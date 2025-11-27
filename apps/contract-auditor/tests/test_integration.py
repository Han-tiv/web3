#!/usr/bin/env python3
"""
集成测试 - 验证端到端功能
"""

import sys
from pathlib import Path

import pytest

# 添加项目根目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.parser.solidity_parser import SolidityParser
from src.analyzer.vulnerability_detector import VulnerabilityDetector
from src.poc_generator import (
    POCGenerator,
    VulnerabilityMetadata,
    VulnerabilityType,
    POCProjectConfig,
)
from src.rules.vulnerability_rules import VulnerabilityRuleEngine


class TestSolidityParser:
    """Solidity解析器测试"""

    def test_parse_vulnerable_vault(self):
        parser = SolidityParser()
        contracts = parser.parse_file("examples/VulnerableVault.sol")

        assert len(contracts) > 0, "应至少解析出1个合约"
        assert "VulnerableVault" in contracts, "应包含VulnerableVault合约"

        vault = contracts["VulnerableVault"]
        assert len(vault.functions) >= 3, "应至少有3个函数"


class TestVulnerabilityDetector:
    """漏洞检测器测试"""

    def test_detect_tx_origin(self):
        detector = VulnerabilityDetector()
        contract = {
            "name": "BadContract",
            "functions": [
                {
                    "name": "withdrawAll",
                    "parameters": [],
                    "modifiers": [],
                    "body": "require(tx.origin == owner); payable(msg.sender).transfer(balance);",
                    "line": 10,
                }
            ],
        }

        findings = detector.detect(contract)
        assert len(findings) > 0, "应检测出tx.origin漏洞"

        tx_origin_found = any("tx.origin" in f["category"].lower() for f in findings)
        assert tx_origin_found, "应包含tx.origin类漏洞"

    def test_detect_delegatecall(self):
        detector = VulnerabilityDetector()
        contract = {
            "name": "BadContract",
            "functions": [
                {
                    "name": "execute",
                    "parameters": ["target", "data"],
                    "modifiers": [],
                    "body": "(bool success, ) = target.delegatecall(data); require(success);",
                    "line": 15,
                }
            ],
        }

        findings = detector.detect(contract)
        assert len(findings) > 0, "应检测出delegatecall漏洞"


class TestPOCGenerator:
    """POC生成器测试"""

    def test_generate_reentrancy_poc(self, tmp_path: Path):
        generator = POCGenerator()
        metadata = VulnerabilityMetadata(
            vuln_type=VulnerabilityType.REENTRANCY,
            description="测试重入漏洞POC生成",
            target_contract="VulnerableVault",
            target_function="withdraw",
        )
        config = POCProjectConfig(root_output_dir=tmp_path)

        result = generator.generate(metadata, config)

        assert Path(result.project_dir).exists(), "POC目录应存在"
        assert Path(result.test_script).exists(), "测试脚本应存在"
        assert Path(result.config_file).exists(), "Hardhat配置应存在"
        assert Path(result.readme_file).exists(), "README应存在"


class TestRuleEngine:
    """规则引擎测试"""

    def test_load_builtin_rules(self):
        engine = VulnerabilityRuleEngine()
        assert len(engine.rules) >= 6, "应至少有6条内置规则"

    def test_match_rules(self):
        engine = VulnerabilityRuleEngine()
        context = {
            "code": "require(tx.origin == owner);",
            "has_external_call": False,
            "has_state_change": False,
        }

        matched = engine.match_rules(context)
        assert len(matched) > 0, "应匹配到tx.origin规则"

    def test_fund_impact_analysis(self):
        engine = VulnerabilityRuleEngine()

        # CRITICAL: 直接转账
        code1 = "payable(msg.sender).transfer(amount);"
        impact1 = engine.analyze_fund_impact(code1, "withdraw")
        assert impact1.value == "直接盗币", "应识别为CRITICAL"

        # MEDIUM: 价格计算
        code2 = "uint256 price = getPrice(); uint256 amount = balance * price;"
        impact2 = engine.analyze_fund_impact(code2, "calculateAmount")
        assert impact2.value == "价值减少", "应识别为MEDIUM"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

