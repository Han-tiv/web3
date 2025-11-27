"""
POC 生成器通用工具函数
"""
from __future__ import annotations

import logging
import shutil
from pathlib import Path
from string import Template
from typing import Any, Dict

logger = logging.getLogger(__name__)


def slugify(name: str) -> str:
    """
    将任意字符串转换为文件系统友好的 slug
    """
    normalized = "".join(ch.lower() if ch.isalnum() else "_" for ch in name)
    # 合并重复下划线
    while "__" in normalized:
        normalized = normalized.replace("__", "_")
    return normalized.strip("_") or "poc"


def render_template(template_path: Path, context: Dict[str, Any]) -> str:
    """
    渲染简单模板文件

    模板语法基于 string.Template，使用 $VAR 占位符，
    避免与 Solidity/JS 中的大括号冲突。
    """
    if not template_path.exists():
        raise FileNotFoundError(f"模板文件不存在: {template_path}")

    raw = template_path.read_text(encoding="utf-8")
    template = Template(raw)
    rendered = template.safe_substitute(context)
    return rendered


def write_file(path: Path, content: str, overwrite: bool = False) -> bool:
    """
    写入文件

    Args:
        path: 目标文件路径
        content: 文件内容
        overwrite: 是否允许覆盖已有文件

    Returns:
        bool: 是否实际写入（False 表示因已存在而跳过）
    """
    path.parent.mkdir(parents=True, exist_ok=True)

    if path.exists() and not overwrite:
        logger.info("⏭️  跳过已存在文件: %s", path)
        return False

    path.write_text(content, encoding="utf-8")
    logger.info("💾 写入文件: %s", path)
    return True


def copy_file(src: Path, dest: Path, overwrite: bool = False) -> bool:
    """
    复制文件到目标路径

    常用于将被审计合约复制到 POC 工程的 contracts 目录。
    """
    if not src.exists():
        raise FileNotFoundError(f"源文件不存在: {src}")

    dest.parent.mkdir(parents=True, exist_ok=True)

    if dest.exists() and not overwrite:
        logger.info("⏭️  跳过已存在合约文件: %s", dest)
        return False

    shutil.copy2(src, dest)
    logger.info("📄 复制合约文件: %s -> %s", src, dest)
    return True


__all__ = [
    "slugify",
    "render_template",
    "write_file",
    "copy_file",
]

