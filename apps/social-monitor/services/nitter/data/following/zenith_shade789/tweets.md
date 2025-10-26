# Zenith (@Zenith_shade789)

- 抓取时间：2025-10-21T01:33:06.907Z
- 推文数量：200 / 目标 200
- 抓取耗时：26943 ms
- 数据来源：twitter-Following-1760964620895.json
- 分页次数：11
- 抓取尝试：1
- Cursor 链：10 条
- 账号统计：粉丝 5613，关注 938，推文 2532，点赞 4934
- 站外链接：https://t.co/ARdFt0m5yH
- Twitter：https://twitter.com/Zenith_shade789
- 头像：https://pbs.twimg.com/profile_images/1926227882730217472/Tnxld_rC_normal.jpg
- Banner：https://pbs.twimg.com/profile_banners/1647773808080633857/1748083491

---

## 1. Oct 13, 2025 · 11:19 AM UTC · 1977695634586046610#m
- 链接：https://twitter.com/Zenith_shade789/status/1977695634586046610#m
- 作者：@Zenith_shade789

> 恭喜 $lab

## 2. Oct 12, 2025 · 8:46 PM UTC · 1977475941967483386#m
- 链接：https://twitter.com/Zenith_shade789/status/1977475941967483386#m
- 作者：@Zenith_shade789

> $coai  @heyibinance

## 3. Sep 24, 2025 · 10:30 AM UTC · 1970797950290030665#m
- 链接：https://twitter.com/Zenith_shade789/status/1970797950290030665#m
- 作者：@Zenith_shade789

> 🎉 Hubble Giveaway — 300 USDT Prize Pool 🎉
>  
> We’re celebrating our participation at #TOKEN2049 Singapore with a community giveaway — join in!
>  
> How to enter:
> 1) Follow @MeetHubble
> 2) Like + RT this post + Tag 3 friends
> 3) Reply with: what’s Hubble to you OR a create a meme of Hubbo, our owl mascot 🦉 (AI welcome)
>  
> 🏆 Prizes (based on creativity & how well it fits Hubble):
> – 1st prize: 100 USDT
> – 2nd prize: 2 winners × 50 USDT
> – 3rd prize: 5 winners × 20 USDT
>  
> ⏰ Ends Oct 2nd 2025

## 4. Sep 20, 2025 · 10:07 AM UTC · 1969342565938843847#m
- 链接：https://twitter.com/Zenith_shade789/status/1969342565938843847#m
- 作者：@Zenith_shade789

> 中文区都不打嘛？不打我打

## 5. Sep 10, 2025 · 4:30 PM UTC · 1965815098402107771#m
- 链接：https://twitter.com/Zenith_shade789/status/1965815098402107771#m
- 作者：@Zenith_shade789

> 别再点赞了人家都吓的给自己地址停了

## 6. Sep 9, 2025 · 12:49 PM UTC · 1965397215016595537#m
- 链接：https://twitter.com/Zenith_shade789/status/1965397215016595537#m
- 作者：@Zenith_shade789

> 全自动印钞机是否存在？来看看以下全自动印钞地址如何实现的。
> 上周末在刷某积分在链上随手创建了代币刷完几个号后勾子的发现家里进鬼了到底谁偷了我的钱顺着链上交易哈希找到了以下地址
> 地址1：Dsi8ntQziuCPt16TStjG4tgviracSryHsRcH72zbwRgu（gmgn显示钱包创建118d盈利+$694K）
> 地址2：CqPRMxSgUYX5UQiY44ttdhPCqh6sZfEGJvmQFD6kNB22（gmgn显示钱包创建5d盈利+$33.9K）
> 地址3：39H3DGBpHpffjTuwQDR9yv9AgbK4U4hesLdsVZ9yDDc9（gmgn显示钱包创建75d盈利+$1.4M）
> 如果你去Solanascan检查了这些地址你很容易发现他们都出现了setLoadedAccountsDataSizeLimit（调节交易执行时可加载的账户数据的总大小上限。）
> 是的就是字面意思可自定义加载账户数据上限（cu），为什么要调节呢？
> Solana 的一个 block 本质上就是 validator 打包的一堆交易。
> 但每个 block 有“硬性约束”，主要是：
> compute units 上限：一个 block 总共能执行的 CU 有限（大约 48M CU/slot）。
> 内存上限：所有交易加载的账户数据不能超过 validator 给定的内存预算。
> 并行性约束：同一个账户如果被多个交易写，就只能串行。
> 这意味着：
> 如果你的交易“看起来要用很多账户数据”，即便你实际没用那么多，系统也会先预留一大块内存预算给你 → 占用 block 的资源上限 → 成本（C）变大 → 优先级（P）下降。
> P = 优先级分数（越高越容易进 block）
>  
> R = 交易愿意支付的奖励（priority fee，总 lamports）
>  
> C = 交易对 block 限制的消耗成本（以 CU 为度量，不止 compute units）
> 举例哈希：explorer.solana.com/tx/57PPc…
>  
> 简单点理解你点了同城配送在同样小费下你需要派送的物件是很小的那么被接单速度将会很快，同样价格配送人员是愿意送一件水上楼还是愿意送一盒byt上楼呢？lamooo
>  
> 这就是为什么以上地址能够印钞原因，当然你可以模仿他们以及优化它快速狙击新币。有人问能定点收割这类sniper bot吗？当然我尝试过从pump发射一枚新币到丢弃损失在8美金，如果我拿过多的筹码即内盘市值过高那么他们将不会狙击它，在我寻找原因时发现是超出了它们的setComputeUnitLimit、setComputeUnitPrice限制因此你可以发现它们狙击的代币市值不会很高也变相的防止了被收割情况毕竟不会有人傻到亏钱去做这种事。
>  
> 当然setLoadedAccountsDataSizeLimit的作用不止于此可以大胆想象去尝试开始印钞。

## 7. Sep 8, 2025 · 2:14 AM UTC · 1964874859231818191#m
- 链接：https://twitter.com/Zenith_shade789/status/1964874859231818191#m
- 作者：@Zenith_shade789

> $BDOG 这是干嘛？200x了

## 8. Sep 7, 2025 · 4:34 AM UTC · 1964547867886133429#m
- 链接：https://twitter.com/Zenith_shade789/status/1964547867886133429#m
- 作者：@Zenith_shade789

> 如果我现在把 @0xcarl_9 0.1825 $wlfi 多单平掉他会不会捅我菊啊

## 9. Sep 6, 2025 · 1:01 PM UTC · 1964313009477574956#m
- 链接：https://twitter.com/Zenith_shade789/status/1964313009477574956#m
- 作者：@Zenith_shade789

> 现在都这么饥渴的吗？
> A:Z哥咋不开微信群了？
> 我：还没想好怎么骗
> A:随便怎么骗，记得叫我

## 10. Sep 2, 2025 · 4:53 PM UTC · 1962921848431399289#m
- 链接：https://twitter.com/Zenith_shade789/status/1962921848431399289#m
- 作者：@Zenith_shade789

> 跟着仙龙掌
> 买了一点 $MDOG
> 老徐不敢做的事情，让BitGet来做！！！
> 冲！

## 11. Sep 1, 2025 · 2:00 PM UTC · 1962515999594553706#m
- 链接：https://twitter.com/Zenith_shade789/status/1962515999594553706#m
- 作者：@Zenith_shade789

> $launchcoin, $wlfi 几天前在深圳和表姐吃饭时候自己开完，按头开的，至于开仓理由就不提了推上大把老师分析思路。
> 深圳老师太多了待了一段时间也和一些老师面基了。值得分享的点在于周围可交流的人多了起来信息流通快，意外的几个老表也在我大学周边非常近，可惜认识晚了点。
> @10UWINA8
> @0xLonelyGod
> @XRZeth
> @0xsnake_
> 都是04 05的帅哥

## 12. Aug 28, 2025 · 5:42 AM UTC · 1960940946134327563#m
- 链接：https://twitter.com/Zenith_shade789/status/1960940946134327563#m
- 作者：@Zenith_shade789

> 好久没看到三梦了，上次看到还是朋友在外围群看到🤔

## 13. Aug 28, 2025 · 4:49 AM UTC · 1960927704766877876#m
- 链接：https://twitter.com/Zenith_shade789/status/1960927704766877876#m
- 作者：@Zenith_shade789

> 难怪都说00后抗压能力强，还说什么跳了兄弟

## 14. Aug 26, 2025 · 12:59 AM UTC · 1960144935719764378#m
- 链接：https://twitter.com/Zenith_shade789/status/1960144935719764378#m
- 作者：@Zenith_shade789

> 还是喜欢那些地推cx的只教买不教卖，不对是布道者

## 15. Aug 20, 2025 · 6:10 AM UTC · 1958048907596034385#m
- 链接：https://twitter.com/Zenith_shade789/status/1958048907596034385#m
- 作者：@Zenith_shade789

> cnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnmcnm thk

## 16. Aug 19, 2025 · 3:39 AM UTC · 1957648507726360814#m
- 链接：https://twitter.com/Zenith_shade789/status/1957648507726360814#m
- 作者：@Zenith_shade789

> It’s angel sh*t — blessed by heaven, minted for us.🤣
> 🪽 🪽🪽

## 17. Aug 18, 2025 · 5:21 PM UTC · 1957493178250219829#m
- 链接：https://twitter.com/Zenith_shade789/status/1957493178250219829#m
- 作者：@Zenith_shade789

> 最新动态一览@heavendex
> 8月19日主币 $light 已经销毁了约$1.5m，份额2%
> 大盘下跌的时候 $light 走的也挺稳。以及这个台子真的不适合pvp，目前观察pvp很难赚钱，适合找有社区/产品慢煮的
>  
> @heavendex是由 Solana 基金会的黑客马拉松部门@Colosseum支持，集发币➕自动做市商 (AMM)➕飞轮回购一体的平台，具体介绍可以看这篇（https://x.字com/btcly17/status/1956912995466117435）
>  
> 🪽🪽🪽🪽🪽数据情况🪽🪽🪽🪽🪽
> 💰【收入方面】
> 日收入目前仅次于pump，稳定增长中，期待看到大柱子！
> 手续费将回购销毁 $light
> 对于价值超过 10 m的货币，@heavendex每次交易的收入比 Pumpfun 多 5 倍左右，而且用户无需支付任何费用
>  
> 👼【活跃地址】
> 地址稳定增长1500+，删掉持币数量120刀以下的活跃地址是4300个左右，陆陆续续有人进来了，但大部分人还是没参与
>  
> 社区情绪也在bullish中 @cookiedotfun
>  
> 🪽🪽🪽🪽🪽运营情况🪽🪽🪽🪽🪽
> 🩵【最新动态】
> 1、平台新增了看板可以更直观看的看到最新的、低市值的和高市值的面板
> 其实可以从里面找一些概念还不错的小彩票埋伏，今天看到个从归零拉起来好猛的 $toro
>  
> 2、平台的基金会Starseed将对优秀项目进行购买支持，比如 ICM、表情包以及各种类型的代币。ico融了22m应该值得期待一把
> Starseed的具体规则马上就要上线公布了，到时候我也会第一时间进行研究👀
>  
> 3、据创始人说，已经有120+项目类在平台申请上线了，个人比较期待里面跑出来大市值项目
>  
> Let there be light
> 777  Godspeed🪽

## 18. Aug 17, 2025 · 2:28 PM UTC · 1957087211415073168#m
- 链接：https://twitter.com/Zenith_shade789/status/1957087211415073168#m
- 作者：@Zenith_shade789

> 两周前我使用 @MeetHubble  Hubble Signal tg进行地址重合扫描时发现与troll地址与多枚代币地址高度重叠，比如天王星 $URANUS 以及 $memecoin 等。
> $uranus 地址重叠度高达10%，并且在监控的地址集群并未发生持仓下降，这点得夸夸tg bot大多时间无法观察直观观察到持仓下降而Hubble tg bot恰好解决了这个问题，地址集群持仓下降通知，以及多代币地址查重比对做的很好。
> 最后发一张老图

## 19. Aug 15, 2025 · 2:11 PM UTC · 1956358062421516385#m
- 链接：https://twitter.com/Zenith_shade789/status/1956358062421516385#m
- 作者：@Zenith_shade789

> 最近问我关于区块链的大学同窗也多了起来什么rwa系列的，估计应该是他们公司涉及到相关业务，正好想到了我。也确实挺有意思的当时她认为我搞的是zp，因为玩的好非常担心我被抓起来，那天正好和女友在约会也就把她丢给 Carl 了。
> 至于carl一段时间前找我请教关于web3的一些事情，说来也奇怪他是我的大学室友三年前当时强拉着他以及另外一个室友和我一起但是他选择了体验大学生活。现在羡慕上了另外一个室友阿涵，阿涵属于稳定盈利那种收益曲线也是我追求不来的，性格不同导致策略性存在差异，涵没有玩过一次链上，这点我问他为什么。经典的回答“我只做我擅长的”😂 多少人追求不来的，凌乱的时间线，嘈杂的噪音，以至于无法发现自己擅长的点。
> 刚看到 关于Carl $og 空单被套的帖子也挺无奈的，因为我和他同时开的，意味着我也被套了。当然好在上次教学他的策略起到了作用扭亏为盈。就像解题答案时唯一的解题过程五花八门，补仓拉均价是最笨的解法。
> 可惜30天胜率没他高了🫥

## 20. Aug 6, 2025 · 10:46 AM UTC · 1953045118614925595#m
- 链接：https://twitter.com/Zenith_shade789/status/1953045118614925595#m
- 作者：@Zenith_shade789

> 🚨 Hubble Signal 庄家监控Bot使用指南，链上猎手必备工具！直达链接👉 t.me/hubble_data_bot
>  
> 🔥 这不是给链上萌新的玩具，而是为这轮牛市中的Alpha猎手量身打造的神器！你准备好狙击庄家动向了么？
>  
> 🎁 限时福利：参与体验抽奖，100U 奖池，5位幸运用户瓜分：
> 1️⃣ 点赞+转发我👇引用的官号推文
> 2️⃣ 官号推文评论区分享你的Signal使用体验+截图
>  
> ⏰ 48小时后开奖！链上玩家速冲！
>  
> @MeetHubbleCN @MeetHubble

## 21. Aug 6, 2025 · 9:58 AM UTC · 1953033044467601506#m
- 链接：https://twitter.com/Zenith_shade789/status/1953033044467601506#m
- 作者：@Zenith_shade789

> $troll 马上190m了，为什么在我置顶快4个月了从20m之后我硬是没点开看过？

## 22. Jul 30, 2025 · 10:33 AM UTC · 1950504941451452738#m
- 链接：https://twitter.com/Zenith_shade789/status/1950504941451452738#m
- 作者：@Zenith_shade789

> Stashing cash in a bank? Enjoy ur 0.01% APY and stale toast.🤡
> $BANK gives u dreams.
> Bread & milk vs mansions & machines.🤣

## 23. Jul 30, 2025 · 9:47 AM UTC · 1950493409577460076#m
- 链接：https://twitter.com/Zenith_shade789/status/1950493409577460076#m
- 作者：@Zenith_shade789

> W nt degen tkt？

## 24. Jul 29, 2025 · 5:56 AM UTC · 1950073015607099392#m
- 链接：https://twitter.com/Zenith_shade789/status/1950073015607099392#m
- 作者：@Zenith_shade789

> $1.9 million on the tag — smack dab in the sweet zone, $Bonkyo got that déjà vu vibe with a twist.

## 25. Jul 17, 2025 · 5:43 PM UTC · 1945902095884017788#m
- 链接：https://twitter.com/Zenith_shade789/status/1945902095884017788#m
- 作者：@Zenith_shade789

> @derp_trade

## 26. Jul 17, 2025 · 5:41 PM UTC · 1945901725891891203#m
- 链接：https://twitter.com/Zenith_shade789/status/1945901725891891203#m
- 作者：@Zenith_shade789

> streaming = new age clout engine @trendsdotfun
>  
> Any tweet could be a token, fr fr.
> If y’all caught my last tweet — that was the riddle.
> Boxing yourself in? u’ll never make it big like that.
>  
> Now imagine this:
> What if anyone could spin up time-limited derivative contracts on hyped tokens? That’d be wild.
> Think synthetic reserves, relay vaults, and multi-oracle weighted setups — it’s a stretch, but it’s a move.
> People love jumping on trends.
> Let ’em mint phase-based token contracts, share hot takes, and get paid for the sauce. Crazy? Maybe.
> But using streaming as leverage to boost token clout?
> The alpha’s in plain sight, you just ain’t copied it yet.
> @smartdotfun

## 27. Jul 17, 2025 · 11:55 AM UTC · 1945814674790055947#m
- 链接：https://twitter.com/Zenith_shade789/status/1945814674790055947#m
- 作者：@Zenith_shade789

