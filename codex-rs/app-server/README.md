# Model catalog provider requirements

`model/list` and periodic model catalog refreshes check the startup provider against
current managed provider requirements before using the catalog. If that provider no longer
complies, `model/list` returns JSON-RPC error `-32600` asking the client to restart Codex,
and background refreshes skip the old endpoint. Requirement load failures also block these
operations. Checks apply even when the catalog is cached. Existing startup provider selection
and caching behavior remain in effect while the provider satisfies current requirements.

# MCP App UI

`mcpToolCall.mcpAppUi` records the invoked descriptor's `resourceUri`
and `preferredModelDisplayMode` (`inline` or `fullscreen`). Descriptors with a widget
URI default to `inline` when the preference is missing or unsupported. The
UI information is preserved in tool-call events and saved history so clients can
render without waiting for the full MCP catalog.

The field is null for older history and tools that declare widgets only in
result metadata; clients retain catalog discovery for those calls. Existing
resource URI fields remain available for older clients.

# Initial Daybreak choice (experimental)

Persistent threads accept `daybreakEnabled` on `thread/start` with the
`experimentalApi` opt-in. The response and `thread/started` notification both
include the initial choice in `thread.daybreakEnabled`. The choice is staged
with the thread's other initial metadata and saved when the thread is persisted.
An unused thread is not guaranteed to survive restart. Omitted or null leaves
the choice unset. Ephemeral threads cannot save it.
Use `thread/metadata/update` for later changes. This preference does not select
`turn/start.cyberAccessProgram` or grant access to an access program.

# Application network policy

App-server loads application network policy at startup and existing explicit
config/account reloads. Local requirements-file edits take effect on
the next explicit reload or restart. Installing a new policy cancels requests
that it no longer permits; a failed policy load blocks network traffic.

Embedded app-server installs the same policy-aware requirements loader for clients
it constructs. The embedding TUI and exec runtime install the same policy before
creating their telemetry providers, background HTTP clients, and executor
connections. TUI worktree cloud loaders retain that shared policy on reload.

# User verification cancellation (experimental)

Local UI clients can cancel a native user-verification RPC by sending
`userVerification/cancel` with `{requestId}` and the `experimentalApi` opt-in.
The result is an empty acknowledgment (`{}`). This API does not enable desktop
verification capability advertisement.

`requestId` is the original status, enroll, delete, or verify RPC's string or
integer ID on the same connection, not the server elicitation ID. Use fresh IDs
for each operation and a distinct ID for the cancel RPC. Unknown, finished,
unrelated, and other-connection requests are no-ops.

The acknowledgment confirms the cancellation signal without waiting for the OS
prompt to close. The original RPC completes independently, with
`cancelled/interrupted` when cancellation prevents completion. Cancellation
cannot roll back completed effects. It remains effective while a proof waits for
outbound queue capacity, but cannot retract a response already enqueued.

Canceling or resolving an elicitation does not itself stop a separate
`userVerification/verify` RPC. Clients must cancel that RPC separately and discard
late proofs after the approval is canceled or resolved. Only one native worker
runs per app-server; if an OS call remains active after cancellation or timeout,
subsequent local operations return `failed/providerError` until that worker exits.

# Hosted Codex Apps MCP protocol

The host-owned HTTP `codex_apps` server uses Legacy by default in app-server and
standalone Codex. To discover the 2026-07-28 protocol, set
`codex_apps_mcp_2026_07_28 = true` under `[features]`, or send a true runtime
override via `experimentalFeature/enablement/set`. Discovery falls back to Legacy
when the server does not support it. Explicit config takes precedence.
The dedicated setting does not apply to third-party HTTP or local `codex_app`
stdio servers. The existing `mcp_2026_07_28` flag still governs eligible other
servers, regardless of whether their names or URLs resemble hosted Apps.
App-server does not persist this selection.

## Hosted resource reads

`mcpServer/resource/read` accepts `target: {connectorId, linkId}` for direct hosted app reads without tool discovery. A string `linkId` selects that account; `null` explicitly requests no-auth access subject to backend policy. Do not infer no-auth access from unknown or synthetic links.

`originCallId` with `threadId` takes precedence and retains the originating app/account scope. Requests without `target` retain discovery; `connectorId` continues to restrict reads to that connector. Direct targets require backend support for app/account resource reads.

# Project trust

`thread/start` does not persist project trust for a directory where configuration
discovery finds no project-root marker, Git checkout, or project-local `.codex`
directory. Starting a task there does not preapprove project configuration added
later. Existing trust decisions and permission checks for projects are unchanged.

# Thread removal

`thread/archive` and `thread/delete` reject attempts to remove a live internal
worker with JSON-RPC error `-32600`. The worker's owner controls its shutdown.
For example, a Guardian reviewer remains available to its parent conversation
after a client tries to archive or delete it.

After the owner releases the worker, its saved conversation can be archived or
deleted normally. Ordinary client-controlled threads keep their existing behavior.

## User verification (experimental)

Codex app-server advertises `openai/elicitation.userVerification` to the
host-owned plugin service for bundled, in-process TUI sessions (`codex-tui`) and
local stdio desktop sessions (`Codex Desktop`) on devices with supported biometric
hardware and the `experimentalApi` opt-in. This is an app-server decision,
independent of whether a key exists; TUI/Desktop/mobile do not advertise this MCP
capability. Mobile integration requires a separate rollout. Other clients and
network connections do not receive this mode, even with a recognized client name.
Before sending verification requests to desktop sessions, deploy a GUI that
handles the typed verification request, cancellation, and late proofs. The general
`experimentalApi` opt-in does not identify a compatible GUI version.

