use super::*;
use crate::session::context_window::context_window_token_status;
use crate::session::turn::run_pre_sampling_compact;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::mount_sse_sequence;
use pretty_assertions::assert_eq;
use spine_core::host::SOURCE_LEDGER_AUTO_COMPACT_LIMIT;

#[tokio::test]
async fn source_ledger_pressure_compacts_before_sampling_while_tokens_remain() {
    let session = make_session_with_config(|config| {
        let _ = config.features.enable(Feature::SpineJit);
        let _ = config.features.enable(Feature::TokenBudget);
    })
    .await
    .expect("create Spine session");
    let turn_context = session.new_default_turn().await;
    assert!(!session.source_ledger_needs_auto_compact());

    let items = vec![user_message("cell").into(); SOURCE_LEDGER_AUTO_COMPACT_LIMIT];
    session
        .lock_spine_coordinator()
        .as_mut()
        .expect("Spine coordinator")
        .observe_response_items(&items)
        .expect("fill ledger to compact limit");

    let token_status = context_window_token_status(session.as_ref(), turn_context.as_ref()).await;
    assert!(
        !token_status.token_limit_reached,
        "ledger pressure must not depend on the token window"
    );
    assert!(session.source_ledger_needs_auto_compact());

    let mut client_session = session.services.model_client.new_session();
    run_pre_sampling_compact(
        &session,
        &turn_context,
        &mut client_session,
        &CancellationToken::new(),
    )
    .await
    .expect("pre-sampling compact at ledger pressure");

    assert!(!session.source_ledger_needs_auto_compact());
    assert_eq!(
        session
            .lock_spine_coordinator()
            .as_ref()
            .expect("Spine coordinator")
            .runtime
            .epoch(),
        spine_core::host::ContextEpoch::ZERO
            .checked_next()
            .expect("successor epoch")
    );
    session
        .lock_spine_coordinator()
        .as_mut()
        .expect("Spine coordinator")
        .observe_response_items(&[user_message("after compact").into()])
        .expect("append after compact");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn source_ledger_pressure_compacts_mid_turn_before_follow_up_sampling() {
    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call("call-ledger-pressure", "test_tool", "{}"),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let mut provider = built_in_model_providers(/*openai_base_url*/ None)["openai"].clone();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    provider.supports_websockets = false;
    let (session, events) = make_session_with_config_and_rx(move |config| {
        config.model = Some("gpt-5.2".to_string());
        config.model_provider = provider;
        let _ = config.features.enable(Feature::SpineJit);
        let _ = config.features.enable(Feature::TokenBudget);
    })
    .await
    .expect("create Spine session");
    let turn_context = session.new_default_turn().await;

    // Fill to one cell below the compact limit: the pre-sampling check still
    // passes, and the turn's own records cross the limit before the first
    // post-sampling check.
    let items = vec![user_message("cell").into(); SOURCE_LEDGER_AUTO_COMPACT_LIMIT - 1];
    session
        .lock_spine_coordinator()
        .as_mut()
        .expect("Spine coordinator")
        .observe_response_items(&items)
        .expect("fill ledger just below the compact limit");

    let input = vec![TurnInput::UserInput {
        content: vec![UserInput::Text {
            text: "cross the ledger limit mid-turn".to_string(),
            text_elements: Vec::new(),
        }],
        client_id: None,
    }];
    session
        .spawn_task(turn_context, input, crate::tasks::RegularTask::new())
        .await;

    timeout(Duration::from_secs(60), async {
        loop {
            let event = events.recv().await.expect("event");
            match &event.msg {
                EventMsg::TurnComplete(_) => break,
                EventMsg::TurnAborted(_) => panic!("turn aborted unexpectedly"),
                _ => {}
            }
        }
    })
    .await
    .expect("turn should complete");

    let requests = responses.requests();
    assert_eq!(
        requests.len(),
        2,
        "token-budget compact must not sample the model"
    );

    let coordinator = session.lock_spine_coordinator();
    let coordinator = coordinator.as_ref().expect("Spine coordinator");
    assert_eq!(
        coordinator.runtime.epoch(),
        spine_core::host::ContextEpoch::ZERO
            .checked_next()
            .expect("successor epoch"),
        "mid-turn roll-over must install the compact epoch"
    );
    assert!(!coordinator.source_ledger_needs_auto_compact());
}
