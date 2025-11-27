"""
POC 生成器主模块

负责根据漏洞信息与模板生成完整的 Hardhat + ethers.js POC 工程。
"""
from __future__ import annotations

import logging
from pathlib import Path
from typing import Dict, List, Optional

from .models import (
    GenerationContext,
    GeneratedFile,
    POCGenerationResult,
    POCProjectConfig,
    VulnerabilityMetadata,
    VulnerabilityType,
)
from .utils import copy_file, render_template, slugify, write_file

logger = logging.getLogger(__name__)


VULN_READABLE_NAME: Dict[VulnerabilityType, str] = {
    VulnerabilityType.REENTRANCY: "重入攻击",
    VulnerabilityType.ACCESS_CONTROL: "权限绕过",
    VulnerabilityType.PRECISION_LOSS: "精度丢失",
    VulnerabilityType.INTEGER_OVERFLOW: "整数溢出",
    VulnerabilityType.UNCHECKED_RETURN: "未检查返回值",
    VulnerabilityType.TIMESTAMP_DEPENDENCE: "时间戳依赖",
    VulnerabilityType.SIGNATURE_REPLAY: "签名重放",
}

VULN_TEMPLATE_DIR: Dict[VulnerabilityType, str] = {
    VulnerabilityType.REENTRANCY: "reentrancy",
    VulnerabilityType.ACCESS_CONTROL: "access_control",
    VulnerabilityType.PRECISION_LOSS: "precision_loss",
    VulnerabilityType.INTEGER_OVERFLOW: "integer_overflow",
    VulnerabilityType.UNCHECKED_RETURN: "unchecked_return",
    VulnerabilityType.TIMESTAMP_DEPENDENCE: "timestamp_dependence",
    VulnerabilityType.SIGNATURE_REPLAY: "signature_replay",
}

VULN_ATTACK_CONTRACT_NAME: Dict[VulnerabilityType, str] = {
    VulnerabilityType.REENTRANCY: "ReentrancyAttack",
    VulnerabilityType.ACCESS_CONTROL: "AccessControlBypass",
    VulnerabilityType.PRECISION_LOSS: "PrecisionLossExploit",
    VulnerabilityType.INTEGER_OVERFLOW: "IntegerOverflowExploit",
    VulnerabilityType.UNCHECKED_RETURN: "UncheckedReturnExploit",
    VulnerabilityType.TIMESTAMP_DEPENDENCE: "TimestampDependenceExploit",
    VulnerabilityType.SIGNATURE_REPLAY: "SignatureReplayAttack",
}


