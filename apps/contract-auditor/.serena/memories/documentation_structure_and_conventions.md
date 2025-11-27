# contract-auditor 文档结构与约定

本次为 AI 智能合约审计系统补全了 README 主文档，使其与当前实现保持一致，并约定了后续文档更新的基本结构，便于维护：

## 文档角色
- README.md：**总览与对外入口**
  - 面向使用者与贡献者，包含：
    - 项目简介与核心特性
    - 架构图（解析器 + 双引擎检测 + 聚合 + POC + 报告）
    - 项目结构（src/main.py、parser/analyzer/rules/poc_generator/web_ui 等）
    - 快速开始（安装/配置/Ollama/CLI/Web/demo/test_system/pytest）
    - 漏洞能力说明（12 类规则 + 7 类 POC）
    - 配置说明（config.yaml、src/rules/rules.yaml）
    - 性能/实践数据与开发指南
    - 贡献指南与 License、项目信息
- QUICKSTART.md：**操作指南**（已存在，保持不改内容），作为更详细的运行示例与输出样例补充。
- PROJECT_SUMMARY.md：**项目状态与路线图**（已存在，保持不改内容），记录设计目标与迭代规划。

## 与代码的对应关系
- 主入口：`src/main.py` 中 `ContractAuditor`，README 中所有 CLI/Web 示例均以此为准（推荐使用 `audit.sh`、`run_web.sh` 包装）。
- Web 界面：以 `src/web_ui/app.py` 为主，`src/web` 视为旧版多页 Streamlit，README 中注明“兼容保留”。
- 规则引擎：
  - 规则类型枚举、资金影响枚举和规则引擎实现位于 `src/rules/vulnerability_rules.py`；
  - YAML 配置在 `src/rules/rules.yaml`，README 中以示例片段介绍启用状态、严重性和权重；
  - README 中宣称的 12 类漏洞、5 档资金影响和 fund_impact_weights 与当前代码一致。
- POC 生成器：
  - 数据模型位于 `src/poc_generator/models.py`，支持 7 类漏洞类型；
  - 生成逻辑在 `src/poc_generator/poc_generator.py`，模板位于 `src/poc_generator/templates/`；
  - README 中说明每个 POC 工程包含 Hardhat 测试脚本、攻击合约、package.json、hardhat.config.js、.env.example、README.md，与现有模板保持一致。
- 测试体系：
  - `test_system.sh`：一键系统级测试脚本，覆盖解析/规则/POC/CLI/规则配置；
  - `tests/test_integration.py`：pytest 集成测试；
  - `demo.py`：交互式演示，作为 README 中“演示模式”部分的落地实现。

## 维护约定
- 若后续扩展新漏洞类型或 POC 模板：
  1. 先更新 `VulnerabilityCategory` / `VulnerabilityType` 与规则/模板；
  2. 再同步更新 README 中的“漏洞检测能力”和“POC 生成支持”列表；
  3. 若新增测试脚本或演示方式，在 README 的“测试系统”或“使用方式”章节追加。
- 若 Web 前端结构调整（例如完全弃用 `src/web`）：
  - 需要更新 README 的“项目结构”和“Web 模式”描述，同时在 PROJECT_SUMMARY 中记录一次架构变更。
- README 末尾保留 `<repo_url>`、`<docs_url>` 占位符，由项目维护者在仓库公开前替换为真实地址。

该记忆可作为未来更新文档时的参考基线，确保 README 与代码实现保持一致，且不破坏 QUICKSTART/PROJECT_SUMMARY 的既有内容。