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
- Rust, Cargo and just were absent from this WSL environment at start. Installed Rust 1.95.0, just, nextest, insta, DotSlash, Bazelisk and required native build dependencies. Source SDK baseline passed (49 tests).

## Acceptance status

P0-P6 source adaptations are present; package validation is in progress. No migration candidate is ready for use or release. The original main worktree remains unchanged.

## Verified P1 foundations

- Source `just test -p spine-core`: exit 0, 49 passed, 0 skipped.
- Target `just test -p spine-core`: exit 0, 49 passed, 0 skipped.
- Target `just test -p codex-history -p codex-rollout -p codex-thread-store`: exit 0, 383 passed, 0 skipped.
- Initial build needed pkg-config/OpenSSL development packages; installed the missing system dependencies and reran successfully.
- The first P1 test run found the expected schema variant-count update and an upstream nested-lineage fixture whose local ordinals were not rebased with its history_base. The fixture now writes a valid global ordinal stream; complete and bounded readers agree for plain and compressed segments.
- New response-envelope metadata and Guardian checkpoints survive history codec round trips. Full core projection/resume integration remains pending.
- `just bazel-lock-update`: exit 0 before the later core/config dependency adaptations; must rerun after final dependency changes.

## Migration verification in progress

- First `just test -p codex-core`: 4,237 executed, 3,991 passed, 246 failed, 9 skipped. This is an intermediate diagnostic result, not an acceptance pass.
- The first run exposed missing first-party binaries, inherited host proxy variables, stale generated schema, request-version fixture mismatches and real Spine defects. Fixed SDK feature synchronization on resume, shared-fork Responses Lite prefix IDs, atomic readonly memory publication, source-fixture lifetime, and target API test adaptations. Reruns pending below.
- `cargo build -p codex-cli -p codex-code-mode-host -p codex-rmcp-client --bins`: passed with the target release's pinned V8 archive and binding (150.4.0), fetched from `openai/codex` release `rusty-v8-v150.4.0` by the existing setup action; both SHA-256 checks passed. Subsequent source edits require rebuilding final artifacts.
- `just write-config-schema`: passed. The target upstream `write-app-server-schema` recipe referred to a deleted binary; repaired the recipe to invoke the existing test-only exporter through `just test`. Stable and experimental generation both passed (one generator test each).
- Release/product Python checks: 8 passed. V8 artifact script tests: 12 passed; pinned MODULE checksum check passed.
- Host proxy variables are removed only for test commands so fixture children receive the environment expected by the upstream suite. No CODEX_SANDBOX variables or sandbox checks were changed.

## Review notes

- All original customization paths have an explicit port/adaptation disposition in `customization-inventory.tsv`; these dispositions describe source handling, not completed validation.
- The upstream native agent status feed remains active for child threads not represented by a Spine tree; Spine tree events use their own display route without duplicating those children. Successful command and patch rendering retain the target implementation.
- Configuration locks v1/v2 are explicitly converted. v2 stores hashes rather than external source bytes, so reconstruction requires matching original source content. New v3 snapshots and sampling records embed the effective SDK configuration, allowing replay after those source files change or disappear.
- Model-visible Spine fragments remain structs in `core/context/spine_context.rs` implementing `ContextualUserFragment`. Manual large-item review: node prompts, memory, spawn evidence and collaboration instructions can exceed 1K tokens; existing SDK validation caps final serialized provider values at 9,500 bytes (plus reserved framing allowance), with synthetic fragments capped at 8,000 bytes. Original raw evidence stays in annotated history; synthetic projection cannot discard Guardian metadata.
- Spawn initial attempts and retries use the frozen StepContext settings, environment and role catalog; continuation and final-memory reminder turns retain parent/root attribution.
- Product version is 0.4.0; protocol compatibility identity is 0.153.4. `--version` and upstream-facing version headers follow the existing compatibility identity contract; product/package identity remains independent.

## Environment restart (2026-09-05 17:57 CST)

