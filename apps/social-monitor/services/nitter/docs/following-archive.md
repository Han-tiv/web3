# Nitter 子项目分析与关注归档方案（2025-10-20，执行者：Codex）

## 1. 子项目结构概览
- **apps/social-monitor/services/nitter**：核心 Nitter 服务，`src/index.ts` 负责 RSS/关键词监控并写入 Redis，`src/api.ts` 提供健康检查与仪表板 API，`src/userService.ts` 封装多实例轮询与 HTML 解析。
- **apps/social-monitor/services/nitter/dist**：TypeScript 编译产物，供现有 Docker/脚本直接运行。
- **apps/social-monitor/services/nitter/data**：原本空目录，本次用于承载关注账号的 JSON/Markdown 归档输出。
- **apps/social-monitor/services/aggregator**：整合 Telegram/Nitter/ValueScan 数据的聚合器，可参考其结构化输出规范与 Redis 消费流程。
- **apps/social-monitor/services/nitter/test**：包含过滤规则测试，本次新增 `archiveFormat.test.js` 校验 Markdown 格式化与参数过滤逻辑。

## 2. 关注归档工具设计
- **脚本位置**：`src/scripts/archiveFollowing.ts`。复用 `NitterUserService` 抓取推文，新增节流与重试机制，默认每个账号间隔 3 秒。
- **辅助模块**：`src/archive/format.ts`、`src/archive/types.ts` 定义目录安全化、Markdown 模板与归档数据结构。
- **CLI 参数**：
  - `--input <path>`：关注列表 JSON，默认指向仓库根部的 `twitter-Following-1760964620895.json`。
  - `--output <path>`：输出目录，默认 `apps/social-monitor/services/nitter/data/following`。
  - `--tweet-limit <n>`：每个账号抓取的推文数，默认 20。
  - `--delay <ms>`：账号之间的休眠时间（毫秒），默认 3000。
  - `--retries <n>`：抓取失败时的重试次数，默认 2。
  - `--limit-accounts <n>`：仅处理前 N 个账号（便于抽样验证）。
  - `--handles a,b`：按账号列表筛选。
  - `--skip-markdown` / `--dry-run`：可选，分别用于跳过 Markdown 与执行演练。
- **错误处理**：重试仍失败会写入空推文列表并在 `meta.errors` 中保留原因，便于后续重跑。

## 3. 输出目录结构
```
apps/social-monitor/services/nitter/data/following/
├─ <screen_name>/
│  ├─ tweets.json   # account/tweets/meta 三段式结构，含抓取耗时与错误记录
│  └─ tweets.md     # Markdown 摘要，便于 LLM 分析
└─ ...
```

## 4. 使用步骤
1. 在 `apps/social-monitor/services/nitter` 目录执行 `npm run build`，确保最新代码生成至 `dist/`。
2. 可选：`npm test` 验证过滤规则与归档格式逻辑。
3. 试运行单个账号以确认配置，例如：  
   `npm run archive:following -- --limit-accounts 1 --tweet-limit 3 --delay 4000`。
4. 全量运行时建议调高 `--delay`（≥3000 ms）或启用自建 Nitter 实例，以规避公共实例的 429 限流。

## 5. 风险与后续建议
- 公开 Nitter 实例限流严格，批量运行前建议准备备用实例或添加更长退避策略。
- 目前仅抓取基础文本，如需媒体、点赞量等指标需扩展 `NitterUserService` 的解析逻辑。
- 可在未来接入归档任务的调度（如 cron + Bun/Node 脚本）并将结果同步至 aggregator 或对象存储，构建长期知识库。
