from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, Dict, List, Optional

import streamlit as st

APP_STATE_KEY = "contract_auditor_app_state"


@dataclass
class AuditProgress:
    """单次审计的进度状态，存放于 session_state 中。"""

    status: str = "idle"  # idle | running | success | error
    start_time: Optional[datetime] = None
    end_time: Optional[datetime] = None
    current_round: int = 0
    total_rounds: int = 0
    last_message: str = ""
    history: List[Dict[str, Any]] = field(default_factory=list)
    last_vuln_count: int = 0


@dataclass
class AuditSession:
    """当前审计任务的会话数据。"""

    contract_path: Optional[str] = None
    uploaded_contract_name: Optional[str] = None
    output_dir: str = "data/results"
    report: Optional[Dict[str, Any]] = None
    analysis_result: Optional[Dict[str, Any]] = None
    progress: AuditProgress = field(default_factory=AuditProgress)


@dataclass
class WebConfig:
    """Web 端配置（仅在 UI 内生效，不直接写回 config.yaml）。"""

    config_path: str = "config.yaml"
    auto_save_report: bool = True


@dataclass
class HistoryFilter:
    """历史记录页的过滤条件。"""

    severity: str = "ALL"
    search_text: str = ""


@dataclass
class AppState:
    """应用整体的 SessionState 根对象。"""

    config: WebConfig = field(default_factory=WebConfig)
    audit: AuditSession = field(default_factory=AuditSession)
    history_filter: HistoryFilter = field(default_factory=HistoryFilter)


def init_app_state() -> AppState:
    """确保 session_state 中已初始化 AppState 并返回。"""
    if APP_STATE_KEY not in st.session_state:
        st.session_state[APP_STATE_KEY] = AppState()
    return st.session_state[APP_STATE_KEY]


def get_app_state() -> AppState:
    """获取当前 AppState。"""
    return init_app_state()


def reset_audit_state() -> None:
    """重置当前审计会话，但保留全局配置与历史过滤条件。"""
    state = get_app_state()
    state.audit = AuditSession()

