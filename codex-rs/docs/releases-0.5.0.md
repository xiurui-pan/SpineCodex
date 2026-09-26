# SpineCodex 0.5.0

Updates the upstream Codex baseline to `0.157.1` (`rust-v0.157.1`, commit
`36650394c5b38c2990ccf2a3457165ca3e9d9726`) from the author's `0.4.0` mainline.

- Adapt Spine fork, sampling, recursive Spawn, status subscriptions and compaction to the new agent runtime and step context APIs.
- Preserve complete fork history, durable compaction checkpoints and continuation after child failures.
- Preserve live Spine tree rendering and restore its projection after editing an earlier prompt.
- Persist Spine concurrency changes independently of feature toggles; cancelling unchanged experimental settings does not write configuration.
- Keep Spine orchestration tools out of Guardian reviewer sessions.
- Integrate upstream protocol changes and regenerate stable and experimental schemas and the Python SDK.
- Update daemon installation checks to use app-server initialization after upstream removal of the legacy MCP server.
- Include the native voice helper, runtime and manifest in release packaging using upstream build and assembly tools.

The product/package version is `0.5.0`. The public CLI version and upstream HTTP
compatibility identity remain `0.157.1`, following the existing Spine versioning
contract.
