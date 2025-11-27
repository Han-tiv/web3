# Serena rust-trading-bot 健康检查修复
- 日期：2025-11-22
- 背景：在 `serena project health-check apps/rust-trading-bot` 时扫描 1.2 万个文件并被终止，原因是该项目的 Serena 配置只启用了 TypeScript 语言服务器且未忽略 web 子项目的 node_modules/dist。
- 措施：
  - 将 `.serena/project.yml` 中的 `languages` 调整为 `rust` 优先并保留 `typescript`，确保健康检查优先挑选 Rust 源码。
  - 在同一配置内加入 `ignored_paths`（node_modules、web/dist、web/.next 等），阻止工具遍历 web 产物与依赖。
- 结果：健康检查仅扫描 79 个文件便通过，日志见 `apps/rust-trading-bot/.serena/logs/health-checks/health_check_20251122-002531.log`。
