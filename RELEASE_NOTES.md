# SpineCodex 0.4.1

Fixes concurrent Spine branches failing before their first model request when
paginated parent history contains decimal rate-limit values. Fork preparation
now uses the shared rollout decoder, preserving the stored history and numeric
values. Regression coverage exercises real rate-limit response headers,
concurrent branch execution, and result ordering with paginated history.

Upstream baseline: Codex `0.153.4`, tag `rust-v0.153.4`, commit
`3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`.

- Migrate Spine sampling, recursive Spawn, tree presentation and feedback onto the new release.
- Carry response-envelope metadata through the Spine source ledger and model projection.
- Store resolved SDK configuration with sampling boundaries, and convert legacy config-lock v1/v2 inputs to Spine-owned version 3 snapshots.
- Keep complete paginated lineage and frozen sampling boundaries readable from compressed rollouts.
- Retain the release's individual successful-command display and `Explored` groups.

The product/package version is `0.4.1`. The public CLI version and upstream HTTP
compatibility identity remain `0.153.4`, following the existing Spine versioning
contract. Release channels and package names remain unchanged.

Version 3 configuration snapshots are self-contained. Legacy version 2 snapshots
contain source digests rather than source content: initial conversion requires
the original referenced sources. Missing or changed sources are identified by
path and digest; they cannot be reconstructed from a digest alone.

See [migration validation](migration/VALIDATION.md) and
[Spine Spawn validation](migration/SPAWN_FIX_VALIDATION.md) for test coverage and
known upstream baseline failures.
