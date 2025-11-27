"""
可复用的 Streamlit UI 组件

包括：
- 文件上传与审计配置面板
- ReAct 推理进度展示
- 漏洞列表与详情卡片
- 报告导出与 POC 下载入口
"""
from __future__ import annotations

from io import BytesIO
from typing import Any, Dict, Iterable, List, Optional, Tuple

import pandas as pd
import plotly.express as px
import streamlit as st

from . import state


def render_file_uploader() -> Optional[Any]:
    """上传 Solidity 文件组件"""
    st.subheader("📤 合约文件上传")
    uploaded = st.file_uploader(
        "选择要审计的 Solidity 文件（.sol）",
        type=["sol"],
        accept_multiple_files=False,
    )
    if uploaded is not None:
        st.caption(f"已选择文件：`{uploaded.name}`，大小 {uploaded.size} 字节")
    return uploaded


def render_config_panel(defaults: Dict[str, Any]) -> Dict[str, Any]:
    """
    审计配置面板：
    - 模型选择
    - 推理轮次
    - 置信度阈值
    """
    st.subheader("⚙️ 审计配置")

    reasoning_model = st.text_input(
        "推理模型（reasoning_model）",
        value=str(defaults.get("reasoning_model") or ""),
        help="例如 deepseek-coder-v2:32b",
    )
    coding_model = st.text_input(
        "编码模型（coding_model）",
        value=str(defaults.get("coding_model") or ""),
        help="例如 qwen2.5:14b，用于后续 POC 生成等任务",
    )

    col1, col2 = st.columns(2)
    with col1:
        max_rounds = st.slider(
            "最大推理轮次",
            min_value=10,
            max_value=40,
            value=int(defaults.get("max_rounds") or 27),
            step=1,
        )
    with col2:
        min_confidence = st.slider(
            "最小置信度阈值",
            min_value=0.5,
            max_value=0.99,
            value=float(defaults.get("min_confidence") or 0.87),
            step=0.01,
        )

    temperature = float(defaults.get("temperature") or 0.7)
    max_tokens = int(defaults.get("max_tokens") or 4000)

    col3, col4 = st.columns(2)
    with col3:
        temperature = st.slider(
            "采样温度 (temperature)",
            min_value=0.0,
            max_value=1.5,
            value=float(temperature),
            step=0.05,
        )
    with col4:
        max_tokens = st.number_input(
            "最大 Tokens 数 (max_tokens)",
            min_value=512,
            max_value=16000,
            value=int(max_tokens),
            step=512,
        )

    return {
        "reasoning_model": reasoning_model or None,
        "coding_model": coding_model or None,
        "max_rounds": max_rounds,
        "min_confidence": min_confidence,
        "temperature": temperature,
        "max_tokens": max_tokens,
    }


def render_progress_panel(task: state.AuditTaskState) -> None:
    """展示当前任务 ReAct 推理进度"""
    st.subheader("🔍 实时推理进度")

    total = max(task.total_rounds, 1)
    current = min(task.current_round, total)
    progress = current / total

    st.progress(progress, text=f"当前第 {current} / {total} 轮")

    if task.message:
        st.info(task.message)

    if not task.history:
        st.caption("暂无历史记录。任务启动后将实时显示 Reason / Act / Observe。")
        return

    # 展示最近若干轮的详细内容
    recent = task.history[-10:]
    for item in reversed(recent):
        with st.expander(f"第 {item.get('round')} 轮", expanded=False):
            st.markdown(f"**Reasoning**：{item.get('reasoning') or '无'}")
            st.markdown(f"**Action**：{item.get('action') or '无'}")
            st.markdown(f"**Observation**：{item.get('observation') or '无'}")


def _filter_and_sort_vulns(
    vulns: List[Dict[str, Any]],
    severities: Iterable[str],
    keyword: str,
    sort_key: str,
    sort_desc: bool,
) -> List[Dict[str, Any]]:
    """根据筛选条件过滤和排序漏洞列表"""
    allowed = {s.upper() for s in severities}
    keyword_lower = keyword.lower().strip()

    def match(v: Dict[str, Any]) -> bool:
        bucket = state.severity_bucket(v.get("severity"))
        if bucket not in allowed:
            return False
        if not keyword_lower:
            return True
        text = " ".join(
            str(x or "")
            for x in [
                v.get("category"),
                v.get("description"),
                v.get("contract_name"),
                v.get("function_name"),
                v.get("evidence"),
            ]
        ).lower()
        return keyword_lower in text

    filtered = [v for v in vulns if match(v)]

    def sort_value(v: Dict[str, Any]) -> Any:
        if sort_key == "severity":
            order = {"HIGH": 3, "MEDIUM": 2, "LOW": 1, "INFO": 0}
            return order.get(state.severity_bucket(v.get("severity")), 0)
        if sort_key == "confidence":
            return float(v.get("confidence") or 0.0)
        if sort_key == "round":
            return int(v.get("round_detected") or 0)
        return v.get("category") or ""

    filtered.sort(key=sort_value, reverse=sort_desc)
    return filtered


