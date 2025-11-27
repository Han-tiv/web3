# gRPC 迁移指南 (可选)

## ⚠️ 说明

当前系统使用 HTTP REST + JSON,**性能已经足够**。

本文档仅供未来参考,**不建议立即实施**。

---

## 📋 迁移步骤

### 1. 定义 Protobuf Schema

创建 `proto/trading_signals.proto`:

```protobuf
syntax = "proto3";

package trading;

// 交易信号服务
service TradingSignalService {
  // 接收单个信号
  rpc ReceiveSignal (TelegramSignal) returns (SignalResponse);

  // 流式接收多个信号
  rpc ReceiveSignalStream (stream TelegramSignal) returns (SignalResponse);

  // 查询系统状态
  rpc GetStatus (StatusRequest) returns (StatusResponse);
}

// Telegram信号消息
message TelegramSignal {
  string symbol = 1;           // 交易对 e.g. "BTCUSDT"
  string side = 2;             // "LONG" or "SHORT"
  double entry_price = 3;      // 入场价
  double stop_loss = 4;        // 止损价
  optional double take_profit = 5;  // 止盈价 (可选)
  string confidence = 6;       // "HIGH" / "MEDIUM" / "LOW"
  optional uint32 leverage = 7;     // 杠杆倍数
  string source = 8;           // 来源标识
  double timestamp = 9;        // Unix时间戳
  string raw_message = 10;     // 原始消息
}

// 信号响应
message SignalResponse {
  string status = 1;           // "received" / "rejected"
  string symbol = 2;
  string queued_at = 3;        // ISO 8601时间戳
  string message = 4;
}

// 状态查询请求
message StatusRequest {
  // 空消息
}

// 状态查询响应
message StatusResponse {
  bool online = 1;
  uint64 uptime_seconds = 2;
  string last_update = 3;
  uint32 positions_count = 4;
  uint32 trades_count = 5;
  uint32 ai_analysis_count = 6;
}
```

---

### 2. Python 客户端实现

#### 安装依赖
```bash
pip install grpcio grpcio-tools
```

#### 生成 Python 代码
```bash
python -m grpc_tools.protoc \
    -I./proto \
    --python_out=./apps/python-telegram-monitor \
    --grpc_python_out=./apps/python-telegram-monitor \
    proto/trading_signals.proto
```

#### Python 客户端代码
```python
import grpc
from trading_signals_pb2 import TelegramSignal, StatusRequest
from trading_signals_pb2_grpc import TradingSignalServiceStub

class GrpcTradingClient:
    def __init__(self, host='localhost', port=50051):
        self.channel = grpc.insecure_channel(f'{host}:{port}')
        self.stub = TradingSignalServiceStub(self.channel)

    async def send_signal(self, signal_data):
        """发送单个信号"""
        signal = TelegramSignal(
            symbol=signal_data['symbol'],
            side=signal_data['side'],
            entry_price=signal_data['entry_price'],
            stop_loss=signal_data['stop_loss'],
            take_profit=signal_data.get('take_profit'),
            confidence=signal_data['confidence'],
            leverage=signal_data.get('leverage', 10),
            source=signal_data['source'],
            timestamp=signal_data['timestamp'],
            raw_message=signal_data['raw_message']
        )

        response = self.stub.ReceiveSignal(signal)
        return {
            'status': response.status,
            'symbol': response.symbol,
            'queued_at': response.queued_at,
            'message': response.message
        }

    async def send_signal_stream(self, signals):
        """流式发送多个信号"""
        def signal_generator():
            for s in signals:
                yield TelegramSignal(**s)

        response = self.stub.ReceiveSignalStream(signal_generator())
        return response

    async def get_status(self):
        """查询系统状态"""
        request = StatusRequest()
        response = self.stub.GetStatus(request)
        return {
            'online': response.online,
            'uptime_seconds': response.uptime_seconds,
            'last_update': response.last_update,
            'positions_count': response.positions_count
        }
```

---

### 3. Rust 服务端实现

#### Cargo.toml 依赖
```toml
[dependencies]
tonic = "0.11"
prost = "0.12"
tokio = { version = "1", features = ["full"] }

[build-dependencies]
tonic-build = "0.11"
```

