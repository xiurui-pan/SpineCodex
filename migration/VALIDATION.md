# Migration validation ledger

Target: `rust-v0.153.4` / `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`.
Target Spine product version: `0.4.0` (explicit user instruction).
Spine source: `98ee6314e962daa90d83ad122031c0ae4340c2db` (0.3.3).
Source base: `be6e8eac029b183056b7e4402879f15d2c85f61b` (0.147.0).
Worktree: `/home/ray/SpineCodex-upgrade-0.153.4`.
Branch: `spine/upgrade-codex-0.153.4`.

## Baseline

- Both source repositories had no tracked modifications. The original untracked plan is preserved.
- GitHub latest release page redirects to rust-v0.153.4; local tag resolves to the target SHA. The unauthenticated API returned HTTP 403; the release page independently confirms the target.
- `customization-inventory.tsv` records every file in the final Spine diff; pending means not yet ported or verified.
- `baseline-tests.tsv` indexes existing tests for targeted migration; listing a test does not mean it ran.
- Rust, Cargo and just were absent from this WSL environment at start. Rust 1.95.0 installed successfully. Other tools are being installed. No baseline test results yet.

## Acceptance status

P0 in progress. P1-P6 pending. No migration candidate is ready for use or release.

## Verified P1 foundations

- Source `just test -p spine-core`: exit 0, 49 passed, 0 skipped.
- Target `just test -p spine-core`: exit 0, 49 passed, 0 skipped.
- Target `just test -p codex-history -p codex-rollout -p codex-thread-store`: exit 0, 383 passed, 0 skipped.
- Initial build needed pkg-config/OpenSSL development packages; installed the missing system dependencies and reran successfully.
- The first P1 test run found the expected schema variant-count update and an upstream nested-lineage fixture whose local ordinals were not rebased with its history_base. The fixture now writes a valid global ordinal stream; complete and bounded readers agree for plain and compressed segments.
- New response-envelope metadata and Guardian checkpoints survive history codec round trips. Full core projection/resume integration remains pending.
- `just bazel-lock-update`: exit 0 before the later core/config dependency adaptations; must rerun after final dependency changes.