def render_vuln_list_panel(
    task: state.AuditTaskState,
) -> Tuple[Optional[Dict[str, Any]], List[Dict[str, Any]]]:
    """
    漏洞列表面板：
    - 按严重性分类统计
    - 支持筛选 / 排序 / 分页

    Returns:
        (selected_vuln, filtered_vulns)
    """
    st.subheader("🧨 漏洞列表")

    vulns = task.vulnerabilities or []
    if not vulns:
        st.caption("当前任务尚未发现任何漏洞。")
        return None, []

    counts = state.aggregate_severity_counts(vulns)

    col1, col2, col3, col4 = st.columns(4)
    col1.metric("HIGH", counts.get("HIGH", 0))
    col2.metric("MEDIUM", counts.get("MEDIUM", 0))
    col3.metric("LOW", counts.get("LOW", 0))
    col4.metric("INFO", counts.get("INFO", 0))

    with st.expander("筛选与排序", expanded=True):
        col_f1, col_f2 = st.columns([2, 2])
        with col_f1:
            severity_options = ["HIGH", "MEDIUM", "LOW", "INFO"]
            selected_severities = st.multiselect(
                "按严重性筛选",
                options=severity_options,
                default=["HIGH", "MEDIUM", "LOW", "INFO"],
            )
        with col_f2:
            keyword = st.text_input("按关键字筛选（合约名 / 函数名 / 描述）", value="")

        col_s1, col_s2, col_s3 = st.columns([2, 1, 1])
        with col_s1:
            sort_key = st.selectbox(
                "排序字段",
                options=[
                    "severity",
                    "confidence",
                    "round",
                    "category",
                ],
                index=0,
                format_func=lambda v: {
                    "severity": "严重程度",
                    "confidence": "置信度",
                    "round": "发现轮次",
                    "category": "漏洞分类",
                }.get(v, v),
            )
        with col_s2:
            sort_desc = st.checkbox("倒序", value=True)
        with col_s3:
            page_size = st.selectbox(
                "每页数量",
                options=[10, 20, 50, 100],
                index=1,
            )

    filtered = _filter_and_sort_vulns(
        vulns=vulns,
        severities=selected_severities,
        keyword=keyword,
        sort_key=sort_key,
        sort_desc=sort_desc,
    )

    total = len(filtered)
    if total == 0:
        st.warning("无满足筛选条件的漏洞。")
        return None, []

    total_pages = (total - 1) // page_size + 1
    page_index = st.number_input(
        "当前页码",
        min_value=1,
        max_value=total_pages,
        value=1,
        step=1,
    )
    page_index = int(page_index)

    start = (page_index - 1) * page_size
    end = min(start + page_size, total)
    page_vulns = filtered[start:end]

    st.caption(f"共 {total} 条记录，第 {page_index}/{total_pages} 页")

    # 简单表格视图
    table_rows = []
    for idx, v in enumerate(page_vulns, start=start + 1):
        label_id = format_vuln_id(v)
        table_rows.append(
            {
                "序号": idx,
                "ID": label_id,
                "严重性": state.severity_bucket(v.get("severity")),
                "分类": v.get("category"),
                "置信度": round(float(v.get("confidence") or 0.0), 4),
                "轮次": v.get("round_detected"),
            }
        )

    df = pd.DataFrame(table_rows)
    st.dataframe(df, use_container_width=True, hide_index=True)

    # 选择某条漏洞查看详情
    selected_index = st.number_input(
        "选择查看详情的序号",
        min_value=start + 1,
        max_value=end,
        value=start + 1,
        step=1,
    )
    selected_index = int(selected_index)
    selected_vuln = page_vulns[selected_index - start - 1]

    return selected_vuln, filtered


