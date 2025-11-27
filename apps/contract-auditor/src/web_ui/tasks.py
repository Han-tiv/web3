"""
异步任务管理模块

通过后台线程执行合约审计，并将进度写入 `state.AuditTaskState`。
"""
from __future__ import annotations

import logging
import threading
from dataclasses import asdict
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

from src.analyzer.react_engine import ReActEngine, VulnerabilityFinding
from src.parser.solidity_parser import SolidityParser
from src.utils.config import Config
from src.utils.llm_client import DualModelSystem

from . import state

logger = logging.getLogger(__name__)


_THREADS: Dict[str, threading.Thread] = {}
_THREADS_LOCK = threading.Lock()


def _ensure_parent_dir(path: Path) -> None:
    """确保文件父目录存在"""
    path.parent.mkdir(parents=True, exist_ok=True)


def _save_uploaded_file(upload_dir: Path, filename: str, content: bytes) -> Path:
    """将上传文件写入磁盘并返回路径"""
    _ensure_parent_dir(upload_dir)
    target = upload_dir / filename
    target.write_bytes(content)
    return target


def _build_llm_system(config_obj: Config, overrides: Dict[str, Any]) -> DualModelSystem:
    """根据配置和 UI 重载构建 DualModelSystem"""
    llm_conf = dict(config_obj.llm_config or {})
    llm_conf.update({k: v for k, v in overrides.items() if v is not None})

    reasoning_model = llm_conf.get("reasoning_model")
    coding_model = llm_conf.get("coding_model")
    api_url = llm_conf.get("api_url")
    if not reasoning_model or not coding_model or not api_url:
        raise ValueError("LLM 配置不完整，请检查 config.yaml 中的 llm 部分")

    return DualModelSystem(
        reasoning_model=reasoning_model,
        coding_model=coding_model,
        api_url=api_url,
    )


def _build_report(
    contract_path: Path,
    contracts: Dict[str, Any],
    vulnerabilities: List[Dict[str, Any]],
    duration_seconds: float,
    tokens_used: int,
) -> Dict[str, Any]:
    """根据审计结果构建标准化报告结构"""
    high = [v for v in vulnerabilities if state.severity_bucket(v.get("severity")) == "HIGH"]
    medium = [v for v in vulnerabilities if state.severity_bucket(v.get("severity")) == "MEDIUM"]
    low = [v for v in vulnerabilities if state.severity_bucket(v.get("severity")) == "LOW"]

    return {
        "metadata": {
            "auditor": "AI Contract Auditor",
            "version": "1.0.0",
            "contract_path": str(contract_path),
            "audit_date": datetime.utcnow().isoformat(),
            "duration_seconds": duration_seconds,
            "tokens_used": tokens_used,
        },
        "summary": {
            "total_contracts": len(contracts),
            "total_vulnerabilities": len(vulnerabilities),
            "high_risk": len(high),
            "medium_risk": len(medium),
            "low_risk": len(low),
        },
        "contracts": list(contracts.keys()),
        "vulnerabilities": vulnerabilities,
    }


def _enrich_vulnerabilities(
    vulns: List[Dict[str, Any]],
    contract_sources: Dict[str, str],
) -> List[Dict[str, Any]]:
    """
    为漏洞结果补充 contract_name / line_number / function_name 等 UI 友好字段。

    由于 ReAct 引擎目前只在 evidence 中给出 `contract:keyword`，这里做启发式解析。
    """
    enriched: List[Dict[str, Any]] = []

    for v in vulns:
        evidence = v.get("evidence") or ""
        contract_name = None
        keyword = None

        # evidence 形如 `MyContract:onlyOwner => ...`
        if ":" in evidence:
            prefix, _ = evidence.split(":", 1)
            contract_name = prefix.strip()
            if "=>" in evidence:
                _, rest = evidence.split(":", 1)
                keyword = rest.split("=>", 1)[0].strip()

        if not contract_name and contract_sources:
            # 回退：取第一个合约名
            contract_name = next(iter(contract_sources.keys()))

        source = contract_sources.get(contract_name or "", "")
        line_number: Optional[int] = None
        if source and keyword:
            lowered = source.lower()
            idx = lowered.find(keyword.lower())
            if idx != -1:
                # 通过字符偏移估算行号
                line_number = source[:idx].count("\n") + 1

        function_name: Optional[str] = None
        target = v.get("target")
        if isinstance(target, str) and target:
            function_name = target.strip()

        enriched_vuln = dict(v)
        enriched_vuln["contract_name"] = contract_name
        enriched_vuln["line_number"] = line_number
        enriched_vuln["function_name"] = function_name
        enriched.append(enriched_vuln)

    return enriched