> 《  Long or Short？》
> 正好有研究过代币合约衍生品相关内容，借运气超好的宝宝  @btcly17 的帖子补充些内容。
>  
> CEX我们常见的进行永续/期货合约多空场所
> 采用集中式撮合：订单进撮合引擎 (内存订单簿)，撮合后更新内部账本。
> 内部结算层：交易结果先记在数据库，周期性上链（如 Proof-of-Reserves）或不上链。
> 统一账户系统：现货、期货、杠杆、资金划转全部在同一清算后台；风险引擎跨产品净额化。
> 以及做市商挂海量限价单 → 深度厚，滑点小，大单可请求 RFQ / Block Trade，撮合价直接由订单簿决定，标记价另算 (防止操纵)。
> Cross vs Isolated Margin：多产品共享保证金 / 独立仓等
>  
> 与链上合约平台有所不同典型的
> 比如Hyperliquid，
> 采用独立链 + 模块化共识
> Hyperchain 使用高性能共识，针对高频交易提供了优化。
> 订单撮合在链外撮合引擎完成，但交易最终状态上链，一定程度上保证安全性和透明度。
>  
> 订单簿撮合逻辑核心区别于 AMM：Hyperliquid 不走 Bonding Curve，采用传统订单簿（挂单/吃单），撮合后将状态同步到链上。
> 撮合由 验证器集群 或 Hyperliquid Sequencer 完成，最终交易记录通过共识写入链。
>  
> 两者区别
> 1.用户心智
> CEX：撮合=内存订单簿 → 数据库账本；链下风险引擎；用户 IOU。
> Hyperliquid：专用链（Hyperchain）+ 原生链级订单簿共识；撮合产生“执行事件”，再写状态；资产非托管。
> CEX=信任运营商，Hyperliquid=验证链状态。
>  
> 2. 系统分层对照
>  
> 3.撮合模型
> CEX: 毫秒级撮合；撮合先于风险；撮合成功 → 数据库写成交；定期风控扫描。
> Hyperliquid: 链式排序（sequencer/validators）接收批次订单；打包撮合；将撮合结果写入区块；所有成交透明可验证。
> → 性能靠“专用执行层 + 并行优化”，不是通用 L1 的慢吞吞 TPS。
>  
> 4. 托管 & 出入金
> CEX：充值 → 平台托管；内部子账户流水；提现需提审。
> Hyperliquid：资产存在协议控制的链上抵押账户；你签名才动；链可审计总资产与负债；“无对手方挪用”承诺可验证（前提：合约安全）。
>  
> 5.账户模型
> CEX：balance, available, margin, PnL, maintenance 都是数据库字段；你看到的是 UI 显示的内部数值。
> Hyperliquid：这些字段对应链上状态槽或合约变量；第三方可重算；理论上可构建开源风险面板。
>  
> 6.保证金体系
> CEX：Cross / Isolated Margin；未实现盈亏实时记入可用保证金；同一资产集约抵押。
> Hyperliquid：原生多市场保证金共享（设计目标类似 CEX cross margin）；抵押资产（USDC 等）绑定链上仓位；风险计算链上执行/验证。
>  
> 7.杠杆 & 强平
> CEX：标的价跌穿维持保证金 → 部分减仓或全平 → 保险基金 & ADL。
> Hyperliquid：链上计算账户权益 / IM / MM；触发阈值由任何清算人或协议执行；资金流透明。
> 重点：清算成交路径可链查，减少“暗箱爆仓”争议。
>  
> 8.标记价 & Oracle
> CEX：内部指数（多平台成交价加权）→ Mark Price；防止平台内刷单操纵爆仓。
> Hyperliquid：链上指数（多源预言机 + 内部逻辑）；用于保证金、资金费率、清算触发。
>  
> 9.资金费率 (Funding)
> CEX 永续经典式：根据 perp 与指数价基差，多空互付，每 4小时结算一次（或动态）。
> Hyperliquid：复用类似逻辑（指数 vs 合约标记价偏离 → 资金费率）；链上可审计历史 funding 转移；参数由治理调整。
> 可独立复核“到底收了我多少钱”。
>  
> 10.性能与延迟
> CEX：撮合在内存；可达亚毫秒；用户体验基本无对手。
> Hyperliquid：牺牲部分延迟换可验证性；靠专用链提升吞吐；撮合批次化；延迟 > CEX 但 < 普通 L1 DEX（目标接近“人类可感知无差”级别）。
> 适合策略交易，但超高频 HFT 在链上依旧受限。
>  
> 11.可组合性 & 透明度
> CEX：封闭黑箱；API 查询；资产不 composable。
> Hyperliquid：链上数据可被索引；可与 DeFi、分析仪表盘、链上清算机器人对接；未来可支持“自动对冲 / 金库策略 / 社交跟单”。（暗池有待支持）
>  
> 12.风险面对照
>  
> 两者之间各有所长。
>  
> 以上两类都不是AMM 类， @btcly17 最近看到AMM类平台 @derp_trade，@smartdotfun （3AC老板zhusu关注有概率投了），@Quanto_cn （前身oxfun）
>  
> @derp_trade
> 智能合约与协议核心
> DERP 合约：采用 AMM 流动性模型，支持任何基于 oracle 的资产 。
> AMM 模式：
> 无订单簿，价格由智能合约依据资产价格和市场 skew 调整设定。
> 实现定向调节：当多空方向偏向一方，AMM 将调整 funding rate 和价格以诱导反向交易，稳定市场
> 流动性 & 风险机制：
> AMM 不可清算，流动性不足时，按比例缩减 PnL 分配（有点类似 Uniswap 的价格冲击）
> 引入了claimable value机制防止用户提取超过 LP 价值部分，部署 skew 调节机制及过度资本化策略优化风控
>  
> 操作流程：
> 前端界面或 SDK 调用 openPosition({sizeUSD, leverage, side, token})；
> 通过 SDK 发起交易请求，将生成交易 TX 并广播至 Solana；
> 智能合约匹配交易、调整 skew、计算费用、更新资金状态；
> 资金与清算机制：
> 初始保证金设为 1/leverage，维护保证金为 0.5 * max initial margin
> AMM 不会过度清算，资产波动剧烈，按比例削减 AMM 的赔付能力来保护系统稳定 。
>  
> 底层技术架构图
>  
> 尽管无需基础资产作为抵押，使用 skew 驱动机制限制不对称风险且每个市场隔离，控制风险范围 。
> 但是依赖外部 oracle，如 Pyth，理论上存在被攻击或延迟风险 。
>  
> @smartdotfun
> 目前暂未透露太多信息，理解偏向于借贷做空概念存入 ORA 作为抵押品，借入智能币，在市场上出售。（风格上有点偏向于传统AMM以及借贷形式）
>  
> @Quanto_cn
> 底层架构
>  
> 存入任意资产作为抵押（BTC、ETH、SOL、USDC、meme‑coins 等）进行杠杆交易
> 盈亏、保证金、清算均使用 QTO；平台手续费 70% 销毁、30% 分配给 QLP（流动性提供者）
>  
> 以上三家本质上采用预言机来控制协议结果
>  
> 那么基于预言机理论上可能实现的
> Derp Oracle 拉升诱空
> 攻击者在 Oracle 数据源（薄 DEX）拉升小币价。
> Derp 读到高价 → DERP 合约价格上移。
> 攻击者大量开空（AMM 接多）。
> 当真实价格回落，空单获利；AMM 亏损，池资金流出。基础因：价格完全信任 Oracle。
>  
> Smartfun 榜单诱导（信息 + 微结构）
> 外部假成交推高估值；
> 前端榜单升温，用户 FOMO；
> 内部池无深度，早期买单抬价；
> 项目方或机器人出货。资料不足，属推测风险。
>  
> Quanto 双触发清算
> 攻击者借多资产抵押高杠杆仓位；
> 同时在外部市场打压抵押资产 & 拉升 $QTO（或反向）以挤占保证金；
> 指数价轻微波动 + $QTO 价变化 → 账户维持率不足 → 连锁部分清算；
> 清算过程自动将非 QTO 抵押卖成 $QTO，形成更多市场冲击。
>  
> 仅偏向于理论实现不建议尝试，如果有透露更多信息更好。
> 或许这也是个方向有待验证点在于市场导向，基于自身平台进行市场拓展困难度大大提升。

## 28. Jul 17, 2025 · 11:25 AM UTC · 1945807147947937899#m
- 链接：https://twitter.com/Zenith_shade789/status/1945807147947937899#m
- 作者：@Zenith_shade789

> 3月份建议定投 $eth 并不毫无依据如果你了解以下政策红利以及市值估值模型也就有了理由
>  
> 《Responsible Financial Innovation Act》（Lummis‑Gillibrand）第116条：要求 SEC 加速审理现货加密 ETF 申请
> • 《Digital Commodities Exchange Act》（DCEA）：将 ETH 纳入 CFTC 商品监管，巩固商品属性
> • 欧盟 MiCA 自 2024 年 12 月 30 日生效：统一牌照 & 披露，机构大举入场
> • 2025 年 5 月 SEC 公司金融部声明：Custodial Staking 免于注册（进一步验证了理解的政策红利）
>  
> EIP‑1559 基础费燃烧机制
> 自 2021 年8 月实施以来，每笔交易的“基础费”被销毁，理论上在链上活动高峰期可实现净通缩，强化 ETH 的稀缺性与价值支撑。
>  
> 那么基于Metcalfe’s Law（梅特卡夫定律）
> 网络价值 ∝（活跃节点数）²
> 2025 年上半年：
> 日活跃地址数 ≈ 87 百万
> 日交易笔数 ≈ 1.2 百万
> 预期年增长率 ≈ 35%
> 拟合效果：R² ≈ 0.83（历史上与 ETH 价格走势高度相关）
> 理论估值：基于上述增长趋势，ETH 公允价位大约在 $2,200 – $2,500 区间，高于当时市场价格。
>  
> Network Value to Transactions (NVT) 比率
> NVT = 市值 / (日交易量 × 价格)
> 2025 年初 ETH NVT 比率远低于历史均值（约 120 – 140），而不是今年后期的高点 194，表明价格与网络使用存在“被低估”空间。
> 低于长期均值的 NVT 往往对应“性价比优”的买入窗口。
>  
> Stock‑to‑Flow（S2F）稀缺性模型
> 在 PoS 模式下，S2F = 流通总量 / 年新增发行量
> 当时 S2F ≈ 0.07（对比 BTC ≈ 0.42）
> 预计质押转向后，S2F 将进一步下降至 0.05 以下
> 按照 ERC‑20 传统 S2F 方法，ETH 的“稀缺性溢价”尚未完全计入，公允价位在 $2,000 – $2,800。
>  
> $eth 低于2000无脑入，今天也是站起来了

## 29. Jul 17, 2025 · 10:35 AM UTC · 1945794480931742096#m
- 链接：https://twitter.com/Zenith_shade789/status/1945794480931742096#m
- 作者：@Zenith_shade789

> e 卫兵站起来说话

## 30. Jul 17, 2025 · 10:34 AM UTC · 1945794333099389337#m
- 链接：https://twitter.com/Zenith_shade789/status/1945794333099389337#m
- 作者：@Zenith_shade789

> $eth 你们会回来看的

## 31. Jul 15, 2025 · 2:59 PM UTC · 1945136043230945422#m
- 链接：https://twitter.com/Zenith_shade789/status/1945136043230945422#m
- 作者：@Zenith_shade789

> 大家都领到了 $c 空投吧,答案写在纸上照抄就行了。
> 这套合约上线前估值对于大多数山寨都挺适用的。
> 如果想抄底些严重低于估值的代币可以看看0富这条推特总会捡漏几个的，在一级市场应该是叫买到彩票币了是吧😂

## 32. Jul 15, 2025 · 10:15 AM UTC · 1945064578993258745#m
- 链接：https://twitter.com/Zenith_shade789/status/1945064578993258745#m
- 作者：@Zenith_shade789

> 昨天关注这个项目挺久宝宝发现自己漏上了 $c 这个标地，下午发看到 $c 即将在bn上线合约。
> 简单做了fdv估值计算公式如下
> FDVpeak​=Base FDV×FOMO Multiplier​
> （Base FDV=Recent Round Valuation or Market FDV）
> （FOMO Multiplier=Exchange Multiplier×Narrative Strength Multiplier×Liquidity Surge Multiplier）
> 基础数据汇总：（未上线合约前）
> 指标数值当前流通市值（Mcap）$29,735,700
> 完全稀释估值（FDV）$187,000,000
> 总供应量1,000,000,000 C
> 当前 C 价格（估算）$0.187 / C
> 当前流动性$1,813,700
> 持币地址数2,303
> 已披露融资金额$15,000,000（2024年7月，A轮）
>  
> 分配项百分比数量（C）解锁情况
> 假设社区 + 生态 40% 400,000,000部分解锁（激励分期）
> 空投奖励 13% 130,000,000部分已发放
> 员工激励 12% 120,000,000Cliff + Vesting（未解锁）
> 早期支持者 17% 170,000,000 部分解锁（按融资时间推算）
> 核心贡献者 15% 150,000,000 多数应未解锁
> LP流动性 3% 30,000,000基本已全部流通
> 结合bn给到的数据当前市值占 FDV 的比例为 15.9%，意味着实际流通仅占约 15-20%，其余仍处于锁仓或激励状态
> 在上合约之前fdv属于略高合理范围，我们再来算下上线合约后短期内炒作区间
> FOMO Multiplier=1.5×1×1.2=1.8
> 因子：
> Exchange Multiplier：1.5
> Narrative Strength：1
> Liquidity Surge：1.2
> FDVpeak​≈150M×1.8≈270m
> 意思简单点短期内fomo上限位于270m左右，反过来逆推维持270m市值需要Chainbase 每年收入 高于 $8.75M，这对于b端来讲过于理想化了。
> 不给我发空投我自己领

## 33. Jul 5, 2025 · 6:25 PM UTC · 1941564229930422591#m
- 链接：https://twitter.com/Zenith_shade789/status/1941564229930422591#m
- 作者：@Zenith_shade789

> 如果bsc来了个Tom会如何呢？

## 34. Jul 2, 2025 · 1:22 PM UTC · 1940400658399846754#m
- 链接：https://twitter.com/Zenith_shade789/status/1940400658399846754#m
- 作者：@Zenith_shade789

> 回忆只会惩罚恋旧的人，沉默成本不参与重大决策
>  
> 00后毕业后并没有选择上班，回家乡开启了实体之路在园区投资了一家电商公司困难重重小地方的思维习惯也注定了最终的结果于今年 6 月走向破产清算，也因为这个包袱限制了太多也失去了最珍贵的。
>  
> 最近换个新环境路过母校北门翻到一张当时因业务需要就以我公司名义买的 macan照片胆子也挺大没有驾照开着上路
>  
> 同样的位置再也不见那个年轻的我了，想来在校三年自己似乎对于传统教育知识接受程度不高每天都想更近一步，有时候挺感慨自己没有接触这些事物多好，多体验体验校园生活谈一场校园恋爱或许也不错！

## 35. Jun 21, 2025 · 5:44 AM UTC · 1936299183818109429#m
- 链接：https://twitter.com/Zenith_shade789/status/1936299183818109429#m
- 作者：@Zenith_shade789

> 未来即是当下 感谢朋友 @pandaskiing 送来的🍑，每年James 都有福利小礼物。一切尚早，愿 @ForgeX_tools  一路长虹， solana战壕只是开始...
>  
> James什么时候来个🍑memecoin猫猫狗狗玩腻了,Lamo🤣

## 36. Jun 20, 2025 · 8:26 AM UTC · 1935977478184669606#m
- 链接：https://twitter.com/Zenith_shade789/status/1935977478184669606#m
- 作者：@Zenith_shade789

> 🚀Hubble MVP Soft Launch: Phase 1 Begins on June 24!
> Hubble MVP is coming! 🎉 Our AI-powered on-chain intelligence data layer is opening early access to a select group for testing.
>  
> 🔥 Why it's a big deal:
> ➡️ First look at Real-time Solana token dashboards
> ➡️ Hubble AI Assistant for instant, Smart Data Insights
> ➡️ Interact with dashboards using Natural Language
>  
> 💡 This is only Phase 1!
>  
> We'll be:
> ✅ Collecting feedback from early users
> ✅ Iterating & improving fast
> ✅ Gradually opening access to more users
>  
> 😎 RT if you believe in a future where anyone can master on-chain intelligence.

## 37. Jun 16, 2025 · 4:07 PM UTC · 1934643919880593668#m
- 链接：https://twitter.com/Zenith_shade789/status/1934643919880593668#m
- 作者：@Zenith_shade789

> 水果滞销，帮帮大爷？那我们来真的。
>  
> 盘圈的老师们，比如学长@cryptoskanda的《开源镰刀》，和@dov_wo哥的《芒果寓言篇》最喜欢用“水果滞销，大爷跪地求助”来比喻开盘逻辑，大爷和水果早就是盘学的具象符号，也可以说是对现代经济学叙事的一个讽刺式解构。
>  
> “若不懂做市，则在战壕无意义” -- ForgeX  @ForgeX_tools
>  
> ForgeX的定位，一开始是做链上自做市基础设施，解决项目方和meme项目的信任问题、技术问题。结果我们宣传着宣传着，大家就把我们当“狗庄工具”了，说我们是在炒"水果"的。
>  
> 但我们同时也想做战壕里的文豪，希望教会每一位P将军们狗庄是怎么画线的，怎么套你的！如果能学会，我希望这是在每一场PVP战斗中能增加你win rate的技能！
>  
> 话说回来，今天panda主播为大家带来的是家乡特产：脆桃🍑
>  
> 为什么会选水蜜桃呢？是因为....
>  
> 前两天我回外公外婆家，在果园里边摘边随口问了句，“现在桃子好卖吗？”外公头也没抬地说：“现在没人收，村上好几家专门种桃子的都卖不出去，连两块钱一斤都没人要，大概率都要烂在地里。”我一听，心里咯瞪一下，因为大部分我们这边的果农幸苦一年赚得钱都没大伙一次仓位波动的大。再加上在澳洲时，超市卖的4400系列水蜜桃是3.5澳币一 公斤，换算下来是8.2块/斤，还不是特别水多。我们村这批桃子红得发紫，不吹不黑，直接撞色ForgeX的紫。那还想啥，得干。
>  
> 我现场就下单买了老乡家10大箱新鲜脆桃，差不多一千个果，开始联系顺丰做生鲜发货、安排筛果、包装、设计... 一通忙活完，最终筛出约700个优质果子，按每盒12个桃子来算，我们能做出50盒。
>  
> 这50盒中，已经预留了20盒给现有投资人、合作伙伴和ForgeX团队全员。
>  
> 剩下的30盒，决定送给ForgeX的社区朋友们，也当是一次特别的活动纪念，每一盒桃子里，我们还放了一张限量印制的ForgeX限量专属50%手续费+20%返佣二维码卡片，求求大家伙了，一起帮帮"果农"，不仅老乡需要帮助，我们枯竭的链上也需要更多的从供给侧发力，更多更好的角度。
>  
> 多推荐推荐想优先想在Solana链上发行资产的朋友，无论是meme、正经TGE、测试币、发行吉祥物，我们都欢迎，因为我相信我们一定是能助力优质Dev们成功的链上做市终端。
>  
> 最后说段正经的升华一下...
>  
> 我们相信，在这个信息被极度加速、信任被不断重构的时代，Web3并非只能用于投机与套利。哪怕在价值可以被实时提现、协作被压缩进区块时间的今天，人与人之间真正的信任与善意，也依然可以穿越链上与现实世界。我们用一次真实的采购和派送，回应那些关于"坐庄""滞销"的盘学隐喻；也尝试在一个技术冷感、逐利主义的世界里，用最原始的果实，把"抽象的协作"重新变得具象可触。哪怕只是一次小事，我们愿意相信，web3的尽头，不一定是无尽的赌，也可以是温度。
>  
> 我们做的不只是救一颗桃子...

## 38. Jun 11, 2025 · 3:45 PM UTC · 1932826564422463635#m
- 链接：https://twitter.com/Zenith_shade789/status/1932826564422463635#m
- 作者：@Zenith_shade789

> $aura 有时候就是那么巧合一年前 Marcell 还不出名时候大家一起push，不好意思全部抛售代币后面归零也就没在意了。
> 今天男高 @ZephyrTrading 聊天一瞬间想起来似乎还有代币没出售完，翻十几分钟地址找到了遗忘地址，是个意外惊喜了。

## 39. Jun 9, 2025 · 4:03 PM UTC · 1932106228878221723#m
- 链接：https://twitter.com/Zenith_shade789/status/1932106228878221723#m
- 作者：@Zenith_shade789

> 园区我只相信越南园区 因为包赔

## 40. Jun 7, 2025 · 5:58 AM UTC · 1931229167741211037#m
- 链接：https://twitter.com/Zenith_shade789/status/1931229167741211037#m
- 作者：@Zenith_shade789

> 内容不适 请谨慎阅览
>  
> 先来复盘最近喊单的memecoin， $MASK , $joobi 。
> $mask 进场喊单位置位于3m左右具体应该是3.47m我的位置，公开喊退出位置在22m左右可以去找我的历史回复。这个ip确实可惜了，放在去年破百m问题不大。
> 买入逻辑是什么？阴谋盘，叙事成立，传播链路成立，资金还行。
> 为什么我知道？去年一整年在英文区瞎混，几个cabal当时也只是刚刚起步大家一起玩的多，好的叙事大家都有共鸣感也就所谓的网感，时间长了就感受到了。
>  
> $joobi  进场喊单位置在262k左右，退出位置在700k左右。小蓝豆表情包加上无厘头式的语录确实具有不错的链路，社区持续建设中。
>  
> 复盘完接下来讲重点拿出小本本记笔记。
> 抛出几个问题
> 买入后你的定位是什么？
> 支持你买入的理由是什么？
> 你卖出的理由是什么？
>  
> 案例：上周在 @10UWINA8 的社群丢了一个ca位置大概在200k左右，恰好高中生在直播 @ZephyrTrading 进场了被套。当然我发的ca后续跌到100k左右，于心不忍拿地址集群进场做市一手，最后做到了776k顶两手，确实没有量了，公开表明撤出地址结束这次做市。
> 这里我想表达meme共同建设产生合力才会出圈，链上现状互相背刺，浇个朋友😂，好的角度也很难起来，换一种思维尽管大家都是对手盘阶段性合力的助推会产生意想不到的效果。
>  
> 一级推荐几位网感不错的
> @mayangdarana 汉堡
> @feibo03 奶牛
> @Shanks_A9z  香总
> @wangyiduo0404 鲨鱼
> @ZephyrTrading  高中生
> @btc_798 96
> 下次浇慢点k线要被砸坏了
>  
> 二级认可的几位
> @Zenzen_Crypto 04女大学生认知理解全面在线
> @CryptoPainter_X 画师对k线理解独到
> @Lsssss1106 果子哥常青树
> @thankUcrypto 熬鹰耐心强能学到不少
>  
> 最后送一个标的hyper，时间会给出答案。

