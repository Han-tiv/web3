# 💰 Binance账户余额查询工具

> Rust实现的简单、快速、安全的Binance期货账户余额查询工具

## 🚀 快速开始

### 1. 配置API密钥

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑配置文件
nano .env
```

**在.env中填入**:
```env
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_TESTNET=true  # true=测试网(推荐), false=主网
```

### 2. 运行查询

**方法一: 使用脚本（推荐）**
```bash
./check_balance.sh
```

**方法二: 直接运行**
```bash
# 编译
cargo build --release --bin check_balance

# 运行
cargo run --release --bin check_balance
```

**方法三: 使用环境变量**
```bash
export BINANCE_API_KEY=your_api_key
export BINANCE_SECRET_KEY=your_secret_key
export BINANCE_TESTNET=true

cargo run --bin check_balance
```

---

## 📋 输出示例

### 成功输出
```
🚀 Binance账户余额查询工具

📡 连接到 Binance 测试网
════════════════════════════════════════

✅ 账户信息获取成功!

💰 账户余额信息:
   总余额: 1000.00 USDT
   可用余额: 850.50 USDT
   未实现盈亏: 25.30 USDT
   已实现盈亏: 12.50 USDT

════════════════════════════════════════

📦 当前持仓 (2 个):

   1. 📈 ETHUSDT
      方向: LONG
      数量: 1.5
      入场价: $3500.00
      标记价: $3520.00
      未实现盈亏: $30.00 🟢 💰
      杠杆: 3x

   2. 📉 BTCUSDT
      方向: SHORT
      数量: 0.05
      入场价: $98000.00
      标记价: $97900.00
      未实现盈亏: $5.00 🟢 💰
      杠杆: 5x

   📊 总盈亏: $35.00 🟢

════════════════════════════════════════
✅ 查询完成
```

### 无持仓输出
```
💰 账户余额信息:
   总余额: 1000.00 USDT
   可用余额: 1000.00 USDT
   未实现盈亏: 0.00 USDT
   已实现盈亏: 0.00 USDT

════════════════════════════════════════

📦 当前持仓: 无

════════════════════════════════════════
✅ 查询完成
```

---

## 🔐 API密钥获取

### 测试网密钥（推荐新手）
1. 访问: https://testnet.binancefuture.com
2. 注册账号并登录
3. 进入 API Management
4. 创建新的API密钥
5. 保存API Key和Secret Key

**测试网优点**:
- ✅ 免费的虚拟资金
- ✅ 零风险测试
- ✅ 完整的API功能
- ✅ 可以随意尝试

### 主网密钥（生产环境）
1. 访问: https://www.binance.com
2. 登录账号
3. 进入 API Management
4. 创建新的API密钥
5. 启用期货交易权限
6. 配置IP白名单（推荐）

**主网注意事项**:
- ⚠️ 真实资金风险
- ⚠️ 需要完成KYC认证
- ⚠️ 建议设置IP白名单
- ⚠️ 不要分享API密钥

---

## 🛠️ 技术实现

### 代码结构
```
rust-trading-bot/
├── src/
│   ├── bin/
│   │   └── check_balance.rs    # 余额查询工具
│   ├── binance_client.rs       # Binance API封装
│   └── main.rs                 # 主程序
├── check_balance.sh            # 启动脚本
├── .env.example                # 配置模板
└── Cargo.toml                  # 依赖配置
```

### 核心功能
```rust
// 获取账户信息
let account = client.get_account_info().await?;

// 获取持仓
let positions = client.get_positions().await?;

// 计算总盈亏
let total_pnl: f64 = positions.iter()
    .map(|p| p.pnl)
    .sum();
```

### 使用的库
- `binance` - Binance API Rust客户端
- `tokio` - 异步运行时
- `dotenv` - 环境变量加载
- `anyhow` - 错误处理

---

## 🔍 故障排除

### 问题1: API密钥错误
```
❌ 账户信息获取失败: 账户信息获取失败

💡 可能的原因:
   1. API Key 或 Secret Key 错误
   2. API权限不足（需要期货交易权限）
   3. IP白名单限制
   4. 网络连接问题