- WSL restarted while the affected-package run was still in progress. The `/tmp` logs and running processes were lost. The surviving JUnit report is the prior 465-test focused run (459 passed, 6 failed); it is not evidence that the interrupted 32-package run passed.
- Persistent output directory for resumed checks: `/home/ray/SpineCodex-validation/0.4.0`. Re-downloaded the same pinned V8 files and verified both checksums.
- Resumed affected-package tests use `CARGO_BUILD_JOBS=2` and `--test-threads 4` to keep local resource usage bounded. No tests are filtered out beyond selecting affected packages.
- npm wrapper staged and packed as 0.4.0 successfully; no package was published.

## First complete affected-package run

- `just test --test-threads 8` with the 32 packages in the persistent `packages.json`: 13,720 executed, 13,607 passed (4 slow, 1 flaky), 113 failed, 21 skipped, exit 100. The log and JUnit report are saved as `affected-tests.log` and `affected-first.xml`; failed cases are indexed in `failed-tests.tsv`.
- Confirmed environment causes: `/etc/profile.d/wsl-proxy.sh` invokes `ip route` inside sandboxed login shells, adding a netlink error to stderr; Fake-IP DNS resolves example.com to 198.18.0.220. An isolated mount namespace uses an empty login customization and an example.com hosts entry resolved through Google DNS (original JSON retained). The host files are unchanged. Test processes also omit NO_COLOR and terminal identity variables inherited from the interactive environment.
- The checkpoint write-failure regression passed. Remaining real adaptations include rollback observers without a pending request, fork response ordering, native fork instruction checkpoints, initial Spine source seeds for legacy-session adoption, and SDK transport configuration in resume fixtures.
- Product-version-only status snapshots (19), two model-picker branding snapshots, and the first constrained-viewport snapshot were reviewed and accepted. Environment-sensitive snapshots have not been accepted.
- The package builder's Python unit suite passed: 15 tests. Product/release checks passed again: 8 tests.
- The daemon now probes MCP `initialize` `serverInfo.version` for product identity, which is independent of public CLI compatibility output and works with the existing legacy Spine protocol. A subprocess regression distinguishes product 0.4.0 from compatibility 0.153.4.

## Source initialization and native forks

- The first version-2 sampling-start record carries an optional versioned host replay seed. It records the exact annotated source, compact barriers and usage observations used to initialize the SDK, so prior native compaction or migration edits cannot change the canonical digest on cold replay. Later sampling records continue the same ledger without repeating the seed. Version-1 records remain readable.
- Native bounded forks and full forks with developer/role overrides retain the original canonical prefix and append a context checkpoint for the explicitly changed child context. Their source records are not rewritten. Unmodified full forks and Spine Spawn retain the existing frozen sampling-prefix path.

## Resumed verification

- The complete `cargo check --tests` selection (core, history, thread-store, app-server, TUI, daemon, MCP, CLI and exec) passed after adding separate canonical lineage to resumed histories.
- The rebuilt CLI passed the public JSON-RPC smoke with isolated home/workspace and a local model: first turn, process exit, cold resume, second turn and clean shutdown. Two captured requests confirm Spine tool availability and retained prior input. Evidence: `smoke-current.log` and `smoke-current/requests.json` in the persistent validation directory.
- Compact successors are now prepared without mutating the live SDK. After the rollout checkpoint is durable, the SDK, model projection, host history and window are installed together. The failure regression uses a real enabled Spine session and checks model context as well as history/window/epoch; its next execution is pending.
- TUI organic status snapshots use an explicit activity word, preserving the actual Spine rendering while removing random selection from these layout assertions.

## Upstream comparison for shell snapshot failures

