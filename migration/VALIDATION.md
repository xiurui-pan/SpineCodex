# Migration validation ledger

Target: `rust-v0.153.4` / `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`.
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