def start_audit_task(
    uploaded_file,
    ui_config: Dict[str, Any],
    upload_root: Path | None = None,
) -> str:
    """
    启动后台审计任务。

    Args:
        uploaded_file: Streamlit UploadedFile 对象
        ui_config: 来自配置面板的参数（模型、轮次、置信度等）
        upload_root: 上传目录根路径，默认 data/uploads

    Returns:
        task_id: 新任务 ID
    """
    if upload_root is None:
        upload_root = Path("data/uploads")

    filename = uploaded_file.name
    content = uploaded_file.getvalue()
    stored_path = _save_uploaded_file(upload_root, filename, content)

    base_config = Config()
    llm_conf = dict(base_config.llm_config or {})
    total_rounds = int(ui_config.get("max_rounds") or llm_conf.get("max_rounds") or 27)

    task = state.create_task(filename=filename, contract_path=stored_path, total_rounds=total_rounds)
    task_id = task.id

    def worker() -> None:
        """后台执行审计的线程函数"""
        from time import monotonic

        logger.info("开始审计任务 %s: %s", task_id, stored_path)
        state.update_task_status(task_id, "running", "解析合约中...")

        start_ts = monotonic()
        try:
            parser = SolidityParser()
            contracts = parser.parse_file(str(stored_path))

            # 记录源码，用于 UI 高亮
            contract_sources: Dict[str, str] = {}
            for name, contract in contracts.items():
                source = getattr(contract, "source_code", None) or getattr(contract, "source", None) or ""
                contract_sources[name] = str(source)

            state.update_task_status(task_id, "running", "执行多轮 ReAct 推理中...")

            # 构建 LLM 系统与推理引擎
            llm_overrides = {
                "reasoning_model": ui_config.get("reasoning_model"),
                "coding_model": ui_config.get("coding_model"),
                "max_rounds": ui_config.get("max_rounds"),
                "min_confidence": ui_config.get("min_confidence"),
                "temperature": ui_config.get("temperature"),
                "max_tokens": ui_config.get("max_tokens"),
            }
            llm_system = _build_llm_system(base_config, llm_overrides)

            min_confidence = float(
                ui_config.get("min_confidence") or llm_conf.get("min_confidence") or 0.87
            )
            temperature = float(ui_config.get("temperature") or llm_conf.get("temperature") or 0.7)
            max_tokens = int(ui_config.get("max_tokens") or llm_conf.get("max_tokens") or 4000)

            react_engine = ReActEngine(
                llm=llm_system,
                max_rounds=total_rounds,
                min_confidence=min_confidence,
                temperature=temperature,
                max_tokens=max_tokens,
            )

            def on_progress(react_state, step: Dict[str, Any]) -> None:
                """接收 ReActEngine 每轮推理的进度"""
                history_item = step.get("history_item") or {
                    "round": str(step.get("round")),
                    "reasoning": (step.get("reasoning") or {}).get("reasoning", ""),
                    "action": (step.get("action_result") or {}).get("action", ""),
                    "observation": (step.get("observation") or {}).get("outcome", ""),
                }
                state.append_history(task_id, history_item)

                # 记录当前轮的置信度快照
                vulns_snapshot: List[Dict[str, Any]] = [
                    asdict(v) if isinstance(v, VulnerabilityFinding) else dict(v)
                    for v in getattr(react_state, "vulns", [])
                ]
                state.record_round_snapshot(task_id, int(step.get("round") or react_state.round), vulns_snapshot)

            analysis_result = react_engine.analyze(
                contracts=contracts,
                progress_callback=on_progress,
            )

            raw_vulns: List[Dict[str, Any]] = analysis_result.get("vulnerabilities", [])
            enriched_vulns = _enrich_vulnerabilities(raw_vulns, contract_sources)

            duration = monotonic() - start_ts
            report = _build_report(
                contract_path=stored_path,
                contracts=contracts,
                vulnerabilities=enriched_vulns,
                duration_seconds=duration,
                tokens_used=llm_system.total_tokens,
            )

            state.finalize_task_success(
                task_id=task_id,
                report=report,
                vulnerabilities=enriched_vulns,
                contracts=contracts,
                contract_sources=contract_sources,
            )

            logger.info("审计任务 %s 完成，用时 %.1f 秒", task_id, duration)
        except Exception as exc:  # noqa: BLE001
            logger.exception("审计任务 %s 失败: %s", task_id, exc)
            state.finalize_task_error(task_id, f"审计失败: {exc}")

    thread = threading.Thread(target=worker, name=f"audit-{task_id}", daemon=True)
    with _THREADS_LOCK:
        _THREADS[task_id] = thread
    thread.start()

    return task_id


def get_thread_status(task_id: str) -> Optional[bool]:
    """
    查询后台线程是否仍然存活。

    Returns:
        True: 线程还在运行
        False: 线程已结束
        None: 找不到对应线程
    """
    with _THREADS_LOCK:
        thread = _THREADS.get(task_id)
    if thread is None:
        return None
    return thread.is_alive()

