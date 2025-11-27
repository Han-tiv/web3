# 代码风格与约定

## 总体风格
- 语言: Python 3, Solidity
- 遵循 PEP 8 风格, 使用类型注解和 `dataclass` 封装数据结构
- 重要模块使用中文文档字符串说明模块用途和关键流程

## Python 模块结构
- 解析器模块: `src/parser/solidity_parser.py` 使用 `Contract`/`Function`/`StateVariable` 等数据类表示合约结构
- 分析引擎: `src/analyzer/react_engine.py` 使用 `ReActEngine` 进行多轮 Reason→Act→Observe 推理
- 工具模块: `src/utils/config.py`, `src/utils/llm_client.py` 管理配置与 LLM 调用

## 命名约定
- 类名: 使用 `CamelCase` (如 `SolidityParser`, `ReActEngine`)
- 函数与变量: 使用 `snake_case` (如 `parse_file`, `save_json`)
- 常量: 使用全大写 + 下划线 (如 `VULNERABILITY_TAXONOMY`, `CATEGORY_PATTERNS`)

## 文档与注释
- 关键类与函数使用三引号中文 docstring 说明职责
- 复杂逻辑前添加简短中文注释, 说明目的而非逐行翻译代码

## 类型与错误处理
- 重要接口使用 `typing` 模块 (`Dict`, `List`, `Optional`, `Mapping` 等) 明确输入输出
- 对外接口捕获异常后记录日志, 避免影响整体审计流程 (`logger.error`/`logger.warning`)
