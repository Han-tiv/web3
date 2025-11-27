"""
POC 生成器数据模型定义
"""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional


class VulnerabilityType(str, Enum):
    """支持的漏洞类型枚举"""

    REENTRANCY = "REENTRANCY"  # 重入攻击
    ACCESS_CONTROL = "ACCESS_CONTROL"  # 权限绕过
    PRECISION_LOSS = "PRECISION_LOSS"  # 精度丢失
    INTEGER_OVERFLOW = "INTEGER_OVERFLOW"  # 整数溢出
    UNCHECKED_RETURN = "UNCHECKED_RETURN"  # 未检查返回值
    TIMESTAMP_DEPENDENCE = "TIMESTAMP_DEPENDENCE"  # 时间戳依赖
    SIGNATURE_REPLAY = "SIGNATURE_REPLAY"  # 签名重放


@dataclass
class VulnerabilityMetadata:
    """
    描述单个漏洞的关键信息

    POC 生成器只关心与执行相关的字段，其余业务字段可以通过 extra 扩展。
    """

    vuln_type: VulnerabilityType
    description: str
    target_contract: str
    target_function: Optional[str] = None
    source_file: Optional[Path] = None
    severity: Optional[str] = None
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class POCProjectConfig:
    """POC 工程生成配置"""

    root_output_dir: Path
    project_name: Optional[str] = None
    overwrite: bool = False
    use_typescript: bool = False  # 预留扩展能力
    env_overrides: Dict[str, str] = field(default_factory=dict)


@dataclass
class GenerationContext:
    """内部渲染上下文"""

    vulnerability: VulnerabilityMetadata
    project_dir: Path
    template_root: Path
    variables: Dict[str, Any] = field(default_factory=dict)


@dataclass
class GeneratedFile:
    """记录单个生成文件的元信息"""

    path: Path
    content: str
    created: bool


@dataclass
class POCGenerationResult:
    """POC 生成结果"""

    project_dir: Path
    files: List[GeneratedFile] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    def as_dict(self) -> Dict[str, Any]:
        """便于序列化输出的简单字典视图"""
        return {
            "project_dir": str(self.project_dir),
            "files": [str(f.path) for f in self.files],
            "warnings": list(self.warnings),
        }


__all__ = [
    "VulnerabilityType",
    "VulnerabilityMetadata",
    "POCProjectConfig",
    "GenerationContext",
    "GeneratedFile",
    "POCGenerationResult",
]