class POCGenerator:
    """POC 工程生成器"""

    def __init__(self, template_root: Optional[Path] = None):
        # 模板目录默认指向当前包下的 templates 目录
        if template_root is None:
            template_root = Path(__file__).resolve().parent / "templates"
        self.template_root = template_root

    # ------------------------------------------------------------------ #
    # 对外主接口
    # ------------------------------------------------------------------ #
    def generate(
        self,
        vuln: VulnerabilityMetadata,
        project_config: POCProjectConfig,
    ) -> POCGenerationResult:
        """
        根据漏洞信息生成完整的 POC 工程。

        Args:
            vuln: 漏洞元数据
            project_config: 工程生成配置
        """
        project_name = (
            project_config.project_name
            or f"{slugify(vuln.target_contract)}_{vuln.vuln_type.value.lower()}_poc"
        )
        project_dir = project_config.root_output_dir / project_name
        project_dir = project_dir.resolve()

        logger.info("🧩 准备生成 POC 工程: %s", project_dir)

        variables = self._build_variables(vuln, project_name, project_config)
        ctx = GenerationContext(
            vulnerability=vuln,
            project_dir=project_dir,
            template_root=self.template_root,
            variables=variables,
        )

        files: List[GeneratedFile] = []
        warnings: List[str] = []

        # 1. 通用工程文件
        files.extend(
            self._generate_common_files(
                ctx,
                overwrite=project_config.overwrite,
                warnings=warnings,
            )
        )

        # 2. 漏洞特定攻击合约 + 测试脚本
        files.extend(
            self._generate_vuln_files(
                ctx,
                overwrite=project_config.overwrite,
                warnings=warnings,
            )
        )

        # 3. 将目标合约复制到 contracts 目录（如果提供）
        if vuln.source_file:
            try:
                target_dest = project_dir / "contracts" / vuln.source_file.name
                created = copy_file(
                    vuln.source_file, target_dest, overwrite=project_config.overwrite
                )
                files.append(
                    GeneratedFile(
                        path=target_dest,
                        content=target_dest.read_text(encoding="utf-8"),
                        created=created,
                    )
                )
            except FileNotFoundError as exc:
                msg = f"目标合约文件不存在，已跳过复制: {exc}"
                logger.warning(msg)
                warnings.append(msg)

        result = POCGenerationResult(project_dir=project_dir, files=files, warnings=warnings)
        logger.info(
            "✅ POC 工程生成完成: %s（共 %d 个文件，警告 %d 条）",
            project_dir,
            len(result.files),
            len(result.warnings),
        )
        return result

    # ------------------------------------------------------------------ #
    # 内部工具
    # ------------------------------------------------------------------ #
    def _build_variables(
        self,
        vuln: VulnerabilityMetadata,
        project_name: str,
        project_config: POCProjectConfig,
    ) -> Dict[str, str]:
        """构建模板渲染变量"""
        variables: Dict[str, str] = {
            "PROJECT_NAME": project_name,
            "VULN_TYPE": vuln.vuln_type.value,
            "VULN_NAME_READABLE": VULN_READABLE_NAME[vuln.vuln_type],
            "VULN_DESCRIPTION": vuln.description,
            "TARGET_CONTRACT_NAME": vuln.target_contract,
            "TARGET_FUNCTION_NAME": vuln.target_function or "",
            "ATTACK_CONTRACT_NAME": VULN_ATTACK_CONTRACT_NAME[vuln.vuln_type],
            # 常用 env 变量名称（方便模板引用）
            "ENV_RPC_URL": "RPC_URL",
            "ENV_PRIVATE_KEY": "PRIVATE_KEY",
            "ENV_TARGET_ADDRESS": "TARGET_CONTRACT_ADDRESS",
        }

        # 针对重入攻击提供默认函数名，可被 extra 覆盖
        if vuln.vuln_type is VulnerabilityType.REENTRANCY:
            variables.setdefault("TARGET_DEPOSIT_FUNCTION", "deposit")
            variables.setdefault("TARGET_WITHDRAW_FUNCTION", "withdraw")

        # 允许通过 extra 覆盖/补充变量，统一转为大写键
        for key, value in (vuln.extra or {}).items():
            variables.setdefault(key.upper(), str(value))

        # env 文案覆盖
        for key, value in (project_config.env_overrides or {}).items():
            variables.setdefault(key, value)

        return variables

    def _generate_common_files(
        self,
        ctx: GenerationContext,
        overwrite: bool,
        warnings: List[str],
    ) -> List[GeneratedFile]:
        """生成 package.json / hardhat.config.js / .env.example / README.md 等通用文件"""
        common_dir = ctx.template_root / "common"
        mapping = {
            "package.json.tpl": "package.json",
            "hardhat.config.js.tpl": "hardhat.config.js",
            "env.example.tpl": ".env.example",
            "README.md.tpl": "README.md",
        }

        generated: List[GeneratedFile] = []

        for tpl_name, relative_out in mapping.items():
            tpl_path = common_dir / tpl_name
            try:
                content = render_template(tpl_path, ctx.variables)
            except FileNotFoundError as exc:
                msg = f"缺少通用模板文件 {tpl_name}: {exc}"
                logger.error(msg)
                warnings.append(msg)
                continue

            out_path = ctx.project_dir / relative_out
            created = write_file(out_path, content, overwrite=overwrite)
            generated.append(
                GeneratedFile(path=out_path, content=content, created=created)
            )

        return generated

    def _generate_vuln_files(
        self,
        ctx: GenerationContext,
        overwrite: bool,
        warnings: List[str],
    ) -> List[GeneratedFile]:
        """生成攻击合约和测试脚本"""
        vuln_type = ctx.vulnerability.vuln_type
        if vuln_type not in VULN_TEMPLATE_DIR:
            msg = f"不支持的漏洞类型: {vuln_type}"
            logger.error(msg)
            raise ValueError(msg)

        dir_name = VULN_TEMPLATE_DIR[vuln_type]
        vuln_dir = ctx.template_root / dir_name

        attack_tpl = vuln_dir / "Attack.sol.tpl"
        test_tpl = vuln_dir / "poc.test.js.tpl"

        generated: List[GeneratedFile] = []

        # 攻击合约
        try:
            attack_content = render_template(attack_tpl, ctx.variables)
            attack_out = (
                ctx.project_dir
                / "contracts"
                / f"{ctx.variables['ATTACK_CONTRACT_NAME']}.sol"
            )
            created = write_file(attack_out, attack_content, overwrite=overwrite)
            generated.append(
                GeneratedFile(path=attack_out, content=attack_content, created=created)
            )
        except FileNotFoundError as exc:
            msg = f"缺少攻击合约模板: {exc}"
            logger.error(msg)
            warnings.append(msg)

        # 测试脚本
        try:
            test_content = render_template(test_tpl, ctx.variables)
            slug = slugify(vuln_type.value.lower())
            test_out = ctx.project_dir / "test" / f"{slug}_poc.test.js"
            created = write_file(test_out, test_content, overwrite=overwrite)
            generated.append(
                GeneratedFile(path=test_out, content=test_content, created=created)
            )
        except FileNotFoundError as exc:
            msg = f"缺少测试模板: {exc}"
            logger.error(msg)
            warnings.append(msg)

        return generated


__all__ = [
    "POCGenerator",
]

