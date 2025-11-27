#!/usr/bin/env python3
"""
AI智能合约审计系统 - 主程序
整合 ReAct 推理引擎、POC 生成、静态漏洞检测规则, 支持 CLI 与 Web 双模式。
"""

from __future__ import annotations

import argparse
import json
import logging
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional

# 添加项目根目录到路径
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.analyzer.react_engine import ReActEngine
from src.analyzer.vulnerability_detector import VulnerabilityDetector
from src.parser.solidity_parser import SolidityParser
from src.poc_generator.models import (
    POCProjectConfig,
    VulnerabilityMetadata,
    VulnerabilityType,
)
from src.poc_generator.poc_generator import POCGenerator
from src.utils.config import Config
from src.utils.llm_client import DualModelSystem

# 进度回调类型：与 ReActEngine.analyze 的 progress_callback 保持一致
ProgressCallback = Callable[[Any, Dict[str, Any]], None]

# 确保日志目录存在
Path("data/logs").mkdir(parents=True, exist_ok=True)

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler("data/logs/auditor.log"),
    ],
)
logger = logging.getLogger(__name__)


class ContractAuditor:
    """智能合约审计器 - 集成版"""

    def __init__(self, config_path: str = "config.yaml") -> None:
        logger.info("🚀 初始化 AI Contract Auditor（集成版）")

        # 加载配置
        self.config = Config(config_path)
        llm_config = dict(self.config.llm_config or {})

        # 初始化 LLM 双模型系统
        self.llm_system = DualModelSystem(
            reasoning_model=llm_config["reasoning_model"],
            coding_model=llm_config["coding_model"],
            api_url=llm_config["api_url"],
        )

        # 初始化解析器
        self.parser = SolidityParser()

        # 初始化 ReAct 引擎
        max_rounds = int(llm_config.get("max_rounds", 27) or 27)
        min_confidence = float(llm_config.get("min_confidence", 0.87) or 0.87)
        temperature = float(llm_config.get("temperature", 0.7) or 0.7)
        max_tokens = int(llm_config.get("max_tokens", 4000) or 4000)

        self.react_engine = ReActEngine(
            llm=self.llm_system,
            max_rounds=max_rounds,
            min_confidence=min_confidence,
            temperature=temperature,
            max_tokens=max_tokens,
        )

        # 静态规则检测器
        self.rule_detector = VulnerabilityDetector()

        # POC 生成器
        self.poc_generator = POCGenerator()

        logger.info("✅ 初始化完成")

    # ------------------------------------------------------------------ #
    # 对外主入口
    # ------------------------------------------------------------------ #
    def audit(
        self,
        contract_path: str,
        output_dir: str = "data/results",
        generate_poc: bool = True,
        use_rules: bool = True,
        progress_callback: Optional[ProgressCallback] = None,
    ) -> Dict[str, Any]:
        """
        执行完整审计流程: 解析 → 静态规则检测 → ReAct 推理 → POC 生成 → 报告落盘。

        Args:
            contract_path: 合约文件路径
            output_dir: 输出目录
            generate_poc: 是否生成 POC
            use_rules: 是否使用静态规则检测
            progress_callback: ReAct 推理进度回调（供 Web UI 使用）

        Returns:
            审计报告字典
        """
        logger.info("📄 开始审计: %s", contract_path)
        start_time = datetime.now()

        try:
            output_dir_path = Path(output_dir)
            output_dir_path.mkdir(parents=True, exist_ok=True)

            # Step 1: 解析合约
            logger.info("\n" + "=" * 80)
            logger.info("📝 Step 1: 解析 Solidity 合约")
            logger.info("=" * 80)

            contracts = self.parser.parse_file(contract_path)

            # 保存解析结果, 方便调试与复用
            parse_result_path = output_dir_path / "parsed_contracts.json"
            self.parser.save_json(str(parse_result_path))
            logger.info("💾 已保存解析结果: %s", parse_result_path)

            # Step 2a: 静态规则检测
            rule_findings: List[Dict[str, Any]] = []
            if use_rules:
                logger.info("\n" + "=" * 80)
                logger.info("🔎 Step 2a: 执行静态规则检测")
                logger.info("=" * 80)

                for _, contract in contracts.items():
                    contract_dict = self._convert_contract_format(contract)
                    findings = self.rule_detector.detect(contract_dict)
                    rule_findings.extend(findings)

                logger.info("✅ 规则检测发现 %d 个潜在问题", len(rule_findings))
            else:
                logger.info("⏭️  已关闭静态规则检测")

            # Step 2b: ReAct 多轮推理
            logger.info("\n" + "=" * 80)
            logger.info(
                "🤖 Step 2b: 启动 ReAct 多轮推理 (%d 轮)", getattr(self.react_engine, "rounds", 0)
            )
            logger.info("=" * 80)

            analysis_result = self.react_engine.analyze(
                contracts=contracts,
                progress_callback=progress_callback,
            )
            ai_vulnerabilities: List[Dict[str, Any]] = analysis_result.get(
                "vulnerabilities", []
            )
            logger.info("✅ AI 推理发现 %d 个漏洞", len(ai_vulnerabilities))

            # Step 3: 合并去重漏洞
            logger.info("\n" + "=" * 80)
            logger.info("🔀 Step 3: 合并去重漏洞列表")
            logger.info("=" * 80)

            merged_vulns = self._merge_vulnerabilities(
                rule_findings=rule_findings,
                ai_findings=ai_vulnerabilities,
                contracts=contracts,
            )
            logger.info("✅ 合并后共 %d 个唯一漏洞", len(merged_vulns))

            # Step 4: 生成 POC
            poc_paths: Dict[str, str] = {}
            if generate_poc:
                logger.info("\n" + "=" * 80)
                logger.info("⚡ Step 4: 生成 POC exploit 脚本")
                logger.info("=" * 80)

                poc_output_dir = output_dir_path / "poc"
                poc_paths = self._generate_pocs(
                    vulnerabilities=merged_vulns,
                    contract_path=contract_path,
                    poc_output_dir=poc_output_dir,
                )
                logger.info("✅ 已为 %d 个漏洞生成 POC 工程", len(poc_paths))
            else:
                logger.info("⏭️  已关闭 POC 生成")

            # Step 5: 生成报告
            logger.info("\n" + "=" * 80)
            logger.info("📊 Step 5: 生成审计报告")
            logger.info("=" * 80)

            end_time = datetime.now()
            duration = (end_time - start_time).total_seconds()

            report = self._build_report(
                contract_path=contract_path,
                contracts=contracts,
                vulnerabilities=merged_vulns,
                poc_paths=poc_paths,
                analysis_result=analysis_result,
                duration=duration,
                tokens_used=self.llm_system.total_tokens,
            )

            # 兼容旧 Web 界面: 挂载原始 ReAct 结果
            report["analysis"] = analysis_result

            report_path = (
                output_dir_path
                / f"audit_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
            )
            with report_path.open("w", encoding="utf-8") as f:
                json.dump(report, f, indent=2, ensure_ascii=False)

            summary = report.get("summary", {})
            logger.info("\n✅ 审计完成！")
            logger.info("📁 报告: %s", report_path)
            logger.info("🔢 Tokens: %s", self.llm_system.total_tokens)
            logger.info("⏱️  耗时: %.1f 秒", duration)
            logger.info(
                "📊 风险统计 - 高危: %d, 中危: %d, 低危: %d, 信息: %d",
                summary.get("high_count", 0),
                summary.get("medium_count", 0),
                summary.get("low_count", 0),
                summary.get("info_count", 0),
            )

            # 打印合并后漏洞摘要
            self._print_summary(merged_vulns)

            return report

        except Exception as exc:  # noqa: BLE001
            logger.error("❌ 审计失败: %s", exc, exc_info=True)
            raise

    # ------------------------------------------------------------------ #
    # 内部工具方法
    # ------------------------------------------------------------------ #
    def _convert_contract_format(self, contract: Any) -> Dict[str, Any]:
        """
        将 SolidityParser 输出的 Contract 对象转换为 VulnerabilityDetector 输入格式。
        """
        functions: List[Dict[str, Any]] = []
        for func in getattr(contract, "functions", []) or []:
            functions.append(
                {
                    "name": getattr(func, "name", ""),
                    # 规则引擎不依赖参数精确结构, 直接传递原始参数字符串列表
                    "parameters": list(getattr(func, "parameters", []) or []),
                    "modifiers": list(getattr(func, "modifiers", []) or []),
                    "body": getattr(func, "body", "") or "",
                    # 当前解析器尚未提供精确行号, 预留字段供后续扩展
                    "line": getattr(func, "line_number", 0) or 0,
                }
            )

        return {
            "name": getattr(contract, "name", ""),
            "functions": functions,
        }

    def _normalize_severity(self, raw: Any) -> str:
        """统一严重级别表示, 便于统计与后续处理。"""
        if raw is None:
            return "INFO"
        text = str(raw).strip()
        if not text:
            return "INFO"

        upper = text.upper()
        mapping = {
            "CRITICAL": "HIGH",
            "HIGH": "HIGH",
            "H": "HIGH",
            "MEDIUM": "MEDIUM",
            "M": "MEDIUM",
            "LOW": "LOW",
            "L": "LOW",
            "INFO": "INFO",
        }
        return mapping.get(upper, "INFO")

    def _merge_vulnerabilities(
        self,
        rule_findings: List[Dict[str, Any]],
        ai_findings: List[Dict[str, Any]],
        contracts: Dict[str, Any],
    ) -> List[Dict[str, Any]]:
        """
        合并并去重漏洞列表。

        以 (contract, function, category) 作为去重键, 同时保留 source 字段标记来源:
        - rule_based: 静态规则检测
        - ai_react: ReAct LLM 推理
        """
        merged: List[Dict[str, Any]] = []
        seen_keys: set[str] = set()

        default_contract = next(iter(contracts.keys()), "") if contracts else ""

        # 规则检测结果
        for finding in rule_findings:
            finding = dict(finding)
            finding["severity"] = self._normalize_severity(finding.get("severity"))
            finding["source"] = "rule_based"

            key = f"{finding.get('contract', '')}:{finding.get('function', '')}:{finding.get('category', '')}"
            if key in seen_keys:
                continue

            merged.append(finding)
            seen_keys.add(key)

        # AI 检测结果
        for finding in ai_findings:
            finding = dict(finding)

            target = str(finding.get("target") or "").strip()
            contract_name = str(finding.get("contract") or "").strip()
            function_name = str(finding.get("function") or "").strip()

            if not contract_name:
                if ":" in target:
                    contract_part, func_part = target.split(":", 1)
                    contract_name = contract_part.strip()
                    if not function_name:
                        function_name = func_part.strip()
                elif default_contract:
                    contract_name = default_contract

            category = (
                finding.get("category")
                or finding.get("type")
                or "未分类"
            )

            finding["contract"] = contract_name
            finding["function"] = function_name or target
            finding["category"] = category
            finding["severity"] = self._normalize_severity(finding.get("severity"))
            finding["source"] = "ai_react"

            key = f"{contract_name}:{finding['function']}:{category}"
            if key in seen_keys:
                continue

            merged.append(finding)
            seen_keys.add(key)

        return merged

    def _generate_pocs(
        self,
        vulnerabilities: List[Dict[str, Any]],
        contract_path: str,
        poc_output_dir: Path,
    ) -> Dict[str, str]:
        """
        为高危/中危漏洞批量生成 POC 工程。

        返回:
            vuln_id (contract:function) -> POC 工程目录
        """
        poc_paths: Dict[str, str] = {}
        source_path = Path(contract_path)

        vuln_type_map: Dict[str, VulnerabilityType] = {
            "重入攻击": VulnerabilityType.REENTRANCY,
            "权限绕过": VulnerabilityType.ACCESS_CONTROL,
            "精度丢失": VulnerabilityType.PRECISION_LOSS,
            "整数溢出": VulnerabilityType.INTEGER_OVERFLOW,
            "未检查返回值": VulnerabilityType.UNCHECKED_RETURN,
            "时间戳依赖": VulnerabilityType.TIMESTAMP_DEPENDENCE,
            "签名重放": VulnerabilityType.SIGNATURE_REPLAY,
        }

        for vuln in vulnerabilities:
            severity = self._normalize_severity(vuln.get("severity"))
            if severity not in {"HIGH", "MEDIUM"}:
                continue

            vuln_type = vuln_type_map.get(str(vuln.get("category", "")).strip())
            if not vuln_type:
                continue

            try:
                metadata = VulnerabilityMetadata(
                    vuln_type=vuln_type,
                    description=str(vuln.get("description", "")),
                    target_contract=str(vuln.get("contract", "")),
                    target_function=vuln.get("function") or None,
                    source_file=source_path if source_path.exists() else None,
                    severity=severity,
                )

                config = POCProjectConfig(root_output_dir=poc_output_dir)
                result = self.poc_generator.generate(metadata, config)

                vuln_id = f"{metadata.target_contract}:{metadata.target_function or ''}"
                poc_paths[vuln_id] = str(result.project_dir)
            except Exception as exc:  # noqa: BLE001
                logger.warning(
                    "⚠️  生成 POC 失败 (category=%s, contract=%s, function=%s): %s",
                    vuln.get("category"),
                    vuln.get("contract"),
                    vuln.get("function"),
                    exc,
                )

        return poc_paths

    def _build_report(
        self,
        contract_path: str,
        contracts: Dict[str, Any],
        vulnerabilities: List[Dict[str, Any]],
        poc_paths: Dict[str, str],
        analysis_result: Dict[str, Any],
        duration: float,
        tokens_used: int,
    ) -> Dict[str, Any]:
        """构建最终审计报告结构。"""
        severity_counts: Dict[str, int] = {"HIGH": 0, "MEDIUM": 0, "LOW": 0, "INFO": 0}

        # 规范化严重级别并填写 POC 路径
        vuln_entries: List[Dict[str, Any]] = []
        for vuln in vulnerabilities:
            v = dict(vuln)
            sev = self._normalize_severity(v.get("severity"))
            v["severity"] = sev
            if sev in severity_counts:
                severity_counts[sev] += 1

            vuln_id = f"{v.get('contract', '')}:{v.get('function', '')}"
            if vuln_id in poc_paths:
                v["poc_path"] = poc_paths[vuln_id]

            vuln_entries.append(v)

        summary: Dict[str, Any] = {
            # 新版字段
            "total_vulnerabilities": len(vuln_entries),
            "high_count": severity_counts["HIGH"],
            "medium_count": severity_counts["MEDIUM"],
            "low_count": severity_counts["LOW"],
            "info_count": severity_counts["INFO"],
            "contracts_analyzed": len(contracts),
            # 兼容旧 Web 界面字段
            "total_contracts": len(contracts),
            "high_risk": severity_counts["HIGH"],
            "medium_risk": severity_counts["MEDIUM"],
            "low_risk": severity_counts["LOW"],
            "confidence": float(analysis_result.get("confidence", 0.0) or 0.0),
        }

        report: Dict[str, Any] = {
            "metadata": {
                "contract_path": contract_path,
                "audit_date": datetime.now().isoformat(),
                "auditor": "AI Contract Auditor v1.0",
                "engine": "ReAct + Rule-Based Hybrid",
                "duration_seconds": duration,
                "tokens_used": tokens_used,
            },
            "summary": summary,
            "contracts": list(contracts.keys()),
            "vulnerabilities": vuln_entries,
            "analysis_metadata": {
                "rounds": analysis_result.get("rounds"),
                "confidence": analysis_result.get("confidence"),
                "history_length": len(analysis_result.get("history", [])),
            },
        }

        return report

    def _print_summary(self, vulnerabilities: List[Dict[str, Any]]) -> None:
        """打印合并后漏洞摘要到日志。"""
        if not vulnerabilities:
            logger.info("\n" + "=" * 80)
            logger.info("📊 漏洞摘要: 未发现漏洞")
            logger.info("=" * 80)
            return

        logger.info("\n" + "=" * 80)
        logger.info("📊 漏洞摘要（合并后）")
        logger.info("=" * 80)

        for vuln in vulnerabilities:
            severity = self._normalize_severity(vuln.get("severity"))
            category = vuln.get("category") or vuln.get("type") or "Unknown"
            try:
                confidence = float(vuln.get("confidence", 0.0) or 0.0)
            except (TypeError, ValueError):
                confidence = 0.0
            exploitability = vuln.get("exploitability", "UNKNOWN")

            icon = "🔴" if severity == "HIGH" else "🟡" if severity == "MEDIUM" else "🟢"
            contract = vuln.get("contract") or ""
            function = vuln.get("function") or ""
            line = vuln.get("line")
            location_parts = [p for p in [contract, function] if p]
            if line:
                location_parts.append(f"L{line}")
            location = " / ".join(location_parts) or vuln.get("location", "") or "N/A"

            logger.info("%s %s - %s", icon, severity, category)
            logger.info("   置信度: %.1f%%", confidence * 100)
            logger.info("   可利用性: %s", exploitability)
            logger.info("   位置: %s", location)
            logger.info("   描述: %s", vuln.get("description", "N/A"))
            if vuln.get("recommendation"):
                logger.info("   修复建议: %s", vuln["recommendation"])
            if vuln.get("source"):
                logger.info("   来源: %s", vuln["source"])


