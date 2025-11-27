from __future__ import annotations

from datetime import datetime
from pathlib import Path
from typing import Optional

import streamlit as st

from src.main import ContractAuditor

from ..audit_progress import create_progress_callback, render_audit_progress
from ..session_state import get_app_state, init_app_state, reset_audit_state
from ..sidebar import render_sidebar
from ..styling import apply_global_styles
from ..vulnerability_list import render_vulnerability_list


def _save_uploaded_contract(uploaded_file) -> Optional[Path]:
    """将上传的合约文件保存到本地临时目录，返回保存路径。"""
    if uploaded_file is None:
        return None
    upload_dir = Path("data/web_uploads")
    upload_dir.mkdir(parents=True, exist_ok=True)
    dest = upload_dir / uploaded_file.name
    dest.write_bytes(uploaded_file.getbuffer())
    return dest


def main() -> None:
    init_app_state()
    state = get_app_state()
    audit = state.audit

    apply_global_styles()
    render_sidebar()

    st.title("🔍 合约审计")
    st.caption("上传 Solidity 合约或指定路径，系统将执行多轮 ReAct 推理并输出漏洞列表。")

    with st.container():
        col_left, col_right = st.columns([2, 1])
        with col_left:
            uploaded = st.file_uploader(
                "上传 Solidity 合约文件",
                type=["sol", "txt"],
                help="推荐上传单个 .sol 文件。",
            )
            default_example = Path("examples/VulnerableVault.sol")
            default_path = audit.contract_path or (
                str(default_example) if default_example.exists() else ""
            )
            manual_path = st.text_input(
                "或直接输入本地合约路径",
                value=default_path,
                placeholder="例如：examples/VulnerableVault.sol",
            )
        with col_right:
            st.write("")
            st.write("")
            start_button = st.button(
                "🚀 开始审计", type="primary", use_container_width=True
            )

    contract_path: Optional[Path] = None
    if uploaded is not None:
        saved_path = _save_uploaded_contract(uploaded)
        if saved_path:
            contract_path = saved_path
            audit.uploaded_contract_name = uploaded.name
    elif manual_path.strip():
        contract_path = Path(manual_path.strip())

    if start_button:
        if contract_path is None or not contract_path.exists():
            st.error("请提供有效的合约文件路径。")
        else:
            # 保留用户在侧边栏中设置的输出目录
            previous_output_dir = audit.output_dir
            reset_audit_state()
            state = get_app_state()  # 重置后重新获取
            audit = state.audit
            audit.contract_path = str(contract_path)
            audit.output_dir = previous_output_dir or audit.output_dir

            try:
                auditor = ContractAuditor(config_path=state.config.config_path)
                callback = create_progress_callback(auditor.react_engine)
                report = auditor.audit(
                    contract_path=str(contract_path),
                    output_dir=audit.output_dir,
                    progress_callback=callback,
                )

                audit.report = report
                audit.analysis_result = report.get("analysis", {})
                audit.progress.status = "success"
                audit.progress.end_time = datetime.now()

                st.success("审计完成。")
            except Exception as exc:  # noqa: BLE001
                audit.progress.status = "error"
                audit.progress.end_time = datetime.now()
                audit.progress.last_message = f"审计失败: {exc}"
                st.exception(exc)

    render_audit_progress(expanded=True)

    if audit.report and audit.report.get("vulnerabilities"):
        st.markdown("### 🛡️ 漏洞列表")
        render_vulnerability_list(audit.report.get("vulnerabilities"))
    elif audit.progress.status == "success":
        st.info("本次审计未发现漏洞。")


if __name__ == "__main__":
    main()
