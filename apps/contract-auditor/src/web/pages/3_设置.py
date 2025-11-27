from __future__ import annotations

from pathlib import Path
from typing import Any, Dict

import streamlit as st

from src.utils.config import Config

from ..session_state import get_app_state, init_app_state
from ..sidebar import render_sidebar
from ..styling import apply_global_styles


def _load_config(config_path: str) -> Dict[str, Any]:
    """使用项目内 Config 类加载配置，返回原始字典。"""
    cfg = Config(config_path)
    # Config 内部将完整 YAML 保存在 _config 字段中
    return getattr(cfg, "_config", {})  # type: ignore[no-any-return]


def main() -> None:
    init_app_state()
    state = get_app_state()

    apply_global_styles()
    render_sidebar()

    st.title("⚙️ 设置")
    st.caption("查看和调整 Web 界面与后端配置（仅作用于当前服务进程）。")

    config_path = st.text_input(
        "配置文件路径",
        value=state.config.config_path,
        help="用于加载 LLM 与审计参数的 YAML 文件。",
        key="settings_config_path",
    )
    if config_path != state.config.config_path:
        state.config.config_path = config_path

    if st.button("重新加载配置"):
        try:
            config_data = _load_config(config_path)
            st.session_state["__last_loaded_config__"] = config_data
            st.success("配置加载成功。")
        except Exception as exc:  # noqa: BLE001
            st.error(f"加载配置失败：{exc}")

    config_data: Dict[str, Any] | None = st.session_state.get(
        "__last_loaded_config__"
    )
    if config_data is None:
        try:
            config_data = _load_config(config_path)
            st.session_state["__last_loaded_config__"] = config_data
        except Exception as exc:  # noqa: BLE001
            st.warning(f"无法加载配置：{exc}")
            config_data = None

    if config_data is not None:
        st.markdown("### LLM 配置")
        st.json(config_data.get("llm", {}))

        st.markdown("### 审计参数")
        st.json(config_data.get("audit", {}))

    st.markdown("### Web 选项")
    state.config.auto_save_report = st.checkbox(
        "审计完成后自动保存报告到磁盘",
        value=state.config.auto_save_report,
    )

    st.caption(
        f"当前配置文件：`{Path(state.config.config_path).resolve()}`"
    )


if __name__ == "__main__":
    main()
