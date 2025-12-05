#!/usr/bin/env python3
"""
OI (Open Interest) 持仓量异动监控模块
实时监测 Binance USDT 永续合约的持仓量变化
"""

import asyncio
import time
from datetime import datetime, timedelta, timezone
from typing import List, Dict, Optional, Tuple
import httpx


class OIMonitor:
    """OI 持仓量异动监控器"""

    def __init__(
        self,
        threshold: float = 8.0,
        interval_minutes: int = 5,
        concurrency: int = 20,
        http_client: Optional[httpx.AsyncClient] = None,
        on_spike_callback = None
    ):
        """
        初始化 OI 监控器

        Args:
            threshold: OI 变化率阈值(%), 默认 8%
            interval_minutes: 扫描周期(分钟), 默认 5m
            concurrency: 并发请求数, 默认 20
            http_client: 复用的 HTTP 客户端(可选)
            on_spike_callback: OI 异动回调函数 async def callback(spike_data: Dict)
        """
        self.threshold = threshold
        self.interval = timedelta(minutes=interval_minutes)
        self.concurrency = concurrency
        self.base_url = "https://fapi.binance.com"
        self.on_spike_callback = on_spike_callback

        # 复用外部 HTTP 客户端或创建新的
        self.http_client = http_client or httpx.AsyncClient(timeout=10.0)
        self.own_client = http_client is None  # 标记是否需要自己关闭

        # 最新结果缓存
        self.coin_pool: List[str] = []  # 所有 OI 异动币种
        self.oi_top: List[Dict] = []     # 按变化率排序的异动详情

        # 运行状态
        self.running = False
        self.task: Optional[asyncio.Task] = None

    def align_to_kline_period(self) -> datetime:
        """对齐到K线周期边界 (5m)"""
        current_time = datetime.now(timezone.utc)
        interval_min = self.interval.total_seconds() // 60
        aligned_minute = (current_time.minute // interval_min) * interval_min
        return current_time.replace(minute=int(aligned_minute), second=0, microsecond=0)

    async def wait_for_next_kline_period(self):
        """等待到下一个K线周期开始"""
        aligned_time = self.align_to_kline_period()
        next_period_start = aligned_time + self.interval
        wait_seconds = (next_period_start - datetime.now(timezone.utc)).total_seconds()

        if wait_seconds > 0:
            print(f"⏸  [OI监控] 等待 {wait_seconds:.1f} 秒到下一个{self.interval.total_seconds()//60:.0f}m周期...", flush=True)
            await asyncio.sleep(wait_seconds)

    async def fetch_json(self, url: str, params: Optional[Dict] = None) -> Optional[Dict]:
        """HTTP GET 请求"""
        try:
            response = await self.http_client.get(url, params=params, timeout=10)
            response.raise_for_status()
            return response.json()
        except Exception:
            return None

    async def get_usdtm_symbols(self) -> List[str]:
        """获取所有 USDT 永续合约交易对"""
        url = f"{self.base_url}/fapi/v1/exchangeInfo"
        data = await self.fetch_json(url)

        if not data or "symbols" not in data:
            return []

        return [
            item["symbol"]
            for item in data["symbols"]
            if item.get("contractType") == "PERPETUAL"
            and item.get("quoteAsset") == "USDT"
            and item.get("status") == "TRADING"
        ]

    async def get_oi_change(self, symbol: str) -> Optional[Tuple[str, float, float]]:
        """
        获取单个币种的 OI 变化率

        Returns:
            (symbol, change_pct, current_oi) or None
        """
        url = f"{self.base_url}/futures/data/openInterestHist"
        params = {"symbol": symbol, "period": "5m", "limit": 2}

        data = await self.fetch_json(url, params)

        if not isinstance(data, list) or len(data) < 2:
            return None

        try:
            oi_old = float(data[0]["sumOpenInterestValue"])
            oi_now = float(data[1]["sumOpenInterestValue"])

            if oi_old == 0:
                return None

            change_pct = ((oi_now - oi_old) / oi_old) * 100.0
            return (symbol, change_pct, oi_now)
        except (KeyError, ValueError, ZeroDivisionError):
            return None

    async def run_scan(self) -> None:
        """执行一次完整扫描"""
        scan_start = time.time()

        # 1. 获取所有交易对
        symbols = await self.get_usdtm_symbols()
        if not symbols:
            print("⚠️  [OI监控] 无法获取USDT永续交易对列表", flush=True)
            return

        # 2. 并发获取 OI 数据
        semaphore = asyncio.Semaphore(self.concurrency)

        async def task(sym: str):
            async with semaphore:
                return await self.get_oi_change(sym)

        tasks = [task(s) for s in symbols]
        results = []

        for coro in asyncio.as_completed(tasks):
            result = await coro
            if result:
                results.append(result)

        # 3. 筛选异动币种
        spikes = [
            (sym, chg, oi)
            for sym, chg, oi in results
            if abs(chg) >= self.threshold
        ]

        # 4. 更新缓存 (如果本轮无异动,保留上一轮结果)
        if spikes:
            self.coin_pool = [sym for sym, _, _ in spikes]
            self.oi_top = [
                {
                    "symbol": sym,
                    "change_pct": chg,
                    "open_interest": oi,
                    "change_value": oi * (chg / 100),
                    "change_sign": 1 if chg > 0 else -1
                }
                for sym, chg, oi in sorted(spikes, key=lambda x: abs(x[1]), reverse=True)
            ]

            # 5. 调用回调函数 (如果已配置)
            if self.on_spike_callback:
                for spike in self.oi_top:
                    try:
                        await self.on_spike_callback(spike)
                    except Exception as e:
                        print(f"⚠️  [OI监控] 回调函数执行失败: {e}", flush=True)

        # 5. 输出日志
        scan_time = time.time() - scan_start
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
        print(f"🔥 [OI监控] {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}", flush=True)
        print(f"   扫描币种: {len(symbols)} | 用时: {scan_time:.1f}s", flush=True)

        if not spikes:
            print(f"   ℹ️  本周期无 OI 异动 (阈值 {self.threshold}%)", flush=True)
            if self.coin_pool:
                print(f"   📦 保留上一周期结果: {len(self.coin_pool)} 个异动币种", flush=True)
        else:
            print(f"   🎯 发现 {len(spikes)} 个 OI 异动:", flush=True)
            for i, spike in enumerate(self.oi_top[:10], 1):  # 只显示前 10 个
                sym = spike['symbol']
                chg = spike['change_pct']
                oi = spike['open_interest']
                sign = "📈" if chg > 0 else "📉"
                print(f"      {i:2}. {sign} {sym:<12} 变化率={chg:+.2f}%  当前OI=${oi:,.0f}", flush=True)

            if len(self.oi_top) > 10:
                print(f"      ... 还有 {len(self.oi_top) - 10} 个异动币种", flush=True)

        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
        print(flush=True)

    async def _scheduler_loop(self):
        """后台调度循环"""
        print(f"✅ [OI监控] 后台任务已启动", flush=True)
        print(f"   阈值: {self.threshold}% | 周期: {self.interval.total_seconds()//60:.0f}m | 并发: {self.concurrency}", flush=True)
        print(flush=True)

        while self.running:
            try:
                # 等待到下一个K线周期
                await self.wait_for_next_kline_period()

                # 执行扫描
                await self.run_scan()

            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"❌ [OI监控] 扫描失败: {e}", flush=True)
                import traceback
                traceback.print_exc()
                # 发生错误后等待一段时间再重试
                await asyncio.sleep(60)

    def start(self):
        """启动 OI 监控后台任务"""
        if self.running:
            return

        self.running = True
        self.task = asyncio.create_task(self._scheduler_loop())

    async def stop(self):
        """停止 OI 监控"""
        if not self.running:
            return

        self.running = False

        if self.task:
            self.task.cancel()
            try:
                await self.task
            except asyncio.CancelledError:
                pass

        if self.own_client:
            await self.http_client.aclose()

        print("✅ [OI监控] 已停止", flush=True)

    def get_coin_pool(self) -> List[str]:
        """获取 OI 异动币种池"""
        return self.coin_pool.copy()

    def get_oi_top(self) -> List[Dict]:
        """获取 OI 异动详情 (按变化率排序)"""
        return self.oi_top.copy()


# 独立运行测试
if __name__ == "__main__":
    async def test():
        monitor = OIMonitor(threshold=8.0, interval_minutes=5)
        monitor.start()

        try:
            # 运行 30 分钟测试
            await asyncio.sleep(1800)
        except KeyboardInterrupt:
            print("\n⚠️  收到中断信号")
        finally:
            await monitor.stop()

    asyncio.run(test())