Native `openai/userVerification` elicitation requests preserve optional `_meta`
JSON through MCP transport and `mcpServer/elicitation/request`. Clients may use
this metadata for extension-specific presentation and must continue to accept
requests without it. Metadata does not change the challenge bytes or the proof
returned in the acceptance response.

Local UI clients use five methods. They require the existing
`experimentalApi` opt-in. The local provider reports
`unavailable/providerUnavailable` on unsupported platforms or without the required
ChatGPT account identity.

| Method | Params | Result |
| --- | --- | --- |
| `userVerification/status` | `{}` | `{credentialId, unavailableReason, unavailableMessage}` |
| `userVerification/enroll` | `{}` | `{credentialId, algorithm?, publicKey?}` |
| `userVerification/delete` | `{}` | `{}` |
| `userVerification/verify` | `{challenge, title, description}` | `{proof: {credentialId, signature}}` |
| `userVerification/cancel` | `{requestId}` | `{}` |

Status reads local readiness without prompting or contacting a backend. A null
`unavailableReason` means local checks passed, not that registration is valid.
Unsupported platforms and missing account identity are reported in the status
response's `unavailableReason` field.
Enrollment creates or reuses the local key and returns its public metadata. The
`publicKey` is unpadded base64url SPKI-DER; `algorithm` is `ecdsaP256Sha256X962`.
During the experimental rollout, `algorithm` and `publicKey` are optional for
compatibility with older app-servers. Current servers populate both fields;
callers must check that both are present and non-null before backend registration.
The trusted UI host owns backend registration: obtain an enrollment challenge,
sign it with `userVerification/verify`, check that the proof's `credentialId`
matches this response, and submit the public metadata and proof to the backend.
Local success is not server enrollment. The caller must preserve the authenticated
account across this flow and reconcile uncertain registration before retrying.
Deletion removes the local key; the caller owns backend revocation.
Enrollment and deletion coordinate credential lifecycle; callers do not issue
separate generate or rotate commands. Identity comes from the authenticated
account; this API exposes no caller-selected scope.

