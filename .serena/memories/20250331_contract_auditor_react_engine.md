# Contract Auditor ReAct 引擎
- 新增 `apps/contract-auditor/src/analyzer/react_engine.py`，实现 25-27 轮的 ReAct 推理循环。
- 提供 `ReActEngine`、`ReActState`、`VulnerabilityFinding`、`Observation` 三个核心数据类，内置漏洞分类、Prompt 模板和置信度阈值过滤（>=0.87）。
- Reason/Act/Observe 管线可调用 `DualModelSystem` 也可 fallback 启发式逻辑，并支持 exploitability 评估和多轮置信度融合。