from __future__ import annotations

import streamlit as st

from .session_state import init_app_state
from .sidebar import render_sidebar
from .styling import apply_global_styles


def main() -> None:
    st.set_page_config(
        page_title="AI Contract Auditor",
        page_icon="🛡️",
        layout="wide",
        initial_sidebar_state="expanded",
    )

    apply_global_styles()
    init_app_state()
    render_sidebar()

    st.title("🛡️ AI 合约审计器 Web 界面")
    st.write("从左侧导航进入不同页面完成合约审计、查看历史记录与调整设置。")
    st.markdown(
        "- **审计**：上传或选择合约文件，实时查看 ReAct 推理进度与漏洞列表。\n"
        "- **历史记录**：浏览之前生成的审计报告。\n"
        "- **设置**：查看并调整 LLM 与审计参数（仅作用于当前服务）。"
    )


if __name__ == "__main__":
    main()

