use super::*;
use crate::ContextEpoch;
use crate::ThreadNamespace;
use pretty_assertions::assert_eq;

fn ledger() -> SourceLedger {
    SourceLedger::new(
        ThreadNamespace::parse("ledger-compact").expect("valid thread"),
        ContextEpoch::ZERO,
    )
    .expect("empty ledger")
}

fn opaque_cells(start: u64, count: usize) -> Vec<SpineChar> {
    (0..count)
        .map(|offset| SpineChar::Opaque {
            boundary: RawBoundary(start + offset as u64),
        })
        .collect()
}

#[test]
fn ledger_requests_auto_compact_before_the_hard_cap() {
    let mut source = ledger();
    source
        .append(opaque_cells(1, SOURCE_LEDGER_AUTO_COMPACT_LIMIT - 1))
        .expect("fill just below the compact limit");
    assert!(!source.needs_auto_compact());

    source
        .append(opaque_cells(SOURCE_LEDGER_AUTO_COMPACT_LIMIT as u64, 1))
        .expect("cross the compact limit");
    assert!(source.needs_auto_compact());
    assert_eq!(source.cell_count(), SOURCE_LEDGER_AUTO_COMPACT_LIMIT);
    assert!(source.cell_count() < MAX_SOURCE_CELLS);
}
