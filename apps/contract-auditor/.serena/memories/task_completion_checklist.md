# 任务完成检查清单

在本项目中完成一个开发/改动任务后,建议执行以下步骤:

1. **代码自查**
   - 确认新增/修改的模块有清晰中文注释或 docstring 说明关键行为
   - 确保遵循现有命名与模块划分方式(解析在 `parser`, 分析在 `analyzer`, 工具在 `utils`)

2. **静态检查与格式化**
   - 代码格式化(如已配置): `black src tests`
   - 静态检查: `flake8 src tests`
   - 类型检查: `mypy src`

3. **测试**
   - 运行全部单元测试: `pytest tests/`
   - 若只影响部分模块,可仅运行相关测试文件(例如解析器相关: `pytest tests/test_parser.py -v`)

4. **功能验证**
   - 使用示例合约跑一遍完整审计流程,确认无运行时异常:
     - `python -m src.main examples/VulnerableVault.sol`

5. **结果检查**
   - 检查 `data/results` 下生成的 JSON 报告是否包含预期信息
   - 对新增的分析维度/字段(例如新的漏洞规则或评分)进行人工 spot-check,确认字段含义与文档一致
