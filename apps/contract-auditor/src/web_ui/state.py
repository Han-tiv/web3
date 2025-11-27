"""
Web UI 状态管理模块

负责：
- 审计任务的生命周期管理
- 多轮推理进度与历史记录
- 漏洞结果及置信度曲线缓存

该模块与 Streamlit 解耦，可在后台线程安全更新。
"""
from __future__ import annotations

from dataclasses import dataclass, field, asdict
from datetime import datetime
from pathlib import Path
from threading import Lock
from typing import Any, Dict, List, Optional
from uuid import uuid4


@dataclass
class AuditTaskState:
    """单次审计任务的状态"""

    id: str
    filename: str
    contract_path: Path
    status: str  # pending | running | completed | failed
    created_at: datetime
    updated_at: datetime
    total_rounds: int
    current_round: int = 0
    message: str = ""

    # 推理历史（用于 UI 展示实时 Reason / Act / Observe）
    history: List[Dict[str, Any]] = field(default_factory=list)

    # 最终报告与漏洞结果
    report: Optional[Dict[str, Any]] = None
    vulnerabilities: List[Dict[str, Any]] = field(default_factory=list)

    # 合约解析结果与源码，用于定位行号和高亮
    contracts: Dict[str, Any] = field(default_factory=dict)
    contract_sources: Dict[str, str] = field(default_factory=dict)

    # 置信度曲线：key -> [{"round": int, "confidence": float}]
    confidence_series: Dict[str, List[Dict[str, Any]]] = field(default_factory=dict)

    # 错误信息 / 元数据
    error: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """用于调试或导出 JSON 的视图"""
        data = asdict(self)
        data["contract_path"] = str(self.contract_path)
        data["created_at"] = self.created_at.isoformat()
        data["updated_at"] = self.updated_at.isoformat()
        return data


# 全局任务仓库（进程内共享），结合锁保证线程安全
_TASKS: Dict[str, AuditTaskState] = {}
_LOCK = Lock()


def build_vuln_key(category: str, target: Optional[str]) -> str:
    """
    构造漏洞在置信度曲线里的键值。

    使用 category + target 保证同类漏洞按目标区分。
    """
    category = (category or "").strip()
    target = (target or "").strip()
    return f"{category}::{target}"


def create_task(filename: str, contract_path: Path, total_rounds: int) -> AuditTaskState:
    """创建新的审计任务并放入全局任务表"""
    task_id = uuid4().hex
    now = datetime.utcnow()
    task = AuditTaskState(
        id=task_id,
        filename=filename,
        contract_path=contract_path,
        status="pending",
        created_at=now,
        updated_at=now,
        total_rounds=total_rounds,
    )
    with _LOCK:
        _TASKS[task_id] = task
    return task


def get_task(task_id: str) -> Optional[AuditTaskState]:
    """按 ID 获取任务"""
    with _LOCK:
        return _TASKS.get(task_id)


def list_tasks() -> List[AuditTaskState]:
    """按创建时间倒序列出所有任务"""
    with _LOCK:
        return sorted(_TASKS.values(), key=lambda t: t.created_at, reverse=True)


def update_task_status(task_id: str, status: str, message: str | None = None) -> None:
    """更新任务状态与提示信息"""
    with _LOCK:
        task = _TASKS.get(task_id)
        if not task:
            return
        task.status = status
        if message is not None:
            task.message = message
        task.updated_at = datetime.utcnow()


def append_history(
    task_id: str,
    history_item: Dict[str, Any],
) -> None:
    """追加一条 ReAct 历史记录"""
    with _LOCK:
        task = _TASKS.get(task_id)
        if not task:
            return
        task.history.append(history_item)
        # 限制历史长度，避免占用过多内存
        max_len = 300
        if len(task.history) > max_len:
            task.history = task.history[-max_len:]
        task.updated_at = datetime.utcnow()


def record_round_snapshot(
    task_id: str,
    round_index: int,
    vulns_snapshot: List[Dict[str, Any]],
) -> None:
    """
    根据当前轮次的漏洞快照记录置信度曲线。

    Args:
        task_id: 任务 ID
        round_index: 当前轮次
        vulns_snapshot: 本轮次时刻的漏洞列表（字典形式）
    """
    with _LOCK:
        task = _TASKS.get(task_id)
        if not task:
            return
        task.current_round = max(task.current_round, round_index)
        for vuln in vulns_snapshot:
            key = build_vuln_key(vuln.get("category", ""), vuln.get("target"))
            series = task.confidence_series.setdefault(key, [])
            # 避免同一轮重复记录
            if series and series[-1]["round"] == round_index:
                series[-1]["confidence"] = vuln.get("confidence", 0.0)
            else:
                series.append(
                    {
                        "round": round_index,
                        "confidence": vuln.get("confidence", 0.0),
                    }
                )
        task.updated_at = datetime.utcnow()


def finalize_task_success(
    task_id: str,
    report: Dict[str, Any],
    vulnerabilities: List[Dict[str, Any]],
    contracts: Dict[str, Any],
    contract_sources: Dict[str, str],
) -> None:
    """审计成功收尾：写入报告与漏洞结果"""
    with _LOCK:
        task = _TASKS.get(task_id)
        if not task:
            return
        task.report = report
        task.vulnerabilities = vulnerabilities
        task.contracts = contracts
        task.contract_sources = contract_sources
        task.status = "completed"
        task.message = "审计完成"
        task.updated_at = datetime.utcnow()


def finalize_task_error(task_id: str, error_message: str) -> None:
    """审计失败收尾"""
    with _LOCK:
        task = _TASKS.get(task_id)
        if not task:
            return
        task.status = "failed"
        task.error = error_message
        task.message = error_message
        task.updated_at = datetime.utcnow()


def severity_bucket(severity: str) -> str:
    """
    将任意形式的 severity 映射到 HIGH / MEDIUM / LOW / INFO 四档。
    """
    if not severity:
        return "INFO"
    s = severity.strip().upper()
    if s in {"HIGH", "H"}:
        return "HIGH"
    if s in {"MEDIUM", "M"}:
        return "MEDIUM"
    if s in {"LOW", "L"}:
        return "LOW"
    return "INFO"


def aggregate_severity_counts(vulns: List[Dict[str, Any]]) -> Dict[str, int]:
    """统计不同严重等级的数量"""
    counts = {"HIGH": 0, "MEDIUM": 0, "LOW": 0, "INFO": 0}
    for v in vulns:
        bucket = severity_bucket(v.get("severity", ""))
        counts[bucket] = counts.get(bucket, 0) + 1
    return counts


