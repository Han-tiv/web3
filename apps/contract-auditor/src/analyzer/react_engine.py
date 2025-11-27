"""
ReAct 推理引擎
实现 Reason → Act → Observe 的多轮循环推理逻辑。
"""
from __future__ import annotations

import json
import logging
from dataclasses import dataclass, field, asdict
from typing import Any, Callable, Dict, List, Mapping, Optional, Tuple

from src.utils.llm_client import DualModelSystem

logger = logging.getLogger(__name__)


@dataclass
class VulnerabilityFinding:
    """漏洞检测结果"""

    category: str
    description: str
    severity: str
    exploitability: str
    confidence: float
    round_detected: int
    target: Optional[str] = None
    recommendation: Optional[str] = None
    evidence: Optional[str] = None

    def as_dict(self) -> Dict[str, Any]:
        """转换为可序列化的字典"""
        data = asdict(self)
        # 置信度四舍五入，方便展示
        data["confidence"] = round(self.confidence, 4)
        return data


@dataclass
class Observation:
    """Act 阶段返回的观察结果"""

    action: str
    outcome: str
    evidence: Optional[str] = None
    confidence_delta: float = 0.0

    def as_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class ReActState:
    """ReAct 执行状态"""

    contracts: Mapping[str, Any]
    round: int = 0
    vulns: List[VulnerabilityFinding] = field(default_factory=list)
    confidence: float = 0.0
    history: List[Dict[str, str]] = field(default_factory=list)


# CVSS 风格的漏洞分类 & 默认属性
VULNERABILITY_TAXONOMY: Dict[str, Dict[str, str]] = {
    "权限绕过": {"severity": "high", "exploitability": "高"},
    "重入攻击": {"severity": "high", "exploitability": "高"},
    "精度丢失": {"severity": "medium", "exploitability": "中"},
    "Admin 滥用": {"severity": "medium", "exploitability": "中"},
    "Gas 优化": {"severity": "low", "exploitability": "低"},
    "命名问题": {"severity": "low", "exploitability": "低"},
}

# 漏洞模式关键字，用于 Act 阶段进行静态验证
CATEGORY_PATTERNS: Dict[str, Tuple[str, ...]] = {
    "权限绕过": ("onlyOwner", "AccessControl", "require(msg.sender", "role"),
    "重入攻击": ("call.value", "call{value", "send(", "transfer(", "nonReentrant"),
    "精度丢失": ("div(", "mul(", "10**", "decimals", "basisPoints"),
    "Admin 滥用": ("owner", "admin", "multisig", "timelock"),
    "Gas 优化": ("for (", "while (", "storage", "++i"),
    "命名问题": ("TODO", "FIXME", "typo", "var "),
}

# Prompt 模板（中文要求 & JSON 输出约束）
REASONING_SCHEMA = json.dumps(
    {
        "reasoning": "分析过程",
        "action": "下一步分析动作，例如 `trace transfer()`",
        "target": "关注的函数或变量",
        "category": "漏洞分类，必须是权限绕过/重入攻击/精度丢失/Admin 滥用/Gas 优化/命名问题之一",
        "severity": "high|medium|low",
        "exploitability": "高|中|低",
        "confidence": "0-1 间的小数",
        "evidence_needed": "希望从合约中验证的证据",
        "proposed_fix": "潜在修复建议",
    },
    ensure_ascii=False,
    indent=2,
)

REACT_PROMPT_TEMPLATE = """
你是一名资深 Web3 安全专家，熟悉 Solidity、DeFi 攻击路径与合约审计。
当前任务：分析输入的合约结构，识别潜在漏洞，并规划下一步验证动作。
请遵循 ReAct (Reason → Act → Observe) 流程输出决策，并确保：
1. 仅输出合法 JSON，不添加额外文本
2. 明确漏洞分类（权限绕过/重入攻击/精度丢失/Admin 滥用/Gas 优化/命名问题）
3. 评估 exploitability（高/中/低）与置信度（0-1 浮点数）
4. 同时产出下一步“行动”描述和需要的证据

=== 合约上下文 ===
{contract_summary}

=== 历史记录（最近 5 轮） ===
{history}

=== 已确认/高置信度漏洞 ===
{vuln_summary}

请返回如下 JSON 结构：
{schema}
""".strip()


