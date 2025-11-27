"""
Solidity 合约解析模块
"""
import re
import json
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, asdict
import logging

logger = logging.getLogger(__name__)


@dataclass
class Function:
    """函数信息"""
    name: str
    visibility: str  # public, private, internal, external
    mutability: str  # view, pure, payable, nonpayable
    parameters: List[str]
    returns: List[str]
    modifiers: List[str]
    body: str


@dataclass
class StateVariable:
    """状态变量"""
    name: str
    type: str
    visibility: str
    value: Optional[str] = None


@dataclass
class Contract:
    """合约信息"""
    name: str
    inherits: List[str]
    functions: List[Function]
    state_vars: List[StateVariable]
    events: List[str]
    modifiers: List[str]
    source_code: str


class SolidityParser:
    """Solidity 合约解析器"""

    def __init__(self):
        self.contracts: Dict[str, Contract] = {}

    def parse_file(self, file_path: str) -> Dict[str, Contract]:
        """
        解析 Solidity 文件

        Args:
            file_path: .sol 文件路径

        Returns:
            Dict[str, Contract]: 合约名 -> 合约对象
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"文件不存在: {file_path}")

        with open(path, 'r', encoding='utf-8') as f:
            source_code = f.read()

        logger.info(f"📄 解析合约: {path.name}")

        # 提取所有合约
        contract_pattern = r'contract\s+(\w+)(?:\s+is\s+([\w\s,]+))?\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}'
        matches = re.finditer(contract_pattern, source_code, re.DOTALL)

        for match in matches:
            contract_name = match.group(1)
            inherits_str = match.group(2) or ""
            contract_body = match.group(3)

            inherits = [i.strip() for i in inherits_str.split(',') if i.strip()]

            logger.info(f"  📦 合约: {contract_name}")
            if inherits:
                logger.info(f"     继承: {', '.join(inherits)}")

            # 解析函数
            functions = self._parse_functions(contract_body)
            logger.info(f"     函数: {len(functions)} 个")

            # 解析状态变量
            state_vars = self._parse_state_variables(contract_body)
            logger.info(f"     状态变量: {len(state_vars)} 个")

            # 解析事件
            events = self._parse_events(contract_body)

            # 解析修饰符
            modifiers = self._parse_modifiers(contract_body)

            contract = Contract(
                name=contract_name,
                inherits=inherits,
                functions=functions,
                state_vars=state_vars,
                events=events,
                modifiers=modifiers,
                source_code=match.group(0)
            )

            self.contracts[contract_name] = contract

        logger.info(f"✅ 解析完成，共 {len(self.contracts)} 个合约")
        return self.contracts

    def _parse_functions(self, contract_body: str) -> List[Function]:
        """解析函数"""
        functions = []

        # 函数正则
        func_pattern = r'function\s+(\w+)\s*\(([^)]*)\)\s*(public|private|internal|external)?\s*(view|pure|payable)?\s*(returns\s*\(([^)]+)\))?\s*(\{[^}]*\})?'

        matches = re.finditer(func_pattern, contract_body, re.DOTALL)

        for match in matches:
            func_name = match.group(1)
            params_str = match.group(2) or ""
            visibility = match.group(3) or "public"
            mutability = match.group(4) or "nonpayable"
            returns_str = match.group(6) or ""
            body = match.group(7) or ""

            # 解析参数
            parameters = [p.strip() for p in params_str.split(',') if p.strip()]

            # 解析返回值
            returns = [r.strip() for r in returns_str.split(',') if r.strip()]

            # TODO: 解析修饰符
            modifiers = []

            functions.append(Function(
                name=func_name,
                visibility=visibility,
                mutability=mutability,
                parameters=parameters,
                returns=returns,
                modifiers=modifiers,
                body=body
            ))

        return functions

    def _parse_state_variables(self, contract_body: str) -> List[StateVariable]:
        """解析状态变量"""
        state_vars = []

        # 状态变量正则（简化版）
        var_pattern = r'(\w+(?:\[\])?\s+(?:public|private|internal)?\s+\w+)\s*(?:=\s*([^;]+))?;'

        matches = re.finditer(var_pattern, contract_body)

        for match in matches:
            var_def = match.group(1)
            value = match.group(2)

            # 解析类型和名称
            parts = var_def.split()
            if len(parts) >= 2:
                var_type = parts[0]
                var_name = parts[-1]
                visibility = parts[1] if len(parts) > 2 else "internal"

                state_vars.append(StateVariable(
                    name=var_name,
                    type=var_type,
                    visibility=visibility,
                    value=value
                ))

        return state_vars

    def _parse_events(self, contract_body: str) -> List[str]:
        """解析事件"""
        event_pattern = r'event\s+(\w+)\s*\([^)]*\);'
        return re.findall(event_pattern, contract_body)

    def _parse_modifiers(self, contract_body: str) -> List[str]:
        """解析修饰符"""
        modifier_pattern = r'modifier\s+(\w+)\s*\([^)]*\)'
        return re.findall(modifier_pattern, contract_body)

    def to_json(self) -> Dict[str, Any]:
        """导出为 JSON"""
        return {
            name: {
                "name": contract.name,
                "inherits": contract.inherits,
                "functions": [
                    {
                        "name": f.name,
                        "visibility": f.visibility,
                        "mutability": f.mutability,
                        "parameters": f.parameters,
                        "returns": f.returns,
                        "modifiers": f.modifiers
                    }
                    for f in contract.functions
                ],
                "state_vars": [asdict(v) for v in contract.state_vars],
                "events": contract.events,
                "modifiers": contract.modifiers
            }
            for name, contract in self.contracts.items()
        }

    def save_json(self, output_path: str):
        """保存为 JSON 文件"""
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(self.to_json(), f, indent=2, ensure_ascii=False)
        logger.info(f"💾 已保存 JSON: {output_path}")