def format_vuln_id(vuln: Dict[str, Any]) -> str:
    """
    统一生成漏洞 ID：
    {vuln.contract_name}:{vuln.line_number}{vuln.function_name}
    """
    contract_name = vuln.get("contract_name") or "UnknownContract"
    line_number = vuln.get("line_number")
    function_name = vuln.get("function_name") or ""

    line_str = str(line_number) if line_number is not None else "?"
    fn_suffix = f".{function_name}" if function_name else ""
    return f"{contract_name}:{line_str}{fn_suffix}"


def render_vuln_detail(
    vuln: Dict[str, Any],
    task: state.AuditTaskState,
) -> None:
    """渲染单条漏洞详情卡片，包含代码片段、POC 下载与置信度曲线"""
    st.subheader("📄 漏洞详情")

    id_str = format_vuln_id(vuln)
    c1, c2 = st.columns([2, 1])

    with c1:
        st.markdown(f"**漏洞 ID**：`{id_str}`")
        st.markdown(f"**分类**：{vuln.get('category') or '未知'}")
        st.markdown(f"**严重性**：{state.severity_bucket(vuln.get('severity'))}")
        st.markdown(f"**置信度**：{round(float(vuln.get('confidence') or 0.0) * 100, 2)}%")
        st.markdown(f"**可利用性**：{vuln.get('exploitability') or '未知'}")
        st.markdown(f"**发现轮次**：{vuln.get('round_detected')}")

    with c2:
        st.markdown("**修复建议**")
        st.write(vuln.get("recommendation") or "暂无明确修复建议。")

    st.markdown("**漏洞描述**")
    st.write(vuln.get("description") or "无描述。")

    evidence = vuln.get("evidence") or ""
    if evidence:
        st.markdown("**证据片段**")
        st.code(evidence, language="text")

    _render_source_snippet(vuln, task)
    _render_confidence_chart(vuln, task)
    _render_poc_download(vuln, task, id_str)


def _render_source_snippet(vuln: Dict[str, Any], task: state.AuditTaskState) -> None:
    """展示漏洞位置附近的源码高亮"""
    contract_name = vuln.get("contract_name")
    source = task.contract_sources.get(contract_name or "", "")
    if not source:
        st.info("未能定位对应合约源码，无法展示代码片段。")
        return

    line_number = vuln.get("line_number")
    if not isinstance(line_number, int):
        st.markdown("**代码片段**（未能精确定位行号，仅展示前若干行）")
        st.code(source[:800], language="solidity")
        return

    lines = source.splitlines()
    idx = max(0, line_number - 1)
    start = max(0, idx - 5)
    end = min(len(lines), idx + 5)
    snippet = "\n".join(
        f"{i+1:4d}: {lines[i]}"
        for i in range(start, end)
    )

    st.markdown("**代码片段（含行号）**")
    st.code(snippet, language="solidity")


def _render_confidence_chart(vuln: Dict[str, Any], task: state.AuditTaskState) -> None:
    """使用 Plotly 展示单个漏洞的置信度曲线"""
    key = state.build_vuln_key(vuln.get("category", ""), vuln.get("function_name") or vuln.get("target"))
    series = task.confidence_series.get(key)
    if not series:
        st.caption("暂无置信度曲线数据。")
        return

    df = pd.DataFrame(series)
    fig = px.line(
        df,
        x="round",
        y="confidence",
        markers=True,
        title="置信度曲线",
    )
    fig.update_layout(
        xaxis_title="ReAct 轮次",
        yaxis_title="置信度",
        yaxis=dict(range=[0.0, 1.0]),
        margin=dict(l=10, r=10, t=40, b=10),
    )
    st.plotly_chart(fig, use_container_width=True)


def _map_category_to_vuln_type(category: Optional[str]):
    """将内部漏洞分类映射到 POC 生成器的 VulnerabilityType"""
    if not category:
        return None
    from src.poc_generator.models import VulnerabilityType

    mapping = {
        "重入攻击": VulnerabilityType.REENTRANCY,
        "权限绕过": VulnerabilityType.ACCESS_CONTROL,
        "精度丢失": VulnerabilityType.PRECISION_LOSS,
        "整数溢出": VulnerabilityType.INTEGER_OVERFLOW,
    }
    return mapping.get(category)


