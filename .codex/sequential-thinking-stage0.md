# 手动 Sequential Thinking 记录（工具缺失替代方案）

- **目标理解**：需要复用现有 Nitter 子项目能力，实现一个批处理脚本/服务，读取 `twitter-Following-1760964620895.json` 列表，为每个关注账号抓取推文并落盘 JSON + Markdown，支持后续 LLM 分析。
- **已知信息**：
  - `NitterUserService` 已能抓取指定账号的最近推文（含基本文本和时间）。
  - 项目结构基于 pnpm/TypeScript，Nitter 服务有 Express API 与 cron 调度。
  - 当前落盘与归档能力缺失，需要自建目录与文件输出流程。
- **关键未知**：
  1. `twitter-Following-1760964620895.json` 的数据量与字段细节，是否存在重复/空字段。
  2. 批量抓取的速率限制和失败重试机制，如何与现有 Nitter 实例容错策略对齐。
  3. Markdown 格式需要包含哪些要素（原文、时间、链接、互动指标等）。
  4. 推文抓取数量上限与分页策略，是否需要根据账号活跃度调整。
- **风险与假设**：
  - Nitter 实例可能出现 429/超时，必须实现多实例轮换与退避。
  - JSON 文件可能较大，需要考虑文件命名、分片与幂等性。
  - Markdown 转换需确保中文编码及特殊字符处理。
- **实施初步想法**：
  - 在 `apps/social-monitor/services/nitter` 下新增一个 `scripts/archiveFollowing.ts`（或同类）批处理脚本，直接复用 `NitterUserService` 抓取能力。
  - 读取 JSON 输入，遍历账号，调用 `getUserTweets`，扩展返回字段（增加发布时间、URL、互动数）。
  - 为每个账号创建 `data/archives/<username>/YYYY-MM-DD/` 目录，输出 `tweets.json` 与 `tweets.md`。
  - Markdown 模板包含标题、发布时间、原文、链接，后续可扩展标签或引用。
  - 增加速率控制（如每个账号延时 3-5 秒），并在失败时重试其他实例或记录错误报告。
  - 编写单元测试（或集成测试）模拟抓取与格式化逻辑，确保 Markdown/JSON 结构正确。
