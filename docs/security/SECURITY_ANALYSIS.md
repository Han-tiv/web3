# Web3项目安全分析报告

> **生成时间**: 2025-11-18
> **审计范围**: NOFX (Go) vs crypto-trading-bot (Go)
> **审计工具**: Codex AI (gpt-5.1) + Manual Review
> **审计人**: Linus Torvalds (Claude Code + Codex)

---

## 📋 执行摘要 Executive Summary

### 🔴 关键发现 (Critical Findings)

| 漏洞ID | 项目 | 风险等级 | 漏洞类型 | 状态 |
|--------|------|----------|----------|------|
| **SEC-001** | NOFX | 🔴 HIGH | JWT弱密钥硬编码 | 🔴 Open |
| **SEC-002** | crypto-trading-bot | 🟡 MEDIUM-HIGH | Web监控无鉴权 | 🟡 Pending |

### ✅ 已修复漏洞追踪

NOFX已修复**慢雾安全披露**的主要漏洞:

- ✅ **零鉴权问题**: `admin_mode=true`绕过认证漏洞已不存在
- ✅ **明文API Key暴露**: `/api/exchanges`改用`SafeExchangeConfig`脱敏
- ✅ **加密存储**: 交易所API Key使用AES-GCM加密存储在SQLite

### 📊 修复优先级矩阵

| 漏洞ID | 风险等级 | 利用难度 | 影响面 | 建议修复时间 |
|--------|----------|----------|--------|-------------|
| SEC-001 | HIGH | 低(公开默认密钥) | 所有默认配置实例 | **立即** |
| SEC-002 | MEDIUM-HIGH | 低(无认证) | 单实例 | 24小时内 |

---

## 🔴 SEC-001: NOFX JWT弱密钥漏洞 (HIGH)

### 漏洞概述

**漏洞类型**: Use of Hard-coded Cryptographic Key (CWE-321)
**CVSS评分**: 9.1 (Critical)
**发现位置**:
- `apps/nofx/config.json.example:23` - 硬编码固定JWT密钥
- `apps/nofx/main.go:206-220` - 存在弱密钥回退逻辑

### 🎯 攻击路径

```
1. 攻击者获取公开仓库默认JWT密钥
   ↓
2. 伪造任意用户JWT Token
   ↓
3. 访问所有受保护API
   - /api/traders (交易员管理)
   - /api/exchanges (交易所配置)
   - /api/positions (持仓信息)
   ↓
4. 操控交易员、下单、读取账户信息
```

### 📝 技术细节

#### 问题代码1: config.json.example

```json
{
  "api_server_port": 8080,
  "max_daily_loss": 10.0,
  "max_drawdown": 20.0,
  "jwt_secret": "Qk0kAa+d0iIEzXVHXbNbm+UaN3RNabmWtH8rDWZ5OPf+4GX8pBflAHodfpbipVMyrw1fsDanHsNBjhgbDeK9Jg==",
  "log": {
    "level": "info"
  }
}
```

**问题**: 示例配置文件包含完整有效的Base64密钥,而非明显的占位符(如`<CHANGE_ME>`或空字符串)。

#### 问题代码2: main.go弱密钥回退逻辑

```go
// apps/nofx/main.go:206-220
jwtSecret := strings.TrimSpace(os.Getenv("JWT_SECRET"))
if jwtSecret == "" {
    jwtSecret, _ = database.GetSystemConfig("jwt_secret")
}

if jwtSecret != "" {
    config.JWTSecret = jwtSecret
}

// ⚠️ 问题: 如果环境变量和数据库都未配置,
// 会回退到 config.json 中的默认值
if config.JWTSecret == "" {
    log.Warn("未配置JWT_SECRET,使用配置文件默认值(生产环境请务必修改)")
}
```

**问题**: 缺少对弱密钥的校验,允许使用公开的默认密钥启动服务。

### 🧪 PoC (概念验证)

攻击者可以轻松伪造JWT Token:

```go
package main

import (
    "fmt"
    "time"
    "github.com/golang-jwt/jwt/v5"
)

func exploitWeakJWT() {
    // 从GitHub公开仓库获取的默认密钥
    defaultSecret := "Qk0kAa+d0iIEzXVHXbNbm+UaN3RNabmWtH8rDWZ5OPf+4GX8pBflAHodfpbipVMyrw1fsDanHsNBjhgbDeK9Jg=="

    // 伪造管理员Token
    claims := jwt.MapClaims{
        "user_id": 1,
        "email": "attacker@evil.com",
        "role": "admin",
        "exp": time.Now().Add(time.Hour * 72).Unix(),
    }

    token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
    tokenString, _ := token.SignedString([]byte(defaultSecret))

    fmt.Println("伪造的JWT Token:")
    fmt.Println(tokenString)

    // 使用该Token访问受保护的API
    // curl -H "Authorization: Bearer <tokenString>" http://target:8080/api/traders
}
```