def _render_poc_download(
    vuln: Dict[str, Any],
    task: state.AuditTaskState,
    vuln_id: str,
) -> None:
    """在详情卡片中提供 POC ZIP 下载按钮"""
    st.markdown("**POC 脚本下载**")

    vuln_type = _map_category_to_vuln_type(vuln.get("category"))
    if vuln_type is None:
        st.caption("当前漏洞分类暂不支持自动生成 POC。")
        return

    generate = st.button("生成 POC 工程并下载 ZIP", key=f"poc-btn-{vuln_id}")
    if not generate:
        return

    from pathlib import Path
    import json
    import zipfile

    from src.poc_generator.models import POCProjectConfig, VulnerabilityMetadata
    from src.poc_generator.poc_generator import POCGenerator

    with st.spinner("正在生成 POC 工程..."):
        source_file = task.contract_path if isinstance(task.contract_path, Path) else Path(task.contract_path)

        metadata = VulnerabilityMetadata(
            vuln_type=vuln_type,
            description=vuln.get("description") or "",
            target_contract=vuln.get("contract_name") or source_file.stem,
            target_function=vuln.get("function_name"),
            source_file=source_file,
            severity=state.severity_bucket(vuln.get("severity")),
            extra={
                "round_detected": vuln.get("round_detected"),
                "evidence": vuln.get("evidence"),
            },
        )

        output_root = Path("data/poc") / task.id
        config = POCProjectConfig(root_output_dir=output_root, overwrite=True)
        generator = POCGenerator()
        result = generator.generate(metadata, config)

        # 将生成的工程打包为内存中的 ZIP
        zip_buffer = BytesIO()
        with zipfile.ZipFile(zip_buffer, "w", zipfile.ZIP_DEFLATED) as zf:
            for f in result.files:
                # 使用相对路径写入 ZIP，避免暴露本地完整路径
                rel = f.path.relative_to(result.project_dir)
                zf.writestr(str(rel), f.content)

            # 同时写入一个 summary.json 方便后续追踪
            zf.writestr("poc_summary.json", json.dumps(result.as_dict(), ensure_ascii=False, indent=2))

        zip_buffer.seek(0)

        st.download_button(
            "下载 POC ZIP",
            data=zip_buffer,
            file_name=f"{metadata.target_contract}_{vuln_type.value.lower()}_poc.zip",
            mime="application/zip",
        )


def build_export_buffers(task: state.AuditTaskState) -> Dict[str, BytesIO]:
    """
    构建报告导出的内存缓冲区，支持 JSON / Markdown / PDF。
    PDF 这里使用简单的 Markdown 文本转 PDF 占位实现，方便后续替换为更专业的渲染方案。
    """
    buffers: Dict[str, BytesIO] = {}
    report = task.report or {}

    # JSON 导出
    json_buf = BytesIO()
    import json

    json_buf.write(json.dumps(report, ensure_ascii=False, indent=2).encode("utf-8"))
    json_buf.seek(0)
    buffers["json"] = json_buf

    # Markdown 导出（简要版）
    md_buf = BytesIO()
    lines: List[str] = []
    meta = report.get("metadata", {})
    summary = report.get("summary", {})

    lines.append(f"# 合约审计报告 - {meta.get('contract_path', task.filename)}")
    lines.append("")
    lines.append(f"- 审计日期：{meta.get('audit_date', '')}")
    lines.append(f"- 总漏洞数：{summary.get('total_vulnerabilities', 0)}")
    lines.append(
        f"- 高危 / 中危 / 低危：{summary.get('high_risk', 0)} / "
        f"{summary.get('medium_risk', 0)} / {summary.get('low_risk', 0)}"
    )
    lines.append("")
    lines.append("## 漏洞列表")
    for idx, v in enumerate(task.vulnerabilities, start=1):
        vid = format_vuln_id(v)
        sev = state.severity_bucket(v.get("severity"))
        conf = round(float(v.get("confidence") or 0.0) * 100, 2)
        lines.append(f"### {idx}. {vid} ({sev}, {conf}%)")
        lines.append(v.get("description") or "")
        lines.append("")

    md_text = "\n".join(lines)
    md_buf.write(md_text.encode("utf-8"))
    md_buf.seek(0)
    buffers["markdown"] = md_buf

    # PDF 导出：这里简单使用 Markdown 文本作为内容占位
    # 真实环境可以接入 reportlab / weasyprint / wkhtmltopdf 等生成 PDF。
    pdf_buf = BytesIO()
    pdf_buf.write(md_text.encode("utf-8"))
    pdf_buf.seek(0)
    buffers["pdf"] = pdf_buf

    return buffers

