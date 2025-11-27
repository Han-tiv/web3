from __future__ import annotations

from datetime import datetime
from typing import Any, Callable, Dict

import streamlit as st

from src.analyzer.react_engine import ReActEngine

from .session_state import get_app_state
from .styling import render_severity_badge


def create_progress_callback(
    engine: ReActEngine,
) -> Callable[[Any, Dict[str, Any]], None]:
    """
    基于给定的 ReActEngine 创建进度回调函数。

    注意：ReActEngine.analyze 在每轮结束时会调用

        progress_callback(state, step_snapshot)

    其中 step_snapshot 是包含 reasoning/action_result/observation 等信息的字典。
    此函数将这些信息汇总后写入 session_state，供前端实时展示。
    """
    state = get_app_state()
    audit = state.audit
    progress = audit.progress
    progress.status = "running"
    if progress.start_time is None:
        progress.start_time = datetime.now()
    progress.total_rounds = getattr(engine, "rounds", 0)

    def _callback(react_state: Any, step_snapshot: Dict[str, Any]) -> None:
        progress.current_round = int(
            step_snapshot.get("round", getattr(react_state, "round", 0))
        )
        vulns = getattr(react_state, "vulns", []) or []
        progress.last_vuln_count = len(vulns)

        reasoning = step_snapshot.get("reasoning") or {}
        action_result = step_snapshot.get("action_result") or {}
        observation = step_snapshot.get("observation") or {}

        category = action_result.get("category") or "未知分类"
        severity = str(action_result.get("severity", "") or "").upper()
        try:
            confidence = float(action_result.get("confidence", 0.0) or 0.0)
        except (TypeError, ValueError):
            confidence = 0.0

        progress.last_message = (
            f"第 {progress.current_round}/{progress.total_rounds} 轮 · "
            f"{category} · 置信度 {confidence:.2f}"
        )

        item = {
            "round": progress.current_round,
            "category": category,
            "severity": severity,
            "confidence": confidence,
            "reasoning": reasoning.get("reasoning", ""),
            "action": action_result.get("action", ""),
            "observation": observation.get("outcome", ""),
        }
        progress.history.append(item)
        # 限制历史长度，避免 session_state 中数据过大
        max_len = 40
        if len(progress.history) > max_len:
            progress.history = progress.history[-max_len:]

    return _callback


def render_audit_progress(expanded: bool = True) -> None:
    """在页面上渲染当前审计的进度组件。"""
    state = get_app_state()
    progress = state.audit.progress

    with st.expander("🔄 实时推理进度", expanded=expanded):
        status_label = {
            "idle": "等待开始",
            "running": "推理进行中",
            "success": "审计完成",
            "error": "审计失败",
        }.get(progress.status, "未知状态")

        st.write(f"当前状态：**{status_label}**")

        total = max(progress.total_rounds, 1)
        current = max(0, min(progress.current_round, total))
        percent = int(current / total * 100)
        st.progress(percent)

        if progress.last_message:
            st.markdown(
                f"<p class='small-muted'>{progress.last_message}</p>",
                unsafe_allow_html=True,
            )

        if progress.history:
            st.markdown("**最近推理历史：**")
            # 只展示最近 5 条，按时间倒序
            for item in progress.history[-5:][::-1]:
                badge_html = render_severity_badge(item.get("severity", "UNKNOWN"))
                st.markdown(
                    (
                        "<div class='vuln-item'>"
                        f"{badge_html} 第 {item.get('round')} 轮 · "
                        f"{item.get('category') or '未知'}"
                        "</div>"
                    ),
                    unsafe_allow_html=True,
                )
                if item.get("reasoning"):
                    st.caption(item["reasoning"])
