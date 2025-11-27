# 合约审计项目风格与约定

## 总体风格
- 使用 Python 3，核心代码位于 `src/`，Web 前端位于 `src/web/`
- 以类和数据类（`@dataclass`）为主组织核心模型（如 `VulnerabilityFinding`、`ContractAuditor`）
- 日志使用标准库 `logging`，统一在模块顶部配置；日志信息偏中文，带 emoji 增强可读性
- 配置集中在 `config.yaml`，通过 `src.utils.config.Config` 访问

## 命名与结构
- 包名与模块名：`src.main`, `src.analyzer.react_engine`, `src.parser.solidity_parser`, `src.utils.llm_client`
- 类名采用帕斯卡命名：`ContractAuditor`, `ReActEngine`, `SolidityParser`, `DualModelSystem`
- 函数/方法采用小写加下划线：`parse_file`, `analyze`, `generate_code`
- Web 端使用 Streamlit，多页结构：`src/web/app.py` + `src/web/pages/*.py`

## 注释与文档
- 关键逻辑（LLM 调用、ReAct 推理、解析流程）使用中文 docstring 说明目的与输入输出
- 在复杂流程处适度加入中文行内注释说明“为什么这样做”，而非解释语法
- 鼓励在涉及安全逻辑、边界条件处理时增加简洁注释

## 设计约定
- 审计主流程通过 `ContractAuditor.audit` 串联：解析 → 多轮 ReAct 推理 →（预留）POC 生成 → 报告落盘
- ReAct 推理引擎 `ReActEngine` 通过 `progress_callback(state, step_snapshot)` 支持实时进度回调
- Web 前端通过 `src/web/session_state.py` 管理 `AppState`，避免直接在各页面散写 `st.session_state` 键
- 漏洞实体统一使用 `VulnerabilityFinding` 字段：`category`, `severity`, `exploitability`, `confidence`, `round_detected`, `target`, `recommendation`, `evidence`

## Web 前端约定
- 统一从 `app.py` 或各页面顶部调用 `apply_global_styles()` 注入 CSS
- 左侧侧边栏由 `src/web/sidebar.py` 负责渲染，展示配置文件路径、报告输出目录与当前任务状态
- 实时进度展示与漏洞列表分别由 `src/web/audit_progress.py` 与 `src/web/vulnerability_list.py` 封装，页面只通过函数调用而不直接操作底层结构

## 错误处理
- 对外暴露的入口（如 `ContractAuditor.audit`）捕获所有异常并写入日志，再向上抛出
- 进度回调失败不会中断审计流程，只在日志中以 warning 记录
- 文件路径相关操作统一使用 `pathlib.Path`，在写文件前保证目录存在 (`mkdir(parents=True, exist_ok=True)`)
