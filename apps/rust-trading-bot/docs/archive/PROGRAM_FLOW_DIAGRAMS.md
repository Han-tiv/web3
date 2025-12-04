# 🎨 集成AI交易系统 - 完整流程可视化

**生成时间**: 2025-11-29 01:29  
**系统版本**: v2.0.0-refactored

---

## 📋 目录

1. [系统整体架构图](#1-系统整体架构图)
2. [程序启动流程图](#2-程序启动流程图)
3. [并发任务架构图](#3-并发任务架构图)
4. [信号处理流程图](#4-信号处理流程图)
5. [AI分析决策流程图](#5-ai分析决策流程图)
6. [持仓监控流程图](#6-持仓监控流程图)
7. [延迟队列流程图](#7-延迟队列流程图)
8. [数据流向图](#8-数据流向图)
9. [状态转换图](#9-状态转换图)

---

## 1️⃣ 系统整体架构图

```mermaid
graph TB
    subgraph "外部输入"
        A[Telegram消息] --> B[Python监听器]
        B --> C[数据库 telegram_signals]
    end
    
    subgraph "Rust交易系统"
        D[主程序 mod.rs] --> E[配置加载]
        D --> F[数据库初始化]
        D --> G[交易器初始化]
        
        G --> H[IntegratedAITrader<br/>trader.rs]
        
        H --> I[持仓监控线程<br/>monitor_positions]
        H --> J[延迟队列线程<br/>reanalyze_pending_entries]
        H --> K[信号轮询线程<br/>Telegram Polling]
        H --> L[Web服务器<br/>8080端口]
        
        C --> K
        K --> M[信号处理<br/>analyze_and_trade]
        M --> N[AI分析 Gemini]
        N --> O[开仓执行<br/>execute_trial_entry]
        
        I --> P[AI评估 DeepSeek]
        P --> Q[止损/止盈决策]
        
        J --> M
    end
    
    subgraph "外部API"
        N --> R[Gemini API<br/>入场分析]
        P --> S[DeepSeek API<br/>持仓管理]
        O --> T[Binance API<br/>下单交易]
        Q --> T
    end
    
    subgraph "数据存储"
        O --> U[(SQLite Database)]
        Q --> U
        L --> U
        U --> V[positions表<br/>ai_analysis表<br/>orders表]
    end
    
    style D fill:#e1f5ff
    style H fill:#fff3e0
    style I fill:#c8e6c9
    style J fill:#c8e6c9
    style K fill:#c8e6c9
    style L fill:#c8e6c9
    style M fill:#ffe0b2
    style N fill:#f8bbd0
    style O fill:#ffccbc
    style P fill:#f8bbd0
    style Q fill:#ffccbc
```

---

## 2️⃣ 程序启动流程图

```mermaid
flowchart TD
    Start([程序启动]) --> LoadEnv[加载.env环境变量]
    LoadEnv --> InitLog[初始化日志系统]
    InitLog --> LoadConfig[加载配置<br/>load_configuration]
    
    LoadConfig --> CheckKeys{检查API密钥}
    CheckKeys -->|缺失| Error1[❌ 错误退出]
    CheckKeys -->|完整| InitBinance[初始化Binance客户端]
    
    InitBinance --> InitDB[初始化数据库<br/>data/trading.db]
    InitDB --> CreateDir{创建data目录}
    CreateDir -->|失败| Error2[❌ 错误退出]
    CreateDir -->|成功| CreateTrader[创建交易器<br/>IntegratedAITrader::new]
    
    CreateTrader --> InitClients[初始化AI客户端<br/>DeepSeek + Gemini]
    InitClients --> SyncPositions[恢复历史持仓<br/>sync_existing_positions]
    
    SyncPositions --> SpawnTasks[启动并发任务<br/>spawn_concurrent_tasks]
    
    SpawnTasks --> Task1[任务1: 持仓监控]
    SpawnTasks --> Task2[任务2: 延迟队列]
    SpawnTasks --> Task3[任务3: Web服务器]
    SpawnTasks --> Task4[任务4: 信号轮询]
    
    Task1 --> Running[系统运行中...]
    Task2 --> Running
    Task3 --> Running
    Task4 --> Running
    
    Running --> MainLoop[主线程保持运行<br/>每小时sleep]
    
    style Start fill:#4caf50,color:#fff
    style Running fill:#2196f3,color:#fff
    style Error1 fill:#f44336,color:#fff
    style Error2 fill:#f44336,color:#fff
    style MainLoop fill:#9c27b0,color:#fff
```

---

## 3️⃣ 并发任务架构图

```mermaid
graph TB
    subgraph "主线程"
        Main[main 函数] --> Spawn[spawn_concurrent_tasks]
        Spawn --> Keep[保持运行<br/>sleep 3600秒]
    end
    
    subgraph "线程1: 持仓监控"
        T1[monitor_positions] --> T1_Loop{每180秒循环}
        T1_Loop --> T1_Check[检查所有持仓]
        T1_Check --> T1_AI[AI评估 DeepSeek]
        T1_AI --> T1_Action{决策}
        T1_Action -->|HOLD| T1_Loop
        T1_Action -->|CLOSE| T1_Close[平仓]
        T1_Action -->|ADJUST| T1_Adjust[调整止损止盈]
        T1_Close --> T1_Loop
        T1_Adjust --> T1_Loop
    end
    
    subgraph "线程2: 延迟队列"
        T2[reanalyze_pending_entries] --> T2_Loop{每600秒循环}
        T2_Loop --> T2_Check[检查待开仓队列]
        T2_Check --> T2_AI[重新AI分析]
        T2_AI --> T2_Action{决策}
        T2_Action -->|ENTER| T2_Open[执行开仓]
        T2_Action -->|SKIP| T2_Remove[移除队列]
        T2_Action -->|WAIT| T2_Loop
        T2_Open --> T2_Loop
        T2_Remove --> T2_Loop
    end
    
    subgraph "线程3: Web服务器"
        T3[start_web_server] --> T3_Listen[监听 0.0.0.0:8080]
        T3_Listen --> T3_Routes{路由处理}
        T3_Routes --> T3_Health[GET /health]
        T3_Routes --> T3_Balance[GET /balance]
        T3_Routes --> T3_Positions[GET /positions]
        T3_Routes --> T3_Signals[POST /api/signals]
    end
    
    subgraph "线程4: 信号轮询"
        T4[Signal Polling] --> T4_Loop{每5秒循环}
        T4_Loop --> T4_Query[查询未处理信号<br/>list_unprocessed_telegram_signals]
        T4_Query --> T4_Parse[解析并分类]
        T4_Parse --> T4_Filter{BUY信号?}
        T4_Filter -->|是| T4_Spawn[spawn子任务<br/>analyze_and_trade]
        T4_Filter -->|否| T4_Mark[标记已处理]
        T4_Spawn --> T4_Mark
        T4_Mark --> T4_Loop
    end
    
    subgraph "子任务: AI分析"
        T5[analyze_and_trade] --> T5_AI[AI分析 Gemini]
        T5_AI --> T5_Dec{决策}
        T5_Dec -->|ENTER| T5_Open[开仓30%]
        T5_Dec -->|WAIT| T5_Queue[加入延迟队列]
        T5_Dec -->|SKIP| T5_Log[记录原因]
    end
    
    Spawn -.启动.-> T1
    Spawn -.启动.-> T2
    Spawn -.启动.-> T3
    Spawn -.启动.-> T4
    T4_Spawn -.创建.-> T5
    
    style Main fill:#e1f5ff
    style T1 fill:#c8e6c9
    style T2 fill:#fff9c4
    style T3 fill:#f8bbd0
    style T4 fill:#b2dfdb
    style T5 fill:#ffccbc
```

---

## 4️⃣ 信号处理流程图

```mermaid
flowchart TD
    Start([Telegram消息到达]) --> Python[Python监听器接收]
    Python --> Insert[插入数据库<br/>telegram_signals表]
    
    Insert --> Poll{Rust轮询<br/>每5秒}
    Poll -->|暂无| Wait[等待下次轮询]
    Wait --> Poll
    
    Poll -->|有新信号| Query[查询未处理信号<br/>limit 100]
    Query --> ForEach[遍历每条记录]
    
    ForEach --> CheckID{有ID?}
    CheckID -->|无| Skip1[跳过该信号]
    CheckID -->|有| Parse[解析信号]
    
    Parse --> Classify{信号分类}
    Classify -->|评分≥5| Alpha[AlertType::AlphaOpportunity]
    Classify -->|评分3-4| Fomo[AlertType::FomoSignal]
    Classify -->|评分≤-3| Escape[AlertType::FundEscape]
    Classify -->|其他| Inflow[AlertType::FundInflow]
    
    Alpha --> ConvertTime[时间戳转换<br/>String→DateTime]
    Fomo --> ConvertTime
    Escape --> ConvertTime
    Inflow --> ConvertTime
    
    ConvertTime --> CreateAlert[创建FundAlert对象]
    CreateAlert --> CheckAction{recommend_action}
    
    CheckAction -->|BUY| SpawnTask[spawn异步任务<br/>analyze_and_trade]
    CheckAction -->|其他| Skip2[跳过非BUY信号]
    
    SpawnTask --> Mark[标记已处理<br/>processed=true]
    Skip2 --> Mark
    Skip1 --> NextRecord[下一条记录]
    Mark --> NextRecord
    
    NextRecord --> MoreRecords{还有记录?}
    MoreRecords -->|是| ForEach
    MoreRecords -->|否| Poll
    
    SpawnTask -.异步执行.-> AIAnalysis[AI分析流程<br/>见下图]
    
    style Start fill:#4caf50,color:#fff
    style Python fill:#2196f3,color:#fff
    style SpawnTask fill:#ff9800,color:#fff
    style AIAnalysis fill:#e91e63,color:#fff
```

---

## 5️⃣ AI分析决策流程图

```mermaid
flowchart TD
    Start([analyze_and_trade开始]) --> Dedup[信号去重检查<br/>30秒内?]
    
    Dedup -->|重复| Skip1[⏭️ 跳过]
    Dedup -->|不重复| RecordTime[记录分析时间]
    
    RecordTime --> Normalize[标准化交易对<br/>BTC→BTCUSDT]
    Normalize --> GetHistory[获取历史表现<br/>12小时内]
    
    GetHistory --> Risk{风险评估}
    Risk -->|高风险| Skip2[❌ 拒绝<br/>历史表现差]
    Risk -->|可接受| GetKlines[获取K线数据]
    
    GetKlines --> K5m[5分钟K线 200根]
    GetKlines --> K15m[15分钟K线 200根]
    GetKlines --> K1h[1小时K线 200根]
    
    K5m --> CheckData{数据完整?}
    K15m --> CheckData
    K1h --> CheckData
    
    CheckData -->|不足| Skip3[❌ 跳过<br/>数据不足]
    CheckData -->|完整| FindZone[查找入场区域<br/>1h支撑位]
    
    FindZone --> HasZone{找到区域?}
    HasZone -->|否| Skip4[❌ 跳过<br/>无明确区域]
    HasZone -->|是| CheckPrice[检查当前价格<br/>是否在区域内]
    
    CheckPrice -->|不在| AddQueue[➕ 加入延迟队列<br/>等待价格回调]
    CheckPrice -->|在区域内| CheckLaunch[检查启动信号<br/>5m趋势强度]
    
    CheckLaunch -->|无启动| AddQueue
    CheckLaunch -->|有启动| BuildPrompt[构建AI提示词<br/>历史+K线+指标]
    
    BuildPrompt --> CallGemini[调用Gemini V2 API<br/>Flash-thinking模式]
    
    CallGemini --> ParseJSON[解析JSON响应]
    ParseJSON --> CheckFormat{格式正确?}
    
    CheckFormat -->|错误| Retry{重试<br/>3次内?}
    Retry -->|是| CallGemini
    Retry -->|否| Skip5[❌ 跳过<br/>AI解析失败]
    
    CheckFormat -->|正确| Decision{AI决策}
    
    Decision -->|ENTER<br/>confidence≥7| CalcSize[计算仓位<br/>基础2U * risk_multiplier]
    Decision -->|WAIT| AddQueue
    Decision -->|SKIP| LogReason[📝 记录原因]
    
    CalcSize --> CheckBalance{余额充足?}
    CheckBalance -->|不足| Skip6[❌ 跳过<br/>余额不足]
    CheckBalance -->|充足| ExecuteEntry[执行开仓<br/>execute_ai_trial_entry]
    
    ExecuteEntry --> PlaceOrder[下限价单<br/>30%仓位试探]
    PlaceOrder --> SetSL[设置止损单]
    SetSL --> SetTP[设置止盈单]
    SetTP --> SaveDB[保存持仓记录]
    SaveDB --> Success[✅ 开仓成功]
    
    LogReason --> End([结束])
    Skip1 --> End
    Skip2 --> End
    Skip3 --> End
    Skip4 --> End
    Skip5 --> End
    Skip6 --> End
    AddQueue --> End
    Success --> End
    
    style Start fill:#4caf50,color:#fff
    style CallGemini fill:#e91e63,color:#fff
    style ExecuteEntry fill:#ff9800,color:#fff
    style Success fill:#4caf50,color:#fff
    style End fill:#9e9e9e,color:#fff
```

---

## 6️⃣ 持仓监控流程图

```mermaid
flowchart TD
    Start([monitor_positions启动]) --> Loop{每180秒循环}
    
    Loop --> GetSnapshot[获取持仓快照<br/>position_trackers]
    GetSnapshot --> HasPositions{有持仓?}
    
    HasPositions -->|无| Cleanup[执行清理任务<br/>每30分钟]
    HasPositions -->|有| ForEach[遍历每个持仓]
    
    ForEach --> GetPrice[获取当前价格<br/>Binance API]
    GetPrice --> CalcProfit[计算盈亏<br/>profit_pct]
    
    CalcProfit --> CalcHoldTime[计算持仓时长<br/>小时数]
    CalcHoldTime --> Phase{持仓阶段}
    
    Phase -->|第1小时<br/>0-1h| Phase1[严格止损策略<br/>SL: -3%, TP: +2%]
    Phase -->|第2小时<br/>1-2h| Phase2[适度策略<br/>SL: -4%, TP: +3%]
    Phase -->|第3小时<br/>2-3h| Phase3[盈利保护<br/>SL: -5%, TP: +5%]
    Phase -->|第4小时<br/>3-4h| Phase4[最后机会<br/>SL: -6%, TP: +8%]
    Phase -->|超过4小时| Timeout[⏰ 超时强制平仓]
    
    Phase1 --> CheckSL{触发止损?}
    Phase2 --> CheckSL
    Phase3 --> CheckSL
    Phase4 --> CheckSL
    
    CheckSL -->|是| CloseSL[🛑 止损平仓<br/>市价全平]
    CheckSL -->|否| CheckTP{触发止盈?}
    
    CheckTP -->|是| Staged{分批止盈}
    CheckTP -->|否| AIEval[AI实时评估<br/>evaluate_position_with_ai]
    
    Staged -->|+2%| Close50[平仓50%]
    Staged -->|+5%| Close75[平仓75%]
    Staged -->|+10%| Close100[平仓100%]
    
    Close50 --> AdjustSL[调整止损<br/>移至成本价]
    Close75 --> AdjustSL
    Close100 --> UpdateDB[更新数据库]
    
    AIEval --> GetKlines2[获取多周期K线<br/>5m/15m/1h]
    GetKlines2 --> BuildPrompt2[构建持仓管理提示词]
    BuildPrompt2 --> CallDeepSeek[调用DeepSeek API]
    
    CallDeepSeek --> ParseAction{AI建议}
    
    ParseAction -->|CLOSE_FULL<br/>立即全平| CloseMarket[市价全平]
    ParseAction -->|CLOSE_PARTIAL<br/>分批减仓| ClosePartial[减仓30-70%]
    ParseAction -->|ADJUST_SL<br/>移动止损| AdjustSL2[移动止损至建议价]
    ParseAction -->|ADJUST_TP<br/>调整止盈| AdjustTP[调整止盈至建议价]
    ParseAction -->|HOLD<br/>继续持有| Hold[保持现状]
    
    CloseMarket --> UpdateDB
    ClosePartial --> UpdateDB
    AdjustSL --> UpdateDB
    AdjustSL2 --> UpdateDB
    AdjustTP --> UpdateDB
    CloseSL --> UpdateDB
    Timeout --> UpdateDB
    
    UpdateDB --> CheckOrders[检查订单状态<br/>止损/止盈单]
    Hold --> CheckOrders
    
    CheckOrders --> NextPosition{还有持仓?}
    NextPosition -->|是| ForEach
    NextPosition -->|否| Cleanup
    
    Cleanup --> MonitorOrders[监控触发订单<br/>每5次检查]
    MonitorOrders --> CleanOrphaned[清理孤儿订单<br/>每10次检查]
    CleanOrphaned --> Loop
    
    style Start fill:#4caf50,color:#fff
    style AIEval fill:#e91e63,color:#fff
    style CallDeepSeek fill:#9c27b0,color:#fff
    style CloseSL fill:#f44336,color:#fff
    style Timeout fill:#ff9800,color:#fff
    style UpdateDB fill:#2196f3,color:#fff
```

---

## 7️⃣ 延迟队列流程图

```mermaid
flowchart TD
    Start([reanalyze_pending_entries启动]) --> Loop{每600秒循环<br/>10分钟}
    
    Loop --> GetQueue[获取待开仓队列<br/>pending_entries]
    GetQueue --> HasEntries{有待处理?}
    
    HasEntries -->|无| Wait[等待下次循环]
    HasEntries -->|有| ForEach[遍历每个待开仓]
    
    ForEach --> CheckRetry{重试次数}
    CheckRetry -->|≥3次| Remove1[移除队列<br/>超过重试上限]
    CheckRetry -->|<3次| CheckAge{加入时长}
    
    CheckAge -->|>24小时| Remove2[移除队列<br/>信号过期]
    CheckAge -->|≤24小时| GetLatestPrice[获取最新价格]
    
    GetLatestPrice --> GetLatestKlines[获取最新K线<br/>5m/15m/1h]
    GetLatestKlines --> RecalcZone[重新计算入场区域]
    
    RecalcZone --> CheckPriceNow{当前价格<br/>在区域内?}
    
    CheckPriceNow -->|不在| Increment[重试次数+1<br/>继续等待]
    CheckPriceNow -->|在区域内| CheckLaunch2[检查启动信号<br/>5m趋势]
    
    CheckLaunch2 -->|无启动| Increment
    CheckLaunch2 -->|有启动| BuildPrompt3[重新构建AI提示词]
    
    BuildPrompt3 --> CallGemini2[调用Gemini V2 API<br/>重新分析]
    CallGemini2 --> ParseDecision{AI决策}
    
    ParseDecision -->|ENTER| ExecuteNow[执行开仓<br/>execute_ai_trial_entry]
    ParseDecision -->|WAIT| Increment
    ParseDecision -->|SKIP| Remove3[移除队列<br/>AI拒绝]
    
    ExecuteNow --> Success{开仓成功?}
    Success -->|是| RemoveSuccess[✅ 移除队列<br/>已成功开仓]
    Success -->|否| Increment
    
    Remove1 --> NextEntry{还有待处理?}
    Remove2 --> NextEntry
    Remove3 --> NextEntry
    RemoveSuccess --> NextEntry
    Increment --> NextEntry
    
    NextEntry -->|是| ForEach
    NextEntry -->|否| CleanupMemory[清理过期缓存]
    
    CleanupMemory --> Loop
    Wait --> Loop
    
    style Start fill:#4caf50,color:#fff
    style CallGemini2 fill:#e91e63,color:#fff
    style ExecuteNow fill:#ff9800,color:#fff
    style RemoveSuccess fill:#4caf50,color:#fff
    style Remove1 fill:#f44336,color:#fff
    style Remove2 fill:#f44336,color:#fff
    style Remove3 fill:#f44336,color:#fff
```

---

## 8️⃣ 数据流向图

```mermaid
graph LR
    subgraph "外部输入"
        A[Telegram群组] --> B[Python监听器]
    end
    
    subgraph "数据库层"
        B --> C[(telegram_signals表)]
        C --> D[Rust轮询器<br/>每5秒]
    end
    
    subgraph "信号处理层"
        D --> E[信号解析器]
        E --> F[TelegramSignal→FundAlert]
        F --> G[analyze_and_trade]
    end
    
    subgraph "AI决策层"
        G --> H[获取K线数据<br/>Binance API]
        H --> I[Gemini V2 API<br/>入场分析]
        I --> J{决策结果}
    end
    
    subgraph "交易执行层"
        J -->|ENTER| K[execute_ai_trial_entry]
        K --> L[Binance API<br/>下单接口]
        L --> M[订单响应]
    end
    
    subgraph "持仓管理层"
        M --> N[(positions表)]
        N --> O[monitor_positions<br/>每180秒]
        O --> P[DeepSeek API<br/>持仓评估]
        P --> Q{管理决策}
    end
    
    subgraph "执行反馈"
        Q -->|CLOSE| L
        Q -->|ADJUST| R[修改止损止盈]
        R --> L
    end
    
    subgraph "延迟重试"
        J -->|WAIT| S[pending_entries队列]
        S --> T[reanalyze_pending_entries<br/>每600秒]
        T --> G
    end
    
    subgraph "Web接口"
        N --> U[Web Server<br/>8080端口]
        U --> V[前端/API调用]
    end
    
    style A fill:#2196f3,color:#fff
    style I fill:#e91e63,color:#fff
    style P fill:#9c27b0,color:#fff
    style L fill:#ff9800,color:#fff
    style N fill:#4caf50,color:#fff
```

---

## 9️⃣ 状态转换图

```mermaid
stateDiagram-v2
    [*] --> 信号接收: Telegram消息
    
    信号接收 --> 信号解析: 存入数据库
    信号解析 --> AI分析中: BUY信号
    信号解析 --> 已忽略: 非BUY信号
    
    AI分析中 --> 开仓执行: ENTER决策
    AI分析中 --> 延迟队列: WAIT决策
    AI分析中 --> 已拒绝: SKIP决策
    
    开仓执行 --> 持仓中: 开仓成功
    开仓执行 --> 开仓失败: 余额不足/API错误
    
    持仓中 --> AI评估中: 每180秒
    
    AI评估中 --> 持仓中: HOLD决策
    AI评估中 --> 调整中: ADJUST决策
    AI评估中 --> 平仓执行: CLOSE决策
    
    调整中 --> 持仓中: 调整完成
    
    持仓中 --> 平仓执行: 触发止损
    持仓中 --> 平仓执行: 触发止盈
    持仓中 --> 平仓执行: 超时4小时
    
    平仓执行 --> 已平仓: 平仓成功
    平仓执行 --> 平仓重试: 平仓失败
    
    平仓重试 --> 已平仓: 重试成功
    平仓重试 --> 持仓中: 重试失败
    
    延迟队列 --> AI分析中: 10分钟重新分析
    延迟队列 --> 已过期: 24小时超时
    延迟队列 --> 已拒绝: 重试3次失败
    
    已平仓 --> [*]
    已忽略 --> [*]
    已拒绝 --> [*]
    开仓失败 --> [*]
    已过期 --> [*]
    
    note right of 持仓中
        4个阶段管理:
        - 第1小时: 严格
        - 第2小时: 适度
        - 第3小时: 保护
        - 第4小时: 最后
    end note
    
    note right of AI分析中
        Gemini V2 Flash
        - 多周期K线
        - 技术指标
        - 历史表现
        - 入场区域
    end note
    
    note right of AI评估中
        DeepSeek
        - 实时评估
        - 动态调整
        - 风险控制
    end note
```

---

## 🔟 时序图 - 完整交易流程

```mermaid
sequenceDiagram
    participant T as Telegram
    participant P as Python监听器
    participant DB as 数据库
    participant R as Rust轮询器
    participant G as Gemini API
    participant B as Binance API
    participant M as 持仓监控
    participant D as DeepSeek API
    
    T->>P: 📱 发送信号消息
    P->>DB: 💾 存入telegram_signals
    
    loop 每5秒轮询
        R->>DB: 🔍 查询未处理信号
        DB-->>R: 返回信号列表
    end
    
    R->>R: 📋 解析并分类
    
    alt BUY信号
        R->>B: 📊 获取K线数据
        B-->>R: 返回5m/15m/1h K线
        
        R->>G: 🤖 AI入场分析
        Note over R,G: 包含历史表现+技术指标
        G-->>R: 返回决策 ENTER/WAIT/SKIP
        
        alt ENTER决策
            R->>B: 📈 下限价单 (30%仓位)
            B-->>R: 返回订单ID
            R->>B: 🛑 设置止损单
            R->>B: 🎯 设置止盈单
            R->>DB: 💾 保存持仓记录
            
            Note over M: 180秒后...
            
            M->>DB: 🔍 查询所有持仓
            M->>B: 💰 获取当前价格
            M->>M: 📊 计算盈亏
            
            alt 需要AI评估
                M->>B: 📊 获取最新K线
                M->>D: 🤖 AI持仓评估
                D-->>M: 返回建议 HOLD/CLOSE/ADJUST
                
                alt CLOSE建议
                    M->>B: 📉 市价平仓
                    B-->>M: 平仓成功
                    M->>DB: 💾 更新持仓状态
                else ADJUST建议
                    M->>B: 🔧 修改止损止盈
                    B-->>M: 修改成功
                    M->>DB: 💾 更新止损止盈
                end
            else 触发止损/止盈
                M->>B: ⚡ 订单已触发
                M->>DB: 💾 更新持仓状态
            end
            
        else WAIT决策
            R->>R: ➕ 加入延迟队列
            
            Note over R: 600秒后...
            
            R->>B: 📊 重新获取K线
            R->>G: 🤖 重新AI分析
            G-->>R: 返回决策
            
        else SKIP决策
            R->>DB: 📝 记录拒绝原因
        end
        
        R->>DB: ✅ 标记信号已处理
    else 非BUY信号
        R->>DB: ⏭️ 标记已处理
    end
```

---

## 📊 关键指标统计

### 时间参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 信号轮询间隔 | 5秒 | 检查新Telegram信号 |
| 持仓监控间隔 | 180秒 | 检查所有持仓状态 |
| 延迟队列间隔 | 600秒 | 重新分析待开仓队列 |
| 订单监控频率 | 每5次持仓检查 | 检查止损止盈订单 |
| 内存清理频率 | 每30分钟 | 清理过期数据 |
| 超时强制平仓 | 4小时 | 持仓超时自动平仓 |
| 信号去重窗口 | 30秒 | 避免重复分析 |
| 信号过期时间 | 24小时 | 延迟队列最长等待 |
| 最大重试次数 | 3次 | 延迟队列重试上限 |

### 风控参数

| 阶段 | 止损 | 止盈 | 说明 |
|------|------|------|------|
| 第1小时 | -3% | +2% | 严格止损 |
| 第2小时 | -4% | +3% | 适度放宽 |
| 第3小时 | -5% | +5% | 盈利保护 |
| 第4小时 | -6% | +8% | 最后机会 |
| 超时 | 强制平仓 | - | 无条件平仓 |

### 仓位参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 初始开仓 | 30% | 试探性建仓 |
| 基础仓位 | 2 USDT | 最小单位 |
| 风险乘数 | 1-3倍 | 根据信号强度 |
| 分批止盈 | 50%/75%/100% | 盈利时减仓 |

---

## 🎯 流程总结

### 核心流程链路

```
1. 信号采集: Telegram → Python → 数据库
2. 信号处理: Rust轮询 → 解析 → 分类
3. AI决策: K线获取 → Gemini分析 → 决策输出
4. 交易执行: Binance下单 → 设置止损止盈
5. 持仓管理: 定期监控 → DeepSeek评估 → 动态调整
6. 延迟重试: 队列管理 → 定时重分析 → 二次机会
```

### 并发执行模型

```
主线程: 保持运行
  ├── 线程1: 持仓监控 (每180秒)
  ├── 线程2: 延迟队列 (每600秒)
  ├── 线程3: Web服务器 (持续监听)
  └── 线程4: 信号轮询 (每5秒)
      └── 子任务: AI分析 (按需spawn)
```

### 数据流向

```
输入: Telegram消息
  ↓
存储: SQLite数据库 (telegram_signals)
  ↓
处理: 解析→分类→转换 (TelegramSignal → FundAlert)
  ↓
分析: AI决策 (Gemini V2)
  ↓
执行: 交易操作 (Binance API)
  ↓
管理: 持仓监控 (DeepSeek评估)
  ↓
输出: 平仓结果 → 数据库 (positions)
```

---

<div align="center">

# 🎨 流程图完成！

**包含9种视图**:
1. ✅ 系统整体架构图
2. ✅ 程序启动流程图
3. ✅ 并发任务架构图
4. ✅ 信号处理流程图
5. ✅ AI分析决策流程图
6. ✅ 持仓监控流程图
7. ✅ 延迟队列流程图
8. ✅ 数据流向图
9. ✅ 状态转换图
10. ✅ 时序图

**使用方法**:
- 在支持Mermaid的Markdown查看器中打开
- 推荐工具: VSCode + Mermaid插件
- 或使用在线查看器: https://mermaid.live/

**生成时间**: 2025-11-29 01:29  
**文档版本**: v1.0

</div>
