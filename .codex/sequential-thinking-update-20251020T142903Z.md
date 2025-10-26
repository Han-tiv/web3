# sequential-thinking 深度思考（工具缺失手动执行）
- 时间：2025-10-20T14:29:03Z
- 上下文：用户要求梳理 nitter 相关子项目现状，并实现一个从 `twitter-Following-1760964620895.json` 批量抓取关注账户推文、按账户落盘为 JSON 与 Markdown 的归档工具。
- 工具状态：`sequential-thinking` MCP 工具在当前环境中不可用（命令执行返回 127），改以手动记录方式执行深度思考流程。

## 任务理解
1. 需要全面了解 `apps/social-monitor/services/nitter` 及相关数据目录，明确已有 API/服务能力与缺口。
2. 新功能聚焦离线归档：读取 Twitter 导出的关注列表，对每个账号抓取最近推文，并将结果按账号归档为结构化 JSON 与 Markdown，便于后续 LLM 分析。
3. 需评估抓取能力是否可直接复用 `NitterUserService.getUserTweets`，以及如何批量处理大量账号、控制速率与错误。

## 技术方案评估
- 复用现有 `NitterUserService` 的多实例轮询和 HTML 解析，可减少重复造轮子并保持生态一致。
- 需要新增脚本或服务（可能放在 `apps/social-monitor/services/nitter/scripts/`）读取本地 JSON 文件、迭代账号、调用 userService、将结果序列化。脚本建议使用 TypeScript 并复用现有 tsconfig。
- 文件输出层需要明确目录结构（如 `data/following/<username>/tweets.json` 与 `tweets.md`）。Markdown 格式要便于 LLM 消化，可包含元数据、表格或项目符号。
- 需考虑抓取速率限制：Nitter 可能拒绝频繁请求，脚本应添加延时、重试及失败记录。
- 需要存在的日志体系？脚本可复用 `winston`，但也可以使用简单控制台输出以减少依赖。

## 风险识别
- Twitter 导出文件结构复杂，需确认字段可靠性（如 screen_name、name 等是否一定存在）。
- 批量抓取中若所有 Nitter 实例不可用，会导致脚本失败，需要良好错误处理与重试策略。
- Markdown 转换需谨慎处理多行文本及特殊字符，避免破坏格式。
- 需防止覆盖已有归档数据：设计幂等策略（如时间戳文件或覆盖最新一次即可）。
- 大量请求可能触发被封，需在实现中考虑速率限制（例如每个账号之间延迟）。

## 实现步骤规划
1. 梳理 nitter 服务现有结构，记录关键模块、配置、脚本（用于上下文扫描输出）。
2. 分析 `twitter-Following-1760964620895.json` 的 schema，确认关键信息字段（screen_name、name、description 等）。
3. 设计归档目录与文件格式规范，记录于文档/代码常量，确保后续易维护。
4. 在 `scripts` 或 `tools` 目录编写 TypeScript 脚本：读取 JSON → 遍历账户 → 调用 `NitterUserService.getUserTweets` → 写入 JSON + Markdown。
5. 为脚本添加节流、错误重试、日志；为 Markdown 生成器封装辅助函数。
6. 编写或更新测试（如使用模拟 HTML/axios stub）验证核心逻辑，至少覆盖数据格式转换。
7. 编写运行说明及验证流程，并在 `.codex/testing.md`、`verification.md` 记录执行结果。

## 边界条件分析
- JSON 中的账号可能已停用或保护，抓取返回空列表，需优雅处理并仍然生成空数据文件。
- 文件系统路径需要过滤非法字符（如 `/`、`?`），需基于 screen_name 进行安全处理。
- Markdown 输出需避免重复（如多条推文同 ID），可基于 ID 去重。
- 若推文内容包含 emoji 或非 ASCII 字符，需要确认编码；默认使用 UTF-8，Markdown 需兼容。
