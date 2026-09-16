use super::*;
use crate::session::context_window::context_window_token_status;
use spine_core::host::RawBoundary;
use spine_core::host::SOURCE_LEDGER_AUTO_COMPACT_LIMIT;
use spine_core::host::SpineChar;

#[tokio::test]
async fn source_ledger_pressure_requests_auto_compact_while_tokens_remain() {
    let session = make_session_with_config(|config| {
        let _ = config.features.enable(Feature::SpineJit);
    })
    .await
    .expect("create Spine session");
    let turn_context = session.new_default_turn().await;
    assert!(!session.source_ledger_needs_auto_compact());

    let characters: Vec<_> = (1..=SOURCE_LEDGER_AUTO_COMPACT_LIMIT as u64)
        .map(|ordinal| SpineChar::Opaque {
            boundary: RawBoundary(ordinal),
        })
        .collect();
    session
        .lock_spine_coordinator()
        .as_mut()
        .expect("Spine coordinator")
        .runtime
        .observe_source(characters)
        .expect("fill ledger to compact limit");

    let token_status = context_window_token_status(session.as_ref(), turn_context.as_ref()).await;
    assert!(
        !token_status.token_limit_reached,
        "ledger pressure must not depend on the token window"
    );
    assert!(session.source_ledger_needs_auto_compact());
}