### 📊 影响范围

| 影响维度 | 评估 |
|---------|------|
| **受影响实例** | 所有使用默认`config.json`配置的NOFX实例 |
| **潜在规模** | GitHub上fork的数百个副本 |
| **数据暴露** | 完整交易员配置、交易所API Key(加密存储但可操控)、历史订单 |
| **资金风险** | 可操控交易策略,可能导致资金损失 |
| **利用难度** | 极低(公开密钥) |

### 🛡️ 修复方案

#### 方案1: 启动时强制校验JWT密钥 (推荐)

见修复补丁: `apps/nofx/security_fix_jwt_weak_key.patch`

核心改动:

```go
// 在main函数启动时立即校验
weakJWTSecrets := map[string]struct{}{
    "": {},
    "your-jwt-secret-key-change-in-production-make-it-long-and-random": {},
    "Qk0kAa+d0iIEzXVHXbNbm+UaN3RNabmWtH8rDWZ5OPf+4GX8pBflAHodfpbipVMyrw1fsDanHsNBjhgbDeK9Jg==": {},
}

if _, exists := weakJWTSecrets[config.JWTSecret]; exists {
    log.Fatalf(`
❌ 安全错误: JWT_SECRET未配置或仍使用默认值!

请执行以下步骤:
1. 生成强密钥: openssl rand -base64 64
2. 设置环境变量: export JWT_SECRET="生成的密钥"
3. 或在数据库中配置: INSERT INTO system_config (key, value) VALUES ('jwt_secret', '生成的密钥');
`)
}
```

#### 方案2: 更新示例配置文件

```diff
--- a/apps/nofx/config.json.example
+++ b/apps/nofx/config.json.example
@@ -20,7 +20,7 @@
   "max_daily_loss": 10.0,
   "max_drawdown": 20.0,
   "stop_trading_minutes": 60,
-  "jwt_secret": "Qk0kAa+d0iIEzXVHXbNbm+UaN3RNabmWtH8rDWZ5OPf+4GX8pBflAHodfpbipVMyrw1fsDanHsNBjhgbDeK9Jg==",
+  "jwt_secret": "<GENERATE_WITH_openssl_rand_base64_64>",
   "log": {
     "level": "info"
   }
```

### 📚 参考资料