- Exact baseline worktree `/home/ray/codex-baseline-0.153.4` is detached at `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a` and has no source edits.
- `just test -p codex-exec-server --test-threads 1 -E 'test(shell_snapshot_v2)'` in the same isolated environment: both upstream and migration execute 15 tests, with 12 passing and the same 3 failing (exit 100). Evidence: `exec-snapshot-upstream.log/.xml` and `exec-snapshot-debug.log`.
- Identical failures: `shell_snapshot_v2_capture_failure_falls_back_and_retries::{remote_pipe_recovery,remote_tty_recovery}` report four captures instead of two; `shell_snapshot_v2_filters_profile_exports_and_stays_in_memory::remote_sandbox` reports the fixture profile write on a read-only filesystem and exit 41. These are baseline failures, not migration acceptance passes.
- WSL-only shortcut snapshots are run with `/proc/version` masked inside the same isolated mount namespace; this does not change the host kernel identity or application behavior.
- Latest completed focused migration run: 1,128 executed, 1,100 passed, 28 failed, exit 100. Further fixes and the next rerun are in progress.

## Canonical replay and compact checkpoint validation

- Focused rerun after the shared resume loader and staged SDK compact: 1,181 executed, 1,172 passed, 9 failed, exit 100 (`failed-rerun-next.log/.xml`). Three failures are the independently reproduced upstream shell cases.
- The real enabled-Spine writer-failure test preserves host history, model projection, window IDs and SDK epoch. Both staged compact/coordinator tests pass, including unchanged projection before installation.
- Canonical replay preserves raw persisted tool-output envelopes while initializing source identity from the seed; the presentation-difference regression now passes.
- P1/P2 checklist implementation and focused validation are complete. Remaining acceptance work is tracked separately; the candidate is not yet approved for main or release.

## Spawn event routing and packaging rehearsal

- `routing-check.log/.xml`: 1,181 executed, 1,178 passed, three exact upstream baseline failures, exit 100. The embedded app-server/TUI Spawn integration now passes with streamed child activity and terminal retirement.
- Spine child ownership is registered before its UI projection is queued; owned or settling children are admitted by the app-server notification filter. Token-usage tree updates carry their actual turn ID. These changes fix missing terminal child activity and incorrect snapshot attribution.
- The idle async-hook fixture now produces exactly one delayed result; a second unrelated hook result previously consumed an unconfigured third mock response. Guardian policy assertions await the asynchronous root analytics event before shutdown.
- The old npm rehearsal only copied a host GNU executable into a musl vendor directory. It now builds the actual musl target, configures pinned V8 artifacts, assembles the canonical package with its sidecars/resources, and checks the manifest and executable inventory. Eight release contract checks and YAML parsing pass; native CI execution remains pending.
- The subsequent complete affected-package run exposed external-file legacy resume cases. The shared loader now retains a stored rollout path as its authoritative locator instead of replacing it with a thread-ID lookup. That fix awaits the next compiled validation.

## Latest local validation

- Complete 32-package run: 13,719 executed, 13,706 passed (4 slow, 3 flaky), 13 failed, 21 skipped, exit 100 (`affected-final.log/.xml`). Five external-path failures share the corrected locator issue; four UI failures were reviewed branding snapshots; one added picker assertion incorrectly prohibited the target upstream native activity feed and was removed. The other three are the established baseline shell failures.
- Post-fix core/TUI and resume/fork selection: 8,606 executed, 8,605 passed (2 slow, 1 flaky), one failed (`core-tui-final.log/.xml`). The remaining test counted two reads before metadata/history selection was introduced; it now explicitly expects both phases. Final six external-path/store tests pass (`path-final.log`, exit 0).
- Config schema and stable/experimental app-server schema regeneration pass (`schema-final.log`, generator tests 1/1 each). Bazel lock was refreshed after Cargo dependency adaptations.
- Refreshed Linux GNU package passes an expanded public-API smoke: the packaged code-mode host runs a real `exec_command`, returns structured exit code 0 and expected stdout, then the session closes and cold-resumes for another turn. All three mock requests and clean shutdown pass (`smoke-package-code-mode.log` and its captured requests).
- P0-P5 implementation and targeted validation are complete. Three independently reproduced upstream shell failures remain documented. The user has now explicitly authorized full-workspace tests, task-artifact/cache cleanup after verification, and local WSL installation. The unfiltered workspace run is in progress; cross-platform CI, final lint/format and final commit/merge remain outstanding.


## Full workspace and review repairs

