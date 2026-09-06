# SpineCodex 对齐 Codex 0.153.4 开发计划

制定日期：2026-09-05。状态：2026-09-05 用户授权执行，迁移开发进行中。

## 目标与范围

以 OpenAI Codex 最新稳定 release `rust-v0.153.4` 为底座，保留 SpineTree、采样事务、递归 Spawn、Node Memory、投影及既有会话恢复能力，完成 CLI、TUI、app-server 和发布链路适配。

成功命令采用目标 release 的逐条显示行为，保留其原有 `Explored` 分组；不移植 `Ran N commands` 合并补丁，也不新增相关配置。

上游能力沿用目标 release 的默认值、配置开关和适用条件。实验性上下文管理也按上游条件接入；不得通过静默关闭功能、退回旧实现、丢弃历史或增加笼统阻断报错来完成升级。涉及上下文、配置和线程状态的差异，必须在其所属数据结构和生命周期中解决。

用户已授权按本计划开发、合并、完整测试、测试后清理本次任务临时文件及缓存，并将 SpineCodex 0.4.0 安装到本机 WSL。公开发布保留为后续明确操作。

目标 Spine 产品版本：**0.4.0**（用户于执行期间指定）。

## 固定基线

| 项目 | 基线 |
| --- | --- |
| 当前 Spine HEAD | `98ee6314e962daa90d83ad122031c0ae4340c2db` |
| 当前 Spine 产品版本 | `0.3.3` |
| 当前上游底座 | `rust-v0.147.0` / `be6e8eac029b183056b7e4402879f15d2c85f61b` |
| 迁移目标 | `rust-v0.153.4` / `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a` |
| 目标发布时间 | `2026-09-04T23:25:48Z`，北京时间 9 月 5 日 |
| 本地上游仓库 | `../codex`；比较必须指定目标 tag，不使用其 main 工作区作为 release 内容 |
| Rust 工具链 | 当前与目标均为 `1.95.0` |

目标已通过 GitHub latest release API 和本地 tag 核验。开发开始时再次核验版本；若出现更新的稳定 release，先更新本计划的目标 SHA、差分和验收范围，再开始移植。开发过程中固定基线，新增上游 release 作为后续升级处理。

已有只读评估结果：

- 当前底座之后有 97 个 Spine 提交，最终差分为 475 个文件，新增 53,500 行、删除 1,556 行，包含测试、快照和生成文件。
- 当前与目标的共同祖先为 `92b83e226df59dc5ec43a49259d7716821e20c85`；目标相对共同祖先有 1,180 个提交。
- 双方均修改了 226 个文件。隔离对象目录的三方合并预演得到 116 个冲突路径，其中 112 个 content、4 个 modify/delete。
- 冲突集中在 core（35）、TUI（32）、app-server-protocol（13）、rollout（7）、thread-store（5）。这些数字不包括自动合并后可能存在的语义问题。
- 尚未编译或运行迁移测试；预演结果只用于规划。

## 开发方式与交付顺序

在新的 worktree 中从目标 release 创建 `spine/upgrade-codex-0.153.4` 分支。保留当前 Spine 分支与提交，作为行为、数据格式和测试基线。以 Spine 相对 `0.147.0` 的最终差分为移植清单，按功能移植到新底座；每项定制记录为已移植、已由上游覆盖或已按新接口替代，并关联验证结果。

按下面的依赖顺序开发：

```text
P0 基线与差分清单
  → P1 独立 SDK、构建和历史格式
  → P2 配置快照与旧会话恢复
  → P3 采样、工具、投影与上下文窗口
  → P4 Spawn、线程生命周期与预算
  → P5 app-server 与 TUI
  → P6 发布身份、完整回归与交付
```

保留 `spine-core` 的独立算法和 `host` 接口，优先将适配放在已有模块或职责明确的新模块中，减少向 `codex-core` 中央编排文件添加代码。新公共接口保持最小范围。

每个阶段拆成可审查提交；普通非机械差分控制在 800 行以内，复杂逻辑控制在 500 行以内。生成文件、机械类型迁移与行为改动分别整理。阶段最终必须完成其依赖范围的验证，不能将“没有冲突标记”视为完成。

## P0：准备迁移基线

- [x] 核验两个仓库状态、目标 tag 和 SHA；保留已有未提交改动，创建独立开发 worktree。
- [x] 建立最终差分清单，覆盖 SDK、历史、配置、采样、Spawn、协议、TUI、身份、打包和测试。
- [x] 将 Spine 现有测试映射到本计划的验收矩阵；记录原版本已有失败与环境限制。
- [x] 准备人工构造或脱敏的旧版会话 fixtures，覆盖普通会话、分页 lineage、压缩、递归 Spawn、fork、rollback 和中断事务。
- [x] 使用隔离的测试配置目录和模型响应 mock，测试 fixtures 不依赖个人 `~/.codex` 数据或真实凭证。

