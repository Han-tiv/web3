from __future__ import annotations

from pathlib import Path

import streamlit as st

from .session_state import get_app_state, reset_audit_state


def render_sidebar() -> None:
    """渲染应用统一侧边栏。"""
    state = get_app_state()
    audit = state.audit

    with st.sidebar:
        st.markdown("## ⚙️ 配置")
        config_path = st.text_input(
            "配置文件路径",
            value=state.config.config_path,
            help="用于加载 LLM 与审计参数的 YAML 配置文件路径。",
        )
        if config_path != state.config.config_path:
            state.config.config_path = config_path

        output_dir = st.text_input(
            "报告输出目录",
            value=audit.output_dir,
            help="审计报告保存目录，默认为 data/results。",
        )
        if output_dir != audit.output_dir:
            audit.output_dir = output_dir

        st.markdown("## 📊 当前任务")
        if audit.contract_path:
            st.caption(f"目标合约：`{Path(audit.contract_path).name}`")
        else:
            st.caption("暂无正在审计的合约。")

        status = audit.progress.status
        if status == "running":
            st.success("正在审计中...", icon="⚙️")
        elif status == "success":
            st.success("最近一次审计已完成。", icon="✅")
        elif status == "error":
            st.error("最近一次审计失败。", icon="❌")
        else:
            st.info("等待开始新的审计。", icon="⏳")

        if st.button("重置当前审计", use_container_width=True):
            reset_audit_state()
            st.success("已重置当前审计状态。")

        st.markdown("---")
        st.markdown("#### ℹ️ 关于")
        st.caption(
            "本界面基于 Streamlit 构建，用于对 Solidity 合约执行自动化安全审计。"
        )

