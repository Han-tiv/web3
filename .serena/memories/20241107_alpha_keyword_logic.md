## 2024-11-07 Alpha/FOMO 解析补丁
- 位置：apps/rust-trading-bot/test_zec_message.rs
- 关键调整：在未检测到【资金异动】与出逃/撤离时，新增对【Alpha】/【FOMO】关键字的容错校验，并输出调试日志。
- 目的：兼容 Alpha 消息模板，确保解析程序不会误判缺少关键字。
- 测试：使用 rustc 独立编译 test_zec_message.rs 并运行 ./test_zec_message，通过示例消息验证成功。