完成标准：目标固定、定制改动可追踪、旧格式 fixtures 和测试入口明确。

## P1：独立 SDK、构建与历史格式

主要范围：`codex-rs/spine-core/`、目标新增的 `history/`、`protocol/`、`rollout/`、`thread-store/`、`state/` 和相关 Cargo/Bazel 文件。

- [x] 将独立 `spine-core` 接入目标 workspace，保留现有采样、源账本和重放合同。
- [x] 将 Spine 持久化扩展适配到 `codex-history::RolloutItem` 和对应 wire codec；明确持久化类型与 app-server 公共类型的边界。
- [x] 保留旧 `spine_sampling_started`、`spine_transition` 记录的确定性读取和版本转换。
- [x] 将源记录和投影接口适配到 `ResponseItemEnvelope`，保存 `client_authored`、工具输出预算等宿主元数据，防止裸 `ResponseItem` 转换丢失信息。
- [x] 适配 `CompactedItem` 的审查历史、MCP 来源、窗口标识和 usage checkpoint。
- [x] 更新历史新增变体在分页、索引、统计、状态抽取和导出中的完整匹配；按实际含义处理新增记录。
- [x] 更新依赖和 Bazel 输入声明；新增编译期资源读取时同步维护 `compile_data` 等声明。

完成标准：新旧 Spine 记录可往返，元数据在持久化和恢复后保持一致；SDK 与受影响历史/存储测试通过。

## P2：配置快照与旧会话恢复

主要范围：`config/`、`core/src/spine/config.rs`、`core/src/spine/session_config.rs`、`core/src/session/rollout_reconstruction.rs`、历史和线程存储。

上游已通过 `279b93242c` 删除通用 config lock 支持；旧 Spine 的 config-lock v2 保存宿主有效配置及 SDK 来源摘要，没有内嵌外部 SDK 文件内容。迁移后的 v3 记录完整 SDK 配置。这是明确的适配点。

- [x] 梳理旧 config lock 的实际入口、用户可见配置/参数和持久化读取方，列明兼容合同。
- [x] 将 Spine 必需的有效配置快照归属到其自有配置/归档模块，与目标 release 配置生命周期衔接。
- [x] 为已有 config-lock v1/v2 中合法且受支持的输入建立显式格式转换，避免恢复时重新读取变化后的默认配置。
- [x] 新数据使用确定的版本化格式；仅在数据确实损坏或语义不可恢复时报告具体记录和原因。
- [x] 恢复 cwd、权限 profile、环境、模型、推理设置、Spine 配置和分支归属，保持与目标 release 的字段及继承语义一致。
- [x] 验证完整分页 lineage、共享/压缩历史、fork 截断点和 rollback 边界；保留 Spine 已有有效历史恢复修复。

完成标准：同一旧会话恢复后，配置、活动分支和下一次模型输入可解释且一致；所有旧输入兼容项都有实现或明确的格式迁移。

## P3：采样、工具、投影与上下文窗口

主要范围：`core/src/spine/`、`core/src/session/turn.rs`、`step_context.rs`、`context_manager/`、`context/`、`tools/`、`compact*` 及 MCP 管理接口。

- [x] 将 Spine 采样事务绑定到目标 `StepContext` 的 settings、token budget、环境、MCP binding 和 tool router。
- [x] 适配 `Prompt.tools`、直接 Spine 工具和 code-mode 工具路径；模型看到的工具声明与实际执行使用同一份采样快照。
- [x] 明确顺序：开始采样并持久化边界 → 执行响应及工具 → 结算事实 → 持久化提交 → 安装投影。
- [x] 保留有副作用失败与中断的事实，确保恢复或网络重试不会重复执行、重复提交或重复投影。
- [x] 适配异步 hook 输出、MCP 结果处理和工具级 `output_token_limit`；相同结果在实时请求与恢复重放后保持一致。
- [x] 将模型投影与原始证据、审查历史分开维护，正确更新 history version、user-message revision 和 world-state baseline。
- [x] 把上游 token budget、history notes、`new_context` 和 remote compact 的窗口切换映射到 Spine epoch/compact barrier；由统一的窗口切换事务协调两者。
- [x] 保留上游实验开关、默认值和账号/provider 适用条件；验证符合条件且显式启用时与 Spine 的组合行为。
- [x] 维护稳定前缀、fork 缓存亲和性、模型片段上限和重放一致性；由增量检查决定客户端会话复用，避免不必要的 `reset_client_session`。