Verify signs 1–4096 decoded challenge bytes using P-256 ECDSA with SHA-256. The
challenge and DER signature use unpadded base64url. Title is 1–256 UTF-8 bytes;
description is at most 4096 bytes. The UI obtains approval for that display
context before calling. Verify does not require a pending elicitation; a UI with
its own authenticator can return proof directly in elicitation response content.
The calling flow owns pending-request checks and discards late proofs.
Native enroll, delete, and verify accept local stdio and in-process connections.
WebSocket and remote-control peers must use their own device authenticator;
status remains available for local readiness. Dropping an embedded RPC, disconnecting,
or changing authentication cancels its native operation. Responses recheck the
captured identity after waiting for outbound queue capacity.
Canceling or resolving an elicitation does not itself stop a separate
`userVerification/verify` RPC. The GUI must use `userVerification/cancel` to
cancel that RPC and discard late proofs when an approval is canceled or resolved.
See [User verification cancellation](#user-verification-cancellation-experimental)
for request ID and acknowledgment semantics.
Only one native worker runs per app-server. If an OS call remains active after
cancellation or timeout, subsequent local operations return `failed/providerError`
until that worker exits.

Failures use the normal JSON-RPC error envelope with closed `{type, reason}` data:
`invalidRequest`, `unavailable`, `cancelled`, or `failed`. UI clients branch on
these values rather than message text. Native diagnostic payloads stay private.

## Local rollout compression

The experimental `rollout/compress` method takes no parameters and immediately
returns `{}` after scheduling one best-effort background pass over the app-server's
local rollout storage. It does not change `features.local_thread_store_compression`
or require that startup flag to be enabled. Non-local thread stores do not support
this method.

The worker retains its existing cold-file checks, maintenance and writer locks,
concurrency limit, and cooldown. Acknowledgement does not imply completion or that
any files were compressed; failures are reported through existing logs and metrics.
There are no progress notifications or cancellation API. Clients sharing this
Codex home must support compressed rollout files, including shared histories.

## Managed model provider requirements

Existing threads retain their provider configuration. Input RPCs reject requests when managed
`model_provider` or `model_providers` requirements no longer match that configuration, or cannot
be loaded. This covers turn start/steer, review, compaction, manual queue start, and active goal
updates. Realtime connections use separate routing configuration and are not checked here.
Interrupt, realtime stop, and goal pause/clear remain available. User and project
configuration changes alone do not invalidate existing threads.

# Amazon Bedrock authentication

If `model_providers.amazon-bedrock.aws.credential_export` is configured, Bedrock setup and
Bedrock login return an error without changing configuration or saved credentials. Remove the
exporter configuration before selecting another credential source. `aws.credential_export` and
`aws.profile` cannot be configured together.

Application network restrictions apply to each AWS credential and region HTTP request and to
the Bedrock destination. Static access keys with an explicit region need no credential discovery.
AWS profile `credential_process` commands are run by the AWS SDK; their network traffic is outside
the application's HTTP policy. Configured credential exporters and AWS reauthentication commands
require unrestricted application policy; policy revocation cancels their active work.

## Stored thread attachments

- `thread/attachment/add` — add a durable resource reference to a stored thread without loading it. Repeated writes with the same attachment type and identity key return the existing attachment.
- `thread/attachment/list` — list attachments for one stored thread in a cursor-paginated request, including a thread that is not loaded.
- `thread/attachment/remove` — remove an attachment by its thread, attachment type, and identity key; returns `{}`.
- `thread/attachment/updated` — notification broadcast after an attachment is created or removed; contains the thread, attachment identity, attachment id, and operation.
### Example: Manage stored thread attachments

Attachments record the resources currently associated with a thread, independently of conversation history. Clients can add, remove, and list attachments for one stored thread at a time without resuming those threads. Adding or removing an attachment does not create or delete the underlying resource or rewrite history. An attachment is idempotently identified by its thread, `attachmentType`, and `identityKey`. For pull requests, clients should reuse the canonical application identity `JSON.stringify([canonicalHostname, lowercaseOwner, lowercaseRepository, pullRequestNumber])` so addition and removal agree across surfaces.

```json
{ "method": "thread/attachment/add", "id": 20, "params": {
    "threadId": "thr_123",
    "attachmentType": "pull_request",
    "identityKey": "[\"github.com\",\"openai\",\"codex\",123]",
    "payload": { "url": "https://github.com/openai/codex/pull/123" }
} }
{ "id": 20, "result": {
    "outcome": "created",
    "attachment": {
        "id": "01984de2-8f74-7c91-a3b2-5c5e937cf318",
        "attachmentType": "pull_request",
        "identityKey": "[\"github.com\",\"openai\",\"codex\",123]",
        "payload": { "url": "https://github.com/openai/codex/pull/123" },
        "createdAt": 1750000000
    }
} }

{ "method": "thread/attachment/list", "id": 21, "params": {
    "threadId": "thr_123",
    "limit": 100
} }
{ "id": 21, "result": {
    "data": [{
        "id": "01984de2-8f74-7c91-a3b2-5c5e937cf318",
        "attachmentType": "pull_request",
        "identityKey": "[\"github.com\",\"openai\",\"codex\",123]",
        "payload": { "url": "https://github.com/openai/codex/pull/123" },
        "createdAt": 1750000000
    }],
    "nextCursor": null
} }

{ "method": "thread/attachment/remove", "id": 22, "params": {
    "threadId": "thr_123",
    "attachmentType": "pull_request",
    "identityKey": "[\"github.com\",\"openai\",\"codex\",123]"
} }
{ "id": 22, "result": {} }

{ "method": "thread/attachment/updated", "params": {
    "threadId": "thr_123",
    "attachmentType": "pull_request",
    "identityKey": "[\"github.com\",\"openai\",\"codex\",123]",
    "attachmentId": "01984de2-8f74-7c91-a3b2-5c5e937cf318",
    "operation": "deleted"
} }
```

`thread/attachment/list` accepts one `threadId` and returns at most 100 attachments per page, ordered by creation time and attachment id. Continue with `nextCursor` and the same `threadId` until the cursor is `null`. Each thread can retain up to 100 attachments. Removing an attachment frees a slot for a new attachment.

A non-ephemeral fork copies the source thread's current attachments, even when forking at an earlier turn. The copies have new attachment IDs and creation timestamps, but retain the same resource identities and payloads. Clients use `forkedFromId` on `thread/started` to detect forks and call `thread/attachment/list` with the new thread ID to load their attachments. Fork copying does not emit per-attachment updates; explicit add/remove operations still do. Copying is awaited before publishing the fork, but is best effort: a copy failure is logged and the conversation fork succeeds without attachments. Membership can then change independently on either thread; the referenced resources themselves are not copied. Resuming a fork does not repeat the copy.

Attachment creation and deletion requests using the same thread ID are serialized across connections. The requesting client receives its response before the compact update is broadcast, and duplicate creates or absent deletes do not emit updates. Deleting the owning thread removes its attachments under the same lifecycle exclusion; queued attachment mutations then report that the thread was not found.

# Thread plugin settings

`thread/settings/update` and `turn/start` accept `disabledPluginIds`, a list of
`PluginSummary.id` values from `plugin/list`, in the
`<plugin-name>@<marketplace-name>` format. A supplied list replaces the selection;
omission or `null` preserves it, and `[]` clears it. Saving this selection does
not yet filter plugin capabilities.

Read the selection from `threadSettings.disabledPluginIds` in
`thread/settings/updated` notifications, or from `disabledPluginIds` in
`thread/start`, `thread/resume`, and `thread/fork` responses. Selections persist
across resume. Forks restore the selection from the history retained at the
requested fork boundary.

# Deprecated thread personality setting

`thread/start`, `thread/resume`, `thread/settings/update`, and `turn/start` still
accept `personality`, but `friendly` and `pragmatic` no longer select a style.
`model/list` returns `supportsPersonality: false` for every model.

`none` removes the literal `# Personality` section when Codex prepares
instructions from the model catalog, for example when starting a thread or
switching models. Setting `friendly` or `pragmatic` can replace a previous
`none` setting for that purpose. Changing the setting does not rewrite the
thread's existing instructions or change explicitly supplied base instructions.
The old `features.personality` flag is ignored.

# MCP server capabilities

`mcpServerStatus/list` returns `serverCapabilities` for each initialized MCP server
in both `full` and `toolsAndAuthOnly` detail modes, including thread-scoped reads.
This is the server's advertised MCP capabilities object, including its `extensions`
map. It is null when the connection has not initialized successfully; capabilities
are never inferred from tools or copied from a shared catalog cache.

# MCP OAuth login

`mcpServer/oauth/login` only returns HTTP(S) authorization URLs. Authorization
endpoints with other schemes fail before client registration or URL return.

# Thread rollback

`thread/rollback` has been removed from the API, including its request and response
types. Requests use the generic unknown-method rejection path. Use `thread/revert`
for paginated threads instead.

Existing rollouts may contain historical `ThreadRolledBack` events. Their replay
and migration remain supported so resuming, reading, and forking those threads
preserves the surviving history. This disk compatibility does not require restoring
support for new `thread/rollback` requests.

# Selected workspace routing

The experimental `account/read.workspaceRouting` response field returns the selected ChatGPT workspace's `chatgptAccountId`, resolved HTTPS `backendOrigin`, and backend-provided `accountRoutingOverride`. The routing value is `us`, `us_cr`, or the explicit `NO_CONSTRAINT` value. API-only and signed-out accounts return `null` and do not need `accounts/check`.

App-server discovers routing for saved ChatGPT logins at startup and for new logins or workspace switches. After requirements and routing are ready, it sends the existing `account/updated` notification. Newly initialized connections also receive this notification once saved-workspace routing is ready, including when discovery finished before the connection initialized. Clients then reread `configRequirements/read` and `account/read`. Saved ChatGPT credentials without a selected workspace ID retain their account information and return `workspaceRouting: null`; app-server does not guess a workspace from the backend's default account. Discovery failures for a selected workspace, including missing or null fields from older backends, return an `account/read` error. They never produce a successful unrestricted result. A later read retries failed discovery. Logout clears the cached routing, and results from earlier authentication owners are discarded. Token refreshes for the same known user and workspace invalidate cached routing without cancelling discovery or failing sign-in. Configuration is reloaded after discovery; a changed backend, model provider, or required backend rejects the result so the next read discovers against current configuration. Account notifications recheck the auth owner generation after waiting for outbound queue capacity. Superseded sign-in attempts emit a failed `account/login/completed` event instead of silently dropping completion. Notifications remain snapshots: clients reread current account and requirements state rather than treating a queued notification as authorization.

Routing compares origins by scheme, host, and effective port, ignoring API paths. A required
`chatgpt_base_url` must match discovery; if neither provides an origin, `NO_CONSTRAINT` uses the
configured base URL.

Responses HTTP (including compaction) and WebSockets wait for discovery and preserve API paths.
Guardian v2 classifier HTTP and pooled WebSockets use the same routing.
HTTP redirects are rejected. `us` and `us_cr` set `X-OpenAI-Account-Routing-Override`;
`NO_CONSTRAINT` omits it.

API-key and explicitly external-auth providers bypass discovery. Custom ChatGPT-auth destinations
require discovery before being treated as independent. Changing a workspace-bound thread's
bootstrap origin requires a new thread.

## Windows sandbox implementation selection

`windowsSandbox/setupStart` applies only to the legacy `elevated` and
`unelevated` backends. `windowsSandbox/readiness` reports `ready` when MXC is
selected so clients do not offer legacy setup. The
`allowedWindowsSandboxImplementations` requirement governs only the legacy
backends and does not restrict MXC. Its `mxc` enum member is retained for wire
compatibility but is not emitted. Non-Windows hosts report `notConfigured`.

MXC uses the standard `command/exec` streaming and process-control path, including
ConPTY when `tty` is enabled. The buffered legacy Windows sandbox restrictions on
process control and custom output caps do not apply to MXC.

### Gateway OAuth sign-in

Providers configured with `gateway_oauth` require a secondary OAuth credential in
addition to their primary authentication. Clients with a gateway sign-in UI set
`initialize.capabilities.explicitGatewayOauth: true`, complete initialization, and
successfully call `account/gatewayOAuth/read` before sending authenticated requests,
including startup `model/list` and inference requests. Repeat this probe on each
new connection. A successful `initialize` alone does not confirm support: older
servers can ignore the unknown capability and retain automatic browser login.

Support for `account/gatewayOAuth/read` and `explicitGatewayOauth` is introduced
together, so a successful read confirms support even when `required` is `false`
or `status` is `notReady`. The returned status determines whether sign-in is needed;
it is separate from the capability check. If the probe fails because the method is
unsupported, require a server upgrade. Other errors and timeouts also leave
authenticated requests blocked until a probe succeeds; do not silently fall back
to automatic login.

With explicit login enabled, app-server refreshes existing credentials, but only the
`account/gatewayOAuth/login` RPC starts browser authorization. Requests needing
sign-in fail promptly so the client can offer that flow.

Clients that omit the capability or set it to `false`, including the TUI, retain
automatic browser authorization after initialization. Startup credential reads
cannot open a browser before initialization. Explicit opt-in is shared by gateway
managers using the same home and network configuration within the process and
cannot be undone by a later connection that omits the capability.

- `account/gatewayOAuth/read` returns the current effective `providerId`,
  `providerName`, `required`, `status`, and `error`. `required` indicates that this
  provider uses gateway OAuth, including when already signed in. This operation
  does not refresh tokens or open a browser; `status` is null for other providers.
  `notReady` means credentials are not ready. `succeeded` means saved credentials
  are locally usable, not that a gateway request has been verified. Reads observe
  usable replacement credentials saved by another process sharing the same home.
- `account/gatewayOAuth/login` starts authorization and returns `{}` after
  the credential has been saved. Providers requiring OpenAI authentication need a
  primary account first. A second login request fails while a login is active.
  The initiating connection receives a `started` notification with `authUrl`; the
  client must open that URL in a browser that can reach the server callback port.
  Other status notifications set `authUrl` to null. Login is rejected if the
  initiating connection opted out of `account/gatewayOAuth/changed` notifications.
- `account/gatewayOAuth/cancel` cancels the calling connection's login and returns
  `{}` after the active login releases its slot, so the client can immediately
  start another login. Closing that connection also cancels its login and releases
  the callback listener. Cancellation makes the pending login request fail.
- `account/gatewayOAuth/changed` reports `notReady`, `started`, `succeeded`, or
  `failed`, with an optional `error`. Notifications apply to the current effective
  gateway configuration. Clients can read readiness when connecting and after
  changing configuration. Notifications follow the standard per-connection
  `optOutNotificationMethods` setting.
  These payloads never contain credentials.

Read, login, and cancel take no params. Read and login use the app's current
provider, reloading configuration and returning an error if it cannot be loaded.
Status notifications include `providerId`. During browser sign-in, inference
requests fail promptly and can be retried when sign-in succeeds.

`model/list` also checks gateway authentication before returning cached models.
If authentication fails after the provider configuration changes, it asks the client
to restart Codex so the retained catalog and gateway sign-in use the same provider.

## Application network policy

Application policy uses the same managed TOML merge as agent-network requirements:
higher-priority layers override conflicting values, including `enabled` and each
domain permission, while non-conflicting domain entries are retained. Omitted
values inherit from lower layers. After merging, a present network block defaults
to `enabled = true` and an empty domain map, meaning no external destinations are
allowed. An effective `enabled = false` disables application destination policy.
Domain keys are exact ASCII names, normalized to lowercase without a trailing dot
before merging; wildcards, URLs, ports, invalid permissions, and duplicate
normalized names are rejected.
App-server enforces these rules for its HTTP and WebSocket traffic before route
resolution or connection work, including redirects and reused clients. An allow
entry permits only HTTPS or WSS to that exact host. Agent-network requirements
remain separate in `network`.

App-server reloads effective requirements on explicit config or account reloads.
Local changes or read failures discovered on reload revoke active requests;
unchanged requirements preserve them. Failed policy loads block traffic until
requirements load successfully. Invalid request or project configuration does not
revoke unrelated traffic. Account changes revoke
outstanding requests and clients retaining the previous account's authorization.
Policy updates also stop active requests to newly denied destinations. Narrow
authentication and requirements-discovery clients use local requirements and
exact endpoint URLs while workspace policy is loading. API-key-only deployments
do not discover ChatGPT workspace requirements.

SDK transports without destination enforcement, including OTLP exporters and AWS
credential discovery/signing, are disabled while restrictions apply. Supported
HTTP, WebSocket, and code-mode gRPC requests use the shared destination checks.
User-directed Git, SSH, shell, and other subprocess traffic retain their existing
execution and sandbox policies.

## Spine feedback and tree events


Spine is optional. Clients that opt into `capabilities.experimentalApi` receive
the experimental boolean `spineFeedbackEnabled` in `thread/start`,
`thread/resume`, and `thread/fork` responses. A value of `true` authorizes the
stable `feedback/spineUpload` request for that active thread; `false` or
an omitted field must be treated as unavailable. The selected active thread
becomes the root of the reported subtree. The request never uploads
without an explicit client call, and the user should be shown the note,
screenshot, and redacted-bundle consent described in the API overview before it
is sent.

Spine state is delivered through typed notifications, never through
`rawResponseItem/*` or `rawResponse/completed`:

- `thread/rolledBack` — stable `{ threadId }` for legacy rollback events.
  New rollback requests use the upstream `thread/revert` API.
- `turn/spineTree/updated` — stable `{ threadId, turnId, snapshotSeq,
  activeNodeId, nodes, settledSpawnCallIds, settledSpawnThreadIds }` whenever a Spine tree
  snapshot changes. `nodes` is the bounded presentation projection and
  `settledSpawnCallIds` identifies Spawn calls whose terminal state has already
  been folded into that live sampling commit. `settledSpawnThreadIds` contains
  child thread IDs from committed Spawn evidence across root epochs, including
  on resume. Clients can hide those completed execution subtrees without
  replaying live progress. Neither field replaces canonical thread history.
- `turn/spineSpawnProgress/updated` — experimental, live-only
  `{ threadId, turnId, callId, tasks }` progress for an active `spine.spawn`.
  It requires `capabilities.experimentalApi`, is not persisted or replayed, and
  clients must tolerate its absence.

All three notification methods can be suppressed with their exact method name
in `capabilities.optOutNotificationMethods`. Opting out affects only delivery
to that connection; it does not disable rollback, tree tracking, or Spawn.

### Fuzzy file search events (experimental)

The fuzzy file search session API emits per-query notifications:

- `fuzzyFileSearch/sessionUpdated` — `{ sessionId, query, files }` with the current matching files for the active query.
- `fuzzyFileSearch/sessionCompleted` — `{ sessionId, query }` once indexing/matching for that query has completed.

### Thread realtime events (experimental)

The thread realtime API emits thread-scoped notifications for session lifecycle and streaming media:

- `thread/realtime/started` — `{ threadId, realtimeSessionId }` once realtime starts for the thread (experimental). `realtimeSessionId` is the upstream Realtime API session identifier, not a Codex session/thread-group id.
- `thread/realtime/itemAdded` — `{ threadId, item }` for raw non-audio realtime items that do not have a dedicated typed app-server notification, including `handoff_request` (experimental). `item` is forwarded as raw JSON while the upstream websocket item schema remains unstable.
- `thread/realtime/transcript/delta` — `{ threadId, role, delta }` for live realtime transcript deltas (experimental).
- `thread/realtime/transcript/done` — `{ threadId, role, text }` when realtime emits the final full text for a transcript part (experimental).
- `thread/realtime/item/started` — `{ threadId, item }` when a realtime item begins. Session boundaries and artifacts complete immediately; transcript segment IDs remain stable through streaming and persistence (experimental).
- `thread/realtime/item/transcript/delta` — `{ threadId, itemId, delta }` for text appended to a started transcript segment (experimental).
- `thread/realtime/item/completed` — `{ threadId, item }` after a session boundary, transcript segment, or promoted backing-agent artifact has been durably committed (experimental).
- `thread/realtime/outputAudio/delta` — `{ threadId, audio }` for streamed output audio chunks (experimental). `audio` uses camelCase fields (`data`, `sampleRate`, `numChannels`, `samplesPerChannel`).
- `thread/realtime/error` — `{ threadId, message }` when realtime encounters a transport or backend error (experimental).
- `thread/realtime/closed` — `{ threadId, reason }` when the realtime transport closes (experimental).

Because audio is intentionally separate from `ThreadItem`, clients can opt out of `thread/realtime/outputAudio/delta` independently with `optOutNotificationMethods`.

### Windows sandbox setup events

- `windowsSandbox/setupCompleted` — `{ mode, success, error }` after a `windowsSandbox/setupStart` request finishes.

### MCP server startup events

- `mcpServer/startupStatus/updated` — `{ threadId, name, status, error, failureReason }` when app-server observes an MCP server startup transition. `threadId` identifies the owning thread when startup is thread-scoped and is `null` when startup is app-scoped. `status` is one of `starting`, `ready`, `failed`, or `cancelled`. `error` and `failureReason` are `null` except for `failed`; `failureReason` is `reauthenticationRequired` when stored OAuth credentials have expired and cannot be refreshed, so clients can prompt the user to reconnect the named server.

### Turn events

The app-server streams JSON-RPC notifications while a turn is running. Each turn emits `turn/started` when it begins running and ends with `turn/completed` (final `turn` status). Token usage events stream separately via `thread/tokenUsage/updated`. Clients subscribe to the events they care about, rendering each item incrementally as updates arrive. The per-item lifecycle is always: `item/started` → zero or more item-specific deltas → `item/completed`.

- `turn/started` — `{ turn }` with the turn id, empty `items`, and `status: "inProgress"`.
- `turn/completed` — `{ turn }` where `turn.status` is `completed`, `interrupted`, or `failed`; successful turns include their final agent message when available, and failures carry `{ error: { message, codexErrorInfo?, additionalDetails?, misalignment? } }`.
- `turn/diff/updated` — `{ threadId, turnId, diff }` represents the up-to-date snapshot of the turn-level unified diff, emitted after every FileChange item. `diff` is the latest aggregated unified diff across every file change in the turn. UIs can render this to show the full "what changed" view without stitching individual `fileChange` items.
- `turn/plan/updated` — `{ turnId, explanation?, plan }` whenever the agent shares or changes its plan; each `plan` entry is `{ step, status }` with `status` in `pending`, `inProgress`, or `completed`.
- `rawResponse/completed` — internal-only; when `thread/start.experimentalRawEvents` is enabled, emits `{ threadId, turnId, responseId, usage }` once for each upstream Responses API completion. `usage` is the exact upstream usage payload mapped to the app-server token breakdown shape and is `null` when the upstream completion omitted usage. Unlike `thread/tokenUsage/updated`, this notification is not accumulated, estimated, persisted, or replayed.
- `model/safetyBuffering/updated` — `{ threadId, turnId, model, useCases, reasons, showBufferingUi, fasterModel }` when a response enters safety buffering. `fasterModel` is nullable. This notification is transient and is not persisted in rollout history.
- `model/rerouted` — `{ threadId, turnId, fromModel, toModel, reason }` when the backend reroutes a request to a different model (for example, due to high-risk cyber safety checks).
- `model/verification` — `{ threadId, turnId, verifications }` when the backend flags additional account verification, such as `trustedAccessForCyber`.
- `modelProvider/authRecoveryStarted` — `{ threadId, turnId, provider, message }` when model-provider authentication recovery begins.
- `modelProvider/authRecoveryCompleted` — `{ threadId, turnId, provider, message }` when model-provider authentication recovery succeeds.
- `turn/moderationMetadata` — experimental; `{ threadId, turnId, metadata }` when a first-party backend supplies turn-scoped moderation metadata for client-side presentation.

`turn/started` carries no items. `turn/completed` carries only the final agent message as a summary fallback; continue consuming `item/*` notifications for the full canonical item list.

#### Items

`ThreadItem` is the tagged union carried in turn responses and `item/*` notifications. Currently we support events for the following items:

- `userMessage` — `{id, clientId, content}` where `clientId` is the optional `clientUserMessageId` supplied to `turn/start` or `turn/steer`, and `content` is a list of user inputs (`text`, `image`, `localImage`, `audio`, or `localAudio`).
- `functionCallOutput` — `{id, name, namespace, output}` for a standalone function-call output without a `call_id`. `namespace` is nullable, and `output` is either a string or structured content items. Clients decide whether to render these tool-authority items; ordinary paired function-call outputs are not emitted separately.
- `agentMessage` — `{id, text, phase, memoryCitation, delivery, questions}` containing the accumulated agent reply. `delivery: "async"` identifies a user-visible message sent without ending the current turn. Async user-input requests also provide `questions`, an ordered array of `{title, options}`; `options: null` means free text only. `text` remains a readable fallback. Replies arrive as ordinary user messages. Ordinary agent messages have `delivery: null` and `questions: null`.
- `plan` — `{id, text}` emitted for plan-mode turns; plan text can stream via `item/plan/delta` (experimental).
- `reasoning` — `{id, summary, content}` where `summary` holds streamed reasoning summaries (applicable for most OpenAI models) and `content` holds raw reasoning blocks (applicable for e.g. open source models).
- `commandExecution` — `{id, pluginId?, scriptPath?, command, cwd, status, commandActions, aggregatedOutput?, exitCode?, durationMs?}` for sandboxed commands; `pluginId` is present only for commands attributed to a trusted first-party plugin, newly attributed items also include `scriptPath` as a safe `/`-separated path relative to the trusted plugin root, older history may omit `scriptPath`, and `status` is `inProgress`, `completed`, `failed`, or `declined`. Ordinary execution items and their replay expose `command` and `commandActions` as redacted display values, not executable commands.
  `cwd` and read `commandActions[].path` use the executor's native path convention, even when the app-server runs on a different operating system. For example, an app-server running on Linux can return `C:\repo\src\main.rs` for a Windows executor; clients must not interpret that path as local to the app-server.
- `fileChange` — `{id, changes, status}` describing proposed edits; `changes` list `{path, kind, diff}` and `status` is `inProgress`, `completed`, `failed`, or `declined`.
- `mcpToolCall` — `{id, server, tool, status, arguments, appContext, mcpAppResourceUri?, pluginId, readOnlyHint, result?, error?}` describing MCP calls; `appContext` is `{connectorId, linkId, resourceUri, appName, actionName}` for calls through a trusted MCP app, where `connectorId` identifies the connector that owns the tool, `linkId` identifies the app link, `resourceUri` points to the widget template, `appName` is the connector's display name, and `actionName` is the stable connector `Action.name`. `readOnlyHint` is `true` for read-only tools, `false` for write-capable tools, and `null` when the annotation is unavailable, including older rollout entries. The hint describes tool capability, not whether an invocation succeeded or performed a write; use `status`, `result`, and `error` to determine the execution outcome. `appName` and `actionName` may be null for older rollout entries. The top-level `mcpAppResourceUri` is deprecated and temporarily duplicated for client migration. `tool` identifies the raw MCP tool. `status` is `inProgress`, `completed`, or `failed`.
- `collabToolCall` — `{id, tool, status, senderThreadId, receiverThreadId?, newThreadId?, prompt?, agentStatus?}` describing collab tool calls (`spawn_agent`, `send_input`, `resume_agent`, `wait`, `close_agent`); `status` is `inProgress`, `completed`, or `failed`.
- `subAgentActivity` — `{id, kind, agentThreadId, agentPath}` describing Multi-Agent V2 lifecycle activity; `kind` is `started`, `interacted`, `interrupted`, or `completed`. A successful child completion is attributed to the parent turn that spawned it, so its `item/completed` notification may arrive after that turn's `turn/completed` notification and is included with that turn when history is read.

  The `CollabAgentTool` schema also includes `sendMessage`, `followupTask`, `interruptAgent`, and
  `listAgents` for private Multi-Agent V2 analytics. These calls do not emit public collaborator tool
  items; their existing `subAgentActivity` notifications are unchanged, and `list_agents` emits no
  activity item. Calls cancelled during handler execution are recorded privately with status
  `interrupted`, distinct from tool failures.
- `webSearch` — `{id, query, action?, results?}` for a web search request issued by the agent; `action` mirrors the Responses API web_search action payload (`search`, `open_page`, `find_in_page`) and may be omitted until completion. For standalone web search, `results` contains the out-of-band structured result DTOs returned by `/v1/alpha/search`; clients should ignore result types and fields they do not understand.
- `imageGeneration` — `{id, status, revisedPrompt, result, transparentBackground, savedPath?}` for a generated image. `transparentBackground` is `true` when the Images API reports a transparent background, `false` when it reports an opaque background, and `null` when the background is automatic, unavailable, or the item has not completed. The field is always present on v2 item payloads, including persisted and resumed items.
- `imageView` — `{id, path}` emitted when the agent invokes the image viewer tool.
- `sleep` — `{id, durationMs}` emitted while the agent waits for a duration or new input.
- `enteredReviewMode` — `{id, review}` sent when the reviewer starts; `review` is a short user-facing label such as `"current changes"` or the requested target description.
- `exitedReviewMode` — `{id, review}` emitted when the reviewer finishes; `review` is the full plain-text review (usually, overall notes plus bullet point findings).
- `contextCompaction` — `{id}` emitted when codex compacts the conversation history. This can happen automatically.
- `compacted` - `{threadId, turnId}` when codex compacts the conversation history. This can happen automatically. **Deprecated:** Use `contextCompaction` instead.

All items emit shared lifecycle events:

- `item/started` — emits the full `item` when a new unit of work begins so the UI can render it immediately; the `item.id` in this payload matches the `itemId` used by deltas.
- `item/completed` — sends the final `item` once that work itself finishes (for example, after a tool call or message completes); treat this as the authoritative execution/result state.
- `item/autoApprovalReview/started` — [UNSTABLE] temporary auto-review notification carrying `{threadId, turnId, targetItemId, review, action}` when approval auto-review begins. This shape is expected to change soon.
- `item/autoApprovalReview/completed` — [UNSTABLE] temporary auto-review notification carrying `{threadId, turnId, targetItemId, review, action}` when approval auto-review resolves. This shape is expected to change soon.
- `autoApprovalReview/strictReviewRequired` — experimental notification carrying `{threadId, turnId, startedAtMs}` whenever elevated or stale Guardian v2 risk requires synchronous approval review.

`review` is [UNSTABLE] and currently has `{status, riskLevel?, userAuthorization?, rationale?}`, where `status` is one of `inProgress`, `approved`, `denied`, or `aborted`. `riskLevel` is one of `"low"`, `"medium"`, `"high"`, or `"critical"` when present. `userAuthorization` is one of `"unknown"`, `"low"`, `"medium"`, or `"high"` when present. `action` is a tagged union with `type: "command" | "execve" | "writeStdin" | "applyPatch" | "networkAccess" | "mcpToolCall" | "requestPermissions"`. Command-like actions include a `source` discriminator (`"shell"` or `"unifiedExec"`). A `writeStdin` action carries `approvalId`, `processId`, `stdin`, and `cwd`; it reviews input to an existing command item without changing that parent item's lifecycle. These notifications are separate from the target item's own `item/completed` lifecycle and are intentionally temporary while the auto-review app protocol is still being designed.

There are additional item-specific events:

#### agentMessage

- `item/agentMessage/delta` — appends streamed text for the agent message; concatenate `delta` values for the same `itemId` in order to reconstruct the full reply.

#### plan

- `item/plan/delta` — streams proposed plan content for plan items (experimental); concatenate `delta` values for the same plan `itemId`. These deltas correspond to the `<proposed_plan>` block.

#### reasoning

- `item/reasoning/summaryTextDelta` — streams readable reasoning summaries; `summaryIndex` increments when a new summary section opens.
- `item/reasoning/summaryPartAdded` — marks the boundary between reasoning summary sections for an `itemId`; subsequent `summaryTextDelta` entries share the same `summaryIndex`.
- `item/reasoning/textDelta` — streams raw reasoning text (only applicable for e.g. open source models); use `contentIndex` to group deltas that belong together before showing them in the UI.

#### commandExecution

- `item/commandExecution/outputDelta` — streams stdout/stderr for the command; append deltas in order to render live output alongside `aggregatedOutput` in the final item.
  Final `commandExecution` items include parsed `commandActions`, `status`, `exitCode`, and `durationMs` so the UI can summarize what ran and whether it succeeded.

#### fileChange

- `item/fileChange/patchUpdated` - when `features.apply_patch_streaming_events` is enabled, streams structured file-change snapshots parsed from the model-generated patch before it is executed.
- `item/fileChange/outputDelta` - deprecated legacy protocol entry for `apply_patch` text output; retained for compatibility but no longer emitted by the server.

### Errors

Ownership rejections for parent-owned Multi-Agent V2 subagents return JSON-RPC error code `-32600` with message `direct app-server input is not allowed for multi-agent v2 sub-agents`.

`error` event is emitted whenever the server hits an error mid-turn (for example, upstream model errors or quota limits). Carries the same `{ error: { message, codexErrorInfo?, additionalDetails?, misalignment? } }` payload as `turn.status: "failed"` and may precede that terminal notification.

`codexErrorInfo` maps to the `CodexErrorInfo` enum. Common values:

- `ContextWindowExceeded`
- `SessionBudgetExceeded`
- `UsageLimitExceeded`
- `rateLimitExceeded`: an upstream rate limit received inside a streaming response; the turn fails with this category only after its existing stream retry budget is exhausted
- `misalignmentPolicyViolation`: a non-retryable request blocked by the misalignment policy
- `HttpConnectionFailed { httpStatusCode? }`: upstream HTTP failures including 4xx/5xx
- `ResponseStreamConnectionFailed { httpStatusCode? }`: failure to connect to the response SSE stream
- `ResponseStreamDisconnected { httpStatusCode? }`: disconnect of the response SSE stream in the middle of a turn before completion
- `ResponseTooManyFailedAttempts { httpStatusCode? }`
- `ActiveTurnNotSteerable { turnKind }`: `turn/start` or `turn/steer` was submitted while the
  current active turn was not steerable, for example `/review` or manual `/compact`
- `BadRequest`
- `Unauthorized`
- `SandboxError`
- `InternalServerError`
- `Other`: all unclassified errors

When an upstream HTTP status is available (for example, from the Responses API or a provider), it is forwarded in `httpStatusCode` on the relevant `codexErrorInfo` variant.

For `misalignmentPolicyViolation`, optional `misalignment` details contain `errorType`,
`detailedExplanation`, and `steer: { message }`. Error categories are open-ended. A category alone
remains a terminal block; clients may offer continuation only when both a substantive explanation
and a steering message are present. To continue after user confirmation, submit the steering
message with the existing `turn/start` method and include
`responsesapiClientMetadata: { misalignment_override: JSON.stringify({ timestamp, feedback }) }`,
where `timestamp` is the confirmation time in Unix milliseconds and `feedback` is the user's
explanation. Misalignment explanation and steering details are delivered live but excluded from
persisted rollout errors, so unavailable details after a restart remain a terminal block.


- `feedback/spineUpload` — submit a user-consented Spine rollout-debug report for any active Spine-enabled thread. The request is stable and accepts `{ threadId, note?, screenshots? }`, where `screenshots` may be omitted, `null`, or an array; it returns `{ reportId }` on a successful upload. The experimental `spineFeedbackEnabled` field on `thread/start`, `thread/resume`, and `thread/fork` responses tells opted-in clients whether the thread is eligible. Calls for a non-Spine thread fail with an invalid-request error. The supplied thread is the root of the selected report subtree: the server redacts its rollout bundle and includes that thread plus its agent descendants. It accepts a note up to 8 KiB and at most three PNG screenshots (5 MiB each, 10 MiB total, 8,192 pixels per side, and 16 million decoded pixels). The rollout bundle and screenshots together are limited to 20 MiB. This is separate from `feedback/upload`; it never emits a raw Responses item.
