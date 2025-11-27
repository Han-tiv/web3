from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, List

import streamlit as st

from ..session_state import get_app_state, init_app_state
from ..sidebar import render_sidebar
from ..styling import apply_global_styles
from ..vulnerability_list import render_vulnerability_list


def _load_reports(base_dir: Path) -> List[Dict[str, Any]]:
    """从指定目录加载所有审计报告。"""
    reports: List[Dict[str, Any]] = []
    if not base_dir.exists():
        return reports

    for path in sorted(base_dir.glob("audit_report_*.json"), reverse=True):
        try:
            with path.open("r", encoding="utf-8") as f:
                data = json.load(f)
            data["_file_path"] = str(path)
            reports.append(data)
        except Exception:
            # 历史文件损坏时忽略该条
            continue
    return reports


def main() -> None:
    init_app_state()
    state = get_app_state()

    apply_global_styles()
    render_sidebar()

    st.title("📚 历史审计记录")

    base_dir = Path(state.audit.output_dir or "data/results")
    reports = _load_reports(base_dir)
    if not reports:
        st.info("当前尚无历史审计报告。运行一次审计后将自动在此展示记录。")
        return

    options: List[str] = []
    for idx, rpt in enumerate(reports):
        metadata = rpt.get("metadata", {})
        summary = rpt.get("summary", {})
        label = (
            f"{idx + 1}. {metadata.get('audit_date', '未知时间')} · "
            f"{Path(metadata.get('contract_path', '')).name or '未知合约'} · "
            f"{summary.get('total_vulnerabilities', 0)} 个漏洞"
        )
        options.append(label)

    selected_label = st.selectbox("选择要查看的审计记录", options)
    selected_index = options.index(selected_label)
    selected = reports[selected_index]

    metadata = selected.get("metadata", {})
    summary = selected.get("summary", {})

    st.markdown("### 概览")
    col1, col2, col3, col4 = st.columns(4)
    col1.metric("合约数量", summary.get("total_contracts", 0))
    col2.metric("总漏洞数", summary.get("total_vulnerabilities", 0))
    col3.metric("高危", summary.get("high_risk", 0))
    col4.metric(
        "中/低危",
        f"{summary.get('medium_risk', 0)} / {summary.get('low_risk', 0)}",
    )

    st.caption(
        f"审计时间：{metadata.get('audit_date', '未知')} · "
        f"报告文件：`{Path(selected.get('_file_path')).name}`"
    )

    st.markdown("### 🛡️ 漏洞列表")
    render_vulnerability_list(selected.get("vulnerabilities", []))


if __name__ == "__main__":
    main()