完成标准：实际出站请求证明采样与执行使用同一快照，事实提交与投影恰好一次，窗口切换与恢复一致，审查所需原始证据完整。

## P4：Spawn、线程生命周期与预算

主要范围：`core/src/agent/control/`、`core/src/spine/spawn*`、`thread_manager.rs`、目标 `agent-roles/` 和 usage/goal 相关接口。

- [x] 适配新的角色加载和带 annotation 的 developer 指令继承。
- [x] 统一父子线程的环境、权限、模型和推理设置继承；不同模型的显式选择按目标接口处理。
- [x] 维护递归 Spawn 的容量准入、并发计数、分支归属和有序结算。
- [x] 对接上游累计 usage 与根 goal budget，明确父线程、子线程和恢复后的计数归属，避免漏计或重复累计。
- [x] 验证失败恢复、继续、重试、取消、关闭、父线程退出和恢复期间的状态转换；修复状态转换根因，避免笼统阻断整个会话。
- [x] 验证已结束子线程不会在恢复后重现为活动线程，未完成分支不会在 rollback 或切换中丢失。

完成标准：递归分支、并发限制、权限和预算在实时执行及恢复后满足相同合同。

## P5：app-server 与 TUI

主要范围：`app-server-protocol/`、`app-server/`、`tui/`、`exec/` 和 Spine feedback。

- [x] 适配目标 v2 请求/响应/通知，保留 SpineTree、SpawnProgress、反馈和线程扩展字段。
- [x] 对齐线程 model/reasoningEffort、异步问题、分页历史、断线重连与恢复后的事件顺序。
- [x] 在新事件路由下处理 Spine 原先删除的 `agent_status_feed` 职责，避免重复显示或遗漏子线程活动。
- [x] 对齐目标 release 的成功命令逐条显示、完整 patch 和终端交互历史。
- [x] 验证 Spine 树、状态栏、子线程选择器、失败恢复界面、线程切换、rollback 和重连。
- [x] 所有 UI 变化增加或更新 insta 快照，审阅 `.snap.new` 后接受预期变化。
- [x] 更新 app-server README 的实际 API 行为及示例；重生成稳定与实验 schema、TypeScript 和 precomputed 导出。

完成标准：公开 JSON-RPC API 和 TUI 快照覆盖新行为；实时显示、恢复重放与线程切换一致，通用命令呈现与目标 release 一致。

## P6：发布身份、回归与交付

主要范围：`Cargo.toml`、目标 `build-info/`、provider/models/login、CLI、npm 包装、Desktop launcher 和 Spine release workflow。

- [x] 同步更新 `codex_compat_version`、`codex_upstream_tag`、`codex_upstream_commit` 到固定目标。
- [x] 保留 Spine 独立产品版本和发布渠道，逐一核对上游新增 build-info 的使用方。
- [x] 核对 `/models?client_version=...`、User-Agent、provider version header、app-server initialize 及 daemon 探测所用的身份合同。
- [ ] 验证更新缓存隔离、npm 二进制解析、CLI/exec 版本输出和 Desktop launcher。
- [x] 更新实际变化的 Cargo/Bazel lock、schema、资源声明和发布元数据检查。
- [x] 完成以下回归矩阵，记录命令、退出状态、测试数量、目标 SHA 和残留问题。
- [ ] 构建本地产物并完成隔离配置下的启动、恢复与关闭冒烟检查，随后完成 Linux/macOS/Windows CI。
- [x] 整理候选版本说明、兼容性说明和构建来源；发布作为后续明确操作执行。

完成标准：候选产物可追溯至固定上游 SHA 与 Spine 提交，身份和包内容正确，要求的测试全部完成，未解决的行为差异有明确记录。

## 验收矩阵

