# Spine Spawn fix for 0.4.1

Product version: `0.4.1`. Codex compatibility baseline: `0.153.4`.

## Failure and repair

Spine Spawn scans persisted parent history to freeze the current sampling
boundary. That scan deserialized a flattened `RolloutLine` directly. With
`serde_json/arbitrary_precision`, valid decimal rate-limit values such as
`used_percent: 28.0` failed with `invalid type: map, expected f64`, preventing
all children from starting.

The scan now uses `codex_rollout::decode_rollout_line`, the same JSON decoder
used by the other history readers. History contents, numeric values, sampling
boundaries, concurrency limits, and branch result handling are preserved.

## Reproduction

The public app-server probe completes one parent request carrying real rate-limit
response headers, then starts three Spine branches. A barrier on the three mock
model requests verifies that the children execute concurrently.

- Original 0.3.3, legacy history: three branches complete.
- Original 0.3.3, explicitly paginated history: all three fail before inference.
- Original 0.4.0, default paginated history: the same three startup failures.
- Patched implementation: all three complete and return terminal memory.
- A copy of the reported real session reproduces the exact original decoding
  failure, then successfully resumes and completes all three branches with the fix.
  The original session file is not edited or included in repository fixtures.

## Regression coverage

- A thread-store test verifies both the selected boundary and the complete
  preserved history, including the decimal rate-limit event.
- The existing concurrent/reverse-completion core test now uses paginated
  history and real rate-limit response headers. Both regressions fail before
  the repair and pass after it.
- All 4,484 core/thread-store tests passed after correcting the local test runner
  environment. The runner removes its own Cargo runner variable from test
  processes so binary discovery resolves the actual CLI and MCP executables.
- The 0.4.1 release workflow and product documentation checks passed (8 tests).
- Cargo.lock changes only the versions of 149 workspace packages. External
  dependencies are unchanged; `just bazel-lock-update` passed without lock drift.

The complete 0.4.1 run executed 17,459 tests with 31 platform/environment skips.
The 25 version snapshots across 24 tests differed only by `0.4.0` becoming
`0.4.1`; every diff was reviewed and accepted. All 4,263 TUI tests passed after
the snapshot updates. Three environment-sensitive cases passed when rerun
with a fresh temporary root and without static DNS host entries: the shared
temporary directory had acquired a Git marker, while the DNS entries allowed
`getent` to resolve a name without using the network.

Three exec-server cases remain failed, as independently reproduced against the
unmodified upstream baseline during migration:

- `shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_pipe_recovery`
- `shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_tty_recovery`
- `shell_snapshot_v2_filters_profile_exports_and_stays_in_memory::remote_sandbox`

Their implementation is unchanged by this fix. These failures are documented,
not counted as acceptance passes.

Scoped `just fix` completed with existing Spine warnings retained, followed by
successful `just fmt`. No Rust test suite was rerun after those final commands.
Installation verification uses the assembled native package and public APIs.
