use super::*;
use crate::session::context_window::context_window_token_status;
use crate::session::turn::run_pre_sampling_compact;
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
