from __future__ import annotations

import streamlit as st

SEVERITY_COLORS = {
    "CRITICAL": "#d32f2f",
    "HIGH": "#f44336",
    "MEDIUM": "#ffa000",
    "LOW": "#43a047",
    "INFO": "#1976d2",
    "UNKNOWN": "#9e9e9e",
}


def apply_global_styles() -> None:
    """注入全局 CSS，统一应用样式。"""
    st.markdown(
        """
        <style>
        /* 页面整体布局优化 */
        .main > div {
            padding-top: 1rem;
            padding-bottom: 2rem;
        }

        /* 标题间距 */
        h1, h2, h3 {
            margin-bottom: 0.4rem;
        }

        .small-muted {
            font-size: 0.85rem;
            color: #666666;
        }

        .severity-badge {
            display: inline-block;
            padding: 0.15rem 0.55rem;
            border-radius: 0.8rem;
            font-size: 0.8rem;
            font-weight: 600;
            color: #ffffff;
        }

        .vuln-item {
            border-radius: 0.5rem;
            padding: 0.4rem 0.6rem;
            background-color: #111827;
        }

        .vuln-item + .vuln-item {
            margin-top: 0.4rem;
        }
        </style>
        """,
        unsafe_allow_html=True,
    )


def render_severity_badge(severity: str) -> str:
    """返回标记严重级别的 HTML 片段。"""
    sev = (severity or "UNKNOWN").upper()
    if sev not in SEVERITY_COLORS:
        mapping = {
            "H": "HIGH",
            "M": "MEDIUM",
            "L": "LOW",
        }
        sev = mapping.get(sev, sev)
    if sev not in SEVERITY_COLORS:
        sev = "UNKNOWN"
    color = SEVERITY_COLORS.get(sev, SEVERITY_COLORS["UNKNOWN"])
    return (
        f'<span class="severity-badge" '
        f'style="background-color:{color}">{sev}</span>'
    )