```

**解决方法**:
1. 检查.env文件中的密钥是否正确
2. 确认API密钥有"期货交易"权限
3. 如果设置了IP白名单，添加当前IP
4. 确认连接到正确的网络（测试网/主网）

### 问题2: 编译失败
```
❌ 编译失败
```

**解决方法**:
```bash
# 检查Rust版本
rustc --version

# 更新Rust
rustup update

# 清理并重新编译
cargo clean
cargo build --release
```

### 问题3: 网络连接问题
```
获取账户信息失败: Connection timeout
```

**解决方法**:
- 检查网络连接
- 如果在中国大陆，可能需要代理
- 测试网连接: `ping testnet.binancefuture.com`
- 主网连接: `ping fapi.binance.com`

### 问题4: 权限不足
```
获取账户信息失败: API-key format invalid
```

**解决方法**:
1. 重新生成API密钥
2. 确保启用"期货交易"权限
3. 检查API密钥状态（是否被禁用）

---

## 📊 显示的信息说明

### 账户余额
- **总余额**: 账户总资产（包括持仓保证金）
- **可用余额**: 可用于开新仓位的余额
- **未实现盈亏**: 当前持仓的浮动盈亏
- **已实现盈亏**: 已平仓的累计盈亏

### 持仓信息
- **方向**: LONG（做多）/ SHORT（做空）
- **数量**: 持仓数量（币的数量）
- **入场价**: 开仓平均价格
- **标记价**: 当前标记价格（用于计算盈亏）
- **未实现盈亏**: 浮动盈亏（🟢盈利 / 🔴亏损）
- **杠杆**: 当前使用的杠杆倍数

---

## 🔐 安全最佳实践

### 1. API密钥安全
- ✅ 永远不要提交.env文件到Git
- ✅ 使用只读权限（如果只查询）
- ✅ 设置IP白名单
- ✅ 定期轮换密钥
- ❌ 不要在公共代码中硬编码密钥

### 2. 测试网优先
- ✅ 先在测试网验证功能
- ✅ 确认无误后再用主网
- ✅ 测试网密钥和主网密钥分开管理

### 3. 权限最小化
```
只查询余额: 启用 "查询" 权限
需要交易: 启用 "查询" + "期货交易" 权限
不需要提现: 不要启用 "提现" 权限
```

---

## 📝 扩展功能

### 添加更多功能

**1. 查看订单历史**
```rust
// 在 binance_client.rs 中已有实现
pub async fn get_open_orders(&self) -> Result<Vec<Order>>
```

**2. 查看特定交易对**
```rust
// 可以添加参数
pub async fn get_positions_for_symbol(&self, symbol: &str) -> Result<Vec<Position>>
```

**3. 导出数据到CSV**
```rust
// 添加导出功能
use csv::Writer;
fn export_to_csv(positions: &[Position]) -> Result<()>
```

---

## 💡 使用技巧

### 1. 快速查看余额
```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
alias balance='cd ~/code/apps/rust-trading-bot && ./check_balance.sh'

# 使用
$ balance
```

### 2. 定时检查
```bash
# 每小时检查一次
crontab -e

# 添加
0 * * * * cd ~/code/apps/rust-trading-bot && ./check_balance.sh >> balance.log 2>&1
```

### 3. 监控盈亏
```bash
# 持续监控（每30秒刷新）
watch -n 30 './check_balance.sh'
```

---

## 🎯 总结

这是一个**简单、快速、安全**的Binance账户余额查询工具：

✅ **特点**:
- 使用Rust实现，极致性能
- 类型安全，编译时错误检测
- 清晰的输出格式
- 完善的错误处理
- 支持测试网和主网

✅ **适用场景**:
- 快速查看账户余额
- 检查持仓状态
- 验证API配置
- 监控盈亏情况

✅ **安全性**:
- 环境变量管理密钥
- 支持测试网零风险验证
- 只读操作，不执行交易

---

## 📞 支持

**文档**:
- [Binance API文档](https://binance-docs.github.io/apidocs/futures/en/)
- [Rust binance crate](https://docs.rs/binance)

**问题反馈**:
- 检查日志输出
- 查看Binance API状态
- 参考故障排除章节

---

**"简洁的工具做好一件事 - 这就是Unix哲学，也是好品味。"** - Linus

---

*最后更新: 2025-09-29*
*版本: 1.0.0*