#### build.rs
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/trading_signals.proto")?;
    Ok(())
}
```

#### Rust 服务端代码
```rust
use tonic::{transport::Server, Request, Response, Status};
use trading_signals::trading_signal_service_server::{TradingSignalService, TradingSignalServiceServer};
use trading_signals::{TelegramSignal, SignalResponse, StatusRequest, StatusResponse};

pub mod trading_signals {
    tonic::include_proto!("trading");
}

#[derive(Debug, Default)]
pub struct TradingService {
    // 共享状态
}

#[tonic::async_trait]
impl TradingSignalService for TradingService {
    async fn receive_signal(
        &self,
        request: Request<TelegramSignal>,
    ) -> Result<Response<SignalResponse>, Status> {
        let signal = request.into_inner();

        log::info!("📨 收到gRPC信号: {} {} @ ${:.4}",
            signal.symbol, signal.side, signal.entry_price);

        // 保存到数据库...

        let response = SignalResponse {
            status: "received".to_string(),
            symbol: signal.symbol.clone(),
            queued_at: chrono::Utc::now().to_rfc3339(),
            message: format!("信号已接收: {} {}", signal.symbol, signal.side),
        };

        Ok(Response::new(response))
    }

    async fn receive_signal_stream(
        &self,
        request: Request<tonic::Streaming<TelegramSignal>>,
    ) -> Result<Response<SignalResponse>, Status> {
        let mut stream = request.into_inner();

        let mut count = 0;
        while let Some(signal) = stream.message().await? {
            log::info!("📨 流式信号 #{}: {}", count, signal.symbol);
            // 处理信号...
            count += 1;
        }

        let response = SignalResponse {
            status: "received".to_string(),
            symbol: format!("{} signals", count),
            queued_at: chrono::Utc::now().to_rfc3339(),
            message: format!("批量接收了{}个信号", count),
        };

        Ok(Response::new(response))
    }

    async fn get_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let response = StatusResponse {
            online: true,
            uptime_seconds: 12345,
            last_update: chrono::Utc::now().to_rfc3339(),
            positions_count: 2,
            trades_count: 15,
            ai_analysis_count: 30,
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = TradingService::default();

    println!("🚀 gRPC服务启动在 {}", addr);

    Server::builder()
        .add_service(TradingSignalServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

---

### 4. 性能对比测试

```bash
# HTTP REST 测试
time for i in {1..1000}; do
  curl -X POST http://localhost:8080/api/signals \
    -H "Content-Type: application/json" \
    -d '{"symbol":"BTCUSDT","side":"LONG",...}' > /dev/null 2>&1
done

# gRPC 测试
time for i in {1..1000}; do
  grpcurl -plaintext -d '{"symbol":"BTCUSDT","side":"LONG",...}' \
    localhost:50051 trading.TradingSignalService/ReceiveSignal > /dev/null 2>&1
done
```

---

## 📊 性能对比 (预期)

| 指标 | HTTP REST | gRPC | 提升 |
|-----|----------|------|-----|
| 单次延迟 | 10-30ms | 5-15ms | 2x |
| QPS (单核) | ~1000 | ~3000 | 3x |
| 数据包大小 | ~400 bytes | ~150 bytes | 2.7x |
| CPU占用 | 中等 | 较低 | - |

---

## ⚠️ 注意事项

1. **调试复杂度增加**
   - 需要 grpcurl 工具
   - 二进制数据不易查看

2. **依赖增加**
   - Python: grpcio (较大)
   - Rust: tonic + prost

3. **学习曲线**
   - 需要理解 Protobuf 语法
   - 需要理解 gRPC 生命周期

4. **兼容性**
   - 现有 HTTP 客户端无法使用
   - 需要重新实现所有 API

---

## 🎯 迁移建议

### ❌ 不建议迁移的情况
- 通信频率 < 10次/秒
- 系统仍在快速迭代
- 团队不熟悉 gRPC

### ✅ 建议迁移的情况
- 通信频率 > 100次/秒
- 需要双向流式传输
- 多个微服务通信
- 对类型安全有极高要求

---

## 📚 参考资源

- [gRPC官方文档](https://grpc.io/docs/)
- [Protobuf语法指南](https://developers.google.com/protocol-buffers)
- [tonic (Rust gRPC)](https://github.com/hyperium/tonic)
- [grpcio (Python)](https://github.com/grpc/grpc/tree/master/src/python/grpcio)

---

**最后更新**: 2025-11-21
**建议**: 当前阶段保持 HTTP REST,性能已经足够
