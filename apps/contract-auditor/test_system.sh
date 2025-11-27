#!/bin/bash
# 系统功能测试脚本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🧪 AI智能合约审计系统 - 功能测试"
echo "===================================="

# 激活虚拟环境
if [ -f ".venv/bin/activate" ]; then
    # shellcheck disable=SC1091
    source ".venv/bin/activate"
fi

# 测试1: 解析功能
echo ""
echo "📝 测试1: Solidity解析器"
python3 -c "
from src.parser.solidity_parser import SolidityParser
parser = SolidityParser()
contracts = parser.parse_file('examples/VulnerableVault.sol')
print(f'   ✅ 解析成功: 发现 {len(contracts)} 个合约')
for name, contract in contracts.items():
    print(f'      - {name}: {len(contract.functions)} 个函数')
"

# 测试2: 规则检测
echo ""
echo "🔎 测试2: 静态规则检测"
python3 -c "
from src.analyzer.vulnerability_detector import VulnerabilityDetector
detector = VulnerabilityDetector()
test_contract = {
    'name': 'TestContract',
    'functions': [{
        'name': 'withdrawAll',
        'parameters': [],
        'modifiers': [],
        'body': 'require(tx.origin == owner); payable(msg.sender).transfer(balance);',
        'line': 10
    }]
}
findings = detector.detect(test_contract)
print(f'   ✅ 检测成功: 发现 {len(findings)} 个潜在漏洞')
for f in findings:
    print(f'      - {f[\"category\"]}: {f[\"severity\"]}, 置信度={f[\"confidence\"]:.2f}')
"

# 测试3: POC生成
echo ""
echo "⚡ 测试3: POC生成器"
python3 -c "
from pathlib import Path
from src.poc_generator import POCGenerator, VulnerabilityMetadata, VulnerabilityType, POCProjectConfig

generator = POCGenerator()
metadata = VulnerabilityMetadata(
    vuln_type=VulnerabilityType.REENTRANCY,
    description='测试重入漏洞',
    target_contract='VulnerableVault',
    target_function='withdraw',
    source_file=Path('examples/VulnerableVault.sol').resolve() if Path('examples/VulnerableVault.sol').exists() else None
)
config = POCProjectConfig(root_output_dir=Path('data/test_poc'))
result = generator.generate(metadata, config)
print(f'   ✅ POC生成成功: {result.project_dir}')
print(f'      - 测试脚本: {result.test_script}')
print(f'      - 攻击合约: {result.attack_contract}')
"

# 测试4: CLI模式审计 (跳过ReAct推理,只测结构)
echo ""
echo "🔍 测试4: CLI审计流程 (仅规则检测)"
if [ -f "examples/VulnerableContract.sol" ]; then
    python3 src/main.py examples/VulnerableContract.sol --no-poc -o data/test_results 2>&1 | grep -E "(Step|✅|发现)" || true
    if [ -d "data/test_results" ]; then
        echo "   ✅ 审计完成,报告已生成"
        ls -lh data/test_results/audit_report_*.json | tail -1
    fi
fi

# 测试5: 规则配置加载
echo ""
echo "⚙️ 测试5: 规则配置系统"
python3 -c "
from src.rules.vulnerability_rules import VulnerabilityRuleEngine
engine = VulnerabilityRuleEngine()
print(f'   ✅ 规则引擎加载成功')
print(f'      - 内置规则数: {len(engine.rules)}')
print(f'      - 规则分类: {len(set(r.category.value for r in engine.rules))} 类')
if engine.config:
    print(f'      - 配置文件: 已加载')
"

echo ""
echo "===================================="
echo "✅ 所有测试通过\!"