## 41. Jun 5, 2025 · 9:15 AM UTC · 1930554062891598251#m
- 链接：https://twitter.com/Zenith_shade789/status/1930554062891598251#m
- 作者：@Zenith_shade789

> 可以撤了，舒服了。

## 42. Jun 1, 2025 · 2:25 AM UTC · 1929001444536943037#m
- 链接：https://twitter.com/Zenith_shade789/status/1929001444536943037#m
- 作者：@Zenith_shade789

> yo  gmask $MASK

## 43. May 31, 2025 · 12:56 PM UTC · 1928797651618701502#m
- 链接：https://twitter.com/Zenith_shade789/status/1928797651618701502#m
- 作者：@Zenith_shade789

> en..... $MASK

## 44. May 31, 2025 · 5:16 AM UTC · 1928682062812426409#m
- 链接：https://twitter.com/Zenith_shade789/status/1928682062812426409#m
- 作者：@Zenith_shade789

> 当我们看到这篇帖子的时候很高兴 $MASK 此时还在15m，我们又踏马吃了几十粒sol。
> 为什么，大盘下跌 $MASK 能够逆势上涨？
>  
> 我们明白好的代币一定是具有以下特性的 共情、传播、叙事、资金所产生的合力。
>  
> $MASK 沙二小猫本身具有二传特质方便传播，社区优质可以看到很多鬼佬的kol换上头像。
> 而go这款fps游戏需要协作能力自然而然的有着凝聚力，玩go饰品的两类人，倒爷和真爱，go学长只是一味的喜欢饰品，正是这些合力让你看到如此美妙的k线。
>  
> 一级不像二级没有那么多瞎几把k线画图，什么压力位回调位的，那不是眼睛就能看见的？画出来是为了让自己心安的？
> gogogo！

## 45. May 31, 2025 · 4:45 AM UTC · 1928674289454100594#m
- 链接：https://twitter.com/Zenith_shade789/status/1928674289454100594#m
- 作者：@Zenith_shade789

> 你们都在跌怎么我在拉盘？
> 是技不如人还是没有拼尽全力？ $joobi

## 46. May 31, 2025 · 12:59 AM UTC · 1928617225457442850#m
- 链接：https://twitter.com/Zenith_shade789/status/1928617225457442850#m
- 作者：@Zenith_shade789

> 刚醒会儿，大盘下跌沙二小猫逆势上涨。怎么说？ $MASK 沙二小猫15m了，我拿嘴拉盘？
>  
> 周六再睡会儿醒来讲整体思路，记得提醒下记性差容易忘了😵

## 47. May 30, 2025 · 3:29 PM UTC · 1928473784325185735#m
- 链接：https://twitter.com/Zenith_shade789/status/1928473784325185735#m
- 作者：@Zenith_shade789

> 回忆成了大问题.....
>  
> 今天问朋友看到过这个视频？
> 3.51 复制打开抖音，看看【糕手小雯的图文作品】我在沙二很想你# dust2 # csgo # f... v.douyin.com/086M7kFQMLM/ 06/13 d@N.wS Hic:/
> 虽然很早就有这个只沙地小猫，今天打开gmgn也正好看到关注地址上了瞅了眼头像这不沙二小猫吗？作为go的老玩家再熟悉不过了，看了眼发币时间在昨晚，知道的朋友应该知道我昨晚事情多出门错过了，不然稳上。
>  
> 今天上的逻辑很简单叙事充分传播链路具有感染力，看了眼社区建设的不错也就果断上车了，先看10m。 #DYOR

## 48. May 30, 2025 · 2:51 PM UTC · 1928464282909294801#m
- 链接：https://twitter.com/Zenith_shade789/status/1928464282909294801#m
- 作者：@Zenith_shade789

> 是笨鸟先飞还是避我锋芒？ $joobi

## 49. May 30, 2025 · 1:50 PM UTC · 1928448853419692049#m
- 链接：https://twitter.com/Zenith_shade789/status/1928448853419692049#m
- 作者：@Zenith_shade789

> 我在沙二很想你。 $MASK

## 50. May 23, 2025 · 9:53 AM UTC · 1925852643517186412#m
- 链接：https://twitter.com/Zenith_shade789/status/1925852643517186412#m
- 作者：@Zenith_shade789

> 看着James老师做起来的，对于庄的看法我从亲身经历来讲，19岁也就21年底刚接触图片期间从什么都不懂认为那些大佬们非常牛逼到后面心智的改变觉得大家思维同质化似乎一切都能理解了，后面拉些大户操盘《唯一数藏》绿马从0.5rmb做到80rmb以及一些其他图片过程其实也很简单疫情期间符合叙事资金没去出大环境日下都想赚钱，好的叙事加上漂亮的涨幅就是最好的合力。（陆续也在50-60出完了）后面也开了台子很简单想做球员和裁判，也有人说这不zp？我直白点讲政策不明朗情况下趁早做才是正解可以认为这是投机倒把行为。无非最后全额退款但是要明白一点退款也只退发售价那么涨幅波动产生的利润，是市场行为，这样你就明白了。
>  
> $neiro 早期也喊了很久大小写阴谋论什么几把玩意儿的，直白点讲我喊因为有货在以及些内幕性质的。现在去搜索ca依旧可以看到我的那些帖子。
>  
> 再说点可能不知道的 $fwog 这个token最早用来孵化kol的，培养一批kol一个好盘子就是互惠互利的，发ca也是因为有货在。
> $billy 那只小狗一个db洗米盘就是这么简单，只是有人先知道些信息，太多太多了。
>  
> 没有什么庄走了，庄来了。无非资金看好ok喜欢。
> 发推内容很少和经历有关私下和朋友们也会聊聊些token怎么样，看好就上。看到最近 $labubu 热度挺高的以及什么勾八rwa 赛道token，也多多少少早期分享过。
>  
> 时间久了给我直观感受是内容的输出似乎不是那么重要了，看结果就ok，发了上不上是大家的行为，涨起来了自有大儒辨经。想拿底部很难更多交易人员看市场情绪做出选择在我逻辑里盈亏比不够也很少看市场情绪做买入行为，
>  
> 最近情感上也出现些问题，这一直是我的劣势无法很好处理感情中的问题，低频社交成为常态，希望能好起来。

## 51. May 10, 2025 · 3:56 PM UTC · 1921232837312704552#m
- 链接：https://twitter.com/Zenith_shade789/status/1921232837312704552#m
- 作者：@Zenith_shade789

> send moon $bob

## 52. May 6, 2025 · 1:45 PM UTC · 1919750492378337663#m
- 链接：https://twitter.com/Zenith_shade789/status/1919750492378337663#m
- 作者：@Zenith_shade789

> 可惜jail当时找一圈只有同名的没人发🤯

## 53. May 6, 2025 · 1:43 PM UTC · 1919749764708761670#m
- 链接：https://twitter.com/Zenith_shade789/status/1919749764708761670#m
- 作者：@Zenith_shade789

> 死没死其实真的不是很重要，这种叙事印象中应该不是第一次。哦~因为他们并不出名大家不买单，当你看到holder只有6000-7000市值却在20-30m时应该也知道大多是不信的，你之所以选择追原因无非在于kol的喊单以及关注地址的买入给了你做出追高行为心理安慰，当然这属于正常心理。（群里公开买了$LEGACOINS本已出完感谢流动性）
> 纯当乐子亏了就认。

## 54. May 6, 2025 · 2:00 AM UTC · 1919572852879327460#m
- 链接：https://twitter.com/Zenith_shade789/status/1919572852879327460#m
- 作者：@Zenith_shade789

> $Zoomie 昨晚炼金群发的上车点位在50k-80k左右
> 5Bin8bMu8Kd3h25thwGwbFDzRNr1nqfDD9yJAmhhbonk（目前以清70%）
> 如果你关注了昨天我的那条并试图找到它或许现在你也埋伏10x,当然不止这些token。

## 55. May 5, 2025 · 3:10 PM UTC · 1919409251426148480#m
- 链接：https://twitter.com/Zenith_shade789/status/1919409251426148480#m
- 作者：@Zenith_shade789

> $uno
> 9Fmqng9XGEqjJMAqFmEFz5aCRDs8Kh1GHiqwEf21pump
> 邪恶实验关闭 uno获救了，拉的好快，出的差不多了。

## 56. May 4, 2025 · 6:44 PM UTC · 1919100891581276162#m
- 链接：https://twitter.com/Zenith_shade789/status/1919100891581276162#m
- 作者：@Zenith_shade789

> 傻逼马斯克，你是这个👍
> 喝了几斤酒又换头像又改名的

## 57. May 4, 2025 · 6:41 PM UTC · 1919100150989062404#m
- 链接：https://twitter.com/Zenith_shade789/status/1919100150989062404#m
- 作者：@Zenith_shade789

> 傻* @elonmusk  卖车赚的没这个来的快是吧？

## 58. May 4, 2025 · 6:37 PM UTC · 1919099058196508848#m
- 链接：https://twitter.com/Zenith_shade789/status/1919099058196508848#m
- 作者：@Zenith_shade789

> 别蒸了别蒸了 也不是第一次换头像了，p的头晕目眩😵

## 59. May 4, 2025 · 2:16 PM UTC · 1919033499027452308#m
- 链接：https://twitter.com/Zenith_shade789/status/1919033499027452308#m
- 作者：@Zenith_shade789

