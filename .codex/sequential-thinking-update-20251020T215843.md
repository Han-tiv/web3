时间：2025-10-20T21:58:43+08:00

## 任务理解
- 目标：修复当前电脑通过 mise 管理的 Node 环境下，`happy-coder` CLI 执行 `rg -n nitter` 时卡住的问题，使其能正常调用 ripgrep。
- 背景：已确认 `happy-coder` npm 包自带的 `tools/unpacked` 目录包含 macOS arm64 版本的 ripgrep 与镜像 `.node`，在 Linux 上导致 `invalid ELF header` 并让 CLI 卡住。
- 范围：限制在本机 mise Node 环境对应的 `happy-coder` 安装目录（可能为 global 或 workspace node_modules），需要清除错误二进制并重新解压对应平台文件。

## 技术要点
1. 查找 mise 管理的 Node 安装位置 (`mise where node` 或查 `~/.local/share/mise/installs/node/...`) 与 npm 全局模块路径。
2. 找出 `happy-coder` 的实际安装目录（可能在全局 npm/pnpm/yarn），确认 `tools/unpacked` 内容。
3. 删除/清空 `tools/unpacked` 后重新运行 `node scripts/unpack-tools.cjs`，确保根据当前平台下载/解压。
4. 若 npm 包发布时已包含错误平台文件，需要强制重新解压或覆盖，或从 `tools/archives/ripgrep-<platform>.tar.gz` 手动提取。
5. 验证 `node scripts/ripgrep_launcher.cjs '["rg","--version"]'` 正常输出后，再让 Codex 执行 `rg -n nitter`。

## 风险
- 找错安装位置导致修改无效；需确认 CLI 实际使用路径。
- mise 环境可能区分多版本 Node，必须定位当前会话使用的版本。
- 如果 happy-coder 版本旧，archives 中缺少对应平台，需检查包内是否包含正确 tarball。
- 操作过程中不要破坏 mise 管理的其他工具。

## 实施步骤
1. 使用 `mise env`/`mise exec` 或 `npm root -g` 查全局包目录；`which happy` 验证 CLI 指向。
2. 在目标目录检查 `tools/unpacked`，通过 `file` 确认架构；若非 Linux x64，执行清理。
3. 运行 `node scripts/unpack-tools.cjs`；若脚本因 `Tools already unpacked` 提前退出，先删除 `tools/unpacked` 后重试。
4. 使用 `file` 再次检查新生成的 `rg` 与 `ripgrep.node`，确保为 `ELF 64-bit LSB`。
5. 通过 `node scripts/ripgrep_launcher.cjs '["rg","-n","nitter","package.json"]'` 或 `happy` 内置命令测试；记录输出，为最終报告提供证据。