def main() -> None:
    parser = argparse.ArgumentParser(
        description="AI 智能合约审计系统",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  # CLI模式 - 审计单个合约
  python src/main.py examples/VulnerableVault.sol

  # CLI模式 - 不生成POC
  python src/main.py examples/VulnerableVault.sol --no-poc

  # Web模式 - 启动Web界面
  python src/main.py --web
  # 或使用: ./run_web.sh
        """,
    )

    parser.add_argument(
        "contract",
        nargs="?",
        help="Solidity合约文件路径",
    )

    parser.add_argument(
        "--web",
        action="store_true",
        help="启动Web界面 (Streamlit)",
    )

    parser.add_argument(
        "--output",
        "-o",
        default="data/results",
        help="审计报告输出目录 (默认: data/results)",
    )

    parser.add_argument(
        "--no-poc",
        action="store_true",
        help="不生成POC脚本",
    )

    parser.add_argument(
        "--no-rules",
        action="store_true",
        help="不使用静态规则检测",
    )

    parser.add_argument(
        "--config",
        "-c",
        default="config.yaml",
        help="配置文件路径 (默认: config.yaml)",
    )

    args = parser.parse_args()

    # Web 模式
    if args.web:
        print("🌐 启动 Web 界面...")
        subprocess.run(["streamlit", "run", "src/web_ui/app.py"], check=True)
        return

    # CLI 模式
    if not args.contract:
        parser.print_help()
        print("\n❌ 错误: 需要提供合约文件路径或使用 --web 启动 Web 界面")
        sys.exit(1)

    contract_path = Path(args.contract)
    if not contract_path.exists():
        print(f"❌ 错误: 文件不存在: {contract_path}")
        sys.exit(1)

    auditor = ContractAuditor(config_path=args.config)
    auditor.audit(
        contract_path=str(contract_path),
        output_dir=args.output,
        generate_poc=not args.no_poc,
        use_rules=not args.no_rules,
        progress_callback=None,
    )

    print(f"\n📄 完整报告输出目录: {args.output}")


if __name__ == "__main__":
    main()