> 今天开了自己小群，命名炼金学院寓意大家在链上都能点石成金。
>  
> 仅仅过去几小时个人发了4个token胜率75%
> 分别是
> 一段：
> $HOTSAUCE HD8tKPbYp2kctueWh4vkX6Y3673MakpLMWmxg97epump( 56k-400k)
> $puma
> Dj4UMcGAgdowpaufj2QwCpjuws3SvJTNzhkqSg3kpump(120k-10k 亏损）
> 二段：
> $mfers
> 5VsVZBRZ34bf6LbpsJ5kwZZUFii5Gj2xKVc5BgRGpump(800k-1.4m)
> $cfx
> RhFVq1Zt81VvcoSEMSyCGZZv5SwBdA8MV7w4HEMpump(未动）
> 根据个人经验而言，Token的涨跌看似无常，其实有迹可循。然而，人的注意力有限，对数据的敏感度常被情绪与惯性所干扰。正如998兄 @Loong998 所言“为何我能抓住$GORK？机会摆在你面前，你会上吗？你又敢上多少？”——市场中的每一次抉择，都是对认知、胆识与纪律的考验。
> 我将这种行为称为“条件反射”：源于神经末梢的惯性。

## 60. May 3, 2025 · 6:01 AM UTC · 1918546437463506983#m
- 链接：https://twitter.com/Zenith_shade789/status/1918546437463506983#m
- 作者：@Zenith_shade789

> 每日炼金
> 昨晚属于是爽吃了几波了，从
> $BeforeGTA6 (AFMyfmGLmZ7VGfqbQ3aUaHtNQnH3mE4T1foJyimppump)
> $Kobushi (DCrwhb7f6LL4dtvsTty1PwT2wL5AoxHzuwYMUqM5pump) $Artemis(G9sTp9mXkM8FyiEV4dM76XzMkdmDAkhg6EBpVahSpump) $10waz2(Hxxmg1atgaEW6zPFLikiR8n923HzE3oX8KafsHKPrqH3)
> $WhiteCoat (HCW4Bv6b5LrNGMC6dd69CiG12D9YLX7fQz3gVUaCpump)
> 很多朋友也在车上多多少少都赚了，当然也有亏损的，这里我讲点自己心得，首先你得明白自己是一名长跑运动员还是短跑选手，核心在于耐力与爆发的区别。
> 长跑选手选择优质标地拿着即可，减少冲刺（扫链）带来的磨损
> 短跑选手短期内接触面广，路程短一波结束，没必要全部拿着一直不出，好在爆发力强
> 那么你有没有你自己的行为习惯我更喜欢称这种为条件反射，举个例子遇到交易量激增的token你下意识选择上？选上多少？什么时候卖？这些你考虑过？不要说在看到时候考虑等你思考完已经翻2-3倍了....（养成自己的条件反射）
> 你有没有自己的分析逻辑与方式，我更希望大家都能找到自己的alpha

## 61. May 2, 2025 · 4:08 AM UTC · 1918155548203336030#m
- 链接：https://twitter.com/Zenith_shade789/status/1918155548203336030#m
- 作者：@Zenith_shade789

> 昨晚boop 太精彩了，后半夜直接睡了。
> 主线任务：领空投
> 支线任务：完成rug

## 62. May 1, 2025 · 6:07 PM UTC · 1918004459449532752#m
- 链接：https://twitter.com/Zenith_shade789/status/1918004459449532752#m
- 作者：@Zenith_shade789

> $pve 略带嘲讽叙事今晚提纯的第二个token Bs7AFCShmQvqeutb6smDQ9Pw472JXMA3i46XhcUtpump（不建议再上了，位置高了以及叙事玩过几次了也就今晚boop事件有个预期）由于没开自己群就300k左右私发了 @Loong998 （搂着妹妹赚钱？）和另外一个匿名朋友。
> 拥抱-质疑-退出

## 63. May 1, 2025 · 7:30 AM UTC · 1917843995188945115#m
- 链接：https://twitter.com/Zenith_shade789/status/1917843995188945115#m
- 作者：@Zenith_shade789

> 昨天一直在高精度提纯中，提纯 $shortcoin 出现了意外自己没怎么吃到但好在群友吃舒服了，自己上的位置在20k左右，群友上的在200k左右，也算长期以来没喂饭的补偿了。（叙事大空头，看过电影的应该知道）
>  
> 后半夜提纯了 $grok 交易量非常大，自己位置在210k左右，当时知道没睡的朋友 @Loong998  问上了没发现他在把妹😵，发他位置在2m左右了虽然晚了点至少吃到了，下次和我一起提纯。（叙事嘲讽grok，疑似官方下场加主力球员push）

## 64. Apr 24, 2025 · 3:26 PM UTC · 1915427157804789787#m
- 链接：https://twitter.com/Zenith_shade789/status/1915427157804789787#m
- 作者：@Zenith_shade789

> 链上p多了发现圈子越来越小，尽管大家没真实见过，但链上总是会碰到熟悉的人。地址也不换，好比明牌德州话说回来也可能一次你的失误露出了马脚以为自己隐藏的很好或者无人在意其实已经被盯上了。

## 65. Apr 23, 2025 · 9:27 PM UTC · 1915155652420526569#m
- 链接：https://twitter.com/Zenith_shade789/status/1915155652420526569#m
- 作者：@Zenith_shade789

> 有回调预期，不做投资建议

## 66. Apr 23, 2025 · 9:26 PM UTC · 1915155456781402457#m
- 链接：https://twitter.com/Zenith_shade789/status/1915155456781402457#m
- 作者：@Zenith_shade789

> 我是谁？sell kfc buy $src

## 67. Apr 23, 2025 · 6:44 PM UTC · 1915114569145684076#m
- 链接：https://twitter.com/Zenith_shade789/status/1915114569145684076#m
- 作者：@Zenith_shade789

> $src 十次疯狂星期四可以换来什么？

## 68. Apr 23, 2025 · 4:48 PM UTC · 1915085479856500812#m
- 链接：https://twitter.com/Zenith_shade789/status/1915085479856500812#m
- 作者：@Zenith_shade789

> 什么时候开骂

## 69. Apr 23, 2025 · 4:48 PM UTC · 1915085322251342208#m
- 链接：https://twitter.com/Zenith_shade789/status/1915085322251342208#m
- 作者：@Zenith_shade789

> 《**的晚宴》

## 70. Apr 23, 2025 · 2:29 PM UTC · 1915050463151751410#m
- 链接：https://twitter.com/Zenith_shade789/status/1915050463151751410#m
- 作者：@Zenith_shade789

> 如果我是你我会怎么做？ $sol #你的强

## 71. Apr 22, 2025 · 6:29 PM UTC · 1914748376434065432#m
- 链接：https://twitter.com/Zenith_shade789/status/1914748376434065432#m
- 作者：@Zenith_shade789

> 什么时候能像他一样努力 #TROLL  #Cabal #白虎 #少萝

## 72. Apr 18, 2025 · 11:45 AM UTC · 1913197267357761664#m
- 链接：https://twitter.com/Zenith_shade789/status/1913197267357761664#m
- 作者：@Zenith_shade789

> 哦~ 原来 $pppp 是我发的pan
>  
> 这两天看到大家对于 $rfc 阴谋内幕舆论挺大的本不想写贴的但流量大就蹭蹭了谁让流量足够大呢？🤣
>  
> 首先rfc我的参与度不高，为什么参与度不高从以下几个角度来看。
> 1.发射时间看到没有？
> （显然参与度并不高发射阶段甚至都没仔细看究其原因发给我期间我并未在链上甚至没注意）
> 2.发觉它的时候为什么不上？
> （比较随缘的发觉时候在2m左右，记得当时忙工作去了也就忘记这茬事情了）
> 3.再次看到时候上了？（上了买的不多属于追了一手没怎么管了）
>  
> 回到主题 $pppp 是不是我的？
> 答案很显然不是我的也不是我在操盘，看到上条贴精准爆点只是运气而已大家赚钱就行，上周日说是我发的只不过一句玩笑话。
> 那么到底有没有阴谋这里我给不了一个准确的答案，“阴谋”存在的定义很多。 $rfc 我不清楚但 $pppp  我清楚，朋友问我这么确定是不是可以梭哈，我直白的说容纳不下这个资金量。
> 我喜欢链上这点在于它是透明的只要你的资金在链上而不是过了cex以及跨链桥那追起来只是时间问题。对于token存不存在上帝视角，大家可以自己去感受这点。（ps： @MeetHubble 正在做的就是这点大家可以关注下）
> 从操盘的角度来看rfc的叙事更具有感染力， $pppp 的故事相对来讲弱很多。（pppp成本已经出完了只有利润）
>  
> $dark能不能上？
> 至于现在的热点 $dark,叙事还行配合有效画线有望ATH。（ps：短期预期不会很高，自己的货不多很散）
> 还是那句话对于是否阴谋内幕不重要你赚了就行了，亏了就认反思自己是不是认知不够。
>  
> 这条帖子点赞过100或评论过20，周六开space来讲讲你们不知道的。( Kill dev,KOL链上不看叙事干拉手法）

## 73. Apr 14, 2025 · 1:58 PM UTC · 1911781162294579564#m
- 链接：https://twitter.com/Zenith_shade789/status/1911781162294579564#m
- 作者：@Zenith_shade789

> 有时候觉得自己界面安排挺合理的，在工作之余看看zen宝推送的token点开看看哪些地址再看看头像就清楚要不要买点，最重要的是一点不影响工作，扫链已经扫不动了会浪费大量的时间在枯坐中。
> 错过的token其实看看地址情况要不要二段即可，$pppp这个标看了资金情况还行准备到位上点。$NORMIES二段在1m也在群里喊了可以稍微上点主要是看到有积累地址买入位置比较可以，换句人话谁家好人吃饱撑了乱搞，事出反常必有妖。

## 74. Apr 12, 2025 · 8:48 AM UTC · 1910978248848359487#m
- 链接：https://twitter.com/Zenith_shade789/status/1910978248848359487#m
- 作者：@Zenith_shade789

> 昨晚 $bang 推出来位置大概在48k左右高点接近400倍，好兄弟 @Divoll_Law 50k上的车。
> 目前zen宝仅仅运作在禁言群没开放基本就几个身边朋友在里面。平常事情太多没太多时间扫链所以基本就看看禁言群推送的token信息值不值得上，可能50个上一个这种频率。
> 一代zen宝推送量大概一天50个token，三代加上行为策略应该会精致很多。

## 75. Apr 7, 2025 · 5:31 PM UTC · 1909297903237202166#m
- 链接：https://twitter.com/Zenith_shade789/status/1909297903237202166#m
- 作者：@Zenith_shade789

> 好哥哥们要么疯了，要么再也不更新了🥲

## 76. Apr 7, 2025 · 5:29 PM UTC · 1909297505982193741#m
- 链接：https://twitter.com/Zenith_shade789/status/1909297505982193741#m
- 作者：@Zenith_shade789

> 今日币圈依犹在，不见当年以太人

## 77. Apr 7, 2025 · 8:00 AM UTC · 1909154190620000584#m
- 链接：https://twitter.com/Zenith_shade789/status/1909154190620000584#m
- 作者：@Zenith_shade789

> 不要浪费任何一场危机

## 78. Apr 7, 2025 · 7:59 AM UTC · 1909154108986319273#m
- 链接：https://twitter.com/Zenith_shade789/status/1909154108986319273#m
- 作者：@Zenith_shade789

> 永远不要被熊市吓到，不要被危机吓到，往往这里存在巨大机会。

## 79. Apr 6, 2025 · 6:15 PM UTC · 1908946754810396980#m
- 链接：https://twitter.com/Zenith_shade789/status/1908946754810396980#m
- 作者：@Zenith_shade789

> 那就对了

## 80. Apr 6, 2025 · 6:13 PM UTC · 1908946183164469269#m
- 链接：https://twitter.com/Zenith_shade789/status/1908946183164469269#m
- 作者：@Zenith_shade789

> “所以，那些明知道结局却还想赌一把的你，赢了吗?”

## 81. Apr 6, 2025 · 8:09 AM UTC · 1908794255960924441#m
- 链接：https://twitter.com/Zenith_shade789/status/1908794255960924441#m
- 作者：@Zenith_shade789

> 在大家都在舔@baosonbnb WL期间我们聊聊项目本身如何以及对应营销手段。
>  
> 辣鸡尼玛，项目方为了舔bn 1亿流动性分配拿个mcp协议做个发射台就来发币，这点本质上是没有祛魅mcp。
> 当下流动性相对来讲大家都清楚什么情况，能不能做起来咱们先不说。营销手段上放钩子撒饵钓kol来发推舔它，尼玛的真给他爽到了。
>  
> 垃圾项目活不久估计也是项目方没钱了想圈一波之前做的没起来想来bsc上套个壳子捞一笔。

## 82. Apr 4, 2025 · 2:45 PM UTC · 1908168976385155582#m
- 链接：https://twitter.com/Zenith_shade789/status/1908168976385155582#m
- 作者：@Zenith_shade789

> 发帖频率越来越低了，思来想去可能是牛熊交替市场给予我的负反馈提不起提笔兴趣，一部分来源于在建设些事物同时并没有什么好的见解分享给大家。
>  
> 上轮熊市开始时这种情况还没有很明显，可能也是因为当时在校没什么钱在币圈，主要处在web2市场打打闹闹，波动并不大。
> 当投入圈内时间越来越久烦躁焦虑频率指数型增长。再次经历牛熊交替期抵制诱惑守住手中的三瓜两枣变成了我当务之急，亏损不可怕如何尽可能的减少大幅回撤成为了难题。
> 流动性的流失也让我很少再去链上频繁冲浪，这种情况下风险远大于回报投入精力大部分浪费了，偶尔晚上内盘大家互相浇个朋友消遣消遣感受链上风向。
> 以上纯吐槽，回到正题我对当下加密市场阶段看法，希望对大家有所帮助（无顺序之分）
>  
> 流动性如何？
> 关于链上流动情况怎么样其实大家也都能感受到从去年底开始流动开始出现疲软现象，600m以上的token越来越稀有，ai概念带来的流动性消化的也很快，热点转移越来越快新的热点出现吸血越来越严重财富转移迅速。项目方也急着发币回笼投入资金，不禁让我想起了22年链上惨状，总结下来，场外资金流入减少，场内资金持续流出。
>  
> 大盘方向？
> 对于大盘我的看法更加偏向于下，早期储备资产带来的预期促使btc等加密资产走向新高，但当预期值被严重高估后市场反映出了不买单此举措政策，加上近期关税事件影响全球经济，导致我并不看好后市。（当然这只是我的期望，因为可以再次拿到被严重低估的打折商品）
>  
> 如何适应当下？
> 聊聊近期三个月的举措，清理了链上80%memecoin，减少链上出手频率，用稳定币去玩些memecoin。（这里推荐链抽象，link就不放了自己可以去找）
> 上新某所做空些项目币，以及某些上新的memecoin，这里提示低倍低倍低倍，降低收益预期会舒服很多。（如果有强大的链上数据处理能力可适当调整）
> 寻找异动token合约适当做多有故意操盘低市值token，扒车上。（不建议重仓，重仓也不能爆富）
> 当然最好的方式就是出去旅游减少噪音，不要因为付了钱就继续坐一辆错误的列车，祝大家财多多!!!

## 83. Apr 1, 2025 · 11:16 AM UTC · 1907029283476000826#m
- 链接：https://twitter.com/Zenith_shade789/status/1907029283476000826#m
- 作者：@Zenith_shade789

> 现货没事

## 84. Mar 23, 2025 · 6:04 AM UTC · 1903689187402174743#m
- 链接：https://twitter.com/Zenith_shade789/status/1903689187402174743#m
- 作者：@Zenith_shade789

> 薯条（ @0xCryptoFries ）  的链上投资哲学
>  
> 薯条（@0xCryptoFries）的故事，是关于如何在波动的市场中，通过冷静的分析和坚定的策略，一步步走向A7的旅程。今天，我有幸与这位A7级别的投资者进行了一次深入的对话，我将以第三人称叙述他的投资哲学和决策过程。
>  
> 1. 叙事的魅力与数据的严谨
> 薯条的投资旅程始于对市场叙事的热度敏感。他告诉我，选择一个token，首先要看它的叙事是否能够引起市场的共鸣。但仅有热度是不够的，成交量、市值、预期值都是他考量的重要因素。他特别强调了预期值的重要性，这源于对叙事和热度的深入分析。
>  
> “买入一个token，首先要看叙事热度和成交量，其次是市值和预期值。”
> ——薯条
> 2. 基本分析与深度挖掘
>  
> 当被问及是先做基本分析立马上车还是详细分析再上车时，薯条的回答显示了他的实战经验。他倾向于先进行基本分析，迅速上车，然后再进行深度挖掘。这种策略让他在市场变动中能够迅速反应，同时也保证了决策的深度。
> “我的链路是先基本分析上车，再深扒。”
> ——薯条
>  
> 3. 中段选手的策略
> 薯条自称为“中段选手”，并不追求最早进入市场，而是在叙事发酵的过程中，根据市值和筹码分布来决定进场时机。这种策略减少了他对市场初期波动的暴露，同时也让他能够在趋势明确时抓住机会。
>  
> “我大多时候是中段选手，我会在叙事发酵过程中根据市值和筹码分布考虑进场。”
> ——薯条
> 4. 市值与仓位的平衡
>  
> 对于市值高的token，薯条并不会盲目进入大仓位。他的决策基于对叙事预期值的判断和盈亏比的分析。如果预期市值空间有限，他可能会选择观望。这种谨慎的态度，是他在市场中稳健前行的关键。
> “如果离预期市值盈亏比空间小，我可能并不会进场。”
> ——薯条
>  
> 5. 数据分析：决策的核心
> 在薯条看来，数据分析是他决策中最重要的因素。尽管他尝试过多种工具，但最终还是回归到区块浏览器的直接分析。他坦言，地址行为分析是他一直想要解决但尚未攻克的问题。如果有产品能够提供这样的分析，并且能够自定义化，他会非常感兴趣。
>  
> “数据分析很重要，地址行为分析也反应了市场情绪”
> ——薯条
>  
> 6. 从A7到A8的征程
> 薯条的资产已经达到了A7级别，通过严格执行自己的决策策略，希望能够达到A8这个资产阶段。他的成功并非偶然，而是建立在对市场的深刻理解和严格的自律之上。
>  
> “我的资产绝大部分来源于严格执行策略。”
> ——薯条
>  
> 结语
>  
> 薯条老师的故事，是每一位区块链投资者的缩影。在这个充满不确定性的市场中，通过不断学习和实践，才能找到属于自己的成功之路。让我们期待薯条早日实现他的A8梦想，也期待每一位链上用户能够在自己的投资旅程中，找到属于自己的叙事。

## 85. Mar 22, 2025 · 6:21 AM UTC · 1903331251790156138#m
- 链接：https://twitter.com/Zenith_shade789/status/1903331251790156138#m
- 作者：@Zenith_shade789

> 除了卷上天的Trading Bot，P小将们还有哪些更高级的工具可以用来打狗？为何 Solana MCP黑客松可以取得第二名的好成绩？AI Agent + onchain 真正的PMF是什么？花两分钟参加一下问卷调查，你将有机会参与到 @hubbledotxyz 四月中开启的邀请内侧！

## 86. Mar 20, 2025 · 7:49 PM UTC · 1902809649801466073#m
- 链接：https://twitter.com/Zenith_shade789/status/1902809649801466073#m
- 作者：@Zenith_shade789

> 《BSC chain 真的memecoin春天吗？》
>  
> 从客观角度来看bsc上memecoin我并未看到具有很强模因角度的token。
>  
> 无论是西兰花，tst还是近期的mubarak，tut等这些token本质上注意力经济还是在 @cz_binance 或者 @heyibinance 身上，那么当下还没看到具有很强的模因文化属性的memecoin，能够进行二创表情包或者社区共识度强等。
>  
> 这种感觉有点像sol上memecoin晚年阶段了，一姐和cz哥对于memecoin的理解可能接触时间短保持在sol当下pvp阶段，一天800个方向。
>  
> 一旦祛魅后注意力经济也随之整体下滑，好比早期@elonmusk 以及 @blknoiz06 等带来的买盘相当强，但是这并不是维持之计。当潮水退去也就会发现原来事情并不是这么回事儿。
>  
> 建议多做些了解或者多进些社区打破信息茧房，还是很看好bsc meme的至少当下币安成员是在努力的。（以下内容不构成投资建议仅分享有趣的模因文化）
>  
> 视频来源：2.33 复制打开抖音，看看【小黄脸的作品】Oia - 烫到脚哩 # 充能计划 # 痞老板# ... v.douyin.com/aCvVxFnyY3M/ sre:/ 01/19 O@x.FH

## 87. Mar 18, 2025 · 8:16 PM UTC · 1902091836669501805#m
- 链接：https://twitter.com/Zenith_shade789/status/1902091836669501805#m
- 作者：@Zenith_shade789

> 来到我们的主场了吗？ $szn  @4Titch_Y2K  @justinsuntron

## 88. Mar 18, 2025 · 9:23 AM UTC · 1901927323601125719#m
- 链接：https://twitter.com/Zenith_shade789/status/1901927323601125719#m
- 作者：@Zenith_shade789

> 撤 差不多了反手了

## 89. Mar 17, 2025 · 12:26 PM UTC · 1901611071087198593#m
- 链接：https://twitter.com/Zenith_shade789/status/1901611071087198593#m
- 作者：@Zenith_shade789

> 一条帖子干 emo 了，测 mcp 回来看这尼玛… （不是吐槽 bn）

## 90. Mar 17, 2025 · 11:48 AM UTC · 1901601594619416780#m
- 链接：https://twitter.com/Zenith_shade789/status/1901601594619416780#m
- 作者：@Zenith_shade789

> 别抄底了兄弟们，给机会给bn拿些流通上现货，需要评估下。

## 91. Mar 17, 2025 · 8:34 AM UTC · 1901552611901518054#m
- 链接：https://twitter.com/Zenith_shade789/status/1901552611901518054#m
- 作者：@Zenith_shade789

> 为什么我确定bn会上现货，昨天有点意淫。。。
>  
> 今天看到大部分cex都上了，只能说现在是赶鸭子上架。
>  
> 换做是你愿意看到自己链上的热度被持续分流其他cex？

## 92. Mar 17, 2025 · 8:00 AM UTC · 1901544244420190564#m
- 链接：https://twitter.com/Zenith_shade789/status/1901544244420190564#m
- 作者：@Zenith_shade789

> 最近忙着写些链上分析工具确实踏空了不少bsc上meme，看来cz已经快掌握meme精髓了。
> 这张单子会拿到200m冰山止盈，@0x_KevinZ 老师快单币a8了，在开之前思考要不要现货追车核算下来盈亏比太低了，索性杠杆进去，不过存在问题mexc我并未kyc不知道会不会因此卡我😅
>  
> 回到链上工具：
> 目前主要是分析Solana chain关于token的解析内容，相比于传统解析更加深入化。无论是从池子还是手法上做出了不同的解析维度，不局限于聪明钱维度更偏向于多元化解析维度。
>  
> 期间 @PandaSkiing 老师也提供了很多灵感，在我应对dev mm等如何做出策略性调整。顺嘴说句James哥家的产品确实不错，建议所有dev们都去感受下。
> @0x_KevinZ 老师在产品这块也是有独到的见解，感谢近期的探讨。不过还没吃上鸡公煲差评....
>  
> 欢迎各位做链上分析的老师们一起交流，太想进步了。🥰
> WeChat：zenithshade

## 93. Mar 16, 2025 · 1:18 PM UTC · 1901261715544973661#m
- 链接：https://twitter.com/Zenith_shade789/status/1901261715544973661#m
- 作者：@Zenith_shade789

> 这张单子要拿到200m 1万暴击！！！
> #mubarak

## 94. Mar 12, 2025 · 4:29 PM UTC · 1899860384527638546#m
- 链接：https://twitter.com/Zenith_shade789/status/1899860384527638546#m
- 作者：@Zenith_shade789

> 开的时候告诉我止损83800😭

## 95. Mar 10, 2025 · 7:41 PM UTC · 1899183936611512649#m
- 链接：https://twitter.com/Zenith_shade789/status/1899183936611512649#m
- 作者：@Zenith_shade789

> 建议大家定投eth
> 1874 你们会回来看的
> #ETH

## 96. Mar 10, 2025 · 3:45 PM UTC · 1899124464828489878#m
- 链接：https://twitter.com/Zenith_shade789/status/1899124464828489878#m
- 作者：@Zenith_shade789

> 思考下能够将 agent 决策具象化比如画 k 标注出于什么因素选择多空🤔

## 97. Mar 10, 2025 · 3:01 PM UTC · 1899113384668528837#m
- 链接：https://twitter.com/Zenith_shade789/status/1899113384668528837#m
- 作者：@Zenith_shade789

> 这里在回测历史数据中并结合未来走势作为决策分析依据，并不用担心

## 98. Mar 10, 2025 · 2:31 PM UTC · 1899105908380385568#m
- 链接：https://twitter.com/Zenith_shade789/status/1899105908380385568#m
- 作者：@Zenith_shade789

> 根据回测跑出来的收益，具体实盘有待测试

## 99. Mar 10, 2025 · 1:17 PM UTC · 1899087265001033768#m
- 链接：https://twitter.com/Zenith_shade789/status/1899087265001033768#m
- 作者：@Zenith_shade789

> 最近大家都在聊链上工具的技术革新，今天来看下昨晚历时一晚的idea实践。 《Agent自动化合约交易》
> 先展示收益图
> 本金：1000u，执行合约交易对 BTC/USDT 净盈利达到 4.34 亿 USD，收益率达 4349 万 %.（这里执行的滚仓交易爆仓后进行了重置1000u后续再优化）
> 回到正题agent能否改变传统合约交易以及链上交易？我的回答一定是可以的!!!!
> 这里讲下当下我的毛胚版agent架构:
>  
> ❇️数据模块：
> 实时数据：
> 使用 OKX WebSocket 获取 K 线和多空合约数据。
> 历史数据：
> 通过 OKX REST API 获取历史 K 线和持仓量数据。
> ❇️分析模块：
> 技术指标：使用 pandas_ta 计算 MA、RSI、MACD 等。
> 交易模块:
> ❇️API 配置：
> 使用 OKX API 进行自动化交易（需要 API Key 和 Secret）。
> ❇️技术架构：使用编程语言是 Python（因为我只懂点js和py）构建 Agent，结合数据库或内存缓存存储数据。
>  
> ❇️回测模块：
> 回测框架：
> 用了历史数据运行信号逻辑，计算胜率和盈亏比。
> 用历史数据（如过去 30 天或 1 年的 K 线和多空数据）运行策略。
>  
> 计算关键指标：收益率：总盈利 / 初始资金。
> 胜率：盈利交易次数 / 总交易次数。
> 夏普比率：衡量风险调整后的收益。
> ❇️自我进化模块：
> 参数优化：
> 采用了网格搜索优化 MA 和 RSI 的周期和遗传算法或强化学习（高级）和 Q-Learning 或 DQN（深度 Q 网络）训练 Agent。（贝叶斯优化环境还在写）
> 动态适应：通过自我进化，根据市场变化调整策略，而不仅仅是静态地输出一个“最佳策略”。
>  
> 奖惩机制：
> 采用的是平滑奖惩，通过移动平均或指数平滑方法，减少短期盈亏波动的权重，避免对单次交易结果过度反应。
>  
> PPO 算法强化学习：
> 提供了训练环境模拟的市场数据或实时交易场景。
> 对Agent 交易决策进行奖惩（如买入、卖出、持有）获得奖励（利润）或惩罚（亏损）。
>  
> 目前来看它还有点笨赌性有点大，采用的是滚仓策略，自我更新不及时回测太少技术指标过少，对于压力位支撑位理解不足够。
>  
> 后续暂时能够优化点完成盈利资金转移避免仓位过大导致回撤利润回吐，训练agent学习更多市场指标添加多家cex数据进行综合分析。
> 这里提出核心问题能否根据当下以及历史数据结合分析从而模拟未来短期走势方向千万次实时推算寻求最优“解”
>  
> 本篇内容灵感来源 @Lxxx_crypto 老师合约交易日记，数据接口来源于 #OKX  @okx @mia_okx

## 100. Mar 7, 2025 · 11:56 AM UTC · 1897979723592212869#m
- 链接：https://twitter.com/Zenith_shade789/status/1897979723592212869#m
- 作者：@Zenith_shade789

> 这年头确实大多数vc项目做不了市值管理，风险太大了。
>  
> 在上大所之前做市商这一环节必不可少的，项目团队自己来做市值管理
> 我说句实话首先出了问题这个第一责任人就是自己。（没护住盘一定被散户骂的很惨）
> 第二点自己来做市在没有成熟做市商背书情况下cex也担心能不能行。迫不得已请三方做市团队首先这个锅不用自己背
> 第三点由于不透明化完全可以从中合谋你背锅我们大家分钱，我想没有人会跟钱过不去。
>  
> 回到GPS事件
> 我更加认为本质上是VC退出需求与做市商逐利行为叠加的结果。通过做市商迅速清仓，而做市商则利用市场机制漏洞放大收益。这种“合谋”并非特例，也是加密市场发展阶段的普遍现象。（只不过大家的记忆像鱼一样也只有7秒了）
> 币安也需要加强对做市商的约束，或引入惩罚机制以遏制恶意砸盘。（这里希望上币前能够放出更多的信息增加散户信心） @cz_binance @heyibinance @0xjiujiu99
> @sisibinance 吐槽下想进 bnb chain 中文区朋友社区群问一圈不是没人回就是进不了🤣
>  
> 建议项目方：与VC和做市商协商更合理的代币释放计划，避免集中抛售；
> 做市商：平衡短期收益与市场健康，增加买单支持，避免单边操纵。
> 交易所：完善做市商审核和监控机制，公开部分交易数据以提升透明度。（我不信砸盘时候你们没在看）
> VC：我知道你很急但先别急我更急。

## 101. Mar 6, 2025 · 5:10 PM UTC · 1897696254660166069#m
- 链接：https://twitter.com/Zenith_shade789/status/1897696254660166069#m
- 作者：@Zenith_shade789

> En：I have seen the popularity of #Manus continue to rise in the past two days, so I would like to share my insights.
>  
> Logical analysis of Manus
>  
> 1. Core concept: closed loop from "knowing" to "doing"
>  
> The name of Manus comes from the Latin "mens et manus" (meaning "hands and brains" or "knowing and doing together"), which reveals its logical core: not only understanding user intentions ("knowing"), but also delivering results ("doing") through tool calls and task execution. This is different from traditional conversational AI (such as ChatGPT, deepseek) in that Manus emphasizes end-to-end task closure rather than just providing suggestions or text output.
>  
> Input layer: users put forward requirements in natural language (such as "analyze Tesla stock price trends and generate visual reports").
>  
> Processing layer: decompose tasks, plan steps, call tools, and perform operations.
>  
> Output layer: directly deliver complete results (such as PDF reports, Excel tables, and visual web pages).
>  
> The core of this logical design lies in "autonomy" and "execution", which upgrades AI from a passive assistant to an active agent.
>  
> 2. Task decomposition and planning
>  
> The key to Manus's ability to handle complex tasks lies in its task decomposition capabilities. For example, in the case of screening resumes, it can:
>  
> Unzip compressed files.
>  
> Analyze the content of resumes page by page.
>  
> Extract key information (such as work experience, skills).
>  
> Sort and generate recommendation lists according to job requirements.
>  
> This shows that its logic contains a dynamic planning module with the following steps:
>  
> Intent parsing: Understand user needs through natural language processing (NLP).
>  
> Task decomposition: Decompose complex requirements into executable subtasks (such as "search data → analyze data → generate charts").
>  
> Prioritization: Arrange subtasks according to task dependencies and logical order.
>  
> Dynamic adjustment: Adjust plans according to feedback or new information during execution.
>  
> 3. Tool call and execution
>  
> Manus's execution logic relies on powerful tool call capabilities. It can operate tools such as browsers, code editors, and file processors in a virtual environment.
>  
> For example, in the real estate screening case, it can:
>  
> Search for community safety data.
>  
> Calculate affordable housing prices.
>  
> Extracting property information from real estate websites.
>  
> Integrating into detailed reports.
>  
> Its logic design embeds a toolchain management module, which may interact with external resources through APIs or simulated human operations (similar to RPA, robotic process automation).
>  
> 4. Learning and optimization
>  
> Manus has memory and learning capabilities.
>  
> For example, after the user asks for the results to be presented in a table, it will remember this preference and give priority to it in subsequent tasks. This logic shows that it has built-in context memory and reinforcement learning mechanisms:
>  
> Context memory: record user interaction history and preferences.
>  
> Feedback optimization: adjust execution strategies based on user satisfaction.
>  
> Manus architecture analysis
>  
> Based on existing information and technical trends in the field of AI Agent, Manus' architecture may be a multiple agent system (Multiple Agent System) running in a cloud virtual machine environment. The following is a detailed deduction:
>  
> 1.Overall architecture: multi-agent collaboration
>  
> Manus adopts a multiple agent architecture, which divides task processing into multiple independent but collaborative agent modules. This design can improve efficiency and robustness, and may include:
>  
> Planner Agent: responsible for task decomposition and step planning.
> Executor Agent: calls tools and completes specific operations.
> Validator Agent: checks the accuracy and completeness of results.
> Memory Agent: manages context and user preferences.
>  
> These agents run independently in a sandbox environment and work together through message passing or shared memory. For example, in the resume screening task:
>  
> The planner agent generates a task list (such as "unzip file → extract information → sort").
> The execution agent calls a Python script to process data.
> The validation agent checks the consistency of the results.
>  
> 2. Technical components
>  
> (1) Computing environment: cloud virtual machine
>  
> Manus runs in an independent cloud virtual machine, similar to Anthropic's Computer Use mode. This design has the following advantages:
>  
> Isolation: avoids local resource limitations, and tasks can be executed asynchronously in the background.
>  
> Scalability: supports multi-task parallel processing.
>  
> Tool integration: The virtual machine is pre-installed with browsers, code editors and other tools to simulate human operations.
>  
> (2) Large Language Model (LLM)
>  
> As the core reasoning engine, Manus may be based on one or more optimized large language models (such as LLaMA, Grok or other open source model variants),
>  
> responsible for:
> Understanding natural language instructions.
> Generate tool call instructions or code.
> Output natural language explanations.
>  
> (3) Tool call interface
>  
> The technical highlight of Manus lies in its tool call capability, which may be achieved in the following ways:
>  
> API integration: directly call external services (such as Yahoo Finance to obtain stock data).
> Simulation operation: operate web pages through browser automation (such as Selenium).
> Code generation: dynamically write and execute Python, HTML and other scripts.
>  
> (4) File management system
>  
> During task execution, Manus can process files (such as decompressing resumes and generating Excel). This requires a virtual file system that supports file addition, deletion, modification and query, and outputs the results in multiple formats.
>  
> 3. Data flow and workflow
>  
> The following is a workflow deduction for a typical task (such as "analyzing bitcoin trends"):
>  
> Input: User input "Analyze bitcoin price trends in 2024 and generate reports".
>  
> Planning: The planning agent is decomposed into "get data → analyze trends → generate charts → integrate reports".
>  
> Execution: The execution agent calls the financial API to obtain price data.
> Write a Python script to calculate trends and generate line charts.
> Integrate the results into PDF.
> Verification: The verification agent checks data accuracy and chart readability.
> Output: Deliver PDF reports and record user preferences.
>  
> 4. Innovation: Less Structure, More Intelligence
>  
> The official mentioned that Manus follows the "Less Structure, More Intelligence" philosophy, which is to reduce preset structures and rely on intelligent emergence to solve problems. This may be reflected in:
>  
> Dynamic path optimization: select the optimal execution path through reinforcement learning instead of a fixed process.
> Adaptive tool chain: automatically select or combine tools according to task requirements.
> Robustness: performs well in long-tail tasks (such as cross-platform data retrieval).
>  
> 5. Performance support: GAIA benchmark performance
>  
> Manus surpassed OpenAI's Deep Research in the GAIA benchmark test, indicating that its architecture has significant advantages in complex problem solving and tool calling. Possible optimizations include:
>  
> Multi-model collaboration: combining the strengths of different models (such as one model is good at reasoning, and the other is good at code generation).
> Asynchronous processing: parallel execution of subtasks in the cloud to shorten response time.
> High-quality training data: using large-scale, multi-domain data to improve generalization capabilities.
>  
> The key point here is the multi-agent cluster collaboration capability of the execution layer, which is also the core point we have been building

## 102. Mar 6, 2025 · 5:04 PM UTC · 1897694881008828750#m
- 链接：https://twitter.com/Zenith_shade789/status/1897694881008828750#m
- 作者：@Zenith_shade789

> 这两天看到 #Manus 热度持续上升，讲讲我的见解。
>  
> 一、Manus的逻辑分析
> 1. 核心理念：从“知”到“行”的闭环
> Manus的名字源自拉丁语“mens et manus”（意为“手脑并用”或“知行合一”），这揭示了其逻辑核心：不仅理解用户意图（“知”），还能通过工具调用和任务执行交付成果（“行”）。这与传统对话型AI（如ChatGPT，deepseek）的区别在于，Manus强调端到端的任务闭环，而非仅提供建议或文本输出。
>  
> 输入层：用户以自然语言提出需求（如“分析特斯拉股价趋势并生成可视化报告”）。
> 处理层：分解任务、规划步骤、调用工具、执行操作。
> 输出层：直接交付完整成果（如PDF报告、Excel表格、可视化网页）。
> 这种逻辑设计的核心在于“自主性”和“执行力”，让AI从被动助手升级为主动代理人。
>  
> 2. 任务分解与规划
> Manus能处理复杂任务的关键在于其任务分解能力。例如，在筛选简历的案例中，它能：
>  
> 解压压缩文件。
> 逐页分析简历内容。
> 提取关键信息（如工作经验、技能）。
> 根据职位需求排序并生成推荐列表。
> 这表明其逻辑包含一个动态规划模块，以下步骤：
>  
> 意图解析：通过自然语言处理（NLP）理解用户需求。
> 任务拆解：将复杂需求分解为可执行的子任务（如“搜索数据→分析数据→生成图表”）。
> 优先级排序：根据任务依赖关系和逻辑顺序排列子任务。
> 动态调整：在执行过程中根据反馈或新信息调整计划。
> 3. 工具调用与执行
> Manus的执行逻辑依赖于强大的工具调用能力。它能在虚拟环境中操作浏览器、代码编辑器、文件处理器等工具。
>  
> 例如，在房产筛选案例中，它能：
>  
> 搜索社区安全数据。
> 计算预算可负担的房价。
> 从房地产网站提取房源信息。
> 整合为详细报告。
> 其逻辑设计中嵌入了工具链管理模块，可能通过API或模拟人类操作（类似RPA，机器人流程自动化）与外部资源交互。
>  
> 4. 学习与优化
> Manus具备记忆和学习能力。
>  
> 例如，用户要求结果以表格形式呈现后，它会记住这一偏好，并在后续任务中优先采用。这种逻辑表明其内置了上下文记忆和强化学习机制：
>  
> 上下文记忆：记录用户交互历史和偏好。
> 反馈优化：根据用户满意度调整执行策略。
>  
> 二、Manus的架构剖析
> 基于现有信息和AI Agent领域的技术趋势，Manus的架构可能是一个多智能体系统（Multiple Agent System），运行于云端虚拟机环境中。以下是详细推演：
>  
> 1. 总体架构：多智能体协同
> Manus采用Multiple Agent架构，将任务处理分为多个独立但协作的智能体模块。这种设计能提高效率和鲁棒性，可能包括：
>  
> 规划代理（Planner Agent）：负责任务分解和步骤规划。
> 执行代理（Executor Agent）：调用工具并完成具体操作。
> 验证代理（Validator Agent）：检查结果准确性和完整性。
> 记忆代理（Memory Agent）：管理上下文和用户偏好。
> 这些代理在沙盒环境中独立运行，通过消息传递或共享内存协同工作。例如，在简历筛选任务中：
>  
> 规划代理生成任务清单（如“解压文件→提取信息→排序”）。
> 执行代理调用Python脚本处理数据。
> 验证代理核对结果一致性。
>  
> 2. 技术组件
> (1) 计算环境：云端虚拟机
> Manus运行在独立的云端虚拟机中，类似Anthropic的Computer Use模式。这种设计有以下优势：
>  
> 隔离性：避免本地资源限制，任务可在后台异步执行。
> 扩展性：支持多任务并行处理。
> 工具集成：虚拟机预装浏览器、代码编辑器等工具，模拟人类操作。
> (2) 大语言模型（LLM）
> 作为核心推理引擎，Manus可能基于一个或多个优化后的大语言模型（如LLaMA、Grok或其他开源模型的变种），
> 负责：
> 理解自然语言指令。
> 生成工具调用指令或代码。
> 输出自然语言解释。
> (3) 工具调用接口
> Manus的技术亮点在于其工具调用能力，可能通过以下方式实现：
>  
> API集成：直接调用外部服务（如Yahoo Finance获取股票数据）。
> 模拟操作：通过浏览器自动化（如Selenium）操作网页。
> 代码生成：动态编写并执行Python、HTML等脚本。
> (4) 文件管理系统
> 在任务执行中，Manus能处理文件（如解压简历、生成Excel）。这需要一个虚拟文件系统，支持文件的增删改查，并将结果以多种格式输出。
>  
> 3. 数据流与工作流
> 以下是一个典型任务（如“分析bitcoin趋势”）的工作流推演：
>  
> ❇️输入：用户输入“分析2024年bitcoin价格趋势并生成报告”。
> ❇️规划：规划代理分解为“获取数据→分析趋势→生成图表→整合报告”。
> ❇️执行：执行代理调用金融API获取价格数据。
> 编写Python脚本计算趋势并生成折线图。
> 将结果整合为PDF。
> ❇️验证：验证代理检查数据准确性和图表可读性。
> ❇️输出：交付PDF报告并记录用户偏好。
> 4. 创新点：Less Structure, More Intelligence
> 官方提到Manus遵循“Less Structure, More Intelligence”哲学，即减少预设结构，依靠智能涌现解决问题。这可能体现在：
> 动态路径优化：通过强化学习选择最优执行路径，而非固定流程。
> 自适应工具链：根据任务需求自动选择或组合工具。
> 鲁棒性：在长尾任务（如跨平台数据检索）中表现出色。
>  
> 5. 性能支撑：GAIA基准表现
> Manus在GAIA基准测试中超越OpenAI的Deep Research，表明其架构在复杂问题解决和工具调用上有显著优势。可能的优化包括：
> 多模型协同：结合不同模型的长处（如一个模型擅长推理，另一个擅长代码生成）。
> 异步处理：云端并行执行子任务，缩短响应时间。
> 高质量训练数据：使用大规模、多领域数据提升泛化能力。
>  
> 个人观点这里核心点在于执行层的多代理集群协作能力以及云端虚拟机，或许我们还能做些什么？
>  
> 希望各位能拿出宝贵的建议看法！！！！

## 103. Mar 6, 2025 · 12:07 PM UTC · 1897619955576901829#m
- 链接：https://twitter.com/Zenith_shade789/status/1897619955576901829#m
- 作者：@Zenith_shade789

> @danielesesta 不要吃kfc了，讲讲你的故事

## 104. Mar 6, 2025 · 12:06 PM UTC · 1897619701418852689#m
- 链接：https://twitter.com/Zenith_shade789/status/1897619701418852689#m
- 作者：@Zenith_shade789

> hey anon 🟧
> Defi是过去。
> AI是当下。
> DeFAI 是草台班子。

## 105. Mar 5, 2025 · 4:17 PM UTC · 1897320578031476834#m
- 链接：https://twitter.com/Zenith_shade789/status/1897320578031476834#m
- 作者：@Zenith_shade789

> 群友问我怎么没很少打狗了？
> 最近一直在做产品转builder了。
> 在加密行业，一直保持着干中学态度，将我刻录进链上吧！
> 大的要来了！！！
>  
> Friends in the group asked me why I don’t do much dog fighting?
> Recently, I have been working on products and switching to builder.
> In the crypto industry, I have always maintained a learning-by-doing attitude, so please record me on the chain!
> The big one is coming!!!

## 106. Mar 5, 2025 · 3:11 PM UTC · 1897303999671595128#m
- 链接：https://twitter.com/Zenith_shade789/status/1897303999671595128#m
- 作者：@Zenith_shade789

> Marketing Manager学起来，撬动市场流量

## 107. Mar 5, 2025 · 3:01 PM UTC · 1897301547228520557#m
- 链接：https://twitter.com/Zenith_shade789/status/1897301547228520557#m
- 作者：@Zenith_shade789

> Defi是过去
>  
> AI是当下
>  
> DeFAI 是未来
>  
> @danielesesta
>  
> Do those woman look good compared to me？！！

## 108. Mar 5, 2025 · 1:22 PM UTC · 1897276539361960413#m
- 链接：https://twitter.com/Zenith_shade789/status/1897276539361960413#m
- 作者：@Zenith_shade789

> 该如何找到庄盘以及庄地址 or 内募地址？
>  
> 什么是庄地址或内募地址？
> “庄”通常指代市场中的操盘者或主力资金，而“内募地址”则可能是项目内部参与者或早期投资者的钱包地址。这些地址往往通过低成本进入市场，并在后续操作中获利。
>  
> 通常具备以下特点：
>  
> ❇️100%获利：包括关联地址在内，整体盈利能力强。
> 进场市值低：在项目早期或底部买入。
> ❇️持仓排名在100-500左右：不是最顶尖的大户，但属于中上层持仓者。
> ❇️资金充足：地址内有足够的资金支持操作。
> 如何抓取内募地址？
>  
> 1. 获取链上数据
> 工具推荐：helius.dev 是一个不错的区块链数据接口服务，可以用来获取交易记录、地址持仓等信息。
> 其他选项： Dune Analytics、Etherscan API 或其他链上分析工具（如 Nansen、Glassnode）来补充数据。
> 2. 数据分析条件设置
> 为了筛选出潜在的“庄”或“内募地址”，可以设置以下条件：
>  
> ❇️持续买入行为：在某个市值范围（如低市值阶段）内，某个地址持续买入特定代币。
> ❇️持仓排名异动：关注排名在100-500之间的地址，观察代币增持或稀释情况。
> ❇️底部买入后转移：在低市值阶段买入的地址，之后出现代币转移（可能是分发或操盘迹象）。
> 通过这些条件，可以初步判断某个代币是否存在“庄”的操控迹象。
>  
> 3. 捕获庄家地址
> 在筛选出可疑代币后，进一步锁定具体地址，可以设置以下条件：
>  
> ❇️资金流向：追踪地址的资金来源和去向，尤其是与早期低成本买入相关的地址。
> ❇️交易模式：是否有规律性的大额买入/卖出，或与项目方地址的关联交易。
> ❇️持仓稳定性：庄家地址往往在关键时刻保持持仓，而非频繁小额交易。
>  
> 如何抓取庄盘再抓取内募地址？
>  
> 问题 1：如何准确筛选有庄的 token？
> 难点：链上数据庞杂，很多 token 的交易可能是散户行为或市场自然波动，难以区分庄家操作。
> 解决思路：量化指标：设定明确的阈值，比如“前 100-500 名地址持仓占比 > 50%”或“某地址买入量占总交易量 > 10%”。
> 时间窗口：关注 token 早期（比如上线后 1-3 个月）的交易模式，庄家往往在此阶段布局。
> 异常检测：用统计方法（如 Z-Score）找出交易量或持仓的异常值。
>  
> 问题 2：内幕地址的伪装
> 难点：庄家可能通过多个地址分散持仓，或通过混淆交易隐藏身份。
> 解决思路：关联分析：追踪代币流向，识别地址间的资金转移模式（比如 A 地址买入后转移到 B 地址）。
> 行为聚类：用机器学习对地址的行为（买入时机、持仓变化）进行聚类，找出潜在的关联地址组。
>  
> 问题 3：逻辑顺序的效率
> 难点：如果先分析庄盘的范围过大（比如扫描所有 token），可能耗时过长。
> 解决思路：预过滤：先用简单指标（如市值 < 1000 万美元、交易量突然放大）筛选出潜在有庄的 token，再深入分析。
> 并行处理：用 Agent 或多线程技术同时分析多个 token。
>  
> 初步筛选 token：条件：低市值、新上线、交易量异常波动。
>  
> 分析庄盘迹象：条件：持续买入、持仓集中、底部转移。
> 输出：标记“疑似有庄”的 token 列表。
> 如何使用agent抓获庄家地址？
> 1. Agent 的基本架构
> 数据输入模块：从链上数据源（如 Helius API）获取实时交易、持仓和地址信息。
> 规则引擎：基于你提到的特征（100%获利、低市值进场、持仓排名100-500、资金充足）设置筛选条件。
> 分析模块：处理数据并标记潜在的庄地址或内募地址。
> 输出模块：生成报告或实时警报，列出疑似地址及其行为。
> 2. Agent 的实现步骤
> Step 1: 数据采集
> 配置 Agent 调用 Helius 或其他区块链 API，订阅特定代币的交易和持仓数据。
>  
> Step 2: 设置筛选规则
> 将特征转化为 Agent 的可执行条件：
>  
> 条件1：100%获利
> Agent 计算地址的买入成本和当前市值，筛选出始终盈利的地址（包括关联地址）。
> 条件2：进场市值低
> 追踪代币历史价格，标记在底部买入的地址。
> 条件3：持仓排名100-500
> 对所有持仓地址排序，聚焦排名在此范围内的地址。
> 条件4：资金充足
> 检查地址余额和交易频率，排除小额或不活跃地址。
> Step 3: 动态分析
> Agent 实时监控符合条件的地址，观察其行为：持续买入：某个地址在低市值阶段大量积累代币。
> 持仓异动：代币增持或稀释的异常波动。
> 转移行为：底部买入后向其他地址转移代币（可能是分发或操盘信号）。
>  
> Step 4: 输出结果
> Agent 自动生成一份疑似庄地址或内募地址的列表，包含：地址详情（余额、持仓排名）。
> 交易历史（买入时间、成本、转移记录）。
> 初步结论（是否符合庄家特征）。
>  
> 3. 用 Agent 捕获庄家地址的进阶条件
> 在筛选出潜在代币后，Agent 可以进一步聚焦庄家地址：
> 资金流追踪：分析地址间的代币流转，识别与项目方或早期投资相关的地址。
> 行为模式识别：通过历史交易数据，检测是否有规律性的大额操作。
> 关联地址检测：利用图分析技术，找出同一控制人下的多个地址。
> 捕获内幕地址：条件：100%获利、低市值进场、持仓排名 100-500、资金充足。
> 输出：具体地址及其行为分析。
>  
> 验证与追踪：通过资金流和关联地址分析，确认庄家身份。
>  
> 用 Agent 实现
> 设计：
> Step 1：Token 扫描 Agent
> 实时监控新上线或低市值 token，标记异常交易行为。
> Step 2：庄盘检测 Agent
> 对筛选出的 token 分析持仓和交易数据，输出“有庄概率”。
> Step 3：地址分析 Agent
> 聚焦疑似庄盘 token，提取符合内幕地址特征的账户。
>  
> 以上仅个人建议还需验证。

## 109. Mar 5, 2025 · 1:22 PM UTC · 1897276417483878553#m
- 链接：https://twitter.com/Zenith_shade789/status/1897276417483878553#m
- 作者：@Zenith_shade789

> 什么样的产品会是一个好的产品？🤔

## 110. Mar 5, 2025 · 9:40 AM UTC · 1897220786962002250#m
- 链接：https://twitter.com/Zenith_shade789/status/1897220786962002250#m
- 作者：@Zenith_shade789

> 5.选择AI图像（每周只能做一次免费）
>  
> 5. Select AI image (can only be done once a week for free)

## 111. Mar 5, 2025 · 9:34 AM UTC · 1897219178387718556#m
- 链接：https://twitter.com/Zenith_shade789/status/1897219178387718556#m
- 作者：@Zenith_shade789

> 4.VeriStar ZK Quiz 答题免费 ：很简单直接答题免费12星
> Prover/verifier
> Succinctness
> Zero-knowledge
> Verify proofs
>  
> VeriStar ZK Quiz Free Answering: Very simple and direct answering, free 12 stars
> Prover/verifier
> Succinctness
> Zero-knowledge
> Verify proofs

## 112. Mar 5, 2025 · 9:32 AM UTC · 1897218663163551934#m
- 链接：https://twitter.com/Zenith_shade789/status/1897218663163551934#m
- 作者：@Zenith_shade789

> 3. 最后一个区块 - 选择 1 到 5 之间的数字 - 要确认，请输入 CONFIRM（对于简单的 10 到 30 颗星，选择 1 代表 30，只需花费 3.5u）
> Ethereum: The Last Block-Choose a number between 1 and 5 - To confirm, type CONFIRM (for a simple 10 to 30 stars, choose 1 for 30, it only costs 3.5u)

## 113. Mar 5, 2025 · 9:30 AM UTC · 1897218074279129273#m
- 链接：https://twitter.com/Zenith_shade789/status/1897218074279129273#m
- 作者：@Zenith_shade789

> 2. FloppyGPU - 点击空格键玩游戏 - Proof of Hit（可能得不到）
> 这个小鸟游戏玩起来非常难，你只是用鼠标的空格键来控制力度，而且从上到下都不能碰到水泥管。我的最高记录是40分，40分可以得20颗星左右。上周有个小鸟比赛，我看到的最高分是96分，太惨了。
> FloppyGPU - Click the space bar to play the game - Proof of Hit (may not be obtained)
> This bird game is very difficult to play. You only use the space bar of the mouse to control the force, and you can't touch the cement pipe from top to bottom. My highest record is 40 points, and 40 points can get about 20 stars. There was a bird competition last week, and the highest score I saw was 96 points

## 114. Mar 5, 2025 · 9:27 AM UTC · 1897217368251834568#m
- 链接：https://twitter.com/Zenith_shade789/status/1897217368251834568#m
- 作者：@Zenith_shade789

> 1. 超级证明者 - 选择任何区块 - 赢得证明，获得星星。
> 有两种结果，一种是损失0.04u左右，不加星，一种是损失0.11，加一颗星，一颗星的成本是0.11u。
> There are two results, one is a loss of about 0.04u without adding a star, and the other is a loss of 0.11 and adding a star. The cost of one star is 0.11u.

## 115. Mar 5, 2025 · 9:19 AM UTC · 1897215355409588247#m
- 链接：https://twitter.com/Zenith_shade789/status/1897215355409588247#m
- 作者：@Zenith_shade789

> 【Succinct】
>  
> 全网都在卷融资 5500 万 ZK 基础层：Succinct
>  
> 最近全网 Kaito Yapper 都在发要为 @SuccinctLabs
> 投票，究竟Succinct是什么？测试网门票哪里来？交易流程为何？
>  
> ✅ Succinct的项目介绍
>  
> @SuccinctLabs是一个专注于零知识证明（Zero-Knowledge Proof）的区块链基础网路，通过其核心技术SP1高效虚拟机，为开发者提供一个高性能通用的工具。
>  
> 让用户使用常用的程序语言快速建立亮点的零知识证明，以提高区块链的可拓展性和可操作性。
>  
> ✅ 融资背景
>  
> Succinct首先在 2023年种子轮融资1200万美元，2024年初A轮融资4300万美元，目前已累积5500万美元的巨额融资，由知名风投@paradigm和@BanklessVC领投，有精湛的技术和实力资金。
>  
> ✅ Succinct测试网任务
>  
> Succinct近期在X上非常火热，主要是因为其测试网任务刚推出，且Succinct为了减少卷度，当前测试网需获得邀请码才可参与，目前只有2w个参与用户。本篇将介绍邀请码的获取方式和测试网络任务的交互策略。
>  
> ➡️测试测试网邀请码获取方式：
>  
> 🔺1.关注Succinct X 官推邀请码活动
> localhost:8080/SuccinctLabs
>  
> 🔺2.Kaito Yapper 投票
> 这是目前最明确的获取方式，仅在@KaitoAI Pre-TGE 排行榜
> @SuccinctLabs 投票 1000 票，即可提供证明私讯
> @0xCRASHOUT 会提供邀请码
>  
> 🔺3.社区推广
> 为@SuccinctLabs做社区推广并标记，@0xCRASHOUT多参与Discord 活动，有机会可以获得邀请码。
>  
> ✅ Succinct的交互流程
>  
> @SuccinctLabs测试网交互模式主要为参与游戏游戏量积分，在网站交互需先充值10美元作为启动资金，因为这些游戏大多需要Gasfee，所以交互的目标是需要用Gas换取更多的积分。
>  
> 🔺1.前往Succinct官网并链接钱包。
> testnet.succinct.xyz/dashboa…
>  
> 🔺2.存入 10 $USDC 作为启动 Gasfee，测试网模拟真实主网交互，本周Succinct为上周参与用户回馈了 7.5 美元的 Gasfee。
>  
> 🔺3.如下图在网站中的游戏 页面，有6个不同的游戏可以参与，玩法各不相同，当前每颗星星积分需约0.1美元成本。
> 游戏参与教程见下
>  
> 【Succinct】
>  
> The whole network is rolling up 55 million ZK base layer: Succinct
>  
> Recently, Kaito Yapper all over the network has been posting to vote for @SuccinctLabs
> . What is Succinct? Where do the test network tickets come from? What is the transaction process?
>  
> ✅ Succinct project introduction
>  
> @SuccinctLabs is a blockchain basic network focusing on zero-knowledge proof (Zero-Knowledge Proof). Through its core technology SP1 efficient virtual machine, it provides developers with a high-performance general tool.
>  
> Allow users to use common programming languages ​​to quickly establish highlight zero-knowledge proofs to improve the scalability and operability of blockchain.
>  
> ✅ Financing background
>  
> Succinct first raised $12 million in the seed round in 2023 and $43 million in the A round in early 2024. It has accumulated a huge amount of financing of $55 million, led by well-known venture capital @paradigm and @BanklessVC, with superb technology and strong funds.
>  
> ✅ Succinct test network task
>  
> Succinct has been very popular on X recently, mainly because its test network task has just been launched, and in order to reduce the volume, Succinct currently requires an invitation code to participate in the test network. Currently, there are only 20,000 participating users. This article will introduce how to obtain the invitation code and the interactive strategy of the test network task.
>  
> ➡️ How to obtain the test network invitation code:
>  
> 🔺1. Follow Succinct X official invitation code event
> localhost:8080/SuccinctLabs
>  
> 🔺2. Kaito Yapper voting
> This is the most clear way to obtain it. Just vote 1,000 times in the @KaitoAI Pre-TGE ranking
> @SuccinctLabs, you can provide proof and send a private message
> @0xCRASHOUT will provide an invitation code
>  
> 🔺3. Community promotion
> Do community promotion for @SuccinctLabs and tag it. @0xCRASHOUT participates in more Discord activities and has a chance to get an invitation code.
>  
> ✅ Succinct interaction process
>  
> @SuccinctLabs test network interaction mode is mainly to participate in the game volume points. You need to recharge $10 as the starting capital before interacting on the website. Because most of these games require Gasfee, the goal of interaction is to use Gas to exchange for more points.
>  
> 🔺1. Go to the Succinct official website and link your wallet.
> testnet.succinct.xyz/dashboa…
>  
> 🔺2. Deposit 10 $USDC as the starting Gasfee. The test network simulates the real main network interaction. This week, Succinct gave back $7.5 in Gasfee to users who participated last week.
>  
> 🔺3. As shown in the following figure, on the game page of the website, there are 6 different games to participate in, and the gameplay is different. Currently, each star point costs about $0.1.
> See the game participation tutorial below

## 116. Mar 4, 2025 · 2:13 AM UTC · 1896745808948773163#m
- 链接：https://twitter.com/Zenith_shade789/status/1896745808948773163#m
- 作者：@Zenith_shade789

> Gate.io Launchpool首发上线：质押赚取 #FORM1 百万美金空投@0xForm
>  
> $GT $BTC $USDT #FORM1 质押池 71,428,571枚 #FORM1 质押奖励还可参与 最高预估年化可达608.33%
>  
> 项目简介：Form (FORM1) 作为去中心化身份 (DID) 和数据管理 (DDM) 基础设施的创新项目，致力于打造更安全、隐私友好的 Web3 生态。
> Gate.io Launchpool，质押 USDT、GT、BTC 即可高效挖矿，最高年化收益达 608.33%！
> 质押入口: gate.io/zh/launchpool/226?gt…
>  
> 🔹 超高 APR——质押即享高收益，轻松赚取 FORM1 代币
> 📷 多种质押池——支持 USDT、GT、BTC，灵活选择适合自己的方式
> 📷 USDT 质押池：稳定收益，适合稳健派
> 📷 GT 质押池：平台币增长潜力大，长期持有更有价值 📷gate.io/zh/rewards_hub?ch=Re……
>  
> Gate.io Launchpool让挖矿更简单，收益更丰厚！ 立即参与！  #Gateio #Launchpool  #空投  #GT

## 117. Mar 3, 2025 · 5:26 AM UTC · 1896432044785520874#m
- 链接：https://twitter.com/Zenith_shade789/status/1896432044785520874#m
- 作者：@Zenith_shade789

> Obviously, @0xCRASHOUT ranks first
>  
> @SuccinctLabs @0xCRASHOUT

## 118. Mar 1, 2025 · 5:47 AM UTC · 1895712593270186107#m
- 链接：https://twitter.com/Zenith_shade789/status/1895712593270186107#m
- 作者：@Zenith_shade789

> 10b+ soonnn $s @SonicLabs @AndreCronjeTech

## 119. Feb 28, 2025 · 6:52 PM UTC · 1895547748419846252#m
- 链接：https://twitter.com/Zenith_shade789/status/1895547748419846252#m
- 作者：@Zenith_shade789

> At this time my hourly wage is 120u/h😂

## 120. Feb 28, 2025 · 6:51 PM UTC · 1895547395649519756#m
- 链接：https://twitter.com/Zenith_shade789/status/1895547395649519756#m
- 作者：@Zenith_shade789

> If you don't like it, don't try it. $s

## 121. Feb 28, 2025 · 11:47 AM UTC · 1895440786004807756#m
- 链接：https://twitter.com/Zenith_shade789/status/1895440786004807756#m
- 作者：@Zenith_shade789

> 感谢人生中的老师，确实学到了很多

## 122. Feb 28, 2025 · 11:46 AM UTC · 1895440535562924096#m
- 链接：https://twitter.com/Zenith_shade789/status/1895440535562924096#m
- 作者：@Zenith_shade789

> 你一定可以的！！！！

## 123. Feb 28, 2025 · 11:45 AM UTC · 1895440238341931312#m
- 链接：https://twitter.com/Zenith_shade789/status/1895440238341931312#m
- 作者：@Zenith_shade789

> 数字游民的Web3回忆录
> 序章：平凡的起点
> 2003年出生于江西小县城的我成绩只能算中等偏下，21年高中毕业落榜的我一度想尽早进入社会，家里的劝导下进入了一所大专。家里没有显赫的背景，学校也不是名校，我的起点就像一个“草台班子”——简陋、粗糙，甚至有点不起眼。在很多人眼里，这样的出身似乎注定与成功无缘。但我不这么认为。在Web3这片充满未知的海洋里，我坚信，只要有自制的桨和一颗不服输的心，哪怕是草台班子，也能乘风破浪。
>  
> “草台班子”，一群非专业或业余的人组成的团队，资源有限，背景普通。但对我来说，它象征的是一种不被出身定义、不依赖传统路径的精神。我的故事，就是从这个草根起点开始的逆袭之旅。
>  
> 第一章：炒鞋与美妆——摸索中的觉醒
> 我的创业之路始于大学时期。那时候，炒鞋和美妆是热门的捞偏门。我从炒鞋入手，学会了如何在二级市场低买高卖，赚取差价。起初，也只是个小玩家，靠着低买高卖，逐渐攒下了一些资金。后来，尝试了美妆代购和分销，接触到了更广阔的市场。
>  
> 这些经历也让我积累了经验，却也让我感到局限。我隐约意识到，真正的机会不在这些传统领域。已经不记得是从哪里知道了Web3这个词（抖音还是知书亦或是朋友圈）。再尝到甜头后，我决定放下手中的鞋子和化妆品，投身这片新大陆。
>  
> 第二章：玩资金盘——风险与教训
> 在迈向Web3之前，我曾短暂涉足资金盘。那是一个充满诱惑和陷阱的世界。高额的回报承诺让我心动，我投入了一些资金，希望能快速致富。然而，现实很快给了我一记重击。资金盘的崩盘让我损失惨重，也让我清醒过来：真正的成功不是靠投机，而是靠实干和创新。
>  
> 这次失败没有击垮我，反而点燃了我对Web3的渴望。只有在新兴领域，才能找到更大的机遇。
>  
> 第三章：不懂NFT——从抢购到建仓
> 2022年初，我第一次接触NFT。那时，我对它一无所知，只知道它是一种非同质化数字资产，可以在市场上买卖。起初我从最简单的玩法开始：参与开售抢购，转手以10倍价格售出。每次这种快速交易都让我兴奋不已，但我很快发现，我严重低估了市场的fomo情绪。
>  
> 开始深入研究NFT的底层逻辑。慢慢学会了建仓、吸筹，甚至尝试联合坐庄，操控市场走势。不再是简单的买进卖出，是对市场趋势和用户心理的博弈了。也从一个懵懂的新手，逐渐变成了一个懂得如何布局的玩家。
>  
> 第四章：创新数藏圈——荷兰拍的尝试
> 在NFT的世界里，我看到了更大的可能性。我开始探索数字藏品（数藏）的玩法，尝试用荷兰拍这种降价拍卖方式与平台方合作作为冷启动吸引用户。组织了几次小型荷兰拍活动，虽然规模不大，但用户的热情让我惊喜。（那时候还没用tg大家都在WeChat群聊里10多个拍卖群用同步机器人）这让我意识到，数藏圈有巨大的潜力，而创新是打开这扇门的钥匙。
>  
> 3月份决定更进一步，创办数藏平台。我想成为这个市场的“球员”和“守门员”——既参与市场竞争，又制定游戏规则。创业的路异常艰辛，资金、技术、市场，三重挑战接踵而至。（挂牌费，开发等等问题）。
>  
> 第五章：被国企收割——阶段性草台班子的终结
> 5月中旬，终于有了起色，却也吸引了国企的注意。他们看到了Web3的潜力，但又担心在市场的政策不明朗情况下一份红头受到牵连。我本以为这是草台班子的胜利，但现实却像一场“收割”。国企凭借雄厚的资源和背景，让我无力抗衡。最终，也不得不退出这个经历几个月日夜心血打造的平台。（每天睡不着的感觉很难熬）
>  
> 这次经历深刻体会到，草台班子虽然能凭努力闯出一片天，但在巨头面前，依然脆弱。也开始反思，如何在Web3的世界里，既保持创新，又避免被“大鱼”吞噬。
>  
> 第六章：后来的种种——草台班子的重生
> 离开数藏平台后，我并没有放弃Web3。无非再找机会从来一遍，此后在校期间一边干合约（高倍杠杆的我经常爆仓😂)，用整整一年创办校园商业综合体。（也是得益于校园政策以及兄弟们给力，阶段性垄断了外面资金进入校园口子）让我积累再来一遍的底气。期间正处在熊市阶段除了校园业务外，我一个个教他们撸猫拿到了不错的结果（期间也认识了身边一些web3朋友 @Divoll_Law  @Amber_Zero0 以及一些老师  @thecryptoskanda @ZTZZBTC @Leoninweb3 @0xamyinthewoods @0xCryptoWing  @mia_okx 就不一 一列举了）
>  
> 尾声：草台班子的意义
> 从炒鞋到美妆，从玩资金盘到不懂NFT，再到创办数藏平台被国企收割，我的Web3之旅充满了起伏。但每一次挫折都都是一次锤炼，每一阶段成功都让我更自信。我的故事证明，草台班子不是局限，而是一种自由——一种不被定义、不被框住的自由。
>  
> 在Web3这个不问出处的舞台上，成功不取决于你是名校精英还是普通院校毕业生，而是取决于你的努力、能力和对机会的把握。我用自己的经历告诉世人，只要有决心和实干精神，哪怕是草台班子，也能在这个风起云涌的世界中书写属于自己的传奇。
>  
> 25年新的开始，愿大家越来越好，一路长虹！你一定可以的！！！！
> 彩蛋环节：

## 124. Feb 27, 2025 · 12:42 PM UTC · 1895092191770616012#m
- 链接：https://twitter.com/Zenith_shade789/status/1895092191770616012#m
- 作者：@Zenith_shade789

> I'm not a gambling man but I would full port here and make heaps of money
>  
> Will cover using pendle momentarily
>  
> (Zero impermanent loss if you hold til maturity btw(
>  
> NFA hahahaha

## 125. Feb 26, 2025 · 2:54 PM UTC · 1894763019558895806#m
- 链接：https://twitter.com/Zenith_shade789/status/1894763019558895806#m
- 作者：@Zenith_shade789

> 欢迎sonic生态alpha伙伴交流

## 126. Feb 26, 2025 · 2:53 PM UTC · 1894762700896653799#m
- 链接：https://twitter.com/Zenith_shade789/status/1894762700896653799#m
- 作者：@Zenith_shade789

> ►关于 Memecoin
>  
> @GOGLZ_SONIC
> @TinHat_Cat
> @fantomsonicinu
> @HedgePot
> @indi_sonic
> @tysonicgodd
> @FROQ_SONIC
> @sDOG_SONIC
> @MoonBaySonic
> @MuttskiTheDog
> @MCLBfbombsFIERY
> @memetoona
> @derpedewdz
> @ShedewOnSonic
> @ruggiespizza
> @shibasonicmeme
> @PassThe_JOINT
> @BeanieZombie
> @ShibaPoCONK
> @Anon_Andre01
> @HOOPS_is_Sonic
> @BEER_on_SONIC
> @SonicKings_K
> @LudwigOnSonic
> @sonicfamilio

## 127. Feb 26, 2025 · 2:50 PM UTC · 1894762000548569381#m
- 链接：https://twitter.com/Zenith_shade789/status/1894762000548569381#m
- 作者：@Zenith_shade789

> Sonic 上的其他协议目前还没有产生 AP，但它们值得探索，因为它们已经上线。  以下是我们的选择（NFA 和 DYOR）：
> ✦@OriginProtocol
> ✦@AmpedFinance
> ✦@magpieprotocol
> ✦@sdotfunapp
> ✦@yokodotlive
> ✦@odosprotocol
> ✦@HeyAnonai
> ✦@Lombard_Finance
> ✦@BoomDEX
> ✦@eulerfinance
> ✦@Contango_xyz
> ✦@zerolendxyz
> ✦@0xfluid
> ✦@GearboxProtocol
> ✦@eggsonsonic
> ✦@SnakeOnSonic
> ✦@degenexpress69
> ✦@moonshot
>  
> 了解最新更新和完整的生态系统覆盖，可以关注
> @SonicEcosystem

## 128. Feb 26, 2025 · 2:48 PM UTC · 1894761413090181165#m
- 链接：https://twitter.com/Zenith_shade789/status/1894761413090181165#m
- 作者：@Zenith_shade789

> 以及
> ✦@machfi_xyz
> ✦@VicunaFinance
> ✦@yearnfi
> ✦@SilverSwapDex
> ✦@yel_finance
> ✦@CurveFinance
> ✦@avalonfinance_
> ✦@Gravity_Finance
> ✦@GammaStrategies
> ✦@Penpiexyz_io
> ✦@steerprotocol
> ✦@stryke_xyz
> ✦@marginzero_xyz
> ✦@spectra_finance
> ✦@NapierFinance
> ✦@Lynx_Protocol
> ✦@vertex_protocol
> ✦@pendle_fi
> ✦@rabbitx_io
> ✦@StableJack_xyz
>  
> 🔶 Games
> ✦@EstforKingdom
> ✦@sacra_fi

## 129. Feb 26, 2025 · 2:45 PM UTC · 1894760753556820274#m
- 链接：https://twitter.com/Zenith_shade789/status/1894760753556820274#m
- 作者：@Zenith_shade789

> ►生态系统
> 目前生态，Sonic 上共有 141 个应用程序，还会进一步扩展。  因此，可以从一些协议开始，这些协议也为您提供 AP，因为它使您有机会获得$S空投：  🔶 DeFi（DEX、借贷、衍生品、稳定币、LST、BTCFi）第 1 部分
> ✦@ShadowOnSonic
> ✦@MetropolisDEX
> ✦@SpookySwap
> ✦@Equalizer0x
> ✦@beets_fi
> ✦@Rings_Protocol
> ✦@ImpermaxFinance
> ✦@SolidlyLabs
> ✦@SolvProtocol
> ✦@wagmicom
> ✦@SwapXfi
> ✦@_WOOFi
> ✦@SuperSonicDEX
> ✦@soneta_xyz
> ✦@beefyfinance
> ✦@DebitaFinance
> ✦@ichifoundation
> ✦@SiloFinance

## 130. Feb 26, 2025 · 2:43 PM UTC · 1894760116081340855#m
- 链接：https://twitter.com/Zenith_shade789/status/1894760116081340855#m
- 作者：@Zenith_shade789

> ► Sonic Earn
>  
> 在进入Sonic之前，请确保您已注册my.soniclabs.com/points （邀请码RZYCH2）
>  
> 为什么？因为您可以通过以下步骤获得 2 亿 $S 空投（约 1.74 亿美元以上）：
> --请使用 RZYCH2 作为您的参考代码--
> ✦被动积分(PP):在您的钱包中保存$scUSD和$scETH等白名单资产。
> ✦活跃点数（AP）：在 Sonic 生态系统中直接使用 WL 资产。
> ✦项目应用积分：应用程序争夺 Sonic Gems，可兑换$S代币，然后分发给用户。
> 根据在 Sonic 生态系统中的参与程度获得被动积分(PP)和/或主动积分 (AP)。

## 131. Feb 26, 2025 · 2:38 PM UTC · 1894758944217956735#m
- 链接：https://twitter.com/Zenith_shade789/status/1894758944217956735#m
- 作者：@Zenith_shade789

> ►连接您的资产  首先，你需要将你的资产桥接到@SonicLabs。  使用原生 Sonic Gateway 从以太坊进行桥接，或者为其他链选择第三方选项：
> ✦@deBridgeFinance
> ✦@rhinofi
> ✦@Owlto_Finance
> ✦@sideshiftai
> ✦@eywaprotocol
> ✦@mesonfi
> ✦@BungeeExchange
> ✦@symbiosis_fi
> ✦@NitroByRouter
> ✦@InterportFi
> ✦@gasdotzip
> ✦@Orbiter_Finance
> ✦@Entanglefi
> ✦@CryptoRubic
> 建议使用@Rabby_io作为首选钱包。

## 132. Feb 26, 2025 · 2:36 PM UTC · 1894758416054432070#m
- 链接：https://twitter.com/Zenith_shade789/status/1894758416054432070#m
- 作者：@Zenith_shade789

> ► Sonic Traction (Data per Feburary 26, 2025)
>  
> Sonic TVL 自今年年初以来增长了约 3500%（截至撰写本文时目前约为 6.7 亿美元）
> ✦总交易量超过 3070 万笔
> ✦已部署 70.3k+ 份合约
> ✦ 847k+唯一地址 ✦ 461k+ $S FeeM 已分发
> ✦仅 DeBridge 就为桥接资金筹集了 1.5 亿美元  到目前为止，这似乎只是 Sonic 的一个开始。
> 高/低@sealaunch_、@DefiLlama、@deBridgeFinance、@0xKhmer

## 133. Feb 26, 2025 · 2:34 PM UTC · 1894757987136483364#m
- 链接：https://twitter.com/Zenith_shade789/status/1894757987136483364#m
- 作者：@Zenith_shade789

> ►@SonicLabs到底是什么？
> 从外观上看，Sonic 似乎是一个高性能的 EVM Layer 1 区块链。
> 然而，它不仅仅拥有令人印象深刻的技术；它还提供可持续的激励措施，创造一个回报用户和建设者的繁荣生态系统。
> ✦每秒 10,000 笔交易
> ✦ 1 秒内完成 ✦每笔交易费用 <0.01 美元
> ✦用户激励：白名单资产总计 2 亿$S
> ✦费用货币化：开发者赚取 90% 的应用程序 gas 费用

## 134. Feb 26, 2025 · 2:33 PM UTC · 1894757670839865754#m
- 链接：https://twitter.com/Zenith_shade789/status/1894757670839865754#m
- 作者：@Zenith_shade789

> 有关@SonicLabs及其生态系统的所有信息。
>  
> 涵盖技术、桥接、生态系统，并最大化您的$S空投。
>  
> 一本实用的指南。 🧵

## 135. Feb 25, 2025 · 2:21 AM UTC · 1894211016747032795#m
- 链接：https://twitter.com/Zenith_shade789/status/1894211016747032795#m
- 作者：@Zenith_shade789

> 哥你说呀是不是偷偷玩狱警手机了？sol危！救！

## 136. Feb 21, 2025 · 7:16 PM UTC · 1893016880547238083#m
- 链接：https://twitter.com/Zenith_shade789/status/1893016880547238083#m
- 作者：@Zenith_shade789

> This is so amazing @AndreCronjeTech  @SonicLabs

## 137. Feb 21, 2025 · 5:54 PM UTC · 1892996282022859103#m
- 链接：https://twitter.com/Zenith_shade789/status/1892996282022859103#m
- 作者：@Zenith_shade789

> $THC (Tin Hat Cat)
> 阴谋论主题迷因币,Sonic DEX 交易，迷因大赛核心。
> 官推:@TinHat_Cat
> 关注列表 (@SonicLabs,@AndreCronjeTech ...)
> 官网:tinhatcat.com/   ca:0x17Af1Df44444AB9091622e4Aa66dB5BB34E51aD5
>  
> $IND
> 疑似 Sonic 官方支持的迷因项目。
> 官推:@indi_sonic
> 关注列表 (@SonicLabs ...)
> 官网:indisonic.xyz/  ca:0x4eec869d847a6d13b0f6d1733c5dec0d1e741b4f
>  
> $DERP
> Sonic 最大 PFP（头像）迷因项目,NFT + 迷因，Sonic NFT 市场交易。
> 官推:@derpedewdz
> 关注列表(@ShadowOnSonic， @0xRoninSam ... ）
> 官网:derpe.xyz/  ca:0xe920d1da9a4d59126dc35996ea242d60efca1304
>  
> $HOOPS
> 篮球主题迷因币,Sonic 社区meme项目。
> 官推:@HOOPS_is_Sonic
> 关注列表 (@SonicLabs，@0xRoninSam ...）
> 官网:linktr.ee/hoopsmeme  ca:0xd4A5C68A1Ed1fC2bb06cbA2d90d6ADEee7503671
>  
> $TOONA
> 基础迷因项目,Sonic 早期社区meme代币
> 官推:@memetoona
> 关注列表 (@0xRoninSam ...）
> 官网:linktr.ee/memetoona  ca:0xf4f9c50455c698834bb645089dbaa89093b93838
>  
> $GOGLZ
> (Goggle Heads) 链游代币，用于 NFT 和奖励,Sonic 主网链游。
> 官推:@GOGLZ_SONIC
> 关注列表（ @blknoiz06 ,@0xRoninSam ... ）
> 官网:goglz.io/  ca:0x9fDbC3f8Abc05Fa8f3Ad3C17D2F806c1230c4564
>  
> $SONIC
> sonic上早期模因，索尼克 sonic生态模因币
> 官推:@MascotSonic
> 官网:mascotsonic.com  ca:0x446649F0727621BDbB76644B1910be2163b62a11
>  
> 以上为部分sonic生态项目听说 AC 最近会带来 Launchpad @AndreCronjeTech

## 138. Feb 21, 2025 · 5:52 PM UTC · 1892995889373028695#m
- 链接：https://twitter.com/Zenith_shade789/status/1892995889373028695#m
- 作者：@Zenith_shade789

> $EQUAL (Equalizer Token)
> Equalizer DEX 治理代币,Sonic 交易基础设施。
> 官推:@Equalizer0x
> 关注列表(@Rings_Protocol,@yel_finance,@SonicLabs,@0xRoninSam,@blknoiz06 ...)
> 官网:equalizer.exchange  ca:0xddf26b42c1d903de8962d3f79a74a501420d5f19
>  
> $SILO (Silo Finance)
> 借贷协议治理代币,Sonic DeFi 服务。
> 官推:@SiloFinance
> 关注列表(@SonicLabs,@0xRoninSam,@AndreCronjeTech,@0xcryptowizard,@ShadowOnSonic,@travisbickle0x ...)
> 官网:silo.finance
>  
> $ECO
> 环保主题迷因币，与 ECOllection NFT 系列（ @Zzvonarka  Sonic DEX（如 Equalizer）交易，社区驱动。
> 官推:@block_ecologist
> 关注列表(@SonicLabs,@AndreCronjeTech ,@michaelfkong ...)
> 官网: fantom.eco/  ca:0x7a08bf5304094ca4c7b4132ef62b5edc4a3478b7

## 139. Feb 21, 2025 · 5:51 PM UTC · 1892995607524180072#m
- 链接：https://twitter.com/Zenith_shade789/status/1892995607524180072#m
- 作者：@Zenith_shade789

> $BEETS
> 流动性协议代币，与 Sonic 集成。 Sonic DeFi 流动性。   官推:@beets_fi
> 关注列表(@Rings_Protocol,@ShadowOnSonic,@0xseg,@SonicLabs,@AndreCronjeTech...)
> 官网:beets.fi/  ca:0x2D0E0814E62D80056181F5cd932274405966e4f0
>  
> $METRO
> DLMM DEX 代币,Sonic 创新 DeFi。
> 官推:@MetropolisDEX
> 关注列表(@AndreCronjeTech,@0xRoninSam ...)
> 官网:metropolis.exchange/  ca:0x71e99522ead5e21cf57f1f542dc4ad2e841f7321
>  
> $WHALE
> DeFAI（DeFi + AI）相关代币。
> 官推:@Whale_Tokens
> 关注列表(@0xRoninSam,@Equalizer0x ...)
> 官网:sonicwhale.ai  ca:0x068e9e009fda970fa953e1f6a43d982ca991f4ba

## 140. Feb 21, 2025 · 5:49 PM UTC · 1892995029842698570#m
- 链接：https://twitter.com/Zenith_shade789/status/1892995029842698570#m
- 作者：@Zenith_shade789

> sonic生态部分token合集
>  
> $SHADOW (ShadowSwap Token)
> ShadowSwap DEX 治理代币
> Sonic DeFi 核心。
> 官推:@ShadowOnSonic
> 关注列表（@SonicLabs，@0xRoninSam，@AndreCronjeTech...)
> 官网:shadow.so/
> ca：0x3333b97138d4b086720b5ae8a7844b1345a33333
>  
> $SONIC / $s
> SonicLabs 主网的原生代币（通常写作 $S，但 X 上也常被提及为 $SONIC），用于交易费用、质押和治理。 Sonic 生态的核心代币，支持所有 dApp 和网络活动。 官推:@SonicLabs
> 关注列表(@Rings_Protocol,@ShadowOnSonic,@0xseg,@SonicLabs,@AndreCronjeTech...)
> 官网:linktr.ee/soniclabs  ca:3rQK45d1ojXR7vtvCmeNjKKVycnVWqaVcP3zk1G39RJR
>  
> $HEDGY
> Sonic 生态内的代币，与 Hedgey Finance 相关，最初在 Fantom（现 Sonic 前身）上推出，专注于去中心化期权交易，现扩展为迷因币或 DeFi 混合代币。 原为期权交易工具代币，通过 Sonic 的 DEX（如 Equalizer、ShadowSwap）提供流动性或参与社区活动。 官推:@hedgycoin
> 关注列表(@yel_finance,@0xRoninSam ...)
> 官网:hedgysonic.com  ca:0x6fb9897896fe5d05025eb43306675727887d0b7c

## 141. Feb 21, 2025 · 1:17 PM UTC · 1892926499193602064#m
- 链接：https://twitter.com/Zenith_shade789/status/1892926499193602064#m
- 作者：@Zenith_shade789

> 现在的 sonic 生态给我的感觉
>  
> 走暗路
> 耕瘦田
> 进窄门
>  
> @AndreCronjeTech @ShadowOnSonic @SonicLabs

## 142. Feb 21, 2025 · 4:37 AM UTC · 1892795830274576792#m
- 链接：https://twitter.com/Zenith_shade789/status/1892795830274576792#m
- 作者：@Zenith_shade789

> 要麻了
> 感谢韦陀老师的三盘理论  @thecryptoskanda
> 最近ghetto在all in sonic @SonicLabs  @AndreCronjeTech

## 143. Feb 18, 2025 · 12:41 PM UTC · 1891830314990960813#m
- 链接：https://twitter.com/Zenith_shade789/status/1891830314990960813#m
- 作者：@Zenith_shade789

> 《CZ表哥学习如何烧币》
>  
> @cz_binance
> BSC（Binance Smart Chain）燃烧Token通常通过以下几种方法实现的：
>  
> 1、通过智能合约燃烧： 使用智能合约来销毁一定数量的代币，通常是通过调用burn()函数。这会将代币转移到一个无法访问的地址（如黑洞地址），或者将代币直接销毁。比如，如果你是一个ERC-20或者BEP-20代币的合约创建者，可以在合约中编写burn()函数，然后调用这个函数来销毁一定数量的代币。
>  
> 2、发送到死地址（黑洞地址）： 另一种常见的燃烧方法是将代币发送到一个“黑洞地址”或“死地址”，即一个没有私钥、无法访问的地址。比如，通常使用的黑洞地址是0x000000000000000000000000000000000000dEaD。这样发送到该地址的代币就永远无法被恢复或转移，从而实现“燃烧”操作。
>  
> 3、通过交易销毁： 可以在代币交易时设定一定的燃烧机制。例如，每次用户转账时，按照一定比例自动燃烧部分代币。通过这个方式，随着交易的进行，代币的总供应量会逐步减少。
> 表哥你说你不会玩meme，这里可以教你如何燃烧
> 你赞助地址里这枚持仓79.8%token memecoin
> 0xa14b0b99c9117ea2f4fb2c9d772d95d9fd3acaab
>  
> Don't sell off your supply chips and feel the joy of the meme.

## 144. Feb 18, 2025 · 6:38 AM UTC · 1891739158214762644#m
- 链接：https://twitter.com/Zenith_shade789/status/1891739158214762644#m
- 作者：@Zenith_shade789

> 一定是还没睡醒

## 145. Feb 18, 2025 · 6:38 AM UTC · 1891739078103544158#m
- 链接：https://twitter.com/Zenith_shade789/status/1891739078103544158#m
- 作者：@Zenith_shade789

> ？

## 146. Feb 18, 2025 · 4:31 AM UTC · 1891706974724575512#m
- 链接：https://twitter.com/Zenith_shade789/status/1891706974724575512#m
- 作者：@Zenith_shade789

> 上海Office刚刚装好，在一个艺术空间租用了一块场地，超哥下楼接我：“不好意思，场地有点小，门也很小。”
>  
> 我说：“所以要走窄门呀！”
>  
> 在Office同事给我做了超好喝的果蔬汁，想说：喝过吃过很多山珍海味都比不上这10块钱一杯的苦瓜汁！
>  
> 牧师吟诵尤在耳边：你们要努力进窄门，因为宽门和阔路通向沉沦。

## 147. Feb 17, 2025 · 7:17 PM UTC · 1891567574653014283#m
- 链接：https://twitter.com/Zenith_shade789/status/1891567574653014283#m
- 作者：@Zenith_shade789

> 亲切问候🤣
> G3zut7f7UZkKjFHyu1o1hcHCnXYhrgtCsUX2sj51pump

## 148. Feb 17, 2025 · 6:40 AM UTC · 1891377162751545823#m
- 链接：https://twitter.com/Zenith_shade789/status/1891377162751545823#m
- 作者：@Zenith_shade789

> pve pve pve

## 149. Feb 15, 2025 · 3:57 PM UTC · 1890792526442856665#m
- 链接：https://twitter.com/Zenith_shade789/status/1890792526442856665#m
- 作者：@Zenith_shade789

> 5号看到扒早期地址看到了这个地址可以在这条推上
> localhost:8080/Zenith_Shade18/status/…
> 再用kaboom看了下这个地址分析往期抽插行为他对链上的判断非常迅速基本大热都在场，入场早出货快。
> 狠狠抽插流动性了。@DoctorMbitcoin

## 150. Feb 14, 2025 · 11:54 PM UTC · 1890550165741736181#m
- 链接：https://twitter.com/Zenith_shade789/status/1890550165741736181#m
- 作者：@Zenith_shade789

> Fuckkkkk

## 151. Feb 14, 2025 · 10:38 PM UTC · 1890531134355751141#m
- 链接：https://twitter.com/Zenith_shade789/status/1890531134355751141#m
- 作者：@Zenith_shade789

> 情人节还在链上看狗，你不发财谁发财

## 152. Feb 14, 2025 · 10:36 PM UTC · 1890530483877904629#m
- 链接：https://twitter.com/Zenith_shade789/status/1890530483877904629#m
- 作者：@Zenith_shade789

> 起床啊！！！！！！！！！！！！
> Bo9jh3wsmcC2AjakLWzNmKJ3SgtZmXEcSaW7L2FAvUsU

## 153. Feb 14, 2025 · 2:29 PM UTC · 1890407993386090809#m
- 链接：https://twitter.com/Zenith_shade789/status/1890407993386090809#m
- 作者：@Zenith_shade789

> Just make something.）

## 154. Feb 14, 2025 · 2:26 PM UTC · 1890407230685565201#m
- 链接：https://twitter.com/Zenith_shade789/status/1890407230685565201#m
- 作者：@Zenith_shade789

> 未约会❌
> 未开房❌
> 未牵手❌
> 未亲嘴❌
> 签到时间：2025 0214 22:26

## 155. Feb 14, 2025 · 12:04 PM UTC · 1890371646638420184#m
- 链接：https://twitter.com/Zenith_shade789/status/1890371646638420184#m
- 作者：@Zenith_shade789

> 妈妈再也不用担心，我找不到cabal地址啦！！！
>  
> KaBoom聪明钱追踪已在@solana重磅升级！
> 💡 新功能亮点：
> 📷 专属页面查看你的聪明钱列表
> 📷 将钱包保存至你的个人观察列表（追踪功能）
> 📷 追踪 Smart Money 钱包实时交易详情
> 📷 自定义追踪： 选择只关注 你感兴趣的钱包！  立即使用 KaBoom，成为你在 Solana 上追踪聪明钱活动的强力工具 🔍🎯
> app.kaboom.meme?invite=VWVeX…

## 156. Feb 14, 2025 · 6:02 AM UTC · 1890280507969269879#m
- 链接：https://twitter.com/Zenith_shade789/status/1890280507969269879#m
- 作者：@Zenith_shade789

> 你俩处上了？

## 157. Feb 14, 2025 · 5:54 AM UTC · 1890278525678334454#m
- 链接：https://twitter.com/Zenith_shade789/status/1890278525678334454#m
- 作者：@Zenith_shade789

> Sol changes the future
>  
> UCYaUNhgB2344N8Qpg4t712V9RHahcpTCejwXKspump @Lxxx_crypto
> Alpha group

## 158. Feb 14, 2025 · 5:34 AM UTC · 1890273287978233910#m
- 链接：https://twitter.com/Zenith_shade789/status/1890273287978233910#m
- 作者：@Zenith_shade789

> Damn,we ruthlessly extract liquidity
> 5bZoxev99XsYi8HT7FnSZ9cdu9ggYasmfP6au7spVUqq

## 159. Feb 13, 2025 · 8:57 AM UTC · 1889962094818361803#m
- 链接：https://twitter.com/Zenith_shade789/status/1889962094818361803#m
- 作者：@Zenith_shade789

> changing changing changing

## 160. Feb 12, 2025 · 6:31 AM UTC · 1889562868313403467#m
- 链接：https://twitter.com/Zenith_shade789/status/1889562868313403467#m
- 作者：@Zenith_shade789

> 一年前
> 朋友：我有几十万枚 pi 初中时候挖的，几毛钱卖给你吧！
> 我：谁尼玛买这种空气币不要
> 今天：哥你还认识我吗？

## 161. Feb 10, 2025 · 9:17 PM UTC · 1889061071216124289#m
- 链接：https://twitter.com/Zenith_shade789/status/1889061071216124289#m
- 作者：@Zenith_shade789

> 今晚流动性太棒了。
> 通宵提取流动性

## 162. Feb 10, 2025 · 7:29 PM UTC · 1889033995872276666#m
- 链接：https://twitter.com/Zenith_shade789/status/1889033995872276666#m
- 作者：@Zenith_shade789

> $Fartnoy
> 2RuDRx9RAcXrSoLupeMLGuBay6w5Q1nUrdPySjA3pump
> 晚上语音冲狗的乐趣？16x

## 163. Feb 10, 2025 · 6:03 PM UTC · 1889012375145300138#m
- 链接：https://twitter.com/Zenith_shade789/status/1889012375145300138#m
- 作者：@Zenith_shade789

> 九字真言
> 买买买，卖卖卖，赚赚赚

## 164. Feb 10, 2025 · 5:55 PM UTC · 1889010335371444702#m
- 链接：https://twitter.com/Zenith_shade789/status/1889010335371444702#m
- 作者：@Zenith_shade789

> 冲不过来了

## 165. Feb 10, 2025 · 5:53 PM UTC · 1889009861192827087#m
- 链接：https://twitter.com/Zenith_shade789/status/1889009861192827087#m
- 作者：@Zenith_shade789

> $peach
> 7qhwYUXBaPTfWkhUpgWTjHAvdG48wRj5TLmTQ5Topump
> 这个按头的群友也挺早的

## 166. Feb 10, 2025 · 5:47 PM UTC · 1889008393618374767#m
- 链接：https://twitter.com/Zenith_shade789/status/1889008393618374767#m
- 作者：@Zenith_shade789

> $pop
> BxB5Vh8taVpRss7Vgw6S1gXE4LW4uWngLveNGuhgpump
> 高点20x，是不是沾了你的气运🤣 @Lxxx_crypto
> @Lxxx_crypto 在发红包正好看到这个阴谋盘有几天前关注的5个阴谋地址上了。
> 果断share到社区了，果然美女自带气运。

## 167. Feb 10, 2025 · 4:08 PM UTC · 1888983387039543467#m
- 链接：https://twitter.com/Zenith_shade789/status/1888983387039543467#m
- 作者：@Zenith_shade789

> 哇塞！阴谋大王除了会给社区喂饭，还这么漂亮
> 多喂点阴谋盘！！！！！ @Lxxx_crypto

## 168. Feb 10, 2025 · 2:05 PM UTC · 1888952436779557312#m
- 链接：https://twitter.com/Zenith_shade789/status/1888952436779557312#m
- 作者：@Zenith_shade789

> 这个地址挺有趣的在 $car 单币a7
> 9CcNFHn1AiSurN6XWZaSGRqNKafkPC8CE3qjLbxcXvzR
> 用kaboom分析后看了他交易风格对于市场把握挺特别的
> 当作参考吧！

## 169. Feb 10, 2025 · 5:45 AM UTC · 1888826585744322574#m
- 链接：https://twitter.com/Zenith_shade789/status/1888826585744322574#m
- 作者：@Zenith_shade789

> next alpha caller🤑

## 170. Feb 10, 2025 · 4:06 AM UTC · 1888801804542185816#m
- 链接：https://twitter.com/Zenith_shade789/status/1888801804542185816#m
- 作者：@Zenith_shade789

> Hnnw2hAgPgGiFKouRWvM3fSk3HnYgRv4Xq1PjUEBEuWM
> 在kaboom看了这个地址在其他token上的情况，Leon老师分析的没有问题该地址交易策略更偏向于量化交易

## 171. Feb 9, 2025 · 7:56 AM UTC · 1888497165212999810#m
- 链接：https://twitter.com/Zenith_shade789/status/1888497165212999810#m
- 作者：@Zenith_shade789

> 喜欢bsc？永远记吃不记打

## 172. Feb 9, 2025 · 6:44 AM UTC · 1888479007102095828#m
- 链接：https://twitter.com/Zenith_shade789/status/1888479007102095828#m
- 作者：@Zenith_shade789

> trenches changing ur live

## 173. Feb 9, 2025 · 6:36 AM UTC · 1888477039013040185#m
- 链接：https://twitter.com/Zenith_shade789/status/1888477039013040185#m
- 作者：@Zenith_shade789

> make trenches great again.

## 174. Feb 9, 2025 · 5:40 AM UTC · 1888462930901176638#m
- 链接：https://twitter.com/Zenith_shade789/status/1888462930901176638#m
- 作者：@Zenith_shade789

> 群友要赚麻了

## 175. Feb 9, 2025 · 5:37 AM UTC · 1888462183006359884#m
- 链接：https://twitter.com/Zenith_shade789/status/1888462183006359884#m
- 作者：@Zenith_shade789

> 金牌辅助工具聪明钱tracker以及ct twitter tracker
> 这里我们可以看到我们的 smart money tracker 看到Dave买入市值在6.77k，以及我们的twitter tracker也在实时更新相关信息。这里重点这两个金牌辅助是悬浮窗便携式，大大减少的我的check时间。后续我会将此金牌辅助需求喂给 @Leoninweb3 老师，让链上小白完成速成升级，同时此条帖子评论人员可以免简历直接dm我进入ghetto社区。

## 176. Feb 9, 2025 · 5:24 AM UTC · 1888459035315437787#m
- 链接：https://twitter.com/Zenith_shade789/status/1888459035315437787#m
- 作者：@Zenith_shade789

> $jailstoo
> AxriehR6Xw3adzHopnvMn7GcpRFcD41ddpiTWMg6pump
> 我们ghetto社区应该是全网除了跟单bot最早发现的这枚token的，这里就要说到金牌辅助，聪明钱tracker以及ct twitter tracker的重要性了。
> 前两天Dave事件参考我们的链上分析师devour这条帖子localhost:8080/Divoll_Law/status/1887… 自行评估。
> 我们也是第一时间找到了dave地址5rkPDK4JnVAumgzeV2Zu8vjggMTtHdDtrsd5o9dhGZHD
> 目前地址换到了99KorR2TqEFSvMJ4UQmxbXRBB5q2GioLLGhjE3nZ2gjq

## 177. Feb 8, 2025 · 10:08 PM UTC · 1888349252633727370#m
- 链接：https://twitter.com/Zenith_shade789/status/1888349252633727370#m
- 作者：@Zenith_shade789

> 忘了贴了Dave新地址99KorR2TqEFSvMJ4UQmxbXRBB5q2GioLLGhjE3nZ2gjq

## 178. Feb 8, 2025 · 10:03 PM UTC · 1888348058041376844#m
- 链接：https://twitter.com/Zenith_shade789/status/1888348058041376844#m
- 作者：@Zenith_shade789

> 今晚Dave笑得我肚子痛
> AxriehR6Xw3adzHopnvMn7GcpRFcD41ddpiTWMg6pump

## 179. Feb 8, 2025 · 9:22 PM UTC · 1888337652711317739#m
- 链接：https://twitter.com/Zenith_shade789/status/1888337652711317739#m
- 作者：@Zenith_shade789

> Are u going to compete with @Cupseyy  for trench position?🤣 @stoolpresidente

## 180. Feb 8, 2025 · 7:07 PM UTC · 1888303746175909988#m
- 链接：https://twitter.com/Zenith_shade789/status/1888303746175909988#m
- 作者：@Zenith_shade789

> 目前来看表哥 @cz_binance  你得继续努力啊，外面都在传
> bsc filp sol

## 181. Feb 8, 2025 · 6:42 PM UTC · 1888297242253226095#m
- 链接：https://twitter.com/Zenith_shade789/status/1888297242253226095#m
- 作者：@Zenith_shade789

> 大家都去bsc了嘛？那我还在sol战壕里吧！
> 市场行为决定资金流向

## 182. Feb 8, 2025 · 3:56 PM UTC · 1888255697898590412#m
- 链接：https://twitter.com/Zenith_shade789/status/1888255697898590412#m
- 作者：@Zenith_shade789

> 阴谋大王 @ZVqNirud9m32331

## 183. Feb 8, 2025 · 3:55 PM UTC · 1888255425222750528#m
- 链接：https://twitter.com/Zenith_shade789/status/1888255425222750528#m
- 作者：@Zenith_shade789

> $live
> 5xFvhaueVRBmiTcsxdYsQUsSsuwV8xmaK4yH9rwo6Krx
> @l3andrnh 推送 @_Ghetto__kids  社区全网最早43k市值最高1m市值

## 184. Feb 8, 2025 · 2:46 PM UTC · 1888237868340093236#m
- 链接：https://twitter.com/Zenith_shade789/status/1888237868340093236#m
- 作者：@Zenith_shade789

> Change ur habits,keep

## 185. Feb 8, 2025 · 7:05 AM UTC · 1888122050830594415#m
- 链接：https://twitter.com/Zenith_shade789/status/1888122050830594415#m
- 作者：@Zenith_shade789

> 干倒车头是p小将的目标,lol

## 186. Feb 8, 2025 · 6:45 AM UTC · 1888116864066482177#m
- 链接：https://twitter.com/Zenith_shade789/status/1888116864066482177#m
- 作者：@Zenith_shade789

> 今天凌晨用kaboom测了Dave的地址5rkPDK4JnVAumgzeV2Zu8vjggMTtHdDtrsd5o9dhGZHD
> 定位很到位，稳重的大象,lol
> @KaBoom_meme 在聪明钱算法这板块也是越来越深入了，我也与 @Leoninweb3 老师沟通过下一阶段也就是便携式以及场景应用了。有更好的建议也可以提我来睡服leon老师,lol
>  
> 关于量化
> 在kol榜单上昨日依然是cupsey稳居日内收益第一。
> suqh5sHtr8HyJ7q8scBimULPkPpA557prMG47xCHQfK
> 不建议大家跟单仔细研究可以看到cupsey地址交易情况大概率是自动化量化交易在跑内盘。这里与leon老师沟通时也有提到他们的@Quant72ai在不久也会进行相关量化测试非常期待

## 187. Feb 7, 2025 · 8:30 PM UTC · 1887962122661085289#m
- 链接：https://twitter.com/Zenith_shade789/status/1887962122661085289#m
- 作者：@Zenith_shade789

> cookingggggg @stoolpresidente

## 188. Feb 7, 2025 · 1:57 PM UTC · 1887863290799468895#m
- 链接：https://twitter.com/Zenith_shade789/status/1887863290799468895#m
- 作者：@Zenith_shade789

> Trenches condition

## 189. Feb 7, 2025 · 7:43 AM UTC · 1887769134035087527#m
- 链接：https://twitter.com/Zenith_shade789/status/1887769134035087527#m
- 作者：@Zenith_shade789

> 我住在69号房子后面的水里-喂我！

## 190. Feb 7, 2025 · 4:13 AM UTC · 1887716225821909491#m
- 链接：https://twitter.com/Zenith_shade789/status/1887716225821909491#m
- 作者：@Zenith_shade789

> 🔥 Yo Boomies, ready for some Wallet IQ FUN?! 🔥 We’re giving away 100U in a Lucky Draw! 🎉 5 winners! 100% FREE entry!
>  
> The Wallet IQ test is gonna help YOU crack the code on Smart Money trading strategies, while boosting our database, and sending MORE Smart Money signals to all users! 🚀
>  
> How to enter:
> 1️⃣ ❤️, RT & tag 3 frens
> 2️⃣ Head to app.kaboom.meme/?tab=iq and test ANY Solana wallet
> 3️⃣ Snap a screenshot of your result and DROP it in the comments below!
>  
> ⚠️ Tip: If the result doesn’t show, just retry or try a different wallet! Make sure the Wallet IQ is discovered by your wallet💡
>  
> ⏳ 48hrs, Time’s ticking ⏰
>  
> KaBoom: Decode On-Chain Signals and Stay Ahead.

## 191. Feb 6, 2025 · 10:58 PM UTC · 1887637069931106492#m
- 链接：https://twitter.com/Zenith_shade789/status/1887637069931106492#m
- 作者：@Zenith_shade789

> ca

## 192. Feb 6, 2025 · 10:54 PM UTC · 1887635975662449113#m
- 链接：https://twitter.com/Zenith_shade789/status/1887635975662449113#m
- 作者：@Zenith_shade789

> 8KMfU13W1ayhBEyWrZTe8hPTNbZo2cLJrH3pTqNqpump
> 这该死的推背感
> montoya por favor

## 193. Feb 6, 2025 · 1:41 PM UTC · 1887496717349888023#m
- 链接：https://twitter.com/Zenith_shade789/status/1887496717349888023#m
- 作者：@Zenith_shade789

> 这是一场刻意行为那会是多么有趣

## 194. Feb 6, 2025 · 1:32 PM UTC · 1887494644629709001#m
- 链接：https://twitter.com/Zenith_shade789/status/1887494644629709001#m
- 作者：@Zenith_shade789

> tst🤠
> @Divoll_Law

## 195. Feb 6, 2025 · 11:37 AM UTC · 1887465649749757981#m
- 链接：https://twitter.com/Zenith_shade789/status/1887465649749757981#m
- 作者：@Zenith_shade789

> 当下链上现状
> 以西部快枪手时代为例，一旦你被公认为最快的枪手你也是众人最希望击倒的对象。

## 196. Feb 6, 2025 · 6:15 AM UTC · 1887384647753212352#m
- 链接：https://twitter.com/Zenith_shade789/status/1887384647753212352#m
- 作者：@Zenith_shade789

> 😅
> ApdzwoRC4ATocqjBFasGGZj9DpwhQGaiLYWoFsJrpump

## 197. Feb 6, 2025 · 6:10 AM UTC · 1887383378175140052#m
- 链接：https://twitter.com/Zenith_shade789/status/1887383378175140052#m
- 作者：@Zenith_shade789

> 这两天扒的一个主播地址从外盘玩到内盘极速pvp，流动性太差了，资金呢？dev呢？
> EhRp3PVPqCe2ms3Cjgxg9LSXygxCkJfehPZnVPGienpg

## 198. Feb 6, 2025 · 6:06 AM UTC · 1887382256618201196#m
- 链接：https://twitter.com/Zenith_shade789/status/1887382256618201196#m
- 作者：@Zenith_shade789

> 真尼玛人才，都别玩了。ponzi都给我

## 199. Feb 6, 2025 · 6:02 AM UTC · 1887381227130470822#m
- 链接：https://twitter.com/Zenith_shade789/status/1887381227130470822#m
- 作者：@Zenith_shade789

> SMCS
> p一晚上从内盘p到外盘，最后还被meow 狠狠干下。

## 200. Feb 5, 2025 · 4:32 AM UTC · 1886996273557070130#m
- 链接：https://twitter.com/Zenith_shade789/status/1886996273557070130#m
- 作者：@Zenith_shade789

> 今早速通盘 $Calicoin，说实话我并没有抓到这个盘子，早上刚哄睡baby便出门。
>  
> 刚到家回来复盘了盘子叙事先去kaboom查了早期买入地址情况。可以看到
> 地址9mXZsKuQGeHeVCcxQntr2LdJfWB9sBnXgRXrFj7Bfsg5在200k左右先买入了5s，应该是检索了叙事以及dev信息，市值不高风控盗号风险。
> 地址ATmKENkRrL1JQQnoUNAQvkiwgjiHKUkzyncxTGxyzQL1应该是看到监控钱包异动去检索了相关信息并且确认了这并不是盗号行为一次性买入了50s
> 市场行为已经是最好的辩伪，对其他meme感兴趣的直接用kaboom去check更多聪明钱
> app.kaboom.meme?invite=VWVeX…
>  
> dev信息
> @BCIcanDoBetter
> 全球首个通过脑机接口创造NFT的赛博朋克艺术家，第一个由实体商品公司支持的硬币。基于Solana的代币旨在培育第一个社区驱动的电子商务品牌，以及社区对@BCIcanDoBetter的口评不错。
> 检索方式x自带的grok非常方便
> localhost:8080/i/grok/share/GJnAwUbww…
>  
> 从叙事角度来看$Calicoin确实符合meme的定位，社区发出来时期在6m左右看到基本叙事，市值已经高于我的预期值因此并未去追fomo情绪。
>  
> 晚点看下回调点位以及是否有庄进来拿筹码再考虑进场 #DYOR