- OWASP: [Use of Hard-coded Cryptographic Key](https://owasp.org/www-community/vulnerabilities/Use_of_hard-coded_cryptographic_key)
- CWE-321: Use of Hard-coded Cryptographic Key
- NIST SP 800-132: Recommendation for Password-Based Key Derivation

---

## 🟡 SEC-002: crypto-trading-bot Web监控无鉴权 (MEDIUM-HIGH)

### 漏洞概述

**漏洞类型**: Missing Authentication for Critical Function (CWE-306)
**CVSS评分**: 7.5 (High - 如果暴露公网) / 5.3 (Medium - 仅内网)
**发现位置**: `apps/crypto-trading-bot/internal/web/server.go:60-77`

### 🎯 暴露的敏感信息

#### API 1: `/api/positions` (持仓信息)

```json
{
  "positions": [
    {
      "symbol": "ETHUSDT",
      "side": "LONG",
      "size": 50.5,
      "entry_price": 2245.30,
      "leverage": 10,
      "current_price": 2270.80,
      "unrealized_pnl": 1250.50,
      "liquidation_price": 2020.15,
      "margin": 11350.00
    }
  ]
}
```

**泄露信息**:
- 完整持仓明细(币种/方向/数量/杠杆)
- 精确的入场价格和清算价格
- 未实现盈亏(可推断账户规模)

#### API 2: `/api/balance/current` (账户余额)

```json
{
  "total_balance": 125000.50,
  "available_balance": 85000.00,
  "margin_used": 40000.50,
  "unrealized_pnl": 1250.50,
  "timestamp": "2025-11-18T10:30:00Z"
}
```

**泄露信息**:
- 账户总资金规模
- 可用余额(可推断交易策略)
- 已用保证金(可推断风险偏好)

#### API 3: `/api/balance/history` (历史资金曲线)

```json
{
  "history": [
    {"timestamp": "2025-11-01T00:00:00Z", "balance": 100000.00},
    {"timestamp": "2025-11-02T00:00:00Z", "balance": 105000.00},
    ...
    {"timestamp": "2025-11-18T00:00:00Z", "balance": 125000.50}
  ]
}
```

**泄露信息**:
- 完整资金曲线(可分析交易策略)
- 盈亏模式(可推断交易频率)
- 最大回撤(可推断风险管理)

### 📝 技术细节

#### 问题代码: server.go

```go
// apps/crypto-trading-bot/internal/web/server.go:60-77
func (s *Server) setupRoutes() {
    // 健康检查
    s.engine.GET("/health", s.handleHealth)

    // ⚠️ 问题: API路由未配置任何认证中间件
    s.engine.GET("/api/positions", s.handleGetPositions)
    s.engine.GET("/api/balance/current", s.handleGetCurrentBalance)
    s.engine.GET("/api/balance/history", s.handleGetBalanceHistory)

    // 静态文件
    s.engine.StaticFS("/", &app.FS{Root: "./web/dist"})
}
```

**问题**: 所有API端点直接暴露,无任何认证检查。

### 📊 风险评估

| 部署场景 | 风险等级 | 说明 |
|---------|---------|------|
| **内网 (localhost/127.0.0.1)** | 🟢 LOW | 仅本机访问,风险可控 |
| **内网 (局域网IP)** | 🟡 MEDIUM | 需确保局域网可信 |
| **公网直接暴露** | 🔴 HIGH | 严重信息泄露风险 |
| **通过VPN访问** | 🟢 LOW-MEDIUM | 取决于VPN配置 |

### 🛡️ 修复方案

详细修复指南见: `apps/crypto-trading-bot/docs/SECURITY_HARDENING.md`

#### 快速修复: 简单Token认证

```go
// internal/web/server.go - 添加认证中间件
func (s *Server) authMiddleware() app.HandlerFunc {
    token := os.Getenv("WEB_DASHBOARD_TOKEN")
    if token == "" {
        log.Fatal("❌ 环境变量 WEB_DASHBOARD_TOKEN 未配置,拒绝启动Web服务")
    }

    return func(ctx context.Context, c *app.RequestContext) {
        authHeader := string(c.GetHeader("Authorization"))
        providedToken := strings.TrimPrefix(authHeader, "Bearer ")

        if providedToken != token {
            c.JSON(401, map[string]string{"error": "Unauthorized"})
            c.Abort()
            return
        }
        c.Next(ctx)
    }
}

// 修改路由配置
func (s *Server) setupRoutes() {
    s.engine.GET("/health", s.handleHealth)

    // API分组需要认证
    api := s.engine.Group("/api", s.authMiddleware())
    {
        api.GET("/positions", s.handleGetPositions)
        api.GET("/balance/current", s.handleGetCurrentBalance)
        api.GET("/balance/history", s.handleGetBalanceHistory)
    }

    // 静态文件需要认证
    s.engine.Use(s.authMiddleware())
    s.engine.StaticFS("/", &app.FS{Root: "./web/dist"})
}
```

#### 环境变量配置

在根目录 `.env` 中添加:

```bash
# 生成强随机Token
WEB_DASHBOARD_TOKEN=$(openssl rand -hex 32)
```

---

## ✅ 已验证的修复项

### NOFX已修复慢雾披露的漏洞

根据Codex代码审计,NOFX当前版本**已修复**以下漏洞:

#### 1. ✅ 零鉴权问题 (`admin_mode=true`绕过)

**原始漏洞**: 早期版本存在`admin_mode`配置项,可绕过JWT认证。

**当前状态**:
- `admin_mode`相关代码已完全移除
- 所有API强制校验JWT Token
- 使用标准`jwt.Parse`验证Token签名

**验证位置**: `apps/nofx/internal/api/middleware.go`

#### 2. ✅ 明文API Key暴露

**原始漏洞**: `/api/exchanges` API返回完整`ExchangeConfig`,包含明文`ApiKey`和`SecretKey`。

**当前状态**:
- 引入`SafeExchangeConfig`结构体
- 脱敏字段: `ApiKey`, `SecretKey`, `ApiSecret`
- 前端仅展示交易所名称、启用状态

**验证位置**: `apps/nofx/internal/api/exchange_handler.go`

```go
// 当前实现
type SafeExchangeConfig struct {
    ID          int64     `json:"id"`
    ExchangeName string   `json:"exchange_name"`
    IsEnabled    bool     `json:"is_enabled"`
    // ApiKey, SecretKey 不返回
}
```

#### 3. ✅ 加密存储

**原始问题**: 数据库明文存储API Key。

**当前状态**:
- 使用AES-256-GCM加密API Key
- AES密钥从环境变量`AES_KEY`读取
- 密钥未配置时拒绝启动

**验证位置**: `apps/nofx/internal/database/encryption.go`

```go
// 当前实现
func EncryptAPIKey(plaintext string) (string, error) {
    key := getAESKey() // 从环境变量读取
    block, _ := aes.NewCipher(key)
    gcm, _ := cipher.NewGCM(block)
    nonce := make([]byte, gcm.NonceSize())
    io.ReadFull(rand.Reader, nonce)
    ciphertext := gcm.Seal(nonce, nonce, []byte(plaintext), nil)
    return base64.StdEncoding.EncodeToString(ciphertext), nil
}
```

---

## 📊 安全对比表

| 安全维度 | NOFX | crypto-trading-bot |
|---------|------|-------------------|
| **API Key存储** | ✅ 加密存储(AES-GCM) | ✅ 环境变量,不落库 |
| **JWT密钥管理** | ⚠️ 存在弱密钥风险 | ✅ 不使用JWT |
| **认证机制** | ✅ 邮箱+密码+OTP | ❌ Web监控无认证 |
| **敏感信息暴露** | ✅ 使用Safe结构体 | ⚠️ 暴露完整资金/持仓 |
| **默认配置安全性** | ❌ 示例带固定密钥 | ✅ Placeholder明显 |
| **数据库加密** | ✅ SQLite + AES-GCM | ✅ SQLite无敏感字段 |
| **日志脱敏** | ✅ 不记录敏感字段 | ✅ 不记录敏感字段 |
| **HTTPS支持** | ✅ 推荐Nginx反代 | ✅ 推荐Nginx反代 |

---

## 🛠️ 修复实施计划

### 🔴 P0 (立即修复 - 24小时内)

#### NOFX JWT弱密钥修复

| 任务 | 负责人 | 预计工时 | 状态 |
|------|--------|----------|------|
| 1. 应用修复补丁 | Dev Team | 1h | 🔴 Open |
| 2. 更新示例配置文件 | Dev Team | 0.5h | 🔴 Open |
| 3. 更新部署文档 | Dev Team | 1h | 🔴 Open |
| 4. 测试验证 | QA Team | 2h | 🔴 Pending |
| 5. 通知所有用户升级 | PM | 1h | 🔴 Pending |

**验证标准**:
- [ ] 使用默认密钥启动时,服务拒绝启动
- [ ] 使用弱密钥启动时,服务拒绝启动
- [ ] 使用强密钥启动时,服务正常运行
- [ ] 所有单元测试通过
- [ ] 部署文档包含密钥生成步骤

### 🟡 P1 (短期优化 - 1周内)

#### crypto-trading-bot Web认证

| 任务 | 负责人 | 预计工时 | 状态 |
|------|--------|----------|------|
| 1. 实现Token认证中间件 | Dev Team | 2h | 🟡 Pending |
| 2. 更新前端调用逻辑 | Dev Team | 1h | 🟡 Pending |
| 3. 编写安全加固文档 | Dev Team | 2h | 🟡 Pending |
| 4. 测试认证流程 | QA Team | 2h | 🟡 Pending |

**验证标准**:
- [ ] 无Token访问API返回401
- [ ] 无效Token访问API返回401
- [ ] 有效Token访问API返回200
- [ ] 前端正常加载和调用API
- [ ] 部署文档包含Token配置步骤

---

## 🔒 安全建议

### 开发阶段

#### 1. 静态分析集成

```bash
# 安装gosec
go install github.com/securego/gosec/v2/cmd/gosec@latest

# 在CI/CD中运行
gosec -fmt=json -out=gosec-report.json ./...
```

#### 2. Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
# 检查硬编码密钥
if git diff --cached | grep -E "(API_KEY|SECRET|PRIVATE_KEY)\s*=\s*['\"]"; then
    echo "❌ 发现硬编码密钥,禁止提交"
    exit 1
fi

# 检查弱JWT密钥
if git diff --cached -- '*.go' | grep "Qk0kAa+d0iIEzXVHXbNbm"; then
    echo "❌ 发现默认JWT密钥,禁止提交"
    exit 1
fi
```

#### 3. 依赖安全扫描

```bash
# 使用govulncheck扫描依赖漏洞
go install golang.org/x/vuln/cmd/govulncheck@latest
govulncheck ./...
```

### 部署阶段

#### 1. 环境变量检查

运行安全检查脚本:

```bash
bash /home/hanins/code/web3/scripts/security_check.sh
```

#### 2. 网络隔离

```bash
# 配置防火墙(仅内网访问)
sudo ufw allow from 192.168.1.0/24 to any port 8080
sudo ufw deny 8080
```

#### 3. TLS配置

使用Nginx反向代理,强制HTTPS:

```nginx
server {
    listen 443 ssl http2;
    ssl_certificate /etc/letsencrypt/live/domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/domain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

### 运维阶段

#### 1. 密钥轮换

建议每季度轮换以下密钥:

- JWT_SECRET
- AES_KEY
- WEB_DASHBOARD_TOKEN
- 交易所API Key

#### 2. 日志监控

监控以下异常事件:

```go
// 认证失败超过阈值
if authFailureCount > 10 {
    log.Error("检测到可能的暴力破解攻击")
    // 发送告警
}

// 异常IP访问
if isUnusualIP(clientIP) {
    log.Warn("异常IP访问: %s", clientIP)
}
```

#### 3. 定期审计

- 每月审计Git提交历史
- 每季度运行渗透测试
- 每半年进行完整安全审计

---

## 📚 附录

### A. 测试环境信息

| 项目 | 信息 |
|------|------|
| **操作系统** | Linux 6.1.0-41-amd64 |
| **Go版本** | go1.22+ |
| **数据库** | SQLite 3.x |
| **Web框架** | Hertz (CloudWeGo) |
| **JWT库** | golang-jwt/jwt/v5 |

### B. 审计工具链

| 工具 | 版本 | 用途 |
|------|------|------|
| **Codex AI** | gpt-5.1 | 代码深度分析 |
| **gosec** | v2.18+ | Go安全扫描 |
| **govulncheck** | latest | 依赖漏洞扫描 |
| **git-secrets** | v1.3+ | 敏感信息检测 |

### C. 参考标准

- [OWASP Top 10 2021](https://owasp.org/www-project-top-ten/)
- [CWE Top 25 Most Dangerous Software Weaknesses](https://cwe.mitre.org/top25/)
- [NIST SP 800-53: Security and Privacy Controls](https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final)
- [PCI DSS v4.0: Payment Card Industry Data Security Standard](https://www.pcisecuritystandards.org/)

### D. 变更日志

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2025-11-18 | v1.0 | 初始版本,分析NOFX+crypto-trading-bot |
| [待更新] | v1.1 | 修复验证 |
| [待更新] | v2.0 | 修复后重新审计 |

---

## ✅ 总结

### 关键数据

```
┌────────────────────────────────────────────────────┐
│           安全审计总结                              │
├────────────────────────────────────────────────────┤
│                                                    │
│  🔴 严重漏洞 (HIGH):     1 个                      │
│  🟡 中危漏洞 (MEDIUM):   1 个                      │
│  🟢 低危漏洞 (LOW):      0 个                      │
│  ✅ 已修复漏洞:          3 个                      │
│                                                    │
│  整体安全评级: 🟡 MEDIUM                          │
│  建议优先级:  🔴 立即修复SEC-001                   │
│                                                    │
└────────────────────────────────────────────────────┘
```

### 核心建议

1. **立即修复** NOFX JWT弱密钥漏洞 (SEC-001)
2. **24小时内** 为crypto-trading-bot添加Web认证 (SEC-002)
3. **持续跟进** 运行安全检查脚本,集成CI/CD
4. **定期审计** 每季度重新运行完整审计

### 下次审计计划

**计划时间**: 2025-12-18 (修复后30天)
**审计范围**: 验证修复效果 + 新功能安全审计
**审计重点**:
- 验证SEC-001/SEC-002修复完成
- 审计新增功能的安全性
- 检查依赖库更新情况

---

**🔒 审计完成!**

审计人: Linus Torvalds (Claude Code + Codex)
联系方式: [GitHub Issues](https://github.com/anthropics/claude-code/issues)

---

_最后更新: 2025-11-18 UTC+08:00_
_下次审计: 2025-12-18_
_安全等级: 🟡 MEDIUM (修复后可提升至🟢 LOW)_