class ReActEngine:
    """多轮推理引擎（25-27轮）"""

    def __init__(
        self,
        llm: Optional[DualModelSystem],
        max_rounds: Optional[int] = None,
        min_confidence: float = 0.87,
        temperature: float = 0.7,
        max_tokens: int = 4000,
    ):
        self.llm = llm
        self.min_confidence = min_confidence
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.rounds = self._determine_rounds(max_rounds)

    def analyze(
        self,
        contracts: Mapping[str, Any],
        progress_callback: Optional[Callable[[ReActState, Dict[str, Any]], None]] = None,
    ) -> Dict[str, Any]:
        """
        对合约字典执行 ReAct 循环，返回 JSON 化的检测结果。

        Args:
            contracts: 合约名称 -> 解析结果/元数据

        Returns:
            Dict[str, Any]: 最终状态，包含轮次、漏洞列表及全局置信度
        """
        state = ReActState(contracts=contracts)
        for _ in range(self.rounds):
            state.round += 1
            logger.info("ReAct round %s/%s", state.round, self.rounds)

            reasoning = self._reason(state)
            action_result = self._act(state, reasoning)
            observation = self._observe(state, reasoning, action_result)

            history_item = {
                "round": str(state.round),
                "reasoning": reasoning.get("reasoning", ""),
                "action": action_result.get("action", ""),
                "observation": observation.outcome,
            }
            state.history.append(history_item)
            # 控制历史长度，避免 prompt 过大
            state.history = state.history[-5:]

            # 实时回调当前轮次的推理进度，方便外部 UI 展示
            if progress_callback is not None:
                step_snapshot = {
                    "round": state.round,
                    "reasoning": reasoning,
                    "action_result": action_result,
                    "observation": observation.as_dict(),
                    "history_item": history_item,
                }
                try:
                    progress_callback(state, step_snapshot)
                except Exception as exc:  # noqa: BLE001
                    # 进度回调失败不应中断核心审计流程
                    logger.warning("进度回调失败: %s", exc)

        return {
            "rounds": state.round,
            "confidence": round(state.confidence, 4),
            "vulnerabilities": [v.as_dict() for v in state.vulns],
            "history": state.history,
        }

    # ------------------------------------------------------------------ #
    # Reason 阶段
    # ------------------------------------------------------------------ #
    def _reason(self, state: ReActState) -> Dict[str, Any]:
        """调用 LLM 或 fallback 生成下一步推理"""
        prompt = self._build_prompt(state)

        if not self.llm:
            logger.warning("未绑定 LLM，使用启发式 Reason 逻辑")
            return self._heuristic_reason(state)

        try:
            response = self.llm.analyze(
                prompt,
                temperature=self.temperature,
                max_tokens=self.max_tokens,
                top_p=0.95,
            )
            return self.llm.reasoning_client.extract_json(response)
        except Exception as exc:  # noqa: BLE001
            logger.error("Reason 阶段失败，fallback 启发式: %s", exc)
            return self._heuristic_reason(state)

    def _build_prompt(self, state: ReActState) -> str:
        """构造 Prompt，注入上下文与历史"""
        contract_summary = self._summarize_contracts(state.contracts)
        history_text = self._format_history(state.history)
        vuln_summary = self._format_vulnerabilities(state.vulns)

        return REACT_PROMPT_TEMPLATE.format(
            contract_summary=contract_summary or "暂无解析数据",
            history=history_text or "无历史记录，当前为第 1 轮。",
            vuln_summary=vuln_summary or "无",
            schema=REASONING_SCHEMA,
        )

    def _summarize_contracts(self, contracts: Mapping[str, Any]) -> str:
        """提取合约主要结构，减少 Prompt token"""
        summary_lines: List[str] = []
        for idx, (name, contract) in enumerate(contracts.items()):
            if idx >= 3:  # 避免 prompt 爆炸
                summary_lines.append("...其余合约略")
                break

            functions = self._safe_len(contract, "functions")
            modifiers = self._safe_len(contract, "modifiers")
            inherits = self._safe_get(contract, "inherits")
            summary_lines.append(
                f"- {name}: 函数 {functions} 个，修饰符 {modifiers} 个，继承 {inherits or '无'}"
            )

        return "\n".join(summary_lines)

    def _format_history(self, history: List[Dict[str, str]]) -> str:
        """历史记录格式化"""
        lines = []
        for item in history:
            lines.append(
                f"Round {item.get('round')}: "
                f"Reason={item.get('reasoning','')} | "
                f"Action={item.get('action','')} | "
                f"Observe={item.get('observation','')}"
            )
        return "\n".join(lines)

    def _format_vulnerabilities(self, vulns: List[VulnerabilityFinding]) -> str:
        """已确认漏洞摘要"""
        parts = []
        for vuln in vulns[-3:]:
            parts.append(
                f"[{vuln.category}] conf={vuln.confidence:.2f} target={vuln.target or 'N/A'}"
            )
        return "\n".join(parts)

    def _heuristic_reason(self, state: ReActState) -> Dict[str, Any]:
        """无 LLM 场景下的简单推理，确保 ReAct 循环仍可运行"""
        categories = list(VULNERABILITY_TAXONOMY.keys())
        category = categories[(state.round - 1) % len(categories)]
        template_action = {
            "权限绕过": "trace access control modifiers",
            "重入攻击": "inspect payable functions with external calls",
            "精度丢失": "verify arithmetic precision handling",
            "Admin 滥用": "check privileged functions and owner-only flows",
            "Gas 优化": "review loops and storage writes",
            "命名问题": "review event/function naming clarity",
        }.get(category, "scan contract")

        confidence = min(0.6 + state.round * 0.01, 0.9)
        return {
            "reasoning": f"Round {state.round} fallback reasoning for {category}",
            "action": template_action,
            "target": None,
            "category": category,
            "severity": VULNERABILITY_TAXONOMY[category]["severity"],
            "exploitability": VULNERABILITY_TAXONOMY[category]["exploitability"],
            "confidence": confidence,
            "evidence_needed": "Identify concrete source lines supporting the suspicion",
            "proposed_fix": "Introduce strict role checks or guards where missing.",
        }

    # ------------------------------------------------------------------ #
    # Act 阶段
    # ------------------------------------------------------------------ #
    def _act(self, state: ReActState, reasoning: Dict[str, Any]) -> Dict[str, Any]:
        """依据 Reason 输出执行静态验证，返回证据与打分"""
        category = reasoning.get("category") or self._guess_category(reasoning.get("action", ""))
        category = category if category in VULNERABILITY_TAXONOMY else "权限绕过"
        severity = reasoning.get("severity") or VULNERABILITY_TAXONOMY[category]["severity"]

        sources = self._collect_sources(state.contracts)
        target = reasoning.get("target")
        evidence, score = self._scan_patterns(category, sources, target)
        evidence = evidence or "暂未命中明显证据"
        action_name = reasoning.get("action") or f"inspect {category}"

        # 将静态检测得分转换为置信度校准值
        base_conf = float(reasoning.get("confidence", 0.0))
        calibrated_conf = min(1.0, base_conf * 0.6 + 0.3 * score + 0.1)

        exploitability = reasoning.get("exploitability") or VULNERABILITY_TAXONOMY[category]["exploitability"]
        exploitability = self._calibrate_exploitability(exploitability, score)

        return {
            "action": action_name,
            "category": category,
            "severity": severity,
            "target": target,
            "evidence": evidence,
            "evidence_score": score,
            "confidence": calibrated_conf,
            "exploitability": exploitability,
            "description": reasoning.get("reasoning", f"{category} suspicion"),
            "proposed_fix": reasoning.get("proposed_fix"),
        }

    def _collect_sources(self, contracts: Mapping[str, Any]) -> Dict[str, str]:
        """收集所有合约源码，返回 name -> source 字典"""
        collected: Dict[str, str] = {}
        for name, contract in contracts.items():
            source = self._safe_get(contract, "source_code") or self._safe_get(contract, "source") or ""
            if not source and isinstance(contract, str):
                source = contract
            collected[name] = str(source)
        return collected

    def _scan_patterns(
        self,
        category: str,
        sources: Dict[str, str],
        target: Optional[str],
    ) -> Tuple[str, float]:
        """根据关键字粗略匹配证据，提高多轮验证可靠性"""
        keywords = CATEGORY_PATTERNS.get(category, ())
        if not keywords:
            return "", 0.0

        hit_snippets: List[str] = []
        for contract_name, source in sources.items():
            lowered = source.lower()
            for keyword in keywords:
                idx = lowered.find(keyword.lower())
                if idx == -1:
                    continue

                snippet = self._extract_context(source, idx)
                prefix = f"{contract_name}:{keyword}"
                hit_snippets.append(f"{prefix} => {snippet}")

        score = min(1.0, len(hit_snippets) / max(1, len(keywords) // 2 or 1))

        if target:
            target_lower = target.lower()
            target_hits = [snippet for snippet in hit_snippets if target_lower in snippet.lower()]
            if target_hits:
                return target_hits[0], score

        return (hit_snippets[0] if hit_snippets else ""), score

    def _extract_context(self, source: str, idx: int, window: int = 80) -> str:
        """截取关键字附近上下文"""
        start = max(0, idx - window)
        end = min(len(source), idx + window)
        snippet = source[start:end].replace("\n", " ")
        return snippet.strip()

    def _calibrate_exploitability(self, base: str, evidence_score: float) -> str:
        """依据静态证据调整 exploitability"""
        base_level = base
        if evidence_score > 0.75:
            return "高"
        if evidence_score > 0.45 and base_level != "低":
            return "中"
        return base_level

    def _guess_category(self, action: str) -> str:
        """通过 action 文本猜测分类"""
        action = action.lower()
        mapping = {
            "reentrancy": "重入攻击",
            "permission": "权限绕过",
            "access": "权限绕过",
            "precision": "精度丢失",
            "admin": "Admin 滥用",
            "owner": "Admin 滥用",
            "gas": "Gas 优化",
            "naming": "命名问题",
        }
        for keyword, category in mapping.items():
            if keyword in action:
                return category
        return "权限绕过"

    # ------------------------------------------------------------------ #
    # Observe 阶段
    # ------------------------------------------------------------------ #
    def _observe(
        self,
        state: ReActState,
        reasoning: Dict[str, Any],
        action_result: Dict[str, Any],
    ) -> Observation:
        """根据 Act 结果更新状态、过滤置信度"""
        confidence = float(action_result.get("confidence", 0.0))
        category = action_result["category"]
        description = action_result.get("description", "")
        evidence = action_result.get("evidence")
        target = action_result.get("target")
        exploitability = action_result.get("exploitability", "中")
        severity = action_result["severity"]

        if confidence >= self.min_confidence:
            vuln = VulnerabilityFinding(
                category=category,
                description=description or f"Potential {category}",
                severity=severity,
                exploitability=exploitability,
                confidence=confidence,
                round_detected=state.round,
                target=target,
                recommendation=action_result.get("proposed_fix"),
                evidence=evidence,
            )
            recorded = self._record_vulnerability(state, vuln)
            outcome = f"记录漏洞 {recorded.category} (conf={recorded.confidence:.2f})"
            state.confidence = max(state.confidence, recorded.confidence)
            confidence_delta = recorded.confidence - confidence
        else:
            outcome = f"置信度 {confidence:.2f} < 阈值 {self.min_confidence}"
            confidence_delta = 0.0

        return Observation(
            action=action_result["action"],
            outcome=outcome,
            evidence=evidence,
            confidence_delta=confidence_delta,
        )

    def _record_vulnerability(
        self,
        state: ReActState,
        candidate: VulnerabilityFinding,
    ) -> VulnerabilityFinding:
        """多轮验证：重复命中则提振置信度，减少误报"""
        for existing in state.vulns:
            same_target = (
                existing.target == candidate.target
                or (existing.target and candidate.target and existing.target.lower() in candidate.target.lower())
            )
            if existing.category == candidate.category and same_target:
                existing.confidence = self._merge_confidence(existing.confidence, candidate.confidence)
                existing.round_detected = min(existing.round_detected, candidate.round_detected)
                # 累积证据
                if candidate.evidence and candidate.evidence not in (existing.evidence or ""):
                    existing.evidence = f"{existing.evidence or ''} || {candidate.evidence}"
                if candidate.recommendation:
                    existing.recommendation = candidate.recommendation
                return existing

        state.vulns.append(candidate)
        return candidate

    def _merge_confidence(self, current: float, new_value: float) -> float:
        """多轮融合置信度 -> 逐渐接近 1"""
        merged = min(1.0, current + (new_value - current) * 0.6 + 0.05)
        return merged

    # ------------------------------------------------------------------ #
    # 实用函数
    # ------------------------------------------------------------------ #
    def _determine_rounds(self, max_rounds: Optional[int]) -> int:
        """确保轮次落在 25-27 之间"""
        if max_rounds is None:
            return 27
        if max_rounds < 25:
            logger.warning("max_rounds=%s 小于要求，自动拉升至 25", max_rounds)
            return 25
        if max_rounds > 27:
            logger.warning("max_rounds=%s 超出要求，自动收敛至 27", max_rounds)
            return 27
        return max_rounds

    def _safe_get(self, contract: Any, attr: str) -> Any:
        """兼容 dataclass / dict 的属性访问"""
        if isinstance(contract, Mapping):
            return contract.get(attr)
        return getattr(contract, attr, None)

    def _safe_len(self, contract: Any, attr: str) -> int:
        """安全获取属性长度"""
        value = self._safe_get(contract, attr)
        if isinstance(value, (list, tuple, set)):
            return len(value)
        if isinstance(value, dict):
            return len(value)
        return 0


__all__ = [
    "ReActEngine",
    "ReActState",
    "VulnerabilityFinding",
    "Observation",
]