| 主要逻辑变化 / 用户行为 | 必须验证的结果 | 优先复用的测试位置 |
| --- | --- | --- |
| 采样提交、网络重试和取消 | 同一有效事实只提交一次，原请求与后续投影顺序确定 | `core/tests/suite/spine_responses_lite.rs`、`spine_effectful_retry.rs` |
| 工具结果和模型可见上下文 | 工具声明与执行一致，预算元数据持久化，恢复后输入一致 | 上述 Spine suite、目标工具输出/MCP suite |
| Spine close/next、压缩和新窗口 | 前缀稳定，epoch/barrier 一致，审查证据与用户消息保留 | `spine_remote_compact.rs`、`compact_resume_fork.rs`、目标 context-management suite |
| 普通与分页历史恢复 | 完整 lineage 和 Spine 状态可读，压缩文件与共享历史正确处理 | `thread-store` model-context/paginated-fork tests、Spine replay tests |
| fork 和 rollback | 截断点正确，父子上下文与保留元数据一致，没有重复用户消息 | `compact_resume_fork.rs`、app-server thread resume/rollback suites |
| Spawn 并发、乱序完成与失败恢复 | 容量和归属正确，取消/重试不重复副作用 | `core/tests/suite/spine_spawn.rs` |
| 子线程预算和权限继承 | 根预算计入后代但不重复，保存权限与环境正确恢复 | Spine spawn suite、目标 goal/usage/permission suites |
| 旧配置快照转换 | 合法旧数据确定性迁移，恢复不依赖变动后的默认值 | Spine config tests、旧格式 fixtures |
| TUI 与 app-server 恢复 | 树和命令顺序正确，活动状态不重复，命令显示符合目标 release | `tui/src/app/tests/spine_*`、chatwidget tests、app-server v2 suites |
| 产品与兼容身份 | 每个请求/本地探测使用正确身份，更新渠道隔离 | provider、models-manager、login、CLI 和 release identity checks |

测试要求：

- agent 逻辑变化使用集成测试，并断言 mock 捕获的实际请求；优先使用 `build_with_auto_env()`，支持 app/exec 位于不同操作系统。
- 使用完整对象比较，避免仅验证静态定义或新增仅供测试的公共接口。
- 新模型可见片段必须有明确上限，符合 `ContextualUserFragment` 合同；可能超过 1K tokens 的单项按仓库规则进行 P0 人工审查，任何单项不得超过 10K tokens。
- 不修改 `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 或 `CODEX_SANDBOX_ENV_VAR` 相关代码。

## 验证命令与执行约束

以下命令是开发阶段的入口，按实际修改范围选取，不在本次计划编写中执行。

在开发 worktree 的 `codex-rs/` 中，先运行受影响项目的测试：

```bash
just test -p spine-core
just test -p codex-history -p codex-rollout -p codex-thread-store
just test -p codex-core
just test -p codex-app-server-protocol -p codex-app-server
just test -p codex-tui
```

目标 release 中 `spine-core` 是需要移植加入的现有包名。其他实际改动的 crate（例如 config、state、exec、CLI、provider 和 login）同样运行对应的 `just test -p <package>`。不直接运行 `cargo test`，不使用常规 `--all-features`。

生成文件使用仓库入口：

```bash
just write-config-schema
just write-app-server-schema
just write-app-server-schema --experimental
just bazel-lock-update
```

`just bazel-lock-update` 从仓库根目录执行，其余从 `codex-rs/` 执行；仅在对应配置/API/依赖有变化时重生成。迁移时采用目标 release 的 justfile，配置 schema 生成器已迁入 `codex-config-schema`。

受影响项目测试通过后，因为涉及 core/protocol，按根目录 `AGENTS.md` 要求，在实际执行前取得用户对完整 `just test` 的许可；这不影响继续完成其他开发、定向检查和本地产物准备。

大改动完成后执行受影响项目的 `just fix -p <package>`，最终运行 `just fmt`。遵循仓库要求，不在 fix/fmt 后重复运行测试。UI 快照需审阅并接受预期更新。等待 Rust 构建锁时不通过 PID 杀掉命令。

## 最终交付清单

- [ ] 上游目标 tag/SHA、Spine 产品版和源码提交清晰可追溯。
- [ ] 定制差分清单的每一项有明确去向，现有 Spine 核心能力与旧会话恢复通过验收。
- [ ] 上游默认功能与实验开关语义保持一致，成功命令显示对齐目标 release。
- [ ] 关键集成测试、UI 快照、schema、锁文件及要求的跨平台检查完成。
- [ ] 没有用静默降级、吞掉历史或笼统报错掩盖接口不兼容。
- [ ] 本地产物、验证记录与候选版本说明齐备；没有把源码完成等同于已经发布。

## 参考

- [目标 release](https://github.com/openai/codex/releases/tag/rust-v0.153.4)
- [官方 changelog](https://learn.chatgpt.com/docs/changelog)
- [上游恢复成功命令逐条显示](https://github.com/openai/codex/commit/32f48598a0609a882e5847f0d3e35d6d67f375bc)
- [Spine 版本身份合同](codex-rs/docs/spinecodex-versioning.md)
- [当前 Spine 采样与提交接口](codex-rs/core/src/spine/coordinator/session.rs)
- [当前 Spine 配置快照](codex-rs/core/src/spine/config.rs)
- [Spine SDK 宿主接口](codex-rs/spine-core/src/lib.rs)
- [根目录开发约束](AGENTS.md)
