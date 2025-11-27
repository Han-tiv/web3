#!/usr/bin/env python3
"""
系统演示脚本 - 展示完整功能链路
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from src.main import ContractAuditor


def demo() -> None:
    print("🎬 AI智能合约审计系统 - 功能演示")
    print("=" * 60)

    # 初始化审计器
    print("\n1️⃣ 初始化审计器...")
    auditor = ContractAuditor(config_path="config.yaml")
    print("   ✅ LLM系统、ReAct引擎、规则检测器、POC生成器就绪")

    # 选择测试合约
    test_contracts = [
        "examples/VulnerableVault.sol",
        "examples/VulnerableContract.sol",
    ]

    available = [c for c in test_contracts if Path(c).exists()]
    if not available:
        print("   ⚠️  未找到测试合约")
        print("\n✅ 演示结束（缺少示例合约）")
        return

    contract_path = available[0]
    print(f"\n2️⃣ 选择测试合约: {contract_path}")

    # 执行审计
    print("\n3️⃣ 开始审计流程...")
    print("   提示: 完整ReAct推理可能需要一定时间，可用 --no-rules 或配置禁用LLM 加速测试")

    try:
        report = auditor.audit(
            contract_path=contract_path,
            output_dir="data/demo_results",
            generate_poc=True,
            use_rules=True,
        )

        print("\n4️⃣ 审计结果摘要:")
        print(f"   - 合约路径: {report['metadata']['contract_path']}")
        print(f"   - 分析合约数: {report['summary']['contracts_analyzed']}")
        print(f"   - 发现漏洞数: {report['summary']['total_vulnerabilities']}")
        print(f"   - 高危: {report['summary']['high_count']}")
        print(f"   - 中危: {report['summary']['medium_count']}")
        print(f"   - 低危: {report['summary']['low_count']}")
        print(f"   - 信息: {report['summary']['info_count']}")

        print("\n5️⃣ 漏洞详情(前3条):")
        for i, vuln in enumerate(report.get("vulnerabilities", [])[:3], 1):
            print(f"   {i}. [{vuln.get('severity', 'N/A')}] {vuln.get('category', 'N/A')}")
            print(
                f"      位置: {vuln.get('contract', 'N/A')}:{vuln.get('function', 'N/A')}"
            )
            print(f"      来源: {vuln.get('source', 'N/A')}")
            if vuln.get("poc_path"):
                print(f"      POC: {vuln['poc_path']}")

        print("\n6️⃣ 报告文件:")
        print("   JSON: data/demo_results/audit_report_*.json")

    except Exception as e:  # noqa: BLE001
        print(f"\n❌ 演示过程出错: {e}")
        import traceback

        traceback.print_exc()

    print("\n" + "=" * 60)
    print("✅ 演示完成!")


if __name__ == "__main__":
    demo()

