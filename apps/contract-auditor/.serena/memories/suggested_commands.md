# 合同审计项目常用命令

## 环境与依赖
- 安装依赖: `pip install -r requirements.txt`

## 核心审计流程
- 运行命令行审计(默认输出目录):
  - `python -m src.main examples/VulnerableVault.sol`
- 指定输出目录:
  - `python -m src.main examples/VulnerableVault.sol data/my_results`

## Web 界面
- 启动 Streamlit Web UI:
  - `streamlit run ui/app.py`

## 测试
- 运行全部测试:
  - `pytest tests/`
- 运行特定模块测试(例如解析器):
  - `pytest tests/test_parser.py -v`
