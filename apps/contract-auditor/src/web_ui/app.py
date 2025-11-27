"""
Streamlit Web 主应用入口

运行方式：
    streamlit run src/web_ui/app.py
或使用项目根目录下的 run_web.sh。
"""
from __future__ import annotations

import sys
import time
from pathlib import Path
from typing import Any, Dict

import streamlit as st

# 确保项目根目录在 sys.path 中，便于导入 src.*
ROOT_DIR = Path(__file__).resolve().parents[2]
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))

from src.utils.config import Config  # noqa: E402
from src.web_ui import components  # noqa: E402
from src.web_ui import state, tasks  # noqa: E402


def _load_default_ui_config() -> Dict[str, Any]:
    """从 config.yaml 加载 UI 默认配置"""
    cfg = Config()
    llm_conf = cfg.llm_config or {}
    return {
        "reasoning_model": llm_conf.get("reasoning_model", ""),
        "coding_model": llm_conf.get("coding_model", ""),
        "max_rounds": llm_conf.get("max_rounds", 27),
        "min_confidence": llm_conf.get("min_confidence", 0.87),
        "temperature": llm_conf.get("temperature", 0.7),
        "max_tokens": llm_conf.get("max_tokens", 4000),
    }


def _ensure_session_state() -> None:
    """初始化 Streamlit 会话级状态"""
    if "current_task_id" not in st.session_state:
        st.session_state.current_task_id = None
    if "ui_config" not in st.session_state:
        st.session_state.ui_config = _load_default_ui_config()
    if "auto_refresh" not in st.session_state:
        st.session_state.auto_refresh = False


def main() -> None:
    st.set_page_config(
        page_title="AI Contract Auditor Web Console",
        layout="wide",
        initial_sidebar_state="expanded",
    )

    _ensure_session_state()

    st.title("🔐 AI 合约审计监控面板")

    # 侧边栏：文件上传与配置
    with st.sidebar:
        st.header("任务配置")
        uploaded_file = components.render_file_uploader()
        ui_config = components.render_config_panel(st.session_state.ui_config)
        st.session_state.ui_config = ui_config

        can_start = uploaded_file is not None
        start_button = st.button("🚀 开始审计", type="primary", disabled=not can_start)

        if start_button and uploaded_file is not None:
            task_id = tasks.start_audit_task(uploaded_file, ui_config)
            st.session_state.current_task_id = task_id
            st.session_state.auto_refresh = True
            st.success(f"已启动审计任务：{uploaded_file.name}")
            st.rerun()

        st.markdown("---")
        st.subheader("历史任务")
        all_tasks = state.list_tasks()
        if all_tasks:
            labels = [f"{t.filename} ({t.status})" for t in all_tasks]
            ids = [t.id for t in all_tasks]
            if st.session_state.current_task_id in ids:
                index = ids.index(st.session_state.current_task_id)
            else:
                index = 0
            selected = st.selectbox("选择任务查看详情", options=list(range(len(ids))), index=index)
            st.session_state.current_task_id = ids[selected]
        else:
            st.caption("暂无历史任务。")

    current_task = (
        state.get_task(st.session_state.current_task_id) if st.session_state.current_task_id else None
    )

    if current_task is None:
        st.info("请在左侧上传合约文件并启动一次审计任务。")
        return

    # 顶部任务概要
    st.markdown(
        f"**当前任务：** `{current_task.filename}` | "
        f"状态：`{current_task.status}` | "
        f"轮次：{current_task.current_round}/{current_task.total_rounds}"
    )

    col_main, col_side = st.columns([2.0, 1.2])
    with col_main:
        components.render_progress_panel(current_task)
    with col_side:
        if current_task.status == "failed" and current_task.error:
            st.error(f"任务失败：{current_task.error}")

        if current_task.report:
            buffers = components.build_export_buffers(current_task)
            st.subheader("📁 审计报告导出")
            st.download_button(
                "下载 JSON 报告",
                data=buffers["json"],
                file_name="audit_report.json",
                mime="application/json",
            )
            st.download_button(
                "下载 Markdown 报告",
                data=buffers["markdown"],
                file_name="audit_report.md",
                mime="text/markdown",
            )
            st.download_button(
                "下载 PDF 报告",
                data=buffers["pdf"],
                file_name="audit_report.pdf",
                mime="application/pdf",
            )

    # 漏洞列表 & 详情
    selected_vuln, _ = components.render_vuln_list_panel(current_task)
    if selected_vuln:
        components.render_vuln_detail(selected_vuln, current_task)

    # 实时刷新：任务运行中时自动轮询更新
    if current_task.status in {"pending", "running"} and st.session_state.auto_refresh:
        cfg = Config()
        refresh_interval = (
            cfg.reporter_config.get("ui", {}).get("refresh_interval", 5)
            if isinstance(cfg.reporter_config, dict)
            else 5
        )
        st.caption(f"任务进行中，将每 {refresh_interval} 秒自动刷新界面。")
        time.sleep(refresh_interval)
        st.rerun()
    else:
        st.session_state.auto_refresh = False


if __name__ == "__main__":
    main()

