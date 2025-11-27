"""
配置管理模块
"""
import yaml
from pathlib import Path
from typing import Dict, Any


class Config:
    """配置管理类"""

    def __init__(self, config_path: str = "config.yaml"):
        self.config_path = Path(config_path)
        self._config: Dict[str, Any] = {}
        self.load()

    def load(self):
        """加载配置文件"""
        if not self.config_path.exists():
            raise FileNotFoundError(f"配置文件不存在: {self.config_path}")

        with open(self.config_path, 'r', encoding='utf-8') as f:
            self._config = yaml.safe_load(f)

    def get(self, key: str, default: Any = None) -> Any:
        """获取配置项"""
        keys = key.split('.')
        value = self._config
        for k in keys:
            if isinstance(value, dict):
                value = value.get(k, default)
            else:
                return default
        return value

    @property
    def llm_config(self) -> Dict[str, Any]:
        """LLM 配置"""
        return self._config.get('llm', {})

    @property
    def audit_config(self) -> Dict[str, Any]:
        """审计配置"""
        return self._config.get('audit', {})

    @property
    def parser_config(self) -> Dict[str, Any]:
        """解析器配置"""
        return self._config.get('parser', {})

    @property
    def poc_config(self) -> Dict[str, Any]:
        """POC 配置"""
        return self._config.get('poc', {})

    @property
    def reporter_config(self) -> Dict[str, Any]:
        """报告配置"""
        return self._config.get('reporter', {})


# 全局配置实例
config = Config()