- The authorized unfiltered `just test --test-threads 8` compiled the entire workspace and executed 17,461 tests: 17,437 passed, 24 failed, 31 skipped, exit 100 (`full-workspace.log/.xml`). Initial compilation exposed the new thread-manager sample's incomplete Config construction and five analytics response fixtures; both were adapted before execution.
- Actual 0.3.3 binary (product metadata and SHA in the frozen fixture provenance) created a Spine branch and successfully cold-resumed it. The target integration then cold-resumed that immutable version-1 sampling archive, retained historical user input/branch context, and completed another inference request. Only host path metadata is rebased when preparing the cross-platform test; model input and sampling payloads remain unchanged.
- The removed/malformed global SDK-source regressions and the media-only annotation regression passed. SDK source loading now happens at session initialization after history selection; config export uses the resolved snapshot. The resolved configuration is heap allocated to avoid growing nested TUI async state.
- Six HTTP-client failures came from Cargo injecting `SSL_CERT_FILE` after the outer runner scrub. A final test runner (`/usr/bin/env -u SSL_CERT_FILE -u CODEX_CA_CERTIFICATE`) made all six pass without changing HTTP implementation or CA handling. The focused final run uses that same isolation.
- Eleven network-proxy failures involved WSL Fake-IP answers for GitHub/OpenAI hosts. The isolated hosts map now uses saved Google DNS responses for the actual public addresses. One skills-root failure saw a pre-existing `/tmp/.git` marker; the final run mounts a task-owned temporary directory on `/tmp` within its private namespace. Host files are unchanged.
- The V8 POC's own optional `sandbox` feature was absent while the shared V8 dependency was correctly sandbox-enabled by code-mode-runtime. Its focused check explicitly selects `codex-v8-poc/sandbox`; the pinned 150.4.0 archive/bindings are unchanged. Final execution pending.
- The Config unit adapter had wrapped its original NotFound cause as an IO Other error. It now returns the original error chain and checks the NotFound cause. The TUI overview stack-overflow case and all environment-affected cases are being rechecked after the configuration allocation adjustment.
- Cross-platform CI run [33973110581](https://github.com/xiurui-pan/SpineCodex/actions/runs/33973110581) passed six native builds (Linux/macOS/Windows, x64/arm64), package staging/audit, and six npm install smokes. It validates commit `49c1dcb25`; post-review changes require a subsequent final run. Release/npm publication jobs were skipped as intended.

- The focused final selection ran 511 tests: 510 passed and one remaining apex-domain test failed only because `openai.com` still resolved to a Fake-IP. After adding its independently resolved public address to the namespace-only hosts file, that test passed (1/1). The boxed configuration TUI regression, real v1 replay, both removed/malformed-source cases, media annotations, config snapshots, TLS tests and V8 sandbox check all passed. Evidence: `review-final.log/.xml`, `network-final.log`.
- Rechecking shell snapshots in the final clean namespace ran 27 tests: 24 passed and the same three remote baseline cases failed (`shell-clean.log`). Their exec-server implementation is unchanged from the fixed upstream target. These remain recorded upstream failures, not acceptance passes.


## Final source preparation

- Scoped `just fix` completed successfully (9m 10s). It fixed unused imports, redundant clones and error propagation. The remaining assertion, async-guard, enum-size and rendering-style warnings are retained in `final-lint-format.log`; this is not a claim of a warning-free `-D warnings` Clippy run. The duplicate TUI debug-test registration and redundant struct update were removed without changing test bodies or product behavior.
- `just fmt` completed for Rust, Just, Bazel/Starlark, Python scripts and Python SDK. PyPI CDN TLS failed from WSL; the two exact Ruff wheels (0.15.12 and 0.15.13) were fetched from the Tsinghua mirror, verified against their checked-in uv.lock SHA-256 values, and installed into the formatter environments before running with dependency synchronization disabled. No dependency pins or source indexes were changed.
- No local unit/integration suite was rerun after fix/format, as required by AGENTS.md. Final delivery still requires the new native-package CI run and installation smoke.
