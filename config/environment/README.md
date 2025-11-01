# 🌍 环境配置目录

本目录用于存储不同环境的配置文件。

## 📋 配置文件规范

### 配置文件命名
```
.env.{environment}
```

### 支持的环境
- `development` - 开发环境
- `staging` - 预发布环境
- `production` - 生产环境
- `test` - 测试环境

## 💡 使用方法

```bash
# 加载特定环境配置
cp config/environment/.env.production .env

# 或通过环境变量指定
NODE_ENV=production npm start
```

## 🔒 安全提示

- ⚠️ 环境配置文件包含敏感信息，已被.gitignore排除
- ✅ 请使用.env.example作为模板创建配置
- ✅ 生产环境配置应通过安全渠道部署
