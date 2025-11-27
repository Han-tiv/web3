"""
LLM 客户端模块
支持 Ollama API 和 OpenAI 兼容接口
"""
import json
import requests
from typing import Dict, Any, Optional, List
from dataclasses import dataclass
import logging

logger = logging.getLogger(__name__)


@dataclass
class LLMResponse:
    """LLM 响应数据类"""
    content: str
    tokens_used: int
    confidence: float = 0.0
    metadata: Dict[str, Any] = None


class LLMClient:
    """LLM 客户端"""

    def __init__(self, api_url: str, model: str, api_key: Optional[str] = None):
        self.api_url = api_url
        self.model = model
        self.api_key = api_key
        self.total_tokens = 0

    def generate(self, prompt: str, **kwargs) -> LLMResponse:
        """
        生成响应

        Args:
            prompt: 输入提示词
            **kwargs: 额外参数（temperature, max_tokens等）

        Returns:
            LLMResponse: 响应对象
        """
        try:
            # 构建请求体
            payload = {
                "model": self.model,
                "prompt": prompt,
                "stream": False,
                **kwargs
            }

            # 添加 API Key（如果有）
            headers = {}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"

            # 发送请求
            response = requests.post(
                self.api_url,
                json=payload,
                headers=headers,
                timeout=300  # 5分钟超时
            )
            response.raise_for_status()

            # 解析响应
            data = response.json()

            # Ollama API 格式
            if "response" in data:
                content = data["response"]
                tokens_used = data.get("eval_count", 0) + data.get("prompt_eval_count", 0)
            # OpenAI 兼容格式
            elif "choices" in data:
                content = data["choices"][0]["message"]["content"]
                tokens_used = data.get("usage", {}).get("total_tokens", 0)
            else:
                raise ValueError(f"未知的响应格式: {data}")

            self.total_tokens += tokens_used

            logger.info(f"✅ LLM 响应成功 ({tokens_used} tokens)")

            return LLMResponse(
                content=content,
                tokens_used=tokens_used,
                metadata=data
            )

        except requests.exceptions.Timeout:
            logger.error("❌ LLM 请求超时")
            raise
        except requests.exceptions.RequestException as e:
            logger.error(f"❌ LLM 请求失败: {e}")
            raise
        except Exception as e:
            logger.error(f"❌ LLM 处理错误: {e}")
            raise

    def extract_json(self, response: LLMResponse) -> Dict[str, Any]:
        """
        从响应中提取 JSON

        Args:
            response: LLM 响应

        Returns:
            Dict: 解析的 JSON 对象
        """
        content = response.content.strip()

        # 尝试直接解析
        try:
            return json.loads(content)
        except json.JSONDecodeError:
            pass

        # 查找 JSON 代码块
        if "```json" in content:
            start = content.find("```json") + 7
            end = content.find("```", start)
            json_str = content[start:end].strip()
        elif "```" in content:
            start = content.find("```") + 3
            end = content.find("```", start)
            json_str = content[start:end].strip()
        else:
            # 尝试提取 {} 或 []
            for char in ['{', '[']:
                if char in content:
                    start = content.find(char)
                    for end_char in ['}', ']']:
                        end = content.rfind(end_char)
                        if end > start:
                            json_str = content[start:end+1]
                            break
                    break
            else:
                raise ValueError("无法从响应中提取 JSON")

        try:
            return json.loads(json_str)
        except json.JSONDecodeError as e:
            logger.error(f"❌ JSON 解析失败: {e}")
            logger.error(f"内容: {json_str[:200]}")
            raise


class DualModelSystem:
    """双模型协作系统"""

    def __init__(self, reasoning_model: str, coding_model: str, api_url: str):
        self.reasoning_client = LLMClient(api_url, reasoning_model)
        self.coding_client = LLMClient(api_url, coding_model)

        logger.info(f"🧠 推理模型: {reasoning_model}")
        logger.info(f"💻 编码模型: {coding_model}")

    def analyze(self, prompt: str, **kwargs) -> LLMResponse:
        """使用推理模型分析"""
        logger.info("🔍 调用推理模型...")
        return self.reasoning_client.generate(prompt, **kwargs)

    def generate_code(self, prompt: str, **kwargs) -> LLMResponse:
        """使用编码模型生成代码"""
        logger.info("💻 调用编码模型...")
        return self.coding_client.generate(prompt, **kwargs)

    @property
    def total_tokens(self) -> int:
        """总消耗 tokens"""
        return self.reasoning_client.total_tokens + self.coding_client.total_tokens
