# BlockSec (@BlockSecTeam)

- 抓取时间：2025-10-20T23:41:54.895Z
- 推文数量：200 / 目标 200
- 抓取耗时：27653 ms
- 数据来源：twitter-Following-1760964620895.json
- 分页次数：11
- 抓取尝试：1
- Cursor 链：10 条
- 账号统计：粉丝 26635，关注 147，推文 2054，点赞 1187
- 站外链接：https://t.co/Kgi6tFe59a
- Twitter：https://twitter.com/BlockSecTeam
- 头像：https://pbs.twimg.com/profile_images/1683851044370145280/YSyAY_kP_normal.jpg
- Banner：https://pbs.twimg.com/profile_banners/1339588489537486848/1690296350
- 认证：Blue Verified

> 账号简介：
> Smart Contract Audit | Security Monitoring | AML/CFT (KYA/KYT) | Crypto Investigation | @Phalcon_xyz @MetaSleuth @MetaDockTeam 👉TG: https://t.co/owokTLanv5

---

## 1. Mar 19, 2024 · 7:57 AM UTC · 1769996703690785011#m
- 链接：https://twitter.com/BlockSecTeam/status/1769996703690785011#m
- 作者：@BlockSecTeam

> 📢 Attention, DeFi projects!
>  
> Secure your protocol's entire lifecycle with BlockSec🛡️.
> From pre-launch security audits to post-launch attack monitoring and blocking (Phalcon), we've got you covered.
>  
> Learn more about our full-stack security solution at blocksec.com/blog/new-websit….

## 2. Oct 20, 2025 · 10:32 AM UTC · 1980220633335349598#m
- 链接：https://twitter.com/BlockSecTeam/status/1980220633335349598#m
- 作者：@BlockSecTeam

> .@SharwaFinance was reported to be exploited and subsequently paused (as claimed by the project). However, several additional suspicious transactions occurred hours later, likely exploiting the same underlying issue through slightly varied attack paths.
>  
> In general, the attacker first created a margin account, then borrowed additional assets through leveraged lending using the provided collateral, and finally executed a sandwich attack targeting the swap operation involving the borrowed assets.
>  
> The root cause appears to be a flawed insolvency check in the MarginTrading contract's swap() function, which exchanges borrowed assets from one token (e.g., WBTC) to another (e.g., USDC). The function only verifies solvency based on the account state at the beginning of the swap, before the asset exchange is executed, leaving room for manipulation during the process.
>  
> Attacker 1 (0xd356c82e0c85e1568641d084dbdaf76b8df96c08) carried out multiple attacks, earning ~$61K in profit.
> - Attacker1's TX1 (create margin account): app.blocksec.com/explorer/tx…
> - Attacker1's TX2 (launch attack): app.blocksec.com/explorer/tx…
>  
> Attacker 2 (0xaa24987bab540617416b77c986f66ae009c55795) executed a single attack, earning ~$85K in profit.
> - Attacker2's TX1 (create margin account): app.blocksec.com/explorer/tx…
> - Attacker2's TX2 (launch attack): app.blocksec.com/explorer/tx…

## 3. Oct 15, 2025 · 4:36 AM UTC · 1978319079963611192#m
- 链接：https://twitter.com/BlockSecTeam/status/1978319079963611192#m
- 作者：@BlockSecTeam

> Our system detected several suspicious transactions (initiated by different EOAs) targeting two unknown contracts deployed by the same address on #Ethereum hours ago, resulting in losses of ~$120K.
>  
> The root cause appears to be a lack of access control on the critical functions approveERC20 and withdrawAll in the victim contracts (which are not open source), allowing attackers to drain the tokens held within them.
>  
> Notably, the withdrawAll function requires burning a sufficient amount of #sil tokens. This explains why, in the second attack transaction (TX2, which caused the majority of the loss), the attacker first acquired #sil tokens via a flashloan followed by multiple swaps before executing the actual exploit.
>  
> TX1:
> app.blocksec.com/explorer/tx…
> TX2:
> app.blocksec.com/explorer/tx…
> TX3:
> app.blocksec.com/explorer/tx…

## 4. Oct 14, 2025 · 12:30 PM UTC · 1978075819575759112#m
- 链接：https://twitter.com/BlockSecTeam/status/1978075819575759112#m
- 作者：@BlockSecTeam

> 🚨 It’s live!
> Phalcon Compliance Self-Service Platform by @BlockSecTeam — your real-time AML solution for crypto compliance.
>  
> ✅ 400M+ labeled addresses
> ⚡️ Millisecond-level response
> 🌍 FATF-aligned across 27+ jurisdictions
> 🔍 Seamless MetaSleuth integration @MetaSleuth
>  
> Detect illicit flows. Prevent freezes. Stay compliant.
>  
> 👉 Explore now: blocksec.com/phalcon/complia…
> #Web3 #Compliance #PhalconCompliance #AML #KYT

## 5. Oct 11, 2025 · 3:20 PM UTC · 1977031656663081148#m
- 链接：https://twitter.com/BlockSecTeam/status/1977031656663081148#m
- 作者：@BlockSecTeam

> Indeed, the AToken price for USDT was incorrectly shown as $154 during the @asterafinance exploit. However, this was not a "textbook oracle misconfiguration". The root cause appears to be token price manipulation resulting from insufficient market liquidity.
>  
> The AToken price = underlying token price (which was correctly around $1 from the oracle) × LiquidityIndex, which the attacker manipulated from 1.001 → 154 through a series of flashloan transactions before the exploit.
>  
> Interestingly, unlike classic one-shot attacks targeting empty markets in #AAVE -forked projects, this incident involved multiple transactions, likely because the market was not empty and required several iterations to accumulate flashloan fees and distort the price.
>  
> AToken (USDT): lineascan.build/address/0x15…
> Attacker address 1 (pool drainer): lineascan.build/address/0x61…
> Attacker address 2 (liquidity-index inflater): lineascan.build/address/0x95…
>  
> 1. Deposit / position creation (attacker address 1)
> Attacker address 1 created positions via deposits.
> TX: app.blocksec.com/explorer/tx…
>  
> 2. LiquidityIndex inflation (attacker address 2)
> Attacker address 2 performed repeated flashloan (borrow+repay) operations (~100 transactions) that inflated the pool’s LiquidityIndex from 1.001 → 154.
> First inflation TX: app.blocksec.com/explorer/tx…
> Final inflation TX: app.blocksec.com/explorer/tx…
>  
> 3. Exploit / pool drain (attacker address 1 profits)
> With LiquidityIndex inflated, attacker address 1 borrowed against the manipulated AToken's price and drained the lending pool.
> Drain TX: app.blocksec.com/explorer/tx…

## 6. Oct 4, 2025 · 5:53 PM UTC · 1974533451408986417#m
- 链接：https://twitter.com/BlockSecTeam/status/1974533451408986417#m
- 作者：@BlockSecTeam

> .@MIM_Spell was attacked hours ago, resulting in a loss of ~$1.7M. The root cause stems from the flawed implementation logic of the cook function, which allows users to execute multiple predefined operations in a single transaction. Specifically, the actions share a common status, which may lead to the bypassing of the insolvency check.
>  
> The details are as follows:
> 1. When action = 5 (ACTION_BORROW), a borrowing operation is executed, and status.needsSolvencyCheck is set to true.
> 2. When action = 0, the internal function _additionalCookAction is called to update the status. However, this function is an empty function and defaults to returning status.needsSolvencyCheck = false, thereby overwriting the previous check flag and skipping the final insolvency check.
>  
> The attacker called the cook function on 6 different addresses, passing in actions = [5, 0], obtained 1,793,755e18 MIM, and subsequently profited through swaps.
>  
> Attack TX: app.blocksec.com/explorer/tx…

## 7. Oct 1, 2025 · 2:21 AM UTC · 1973211741350973620#m
- 链接：https://twitter.com/BlockSecTeam/status/1973211741350973620#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected several suspicious transactions continuously targeting an unknown contract on #SEI. It is a typical "transferFrom" issue, which is caused by an arbitrary low-level call.
>  
> Please revoke any approvals granted to 0xa9b9e1af3cfd8b1c29e0ae4fddf2dadd74a108cc on #SEI!

## 8. Sep 29, 2025 · 9:34 AM UTC · 1972595841245516037#m
- 链接：https://twitter.com/BlockSecTeam/status/1972595841245516037#m
- 作者：@BlockSecTeam

> ALERT! Our system detected a series of suspicious transactions targeting an inactive contract (etherscan.io/address/0xdc827…, inactive for over 200 days) on #Ethereum. Losses exceed 32 Ether, and the remaining Ether is still under attack.
>  
> Since the contract is not open-source, we suspect the root cause lies in flawed game mechanics: it fails to restrict participants to EOAs, and the randomness implementation is also defective.
>  
> One of the attack TXs: app.blocksec.com/explorer/tx…

## 9. Sep 25, 2025 · 10:08 AM UTC · 1971154830447280308#m
- 链接：https://twitter.com/BlockSecTeam/status/1971154830447280308#m
- 作者：@BlockSecTeam

> Yet another attack targeting @Griffin_AI similar to the @seedifyfund incident: fraudulent cross-chain messages from the source chain (in this case, #Ethereum) were accepted and executed on the destination chain (#BSC), allowing the attacker to profit from $GAIN tokens and eventually swap them for other assets.
>  
> The key difference lies in how ownership was compromised. Specifically, the attacker managed to get 0x54A978238984d581EdD3a9359dDA9BE53A930a7e to invoke the setPeer function (likely via phishing). This action designated 0xba159054636e69080ae7c756319e5c85498efeb0 as a trusted peer, enabling the attacker to mint tokens arbitrarily through the cross-chain bridge.
>  
> 1. Set peer on #BSC: app.blocksec.com/explorer/tx…
> 2. Cross-chain TX on #Ethereum: app.blocksec.com/explorer/tx…
> 3. Mint $GAIN tokens on #BSC: app.blocksec.com/explorer/tx…

## 10. Sep 23, 2025 · 3:26 PM UTC · 1970510108506591289#m
- 链接：https://twitter.com/BlockSecTeam/status/1970510108506591289#m
- 作者：@BlockSecTeam

> .@seedifyfund was attacked across multiple chains, likely due to a private-key compromise.
>  
> Specifically, after gaining ownership of the SFUND_OFTv1 contracts on certain destination chains (e.g., #Base), the attacker (0x8030f5bF186d69627aA220FF7d486fd8c8818c56) used the setTrustedRemoteAddress function to set a malicious contract (0xffad4bD0fA118010bA01a3C69C9Ed7fF460E943e) as the trusted remote address. This allowed fraudulent cross-chain messages originating from the source chain (e.g., #Polygon) to be accepted and executed on the destination chain (e.g., #Base), enabling the attacker to profit in $SFUND tokens and ultimately swap them for other assets.
>  
> Key attack steps:
> 1. Ownership takeover:  app.blocksec.com/explorer/tx…
> 2. Malicious trusted remote set (attacker-controlled: 0xffad4bD0fA118010bA01a3C69C9Ed7fF460E943e):  app.blocksec.com/explorer/tx…
> 3. Cross-chain request sent from #Polygon:  app.blocksec.com/explorer/tx…
> 4. Profits realized on #Base (in $SFUND):  app.blocksec.com/explorer/tx…

## 11. Sep 23, 2025 · 10:04 AM UTC · 1970429012179869801#m
- 链接：https://twitter.com/BlockSecTeam/status/1970429012179869801#m
- 作者：@BlockSecTeam

> Phalcon explorer now supports @monad
>  
> A sample tx:
>  
> app.blocksec.com/explorer/tx…

## 12. Sep 22, 2025 · 1:11 PM UTC · 1970113734019633635#m
- 链接：https://twitter.com/BlockSecTeam/status/1970113734019633635#m
- 作者：@BlockSecTeam

> 🧵 WE'RE SO BACK 🔥
>  
> Phalcon Explorer @Phalcon_xyz just dropped the most INSANE update and it's about to change the game forever
>  
> After listening to 2000+ degens, researchers, and devs crying about transaction analysis pain points... we cooked something special 👨‍🍳

## 13. Sep 19, 2025 · 12:49 PM UTC · 1969021136236351836#m
- 链接：https://twitter.com/BlockSecTeam/status/1969021136236351836#m
- 作者：@BlockSecTeam

> Okie Finance has completed the security audit for Okie Swap V3 conducted by @BlockSecTeam 💯
>  
> For more info, audit report 👉 docs.okiedokie.finance/user-…

## 14. Sep 18, 2025 · 3:40 AM UTC · 1968520529046016248#m
- 链接：https://twitter.com/BlockSecTeam/status/1968520529046016248#m
- 作者：@BlockSecTeam

> ALERT! Several hours ago, our system detected an attack transaction targeting the NGP token (issued by @newgoldprotocol) on #BSC, resulting in ~$2M in losses.
>  
> This incident is a price manipulation attack, rooted in two critical design flaws of the NGP token:
> 1) The token’s buying amount limit can be bypassed by setting the recipient address of token transfers to the "DEAD" address;
> 2) When selling NGP tokens, the contract syncs to reflect fee deductions—a process that can be abused to manipulate the token’s price.
>  
> Here’s a breakdown of how the attack unfolded:
> 1. The attacker purchased NGP Token at a normally low price using multiple accounts.
> 2. They then inflated NGP’s price by executing large-volume BUSD-for-NGP swaps on the BUSD-NGP PancakePair. Crucially, by setting the recipient address to the "DEAD" address, the attacker bypassed two key restrictions during acquisition: the "maxBuyAmountInUsdt" limit and the buyer’s cooldown limit.
> 3. Finally, the attacker drained BUSD from the pool using the NGP Token purchased earlier. While certain fees were deducted (e.g., fees allocated to the treasury and reward pools), this action left the pool’s NGP reserves extremely depleted. This depletion, in turn, triggered a sharp surge in the token’s price—completing the manipulation.
>  
> Attack TX: app.blocksec.com/explorer/tx…

## 15. Sep 17, 2025 · 8:27 AM UTC · 1968230202468974649#m
- 链接：https://twitter.com/BlockSecTeam/status/1968230202468974649#m
- 作者：@BlockSecTeam

> 🚀 We're thrilled to announce that BlockSec has completed the security audit for @Upheavalfi!

## 16. Sep 17, 2025 · 4:32 AM UTC · 1968171129039933751#m
- 链接：https://twitter.com/BlockSecTeam/status/1968171129039933751#m
- 作者：@BlockSecTeam

> ALERT! Hours ago, our system detected a series of suspicious transactions on #BSC targeting an unverified contract (0x93fD192e1CD288F1f5eE0A019429B015016061F9), resulting in ~$150K in losses.
>  
> The root cause lies in the contract’s referral reward design: reward calculations depend on the manipulable spot price of the BURN/BUSD trading pair (note: the BURN token is issued by @burnArmy or @Burn_building ?).
>  
> Here’s a breakdown of how the attack unfolded:
> 1. When users stake or lock BURN via a referrer, the contract credits them with referral rewards in BUSD. These rewards are calculated based on the volume of BURN staked/locked and the real-time BURN/BUSD spot price.
> 2. The attacker exploited this flaw by manipulating BURN’s price via flash loans. They then repeatedly created new contracts to bypass two critical restrictions—the "one referral per address" rule and maximum investment limits—allowing them to accumulate artificially inflated BUSD rewards.
> 3. Afterward, the attacker sold off the remaining borrowed BURN to repurchase BUSD—a move that drove down BURN’s price. Finally, they used the previously accumulated BUSD credits to claim BURN at this depressed price, aiming to make profits.
>  
> One of the TXs: app.blocksec.com/explorer/tx…

## 17. Sep 15, 2025 · 11:31 AM UTC · 1967551758118625410#m
- 链接：https://twitter.com/BlockSecTeam/status/1967551758118625410#m
- 作者：@BlockSecTeam

> 🎉 MAJOR LAUNCH: BlockSec Security Incidents Library
>  
> Introducing the industry's first comprehensive blockchain security incident library!
>  
> Highlights of this library:
> ✅ Curated incidents with losses over $100K
> ✅ $2.9B+ total losses recorded across all incidents
> ✅ $500M+ protected by Phalcon, $20M+ successfully rescued
> ✅ Complete vulnerability coverage: Access Control, Business Logic, Price Manipulation & more
>  
> Powerful Features:
> 🔍 Search by project name or attack transaction hash
> 📥 One-click download of attack transactions & attacker addresses
> 🔗 Direct links to detailed root cause analysis
> 📋 Embeddable code for easy team sharing
>  
> More than just data—it's your Web3 security playbook! Every incident verified by Phalcon Security APP real-time detection.
>  
> #Web3Security

## 18. Sep 12, 2025 · 8:26 AM UTC · 1966418055900311556#m
- 链接：https://twitter.com/BlockSecTeam/status/1966418055900311556#m
- 作者：@BlockSecTeam

> 🚀 We are thrilled to announce that BlockSec will provide security auditing services for @Upheavalfi !

## 19. Sep 8, 2025 · 7:58 PM UTC · 1965142760660562210#m
- 链接：https://twitter.com/BlockSecTeam/status/1965142760660562210#m
- 作者：@BlockSecTeam

> securityalliance.org/news/20…

## 20. Sep 2, 2025 · 9:35 AM UTC · 1962811690631983342#m
- 链接：https://twitter.com/BlockSecTeam/status/1962811690631983342#m
- 作者：@BlockSecTeam

> An interesting transaction on #BSC, involving ~$24M in funds—is it a phishing incident or merely a normal position adjustment?
> app.blocksec.com/explorer/tx…
>  
> The approval took place in this transaction: app.blocksec.com/explorer/tx…

## 21. Sep 2, 2025 · 5:05 AM UTC · 1962743751568433416#m
- 链接：https://twitter.com/BlockSecTeam/status/1962743751568433416#m
- 作者：@BlockSecTeam

> ALERT! Our system detected a suspicious transaction targeting @bunni_xyz ’s contract on #Ethereum, and the loss is ~$2.3M. Please take actions ASAP.

## 22. Aug 30, 2025 · 4:18 PM UTC · 1961825928876642766#m
- 链接：https://twitter.com/BlockSecTeam/status/1961825928876642766#m
- 作者：@BlockSecTeam

> It’s been an incredible journey at Virtual Asset Technical Exchange 2025 in New Orleans! 🇺🇸
>  
> Our cofounder @yajinzhou presented our team’s work on how crypto is used in Southeast Asia scam compounds and shared insights on tracing illicit funds. Excited to see our technology helping investigations into cryptocurrency abuse in cybercrime.
>  
> #CryptoSecurity #Web3 #BlockchainForensics

## 23. Aug 25, 2025 · 3:48 AM UTC · 1959825233453650293#m
- 链接：https://twitter.com/BlockSecTeam/status/1959825233453650293#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected two attack transactions targeting an unknown contract (0x5a46c6) on #BSC, resulting in a loss of ~$85K.
>  
> Specifically, the attacker utilized EIP-7702 on its own address to leverage the flash loan callback function, manipulating prices and conducting staking activities. By exploiting the flawed time check mechanism, the attacker directly executed two transactions (stake and unstake) to illicitly profit.
>  
> Notably, three key vulnerabilities enabled this attack:
> 1) Flawed flash loan protection check: The attacker exploited EIP-7702 to bypass the flash loan protection mechanism.
> 2) Spot price dependency.
> 3) Flawed time interval check for staking and unstaking: The duration calculation incorrectly uses "timestamp modulo day" (timestamp % day) instead of relying on the end time of the previous stake.
>  
> TX1: app.blocksec.com/explorer/tx…
> TX2: app.blocksec.com/explorer/tx…

## 24. Aug 19, 2025 · 6:07 AM UTC · 1957685945194672238#m
- 链接：https://twitter.com/BlockSecTeam/status/1957685945194672238#m
- 作者：@BlockSecTeam

> 🚀 We're thrilled to announce that BlockSec has completed the security audit for @StableStock !

## 25. Jun 26, 2025 · 5:24 AM UTC · 1938106182989979718#m
- 链接：https://twitter.com/BlockSecTeam/status/1938106182989979718#m
- 作者：@BlockSecTeam

> We released the SoK paper on stablecoin collaborated with City University of Hongkong.
>  
> See the link: arxiv.org/abs/2506.17622
>  
> Comments and suggestions are appreciated.

## 26. Aug 16, 2025 · 10:18 AM UTC · 1956661877473476962#m
- 链接：https://twitter.com/BlockSecTeam/status/1956661877473476962#m
- 作者：@BlockSecTeam

> Alert! Our system has detected suspicious transactions targeting an unknown contract (named D3XAT) on #BSC, resulting in an estimated loss of ~$160K. Although the contract is not open-source, preliminary analysis indicates a likely case of price manipulation due to spot price dependency.
>  
> TX: app.blocksec.com/explorer/tx…

## 27. Jul 25, 2025 · 12:59 AM UTC · 1948548552143040793#m
- 链接：https://twitter.com/BlockSecTeam/status/1948548552143040793#m
- 作者：@BlockSecTeam

> Blockchain Security Award Ceremony
>  
> Pioneering blockchain security talent development to strengthen Hong Kong's Web3 industry. 🚀
>  
> BlockSec will award the BlockSec Blockchain Security Award at @HongKongPolyU, recognizing outstanding talent and celebrating the future of blockchain security.
>  
> 📍 Location: The Hong Kong Polytechnic University
> 🗓️ Date: 25th July, 2025
>  
> Empowering the next generation of Web3 leaders through innovation and education.
>  
> #Web3 #BlockchainEducation #BlockSec #PolyU

## 28. Jul 23, 2025 · 11:12 AM UTC · 1947978159149994231#m
- 链接：https://twitter.com/BlockSecTeam/status/1947978159149994231#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected a series of malicious transactions targeting an unverified contract (0x16d7c6f43df19778e382b7a84bcb8c763971a551) on the Binance Smart Chain (BSC), with losses exceeding $600K. Since the contract is not open-source, our investigation indicates the root cause lies in a lack of slippage protection. This vulnerability allowed attackers to exploit the contract by using a fake liquidity pool to harvest #TA tokens (0x539ae81a166e5e80aed211731563e549c411b140), which were then sold in the legitimate pool to make profits.
>  
> Users who have granted approvals to this contract (0x16d7c6f43df19778e382b7a84bcb8c763971a551) are strongly advised to revoke those permissions immediately to prevent further losses!!!
>  
> Attack TXs:
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…

## 29. Jul 24, 2025 · 3:59 AM UTC · 1948231611654484161#m
- 链接：https://twitter.com/BlockSecTeam/status/1948231611654484161#m
- 作者：@BlockSecTeam

> The submission deadline to the ACM CCS workshop on Decentralized Finance and Security has been extended to July 28, 2025 (AoE).
>  
> Thanks to our incredible program committee & chairs.
>  
> @yaish_aviv @christoftorres @alexcryptan @chendaLiu @PulpSpy @jgorzny @0xlf_ @manv_sc @pszalach @mysteryfigure @KaihuaQIN @flotschorsch @zzzihaoli @masserova @dmoroz @ObadiaAlex @chiachih_wu @VeroCEG @KushalBabel @0xFanZhang @lzhou1110 @lzhou1110 @chunghaocrypto
> …and to our steering committee: @TheWattenhofer @dawnsongtweets @HatforceSec @Daeinar
>  
> Learn more & submit: defiwork.shop

## 30. Jul 14, 2025 · 2:45 PM UTC · 1944770132280017053#m
- 链接：https://twitter.com/BlockSecTeam/status/1944770132280017053#m
- 作者：@BlockSecTeam

> Phalcon Explorer supports HyperEVM now @HyperliquidX
>  
> You can see the call trace of HyperEVM to understand the transaction.
>  
> A simple transaction:
>  
> app.blocksec.com/explorer/tx…

## 31. Jul 14, 2025 · 7:11 AM UTC · 1944656034653741122#m
- 链接：https://twitter.com/BlockSecTeam/status/1944656034653741122#m
- 作者：@BlockSecTeam

> Thrilled to partner with @InterlaceMoney, a true innovator bridging Web2 and Web3 finance. We're excited to work with a team that places such a high priority on security and compliance.
>  
> This is precisely why we built our Phalcon Compliance App. It empowers partners like Interlace with real-time AML/CFT screening and on-chain risk evaluation, making trusted digital finance a reality. @Phalcon_xyz
>  
> Learn more about our solution: blocksec.com/phalcon/complia…
>  
> #Web3Security #Compliance #AML

## 32. Jul 11, 2025 · 5:59 PM UTC · 1943731901392699845#m
- 链接：https://twitter.com/BlockSecTeam/status/1943731901392699845#m
- 作者：@BlockSecTeam

> Friday night is hack night. 🧪🕵️‍♂️
>  
> Let’s analyze an attack (happened a couple of hours ago) no one really talks about.
>  
> app.blocksec.com/explorer/tx…

## 33. Jul 11, 2025 · 9:09 AM UTC · 1943598560689160619#m
- 链接：https://twitter.com/BlockSecTeam/status/1943598560689160619#m
- 作者：@BlockSecTeam

> x.com/i/article/194359504479…

## 34. Jul 11, 2025 · 3:51 AM UTC · 1943518566831296566#m
- 链接：https://twitter.com/BlockSecTeam/status/1943518566831296566#m
- 作者：@BlockSecTeam

> Our system has detected a series of attacks targeting @Bankroll_Status across both Ethereum and BSC, resulting in total losses of ~$400K since Sep 2024. Attempts to contact the project team have received no response.
>  
> These attacks exploit the same root vulnerability in the distribute() function, which updates the global profitPerShare_ only after syncing a user's _updatedPayouts during buy/sell operations. Since rewards are distributed over time based on the pool's payoutRate, an attacker can buy a large amount tokens via several staking contracts, immediately sell them, and gain more than their fair share of dividends. This sequencing flaw allows the attacker to claim excess rewards before the system correctly accounts for them.
>  
> The most recent attack transaction—resulting in ~$115K in losses—was front-run by c0ffeebabe.eth: app.blocksec.com/explorer/tx…
> Note that in this transaction, ETH was first donated to the pool to ensure there were enough rewards to drain.

## 35. Jul 9, 2025 · 4:52 PM UTC · 1942990338731229379#m
- 链接：https://twitter.com/BlockSecTeam/status/1942990338731229379#m
- 作者：@BlockSecTeam

> x.com/i/article/194298618589…

## 36. Jul 9, 2025 · 3:14 PM UTC · 1942965515007455521#m
- 链接：https://twitter.com/BlockSecTeam/status/1942965515007455521#m
- 作者：@BlockSecTeam

> Our initial analysis indicates that GMX’s order-keeper account (0xd4266f8f82f7405429ee18559e548979d49160f3) issued a transaction,  which passes a contract address as the first parameter of executeDecreaseOrder, and then the attacker leveraged a reentrancy to carry out the attack.

## 37. Jul 9, 2025 · 2:05 PM UTC · 1942948251671433285#m
- 链接：https://twitter.com/BlockSecTeam/status/1942948251671433285#m
- 作者：@BlockSecTeam

> app.blocksec.com/explorer/tx…
>  
> Worth a further analysis  @GMX_IO

## 38. Jul 8, 2025 · 2:38 PM UTC · 1942594222353260893#m
- 链接：https://twitter.com/BlockSecTeam/status/1942594222353260893#m
- 作者：@BlockSecTeam

> . @PeapodsFinance’s Ethereum contracts have reportedly been attacked due to a price oracle issue. It looks like the original attacker successfully executed the first exploit, but its second attempt was front-run by Yoink—a well-known MEV frontrunner. It may be possible to contact them to mitigate losses.
>  
> Attack TXs:
> 1. app.blocksec.com/explorer/tx…
>  
> 2. app.blocksec.com/explorer/tx…

## 39. Jul 7, 2025 · 4:03 AM UTC · 1942071854522380498#m
- 链接：https://twitter.com/BlockSecTeam/status/1942071854522380498#m
- 作者：@BlockSecTeam

> Best DeFi Papers Award 2024 – Call for Nominations!
>  
> We honor research advancing Decentralized Finance across:
> * Track A: Best Theoretical Paper
> * Track B: Best Practical Paper
> * Track C: Best Industry Paper
>  
> For the nominations aim for industry track, one of the authors should come from industry, and please describe how the research has been practically used in industry.
>  
> Each winning paper receives:
> • $500 in USDT/DAI/USDC
> • A commemorative statue & certificate
>  
> Eligibility: Peer-reviewed papers published between Jan 1 – Dec 31, 2024
>  
> Timeline:
> July 1 – Nominations open
> July 21 – Nominations close
> Aug 20 – Recipients notified
>  
> Awarded authors will present at DeFi’25 Workshop @ ACM CCS 2025 on Oct 17 (remote/on-site) and provide a 2-min video summary.
>  
> Nominate your paper now: defiwork.shop

## 40. Jul 6, 2025 · 9:16 AM UTC · 1941788315549946225#m
- 链接：https://twitter.com/BlockSecTeam/status/1941788315549946225#m
- 作者：@BlockSecTeam

> A hack in the weekend:
>  
> app.blocksec.com/explorer/tx…
>  
> around 204k loss

## 41. Jul 6, 2025 · 9:10 AM UTC · 1941786763971948631#m
- 链接：https://twitter.com/BlockSecTeam/status/1941786763971948631#m
- 作者：@BlockSecTeam

> x.com/i/article/194178468800…

## 42. Jul 6, 2025 · 8:53 AM UTC · 1941782542870536288#m
- 链接：https://twitter.com/BlockSecTeam/status/1941782542870536288#m
- 作者：@BlockSecTeam

> We published our in-depth analysis and reflections on the Resupply protocol attack incident
>  
> See:
>  
> blocksec.com/blog/in-depth-a…

## 43. Jul 5, 2025 · 6:25 PM UTC · 1941564153765954025#m
- 链接：https://twitter.com/BlockSecTeam/status/1941564153765954025#m
- 作者：@BlockSecTeam

> Stay safe.

## 44. Jul 4, 2025 · 3:20 PM UTC · 1941155150514508194#m
- 链接：https://twitter.com/BlockSecTeam/status/1941155150514508194#m
- 作者：@BlockSecTeam

> “Moving Bricks”: Inside the Scam Industry’s Secret Logistics Network
>  
> 1/ Imagine you send your life savings to what you think is a safe investment.
>  
> Ten minutes later, that money is split into dozens of smaller transfers, wired across continents, converted into crypto, and gone forever.
>  
> This is not a movie plot. It’s happening every day, in a system criminals call Moving Bricks.
>  
> Read more: globalchinapulse.net/moving-…

## 45. Jul 3, 2025 · 10:12 AM UTC · 1940715252011937848#m
- 链接：https://twitter.com/BlockSecTeam/status/1940715252011937848#m
- 作者：@BlockSecTeam

> Regarding compliance risk management, BlockSec's CEO stated: "As a globally leading blockchain security company, BlockSec not only possesses exceptional technical capabilities in security protection, but its Phalcon Compliance APP (compliance monitoring platform) also provides virtual asset service providers with real-time risk identification and compliance management capabilities." @Phalcon_xyz
>  
> Learn more: blocksec.com/phalcon/complia…

## 46. Jul 3, 2025 · 10:12 AM UTC · 1940715248287412574#m
- 链接：https://twitter.com/BlockSecTeam/status/1940715248287412574#m
- 作者：@BlockSecTeam

> BlockSec is proud to announce a strategic partnership with @FinTax_Official.
>  
> As a cryptocurrency tax software and service provider, FinTax has developed comprehensive Crypto tax systems and professional tax teams, providing customized solutions for numerous industry institutions and participating in cryptocurrency tax industry standards development across multiple countries and regions.
>  
> This partnership will create an integrated solution combining security protection, compliance risk management, and tax management, building safer, more compliant, and more professional infrastructure services for the industry.

## 47. Jul 3, 2025 · 3:22 AM UTC · 1940612190886903910#m
- 链接：https://twitter.com/BlockSecTeam/status/1940612190886903910#m
- 作者：@BlockSecTeam

> . @QuickswapDEX
>  
> 7702 breaks the assumption in the code, and creates an attack surface.
>  
> app.blocksec.com/explorer/tx…

## 48. Jun 27, 2025 · 9:56 AM UTC · 1938536823539499326#m
- 链接：https://twitter.com/BlockSecTeam/status/1938536823539499326#m
- 作者：@BlockSecTeam

> “Compliance isn’t just a requirement—it’s the key to unlocking stablecoin adoption.”
>  
> At the Scale Stablecoin Summit, BlockSec CEO Prof. Yajin Zhou @yajinzhou  showcased how Phalcon @Phalcon_xyz empowers stablecoin issuers with real-time risk blocking and a one-stop compliance solution to meet growing regulatory demands and ensure operational security.
>  
> He also highlighted that trust and security are essential to building a sustainable stablecoin ecosystem.
>  
> #Stablecoin #Compliance #Blockchain #Security #ScaleSummit2025

## 49. Jun 26, 2025 · 3:54 AM UTC · 1938083408762311100#m
- 链接：https://twitter.com/BlockSecTeam/status/1938083408762311100#m
- 作者：@BlockSecTeam

> 🌟 Stablecoins in Focus | Scale Stablecoin Summit
>  
> As stablecoins enter the era of strong regulation, the industry faces unprecedented compliance and security challenges.
>  
> BlockSec will join the panel "The Forces Behind Stablecoin Adoption" to share how our Phalcon @Phalcon_xyz delivers comprehensive compliance and security solutions, offering in-depth insights into stablecoin issuance, anti-money laundering, and future market trends, empowering stablecoins to navigate this evolving landscape 🚀.
>  
> See you in Hong Kong Tonight! 🇭🇰
>  
> #Stablecoin #Blockchain #Compliance #Security

## 50. Jun 26, 2025 · 3:30 AM UTC · 1938077421665980859#m
- 链接：https://twitter.com/BlockSecTeam/status/1938077421665980859#m
- 作者：@BlockSecTeam

> 😔 $9.5M lost in today’s attack...

## 51. Jun 26, 2025 · 3:17 AM UTC · 1938074034119516504#m
- 链接：https://twitter.com/BlockSecTeam/status/1938074034119516504#m
- 作者：@BlockSecTeam

> Bypassing insolvency checks—a classic attack vector for lending protocols—demands attention!

## 52. Jun 26, 2025 · 2:32 AM UTC · 1938062681866678593#m
- 链接：https://twitter.com/BlockSecTeam/status/1938062681866678593#m
- 作者：@BlockSecTeam

> This attack transaction occurred on Ethereum. If @ResupplyFi were using Phalcon, they could have detected it in the mempool stage (before the attack goes on-chain) and automatically paused the protocol— preventing the full $9.5M loss! 🔍

## 53. Jun 25, 2025 · 4:04 PM UTC · 1937904699610718520#m
- 链接：https://twitter.com/BlockSecTeam/status/1937904699610718520#m
- 作者：@BlockSecTeam

> Our system detected several attack transactions targeting @SiloFinance's smart contracts on different chains, with the root cause identified as flawed parameter validation logic in the flashloan callback function. By exploiting this vulnerability, the attacker could borrow assets using victims' collateral. The protocol team has paused the affected smart contracts.
>  
> While the team released a somewhat vague statement, forensic analysis of transactions reveals characteristics consistent with an attack:
> 1) Legitimate testing scenarios rarely necessitate moving substantial funds.
> 2) An address which launched attack transactions on Ethereum in the incident was funded via #TornadoCash.
>  
> Timeline (UTC):
> 14:11:23  - Attack on Ethereum ~$546K loss
> 14:20:23 - Attack on Ethereum ~$3K loss
> 14:38 - @SiloFinance 's official announcement on X
> 14:44:02 -  Attack on Sonic ~$1K loss
> 14:44:27  - Attack on Sonic ~$2K loss
>  
> Attack TXs:
> 1) 1st TX on Ethereum : app.blocksec.com/explorer/tx…
> 2) 2nd TX on Ethereum : app.blocksec.com/explorer/tx…
> 3) 1st TX on Sonic : app.blocksec.com/explorer/tx…
> 4) 2nd TX on Sonic : app.blocksec.com/explorer/tx…
>  
> Attacker addresses:
> 1) Ethereum: 0x04377cfaf4b4a44bb84042218cdda4cebcf8fd62
> 2) Sonic: 0xba321eb613d76c758995fb33c64d71b9db0aaf6e

## 54. Jun 25, 2025 · 6:06 AM UTC · 1937754321111908577#m
- 链接：https://twitter.com/BlockSecTeam/status/1937754321111908577#m
- 作者：@BlockSecTeam

> Some vToken losses reported, but unrelated to @VenusProtocol

## 55. Jun 23, 2025 · 9:34 AM UTC · 1937081834661351425#m
- 链接：https://twitter.com/BlockSecTeam/status/1937081834661351425#m
- 作者：@BlockSecTeam

> 🚀 Calling all Phalcon Explorer users!
>  
> We want your feedback — our User Experience Survey takes just 2–3 mins. Got ideas or suggestions? Let us know!
>  
> 👉 Survey link: docs.google.com/forms/d/e/1F…
>  
> 🎁 Bonus: All survey participants will get a free 1-week @MetaSleuth Pro trial (worth $180/month)! Thank you for helping us improve!
>  
> #Web3 #DeFi #Blockchain #PhalconExplorer

## 56. Jun 23, 2025 · 4:12 AM UTC · 1937000909374140425#m
- 链接：https://twitter.com/BlockSecTeam/status/1937000909374140425#m
- 作者：@BlockSecTeam

> Interesting BSC transactions exploiting EIP-7702’s EOA code delegation.

## 57. Jun 20, 2025 · 1:34 PM UTC · 1936055015229030612#m
- 链接：https://twitter.com/BlockSecTeam/status/1936055015229030612#m
- 作者：@BlockSecTeam

> 🚀 We’ve just released a new version of MetaSuites!
>  
> 🔒 Fix: Thanks to @elyx0 for reporting a potential issue —we’ve replaced it with a safer implementation.
>  
> 📝 New feature: Local labels in MetaSuites now sync with Phalcon Explorer (locally only—your data stays private).
>  
> 🙈 Quality-of-life: You can now hide zero-amount token transfers to filter out noisy address-posting txs.

## 58. Jun 20, 2025 · 8:13 AM UTC · 1935974251334627648#m
- 链接：https://twitter.com/BlockSecTeam/status/1935974251334627648#m
- 作者：@BlockSecTeam

> Our @Phalcon_xyz system detected an attack tx to
> @libertum_token caused ~4.8K USD loss.
>  
> Book a demo to know how Phalcon can help protocol to detect hacks and take automated actions.
>  
> blocksec.com/book-demo

## 59. Jun 18, 2025 · 11:55 AM UTC · 1935305319732887834#m
- 链接：https://twitter.com/BlockSecTeam/status/1935305319732887834#m
- 作者：@BlockSecTeam

> localhost:8080/i/spaces/1RDGlzvvbpDxL

## 60. Jun 16, 2025 · 3:10 AM UTC · 1934448470544515294#m
- 链接：https://twitter.com/BlockSecTeam/status/1934448470544515294#m
- 作者：@BlockSecTeam

> The #BNBKickstart program offers a range of essential tools and services from across the BNB Chain ecosystem to help developers and projects kickstart their building journey.
>  
> 👉 Apply now bnbchain.org/en/programs/kic…

## 61. Jun 16, 2025 · 3:10 AM UTC · 1934448467059114248#m
- 链接：https://twitter.com/BlockSecTeam/status/1934448467059114248#m
- 作者：@BlockSecTeam

> 🎉 We're proud to announce that BlockSec is now an official @BNBCHAIN Kickstart Program Service Provider!
>  
> In BNB Chain Kickstart Program, we offer:
> 🔸Audit: 5% exclusive discount with priority onboarding—get started in 8 working days.
> 🔸Phalcon Security APP: Free real-time monitoring for 3 addresses to promptly identify and mitigate security threats. First-time Standard Plan subscribers receive an extra month free.
> 🔸Phalcon Compliance APP: Ensure project compliance with KYA & KYT. Enjoy a 20% discount on your first-year subscription.
> 🔸Dedicated 1-on-1 Support: Immediate SOP emergency response from our expert team for fast and effective issue resolution.
> 🔸Co-Branding Exposure: Get featured on our official social media channels.

## 62. Jun 9, 2025 · 9:32 AM UTC · 1932007814400229696#m
- 链接：https://twitter.com/BlockSecTeam/status/1932007814400229696#m
- 作者：@BlockSecTeam

> Proud to partner with @minelabs_
>  
> Let's make on-chain safer for everyone! 🫡
>  
> #BlockchainSafety

## 63. Jun 6, 2025 · 10:52 AM UTC · 1930940746938696122#m
- 链接：https://twitter.com/BlockSecTeam/status/1930940746938696122#m
- 作者：@BlockSecTeam

> “Security isn’t just about audits—it’s about full-lifecycle protection.”
>  
> At #SolanaSummitAPAC, BlockSec’s Director of Security Service, Dr. Jiang, emphasized the importance of securing protocols across their entire lifecycle.
>  
> With BlockSec Phalcon (@Phalcon_xyz), we showcased how proactive, real-time monitoring and automatic blocking effectively mitigate live attacks.
>  
> More importantly, we’re encouraged to see more builders prioritizing security as a foundation for creating a stronger Web3 ecosystem. ⚡🌐
>  
> #SolanaSummitAPAC #BlockSec #Phalcon

## 64. Jun 6, 2025 · 6:39 AM UTC · 1930877084999586066#m
- 链接：https://twitter.com/BlockSecTeam/status/1930877084999586066#m
- 作者：@BlockSecTeam

> A big thank you to everyone who joined BlockSec's workshop at #SolanaSummitAPAC2025!
> @SolanaSummitOrg
>  
> 🌟 BlockSec shared how we safeguard the Solana ecosystem across its full lifecycle through 4 KEY DIMENSIONS:
> 🔒  Security Audit: In-depth security audits for both smart contracts and blockchains.
> ⚡ BlockSec @Phalcon_xyz : Provides real-time monitoring and automatic threat-blocking protections.
> 🕵️ BlockSec @MetaSleuth : Provides advanced on-chain investigation and tracking to analyze efficiently.
> 🔗 Swap Actions API: Subscribe to DEX Swap Actions on Solana with One Single API -- receive parsed swap actions within a transaction.
> docs.blocksec.com/transactio…
>  
> Building a safer Web3, one step at a time. 🌐
>  
> #Web3Security #SolanSummitAPAC #BlockSec

## 65. Jun 5, 2025 · 6:02 AM UTC · 1930505401247867130#m
- 链接：https://twitter.com/BlockSecTeam/status/1930505401247867130#m
- 作者：@BlockSecTeam

> Meet BlockSec at Solana Summit 🤘
>  
> @SolanaSummitOrg @SolanaFndn

## 66. Jun 4, 2025 · 4:59 PM UTC · 1930308444839653591#m
- 链接：https://twitter.com/BlockSecTeam/status/1930308444839653591#m
- 作者：@BlockSecTeam

> . @CetusProtocol hacker was moving funds to tornado cash.
>  
> etherscan.io/tx/0xdcfb8d40b6…

## 67. Jun 3, 2025 · 10:25 AM UTC · 1929846817338089480#m
- 链接：https://twitter.com/BlockSecTeam/status/1929846817338089480#m
- 作者：@BlockSecTeam

> 🔥 BlockSec is excited to join Solana Summit APAC 2025！@SolanaSummitOrg
>  
> 👉 Register now: lu.ma/qzpui3j6
> 🛠️ Workshop: Securing Solana Ecosystem Across Its Full Lifecycle
> 🕙 2025.6.6 10:00-10:25（UTC+7）
> 📍 Workshop Room 2: The Boardroom Round Table Room (44PAX)
>  
> 🎤 Panel: Security Blind Spots: What You’re Overlooking (Until It’s Too Late)
> 🕒 2025.6.6 15:40-16:10（UTC+7）
> 📍 Main Stage
>  
> From audits to real-time attack monitoring (@Phalcon_xyz), crypto tracking and investigation platform (@MetaSleuth), and Swap Actions API, BlockSec is committed to securing the Solana ecosystem across its full lifecycle.@SolanaFndn
>  
> We’re shaping the way for a safer Solana.
>  
> Join us in Da Nang! 🏄🏻‍♂️
>  
> #SolanaSummitAPAC #BlockchainSecurity #Solana

## 68. Jun 3, 2025 · 9:17 AM UTC · 1929829660935819355#m
- 链接：https://twitter.com/BlockSecTeam/status/1929829660935819355#m
- 作者：@BlockSecTeam

> BlockSec Wallet Label & Screening APIs have been launched on @OmniMCP 🙌
>  
> Thanks for @Footprint_Data 's integration efforts! Excited to see AI and Crypto spark new innovations together!
>  
> #AI #Web3 #Crypto #Security #Data

## 69. May 30, 2025 · 2:45 PM UTC · 1928462800621633590#m
- 链接：https://twitter.com/BlockSecTeam/status/1928462800621633590#m
- 作者：@BlockSecTeam

> From Security to Trust: BlockSec at Trust Day@DASFAA 2025! 🚀✨
>  
> Stablecoins face increasing compliance and security challenges, from smart contract vulnerabilities to illegal transactions and regulatory scrutiny. At this inspiring event, BlockSec showcased end-to-end solutions to address these issues:
> -Conducting security audits to reduce vulnerabilities at the source.
> -24/7 risk monitoring and automated response with Phalcon Security APP.
> -Real-time risk exposure analysis and illegal activity detection with Phalcon Compliance APP. 🔐⚡
>  
> Big thanks to @Cobo_Global and @FOMOPayOfficial for joining the conversation!
>  
> Let’s keep building a safer world together. 💻🌟
>  
> #DASFAA2025 #TrustDay #BlockchainSecurity #Stablecoins

## 70. May 29, 2025 · 3:46 PM UTC · 1928115801573179579#m
- 链接：https://twitter.com/BlockSecTeam/status/1928115801573179579#m
- 作者：@BlockSecTeam

> Thanks for mentioning us! Let's make a more secure Web3 ecosystem.

## 71. May 29, 2025 · 11:26 AM UTC · 1928050300780433553#m
- 链接：https://twitter.com/BlockSecTeam/status/1928050300780433553#m
- 作者：@BlockSecTeam

> Oracle Monitoring Feature Now Live in BlockSec Phalcon!
> blocksec.com/blog/phalcon-or…
>  
> Mango Markets ($116M), Venus ($11.2M), and Rho Markets ($7.6M) all suffered massive losses from oracle exploits. With Phalcon's Oracle Monitors, these attacks could have been stopped cold.
>  
> 🔥 How?
> → Live price deviation & range alerts
> → Oracle price feed comparison
> → Update delays + health checks
> → Instant alerts + auto-response to anomalies
> 🟰 No window for exploiters—losses can be fully prevented.
>  
> Read more👇

## 72. May 29, 2025 · 3:46 AM UTC · 1927934439713296526#m
- 链接：https://twitter.com/BlockSecTeam/status/1927934439713296526#m
- 作者：@BlockSecTeam

> BlockSec is live at the world’s biggest Bitcoin event Bitcoin2025 Las Vegas! 🚀✨@TheBitcoinConf
>  
> With Bitcoin's market cap making history in 2025, security is more crucial than ever. 🔐 From auditing services to real-time risk monitoring and blocking, we’re here to safeguard Bitcoin Layer 2 and cross-chain protocols.
>  
> Swing by our booth 2339 @exSatNetwork for epic moments! 🌟💻
>  
> #Bitcoin2025 #Blockchain

## 73. May 28, 2025 · 3:00 PM UTC · 1927741755400884661#m
- 链接：https://twitter.com/BlockSecTeam/status/1927741755400884661#m
- 作者：@BlockSecTeam

> Since @Corkprotocol has paused the protocol, I'd like to share some findings from our initial investigation based on the attack transaction trace: it appears that the protocol fails to properly verify the arguments passed to the CorkCall function, allowing the attacker to specify a fake paymentToken and profit from it.

## 74. May 28, 2025 · 12:38 PM UTC · 1927706024414158987#m
- 链接：https://twitter.com/BlockSecTeam/status/1927706024414158987#m
- 作者：@BlockSecTeam

> A bad day… @Corkprotocol was attacked.  Detected by @Phalcon_xyz
>  
> app.blocksec.com/explorer/tx…

## 75. May 28, 2025 · 6:08 AM UTC · 1927607817378177316#m
- 链接：https://twitter.com/BlockSecTeam/status/1927607817378177316#m
- 作者：@BlockSecTeam

> Our @Phalcon_xyz system detected an possibly attack tx to @usualmoney . Now the vault has paused, we can share the link here:
>  
> app.blocksec.com/explorer/tx…
>  
> Book a demo to know how Phalcon can help protocol to monitor hacks and take automated actions.
>  
> blocksec.com/phalcon/securit…

## 76. May 27, 2025 · 5:05 PM UTC · 1927410963315060823#m
- 链接：https://twitter.com/BlockSecTeam/status/1927410963315060823#m
- 作者：@BlockSecTeam

> Good to help secure protocols in @StoryEcosystem.

## 77. May 25, 2025 · 5:44 PM UTC · 1926695761921634709#m
- 链接：https://twitter.com/BlockSecTeam/status/1926695761921634709#m
- 作者：@BlockSecTeam

> We have released the dataset (the phishing contracts) of our SIGMETRICS 2025 paper. Feel free to use the dataset and cite our paper :)
>  
> github.com/blocksecteam/phis…

## 78. May 23, 2025 · 3:44 PM UTC · 1925941031884804493#m
- 链接：https://twitter.com/BlockSecTeam/status/1925941031884804493#m
- 作者：@BlockSecTeam

> 1/ Two lines of code. One giant mess.
>  
> That’s all it took to drain $223 million from @CetusProtocol.
>  
> * $162M was paused (thanks, Sui)
> * $60M was bridged to Ethereum, now chilling in two wallets
> * And yes — AI can help explain the whole mess.
>  
> Here’s the story 🧵

## 79. May 23, 2025 · 3:34 AM UTC · 1925757100304515446#m
- 链接：https://twitter.com/BlockSecTeam/status/1925757100304515446#m
- 作者：@BlockSecTeam

> Yep, the overflow check can be bypassed:
> github.com/CetusProtocol/int…

## 80. May 22, 2025 · 7:09 PM UTC · 1925630025585312068#m
- 链接：https://twitter.com/BlockSecTeam/status/1925630025585312068#m
- 作者：@BlockSecTeam

> The gas fee of @CetusProtocol hacker is from 0xff1070d40277a3d748a3222b44eab08ed588b256094ecdc2090badf0f9f5e9f4.
> This address looks the wallet address of Kucoin @kucoincom ? Anyone can help double check this?

## 81. May 22, 2025 · 5:24 PM UTC · 1925603580113035298#m
- 链接：https://twitter.com/BlockSecTeam/status/1925603580113035298#m
- 作者：@BlockSecTeam

> Our preliminary investigation on the @CetusProtocol security incident.
>  
> Without @Phalcon_xyz, it's really a headache when analyzing hacks.

## 82. May 22, 2025 · 2:44 PM UTC · 1925563331965780343#m
- 链接：https://twitter.com/BlockSecTeam/status/1925563331965780343#m
- 作者：@BlockSecTeam

> A sad day (and night) ...

## 83. May 21, 2025 · 11:47 AM UTC · 1925156444862832893#m
- 链接：https://twitter.com/BlockSecTeam/status/1925156444862832893#m
- 作者：@BlockSecTeam

> 👀 This is a must-read for risk management teams! blocksec.com/blog/phishing-c…
>  
> From EOAs to phishing contracts, scammers are getting smarter. 37K phishing contracts, 171K victims, and $190M in losses—don't let your users become the next target!
>  
> Our SIGMETRICS 2025 research dives deep into phishing contracts and provides actionable strategies to protect your users. Read more👇

## 84. May 20, 2025 · 4:02 AM UTC · 1924677205084536915#m
- 链接：https://twitter.com/BlockSecTeam/status/1924677205084536915#m
- 作者：@BlockSecTeam

> Proxy and implementation misused in 2025, ~64k loss
>  
> app.blocksec.com/explorer/tx…
>  
> app.blocksec.com/explorer/tx…

## 85. May 19, 2025 · 12:07 PM UTC · 1924436807363817730#m
- 链接：https://twitter.com/BlockSecTeam/status/1924436807363817730#m
- 作者：@BlockSecTeam

> 🛡️.@soon_svm SOON Bridge is powered by BlockSec Phalcon's real-time protection, offering top-tier security through cross-chain asset depegging detection, contract security, and operational threat monitoring.
>  
> 🚨 Any detected anomalies will trigger immediate alerts and automated responses, ensuring assets remain secure. 💪
>  
> Learn more about BlockSec Phalcon at blocksec.com/phalcon/securit…

## 86. May 12, 2025 · 3:18 PM UTC · 1921948087892705472#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948087892705472#m
- 作者：@BlockSecTeam

> 📢 6/ Final Call to Action
>  
> Web3 = trustless systems, but phishing is still your Achilles’ heel.
>  
> DMARC isn’t optional – it’s your first line of defense against phishing to your community.
>  
> Lock it down. Now. 🔒

## 87. May 12, 2025 · 3:18 PM UTC · 1921948085552353531#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948085552353531#m
- 作者：@BlockSecTeam

> 🔧 5/ How Web3 Projects Should Set DMARC
>  
> 1️⃣ Publish a strict DNS record with p=reject
>  
> 2️⃣ Use tools like EasyDMARC to monitor spoofing attempts.
>  
> 3️⃣ Educate your community to never trust emails without DMARC enforcement.

## 88. May 12, 2025 · 3:18 PM UTC · 1921948083216175364#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948083216175364#m
- 作者：@BlockSecTeam

> 🛡️ 4/ Fix: Enforce “p=quarantine” or “p=reject”
>  
> * p=reject: Block unauthorized emails pretending to be you.
>  
> * Protects users from fake:
> - Wallet updates
> - Smart contract interactions
> - Team announcements

## 89. May 12, 2025 · 3:18 PM UTC · 1921948080821191017#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948080821191017#m
- 作者：@BlockSecTeam

> 📉 3/ Real Web3 Nightmares
>  
> * Fake “token sale” emails stealing ETH.
> * “KYC verification” scams draining wallets.
> * Spoofed “governance alerts” redirecting to malicious dApps.
>  
> All possible if your domain isn’t locked down.

## 90. May 12, 2025 · 3:18 PM UTC · 1921948078535274759#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948078535274759#m
- 作者：@BlockSecTeam

> 💥 2/ “p=none” = Open Season for Crypto Phishing
>  
> Example:
> Attacker sends: “URGENT: Claim your airdrop at phishy-link.xyz” from support@yourproject.com.
>  
> Email providers deliver it because your DMARC policy says “no enforcement.”
>  
> Result? Users lose funds → your reputation burns. 😱

## 91. May 12, 2025 · 3:18 PM UTC · 1921948076173828543#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948076173828543#m
- 作者：@BlockSecTeam

> 🔍 1/ Why DMARC matters for Web3
>  
> You’re not just protecting emails – you’re safeguarding private keys, wallets, and community trust.
>  
> Phishers spoofing admin@yourproject.com? With p=none, their fake "wallet migration" emails hit inboxes. 💸🔥

## 92. May 12, 2025 · 3:18 PM UTC · 1921948073783144933#m
- 链接：https://twitter.com/BlockSecTeam/status/1921948073783144933#m
- 作者：@BlockSecTeam

> 🚨 Web3 projects, listen up!
>  
> A weak DMARC policy (p=none) could turn your community into a phishing goldmine.
>  
> Your token holders’ crypto is at risk. Let’s unpack why. 👇
>  
> #Web3 #Cybersecurity

## 93. Apr 24, 2025 · 11:31 AM UTC · 1915367915442102377#m
- 链接：https://twitter.com/BlockSecTeam/status/1915367915442102377#m
- 作者：@BlockSecTeam

> 🚀 Big News for Crypto Compliance: the Phalcon Compliance APP is LIVE!
>  
> 🔥 Why it’s a game-changer:
> ✅ Detect illegal activities via AI-powered exposure and behavioral pattern analysis
> ✅ Pre-set FATF-compliant risk engines and custom rules
> ✅ Collaborate: Assign tasks, comment & blacklist
> ✅ 1-click STR, ready for regulators
> ✅ Security + Compliance in one platform
>  
> ⚡ Exclusive Launch Offer: The first 30 demos get FREE access!
> 👉 Request a demo NOW: blocksec.com/request-demo-co…
> 👀 Read the blog to learn more: blocksec.com/blog/phalcon-co…

## 94. Apr 22, 2025 · 2:14 PM UTC · 1914684330758070502#m
- 链接：https://twitter.com/BlockSecTeam/status/1914684330758070502#m
- 作者：@BlockSecTeam

> This API has multiple use cases. For example, users can subscribe to smart money accounts to track their swap actions and gain insights into their trading strategies. The following code snippet demonstrates how to use this API.

## 95. Apr 22, 2025 · 2:14 PM UTC · 1914684326928622008#m
- 链接：https://twitter.com/BlockSecTeam/status/1914684326928622008#m
- 作者：@BlockSecTeam

> BlockSec's Solana Swap Actions API is now available. With this API, you can access well-parsed swap actions without the need for an RPC endpoint, requiring just a few lines of code. Currently, it supports 48 different DEX protocols.
>  
> Document: docs.blocksec.com/transactio…

## 96. Apr 22, 2025 · 3:45 AM UTC · 1914525889934901451#m
- 链接：https://twitter.com/BlockSecTeam/status/1914525889934901451#m
- 作者：@BlockSecTeam

> 🚀 Thrilled to power the future of secure staking! 🚀
>  
> As the security partner behind @puffer_finance 's Institutional-Grade Staking & Restaking Solution, we at @BlockSecTeam are proud to bring battle-tested audits and cutting-edge safeguards to this game-changing EigenLayer-powered ecosystem.

## 97. Apr 16, 2025 · 11:46 AM UTC · 1912472774829801709#m
- 链接：https://twitter.com/BlockSecTeam/status/1912472774829801709#m
- 作者：@BlockSecTeam

> Scammers are becoming more cunning, using increasingly stealthy methods to conceal their rug pull intentions.
>  
> app.blocksec.com/explorer/tx…

## 98. Apr 16, 2025 · 7:46 AM UTC · 1912412347232317711#m
- 链接：https://twitter.com/BlockSecTeam/status/1912412347232317711#m
- 作者：@BlockSecTeam

> At 18:52 UTC on April 14, Phalcon @Phalcon_xyz  detected and blocked an initial exploit attempt targeting KiloEx, preventing $927K in potential losses. This critical intervention created a 47-minute window for the related team to respond, securing an additional $982K.
>  
> This fully validates the effectiveness of Phalcon and our STOP solution. Learn more:
> 🔗 Phalcon: blocksec.com/phalcon
> 🔗 STOP: blocksec.com/stop
>  
> Currently, our team is assisting @KiloEx_perp in tracing the stolen funds. For post-incident support (war room, fund tracking, root cause analysis), contact us: contact@blocksec.com

## 99. Apr 11, 2025 · 7:01 AM UTC · 1910588902026461358#m
- 链接：https://twitter.com/BlockSecTeam/status/1910588902026461358#m
- 作者：@BlockSecTeam

> RT @Phalcon_xyz: It appears that the victim, 0xa8292c010522a3d65880bad8e75fb9ac8c8da3e5, was likely tricked into signing an approve transac…

## 100. Apr 11, 2025 · 6:51 AM UTC · 1910586410551177368#m
- 链接：https://twitter.com/BlockSecTeam/status/1910586410551177368#m
- 作者：@BlockSecTeam

> woo, an interesting transaction.
>  
> app.blocksec.com/explorer/tx…

## 101. Apr 11, 2025 · 2:27 AM UTC · 1910520080183812553#m
- 链接：https://twitter.com/BlockSecTeam/status/1910520080183812553#m
- 作者：@BlockSecTeam

> Excited to collaborate with SoSoValue to create a safer on-chain ecosystem! 🫡
>  
> #SoSoValue #SSI #BlockchainSecurity

## 102. Apr 8, 2025 · 4:15 AM UTC · 1909460134830678454#m
- 链接：https://twitter.com/BlockSecTeam/status/1909460134830678454#m
- 作者：@BlockSecTeam

> 🎓​Prof. Yajin Zhou @yajinzhou, CEO of BlockSec @BlockSecTeam and Professor at Zhejiang University, is giving a speech on "​System Research for Blockchain: From Storage Optimization to Parallel Execution" at #WSC2025.
>  
> 🗓️Apr 8, 2025 | 09:30-17:30
> 📍STAGE 2, HK WEB3 FESTIVAL
>  
> 🔗Livestream: piped.kavin.rocks/live/Qq2mu_C-Y9w
>  
> #WSC2025 #Web3Festival #HK

## 103. Apr 6, 2025 · 1:31 PM UTC · 1908875254656868410#m
- 链接：https://twitter.com/BlockSecTeam/status/1908875254656868410#m
- 作者：@BlockSecTeam

> . @MochiDeFi was hacked with a loss around 50K in multiple transactions.
>  
> app.blocksec.com/explorer/tx…
>  
> Note that, Mochi was blocked by @CurveFinance  emergency DAO due to alleged attempted governance attack.
>  
> gov.curve.fi/t/the-curve-eme…

## 104. Apr 6, 2025 · 8:21 AM UTC · 1908797072771604648#m
- 链接：https://twitter.com/BlockSecTeam/status/1908797072771604648#m
- 作者：@BlockSecTeam

> We Do Everything To Ensure Your Transaction Security! 🛡️
> From shielding users against phishing, rug pulls, and sandwich attacks to protecting protocols from hacks, operational risks, compliance issues, and social engineering attacks targeting Safe{Wallet} signers.
>  
> Learn more at blocksec.com
> #Web3Festival

## 105. Apr 6, 2025 · 2:53 AM UTC · 1908714758511202507#m
- 链接：https://twitter.com/BlockSecTeam/status/1908714758511202507#m
- 作者：@BlockSecTeam

> 🚨 BlockSec at the 2025 Hong Kong Web3 Festival! 🚨
>  
> As a trusted guardian of Web3, BlockSec is excetied to participate in the 2025 Hong Kong Web3 Festival, the premier event driving the future of blockchain innovation! 🌐@festival_web3 @WXblockchain
> @HashKeyGroup
>  
> 📅 April 6 - 9, Hong Kong Convention Center. Here’s where you’ll find us:
>  
> 1️⃣ Keynote Speech:📅 April 6 | 14:30-14:45📍 Main Stage, Hall 3·Winter
>  
> BlockSec will reveal critical vulnerabilities in Web3 ecosystems and share actionable strategies to protect projects and users in this "dark forest."
>  
> 2️⃣Side Event - Stablecoins & Global Payments:📅 April 7 | 17:00-22:00
>  
> BlockSec will deliver an exclusive talk on Crypto Payment Security and the regulatory challenges facing the future of global payments.
>  
> 3️⃣ Web3 Scholars Conference📅 April 8 | 11:25-11:45📍Hall 5BCDE
>  
> Prof. Yajin will present BlockSec’s insights - System Research for Blockchain: “From Storage Optimization to Parallel Execution” pushing the boundaries of blockchain technology.
>  
> 🎭 Mystery COSER Alert!Look out for our “Security Guardian” COSER in black shades and trench coat, wielding the BlockSec Blade! Find them for interactive photos and exclusive BlockSec merch!
>  
> #BlockSec #Web3Festival #Web3Security

## 106. Apr 3, 2025 · 7:13 AM UTC · 1907692882510884998#m
- 链接：https://twitter.com/BlockSecTeam/status/1907692882510884998#m
- 作者：@BlockSecTeam

> Thrilled to share BlockSec's latest security research insights at the event! Honored by the invitation & can't wait to engage with the community.
>  
> See you there! 🔐 #Cybersecurity

## 107. Mar 25, 2025 · 6:50 PM UTC · 1904606890438648240#m
- 链接：https://twitter.com/BlockSecTeam/status/1904606890438648240#m
- 作者：@BlockSecTeam

> Our initial analysis suggests that the root cause may lie in liquidating without updating the collateral's value in the order contract of @MIM_Spell .
> Specifically, the inputAmount variable records an order's collateral value and is later used in the _isSolvent function (part of @GMX_IO ) to perform the insolvency check via the orderValueInCollateral function in the order contract. However, during liquidation in the sendValueInCollateral function, inputAmount is not properly decreased.
> As a result, the _isSolvent check can be bypassed, allowing an attacker to borrow additional assets after liquidation and profit from the exploit.

## 108. Mar 23, 2025 · 4:38 PM UTC · 1903848877100613858#m
- 链接：https://twitter.com/BlockSecTeam/status/1903848877100613858#m
- 作者：@BlockSecTeam

> 🫡

## 109. Mar 21, 2025 · 10:26 AM UTC · 1903030334897233966#m
- 链接：https://twitter.com/BlockSecTeam/status/1903030334897233966#m
- 作者：@BlockSecTeam

> . @zothdotio was hacked through contract upgrade. The loss is around 8M. It's probably the private key leakage. The fund is now in 0x7b0cd0d83565adbb57585d0265b7d15d6d9f60cf.
>  
> metasleuth.io/result/eth/0x3…

## 110. Mar 21, 2025 · 10:11 AM UTC · 1903026639522418810#m
- 链接：https://twitter.com/BlockSecTeam/status/1903026639522418810#m
- 作者：@BlockSecTeam

> Looks the contract was upgraded to unverified one, which was funded through changnow.
>  
> app.blocksec.com/explorer/tx…

## 111. Mar 20, 2025 · 4:55 AM UTC · 1902584640113717680#m
- 链接：https://twitter.com/BlockSecTeam/status/1902584640113717680#m
- 作者：@BlockSecTeam

> Upgrade MetaSuites to 5.7.0 to sync local labels between Etherscan (or other scans) to Phalcon Explorer.
>  
> chromewebstore.google.com/de…

## 112. Mar 19, 2025 · 8:53 AM UTC · 1902282316778496418#m
- 链接：https://twitter.com/BlockSecTeam/status/1902282316778496418#m
- 作者：@BlockSecTeam

> Sandwich attacks cost users money daily. We've deployed Anti-MEV RPC protection for BSC/Ethereum with a real-time monitoring dashboard (beta) to combat these attacks.
>  
> Secure your transactions now:
>  
> anti-mev.blocksec.com/
>  
> #DeFiSecurity #MEV

## 113. Mar 18, 2025 · 9:14 AM UTC · 1901925111386775662#m
- 链接：https://twitter.com/BlockSecTeam/status/1901925111386775662#m
- 作者：@BlockSecTeam

> RT @yajinzhou: The add liquidity tx is not using private rpc. Our system listened this tx (0x7f87a7cd02bbdab255c6245942502d958caabd7eea3da8…

## 114. Mar 15, 2025 · 7:22 AM UTC · 1900809936906711549#m
- 链接：https://twitter.com/BlockSecTeam/status/1900809936906711549#m
- 作者：@BlockSecTeam

> 1/ @WebKey01 WebKeyDao was attacked, resulting in a ~$73K loss. The attacker leveraged an unprotected function to buy wkeyDao tokens at a low price and sold them on DEX for profit.
>  
> app.blocksec.com/explorer/tx…
>  
> Since the vulnerable contract can no longer be exploited, we're releasing the full details to the community.

## 115. Mar 6, 2025 · 12:32 PM UTC · 1897626232860410308#m
- 链接：https://twitter.com/BlockSecTeam/status/1897626232860410308#m
- 作者：@BlockSecTeam

> Make Safe{Wallet} Safe Again!
>  
> blocksec.com/safe-wallet-mon…

## 116. Feb 26, 2025 · 11:33 AM UTC · 1894712439020966161#m
- 链接：https://twitter.com/BlockSecTeam/status/1894712439020966161#m
- 作者：@BlockSecTeam

> The @solana ecosystem is now under enhanced protection with BlockSec Phalcon.
> blocksec.com/blog/blocksec-p…
>  
> Protocols on Solana can now leverage Phalcon's on-chain risk detection capabilities to:
> 🔍 Monitor potential security threats in real-time
> 🚨 Receive instant alerts for suspicious activities
> 🛡️ Implement proactive measures to prevent losses
>  
> Additionally, Phalcon Explorer empowers Solana users with visualized insights into transaction details such as fund flows, balance changes, and function calls.
>  
> Learn more in the blog 👇
> #Solana #Web3Security

## 117. Feb 24, 2025 · 10:54 AM UTC · 1893977820893950151#m
- 链接：https://twitter.com/BlockSecTeam/status/1893977820893950151#m
- 作者：@BlockSecTeam

> 🚨 Twitter Space: Rethinking Multisig Security – Lessons from the Bybit Hack 🚨
>  
> Join Cobo’s CEO & Co-founder, Discus Fish (@bitfish1), alongside leading security experts as we analyze the recent Bybit security incident, explore key vulnerabilities, and discuss how institutions can strengthen their security frameworks.
>  
> This attack has raised critical questions about multisig security and risk management for exchanges—let’s break down what happened and what it means for the future of digital asset protection.
>  
> 📅 Feb 25 | 20:00 (UTC+8)
> 🎙 Speakers:
> 🔹 Discus Fish (@bitfish1) – Cobo CEO & Co-founder | @Cobo_Global
> 🔹 Yajin Zhou (@yajinzhou) – BlockSec CEO & Co-founder | @BlockSecTeam
> 🔹 Niqi (@niqislucky) – OneKey Chief Growth | @OneKeyHQ
> 🔹 Moon (@MoonL1ang) – Cobo Head of Blockchain Security
>  
> 🎤 Host: Wu Blockchain (@WutalkWu | @wublockchain12)
> 🎤 Co-host: Alex Zuo (@alexzuo4) – Cobo VP💡 Key Topics:
> 🔹 Understanding the attack: What happened and key takeaways
> 🔹 The evolving security needs of exchanges and custodians
> 🔹 Risks associated with transaction approvals and operational security
> 🔹 Best practices for securing digital assets in an institutional setting.
>  
> Join the discussion & stay ahead of emerging security risks: localhost:8080/i/spaces/1yNxaLjPyPNJj
>  
> Security is a continuous process — let’s discuss what’s next for safeguarding digital assets. Don’t miss it!

## 118. Feb 24, 2025 · 4:25 AM UTC · 1893879953118134589#m
- 链接：https://twitter.com/BlockSecTeam/status/1893879953118134589#m
- 作者：@BlockSecTeam

> . A sad day: an attack on an unverfied contract on Ethereum, around 40M loss.
>  
> etherscan.io/address/0xc49b5…

## 119. Feb 21, 2025 · 5:20 PM UTC · 1892987728452538643#m
- 链接：https://twitter.com/BlockSecTeam/status/1892987728452538643#m
- 作者：@BlockSecTeam

> The tx to update the master copy of the Safe wallet in @Bybit_Official incident.
>  
> app.blocksec.com/explorer/tx…

## 120. Feb 21, 2025 · 5:10 PM UTC · 1892985284482211961#m
- 链接：https://twitter.com/BlockSecTeam/status/1892985284482211961#m
- 作者：@BlockSecTeam

> The bybit @Bybit_Official  hacker is dispersing the Ether across multiple addresses.
>  
> metasleuth.io/result/eth/0x4…

## 121. Feb 21, 2025 · 2:44 PM UTC · 1892948641171542030#m
- 链接：https://twitter.com/BlockSecTeam/status/1892948641171542030#m
- 作者：@BlockSecTeam

> By switching your wallet’s RPC settings to BlockSec’s RPC endpoint (bsc.rpc.blocksec.com) , you protect yourself from front-runners and keep Four.meme @four_meme_  trading fair for everyone.
>  
> Dive into our blog for the full story!

## 122. Feb 21, 2025 · 2:43 PM UTC · 1892948346756575462#m
- 链接：https://twitter.com/BlockSecTeam/status/1892948346756575462#m
- 作者：@BlockSecTeam

> 4/ The key to avoiding these attacks is transaction privacy. BlockSec’s Anti-MEV RPC (bsc.rpc.blocksec.com) hides your pending trades until they’re confirmed, reducing the chance bots can slip in front of you.

## 123. Feb 21, 2025 · 2:43 PM UTC · 1892948344672006248#m
- 链接：https://twitter.com/BlockSecTeam/status/1892948344672006248#m
- 作者：@BlockSecTeam

> 3/ We’ve highlighted a BNX case where the attacker spotted the victim’s purchase, bought BNX at a lower price, and sold right after the victim’s transaction finalized—pocketing easy profits.

## 124. Feb 21, 2025 · 2:43 PM UTC · 1892948341899628775#m
- 链接：https://twitter.com/BlockSecTeam/status/1892948341899628775#m
- 作者：@BlockSecTeam

> 2/ In a sandwich attack, a malicious bot sees your pending trade, buys first to push up the price, and then sells immediately after you, leaving you paying a premium.

## 125. Feb 21, 2025 · 2:43 PM UTC · 1892948338175013342#m
- 链接：https://twitter.com/BlockSecTeam/status/1892948338175013342#m
- 作者：@BlockSecTeam

> 1/ Want a quick rundown on sandwich attacks plaguing Four.meme @four_meme_? Check out our latest blog for a real-world example of how attackers front-run trades —plus tips on how to protect yourself using BlockSec's Anti-MEV RPC.
>  
> blocksec.com/blog/how-to-pla…

## 126. Feb 21, 2025 · 5:52 AM UTC · 1892814617883161068#m
- 链接：https://twitter.com/BlockSecTeam/status/1892814617883161068#m
- 作者：@BlockSecTeam

> Use BlockSec Phalcon @Phalcon_xyz  to monitor hacks and take automatic actions. blocksec.com/phalcon

## 127. Feb 20, 2025 · 11:38 AM UTC · 1892539327441240224#m
- 链接：https://twitter.com/BlockSecTeam/status/1892539327441240224#m
- 作者：@BlockSecTeam

> In our latest blog, we dive deep into the recent @zkLend incident, offering a detailed security analysis and clearing up misunderstandings about the attack within the security community.
>  
> blocksec.com/blog/zklend-exp…

## 128. Feb 19, 2025 · 5:54 AM UTC · 1892090307707011157#m
- 链接：https://twitter.com/BlockSecTeam/status/1892090307707011157#m
- 作者：@BlockSecTeam

> Day 1 at Consensus Hong Kong 2025 was a blast! 🚀
>  
> BlockSec is here, making waves with cutting-edge blockchain security insights 🔐 and our amazing "Security Guardian" Coser😎.
>  
> Thank you to everyone who stopped by to chat, take photos, and explore the future of Web3 with us! 📸
>  
> #Consensus2025 #BlockSec #BlockchainSecurity #Web3

## 129. Feb 18, 2025 · 10:25 AM UTC · 1891796253425512819#m
- 链接：https://twitter.com/BlockSecTeam/status/1891796253425512819#m
- 作者：@BlockSecTeam

> In a word: Change your RPC to bsc.rpc.blocksec.com to avoid sandwich attacks and happily play @four_meme_! 🛡️🎮

## 130. Feb 18, 2025 · 10:23 AM UTC · 1891795783004951001#m
- 链接：https://twitter.com/BlockSecTeam/status/1891795783004951001#m
- 作者：@BlockSecTeam

> To play @four_meme_ and avoid being a victim of a sandwich attack, you can use BlockSec's anti-MEV RPC. Here's how you can set it up.
>  
> docs.blocksec.com/blocksec-a…

## 131. Feb 17, 2025 · 3:40 PM UTC · 1891513152136999180#m
- 链接：https://twitter.com/BlockSecTeam/status/1891513152136999180#m
- 作者：@BlockSecTeam

> Interested in how we approach protocol security?
>  
> Join us on February 18th at 2:30am UTC to welcome @Phalcon_xyz to our security lineup.
>  
> We'll cover what BlockSec Phalcon is capable of and how your assets on Yei are better secured with the @BlockSecTeam on our side.
>  
> Reminder below 👇

## 132. Feb 17, 2025 · 12:55 PM UTC · 1891471575620231360#m
- 链接：https://twitter.com/BlockSecTeam/status/1891471575620231360#m
- 作者：@BlockSecTeam

> 📍 Mark your calendars and find us at HKCEC — let’s unlock the ultimate Web3 defense strategies together! 💪
>  
> #Consensus2025 #BlockSec #BlockchainSecurity #Web3Gaming

## 133. Feb 17, 2025 · 12:54 PM UTC · 1891471224217141541#m
- 链接：https://twitter.com/BlockSecTeam/status/1891471224217141541#m
- 作者：@BlockSecTeam

> 🔥 Don’t miss our highlights:
>  
> 🤖 Meet our "Security Guardian" Coser for a fun photo op & exclusive BlockSec merch!
>  
> 🎤 Dive into the latest blockchain security insights & innovations with the BlockSec team.

## 134. Feb 17, 2025 · 12:53 PM UTC · 1891471020323713396#m
- 链接：https://twitter.com/BlockSecTeam/status/1891471020323713396#m
- 作者：@BlockSecTeam

> 🔐 BlockSec is your Web3 Guardian, and we’re coming to Consensus Hong Kong 2025!
>  
> From Feb 18-20, we’ll bring cutting-edge blockchain security solutions and insights to the biggest Web3 stage at HKCEC @consensus_hk @CoinDesk🇭🇰.

## 135. Feb 14, 2025 · 12:29 PM UTC · 1890377906980819365#m
- 链接：https://twitter.com/BlockSecTeam/status/1890377906980819365#m
- 作者：@BlockSecTeam

> Happy to work together with Unleash Protocol @UnleashProtocol on Story @StoryProtocol.

## 136. Feb 14, 2025 · 6:14 AM UTC · 1890283371281997874#m
- 链接：https://twitter.com/BlockSecTeam/status/1890283371281997874#m
- 作者：@BlockSecTeam

> 🚨 ALERT! 🚨 Our system has detected a series of suspicious transactions targeting several unknown contracts (likely MEV bots) across multiple chains, including #Ethereum, #BSC, #Avalanche, and #Base, resulting in a loss of approximately $170K. These transactions were initiated by the same address (0x6c88be3ff0a1c98cd3ddbad1203507d0bc2e17f1).
>  
> Our analysis indicates that due to inadequate access control, the attacker was able to force investments into fraudulent Uni-V3 pools for profit.
>  
> Transactions:
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…

## 137. Feb 12, 2025 · 2:38 PM UTC · 1889685529110749442#m
- 链接：https://twitter.com/BlockSecTeam/status/1889685529110749442#m
- 作者：@BlockSecTeam

> .@YeiFinance 🤝 BlockSec Phalcon
>  
> 24/7 attack monitoring ➕ millisecond automated blocking
> 🟰 Proactive defense 🟰 $140M+ assets secured 🛡️
>  
> Learn more in the blog👇: blocksec.com/blog/blocksec-p…
>  
> #DeFiSecurity #YeiFinance #Phalcon #Sei

## 138. Feb 8, 2025 · 9:53 AM UTC · 1888164245831131418#m
- 链接：https://twitter.com/BlockSecTeam/status/1888164245831131418#m
- 作者：@BlockSecTeam

> Subscribe to BlockSec Phalcon today to get alerted in real time and take automatic actions to protect your assets.  blocksec.com/phalcon

## 139. Feb 8, 2025 · 8:32 AM UTC · 1888143950990778773#m
- 链接：https://twitter.com/BlockSecTeam/status/1888143950990778773#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected several attack transactions targeting contracts on #BSC and #Arbitrum hours ago, resulting in losses of ~ $107K and $188K, respectively. Both incidents are price manipulation attacks: the first involves a price manipulation attack by forced investment, while the second exploits the lack of access control and slippage protection, enabling the attacker to forcibly burn victims' positions.
>  
> Attack TX on #BSC: app.blocksec.com/explorer/tx…
>  
> Attack TX on #Arbitrum:
> app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.  blocksec.com/phalcon

## 140. Feb 7, 2025 · 2:44 AM UTC · 1887693987839971583#m
- 链接：https://twitter.com/BlockSecTeam/status/1887693987839971583#m
- 作者：@BlockSecTeam

> ALERT! Our system detected a series of attacks targeting an unknown #XSD token on #ETH , #Base and #Optimism , resulting in ~$45k in losses.  It is a price manipulation attack that leverages the flawed 'swapXSDforETH' function through two key steps to manipulate the #XSD token's price: 1) exploiting a reentrancy vulnerability, and 2) burning tokens held by the pool.
>  
> Attack TX:
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.  blocksec.com/phalcon

## 141. Jan 24, 2025 · 3:23 AM UTC · 1882630151583981787#m
- 链接：https://twitter.com/BlockSecTeam/status/1882630151583981787#m
- 作者：@BlockSecTeam

> ALERT! Our system detected a series of attacks targeting the @odosprotocol protocol on #ETH #Base, resulting in ~$50k in losses.
>  
> The root cause is arbitrary call vulnerability caused by unverified user input. We notice that the attacker exploited the precompile contract (0x4) to bypass the signature verification. Protocols utilizing this method should exercise caution to mitigate similar risks.
>  
> Attack TX:
> app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.
>  
> blocksec.com/phalcon

## 142. Jan 17, 2025 · 9:12 AM UTC · 1880181450387124272#m
- 链接：https://twitter.com/BlockSecTeam/status/1880181450387124272#m
- 作者：@BlockSecTeam

> 🚀Enhancing On-Chain Security on Ethereum and BSC🚀
> Read the full analysis👉blocksec.com/blog/blog-4
>  
> Blocksec has partnered with leading BSC Builder service provider @BlockRazor_Inc to launch an RPC for MEV protection.
> Start Now👉:docs.blocksec.com/blocksec-a…
>  
> With 100% effectiveness against sandwich and front-running attacks, and 98% of transactions included in the next block, it offers a safer, more secure trading environment while protecting users from potential losses. 🛡️
>  
> #BSC #MEV #BlockchainSecurity

## 143. Jan 15, 2025 · 10:28 AM UTC · 1879475749138382896#m
- 链接：https://twitter.com/BlockSecTeam/status/1879475749138382896#m
- 作者：@BlockSecTeam

> Happy to collaborate to secure Web3 users. @MetaDockTeam @okxexplorer
>  
> blocksec.com/metasuites

## 144. Jan 15, 2025 · 3:27 AM UTC · 1879369751694291449#m
- 链接：https://twitter.com/BlockSecTeam/status/1879369751694291449#m
- 作者：@BlockSecTeam

> Subscribe to Phalcon to monitor and block hacks to protect users.
>  
> blocksec.com/phalcon

## 145. Jan 13, 2025 · 1:13 PM UTC · 1878792587500540041#m
- 链接：https://twitter.com/BlockSecTeam/status/1878792587500540041#m
- 作者：@BlockSecTeam

> #BTCFi projects can utilize BlockSec Phalcon to monitor the depegging and cross-chain risks of BTC-wrapped assets, and take automated actions to prevent them.
> blocksec.com/blog/btc-cross-…
>  
> Additionally, our address ownership verification API provides data for #PoR on third-party platforms like Chainlink. Learn more 👇

## 146. Jan 13, 2025 · 6:55 AM UTC · 1878697312731762811#m
- 链接：https://twitter.com/BlockSecTeam/status/1878697312731762811#m
- 作者：@BlockSecTeam

> ✅ Audit complete and mainnet underway!
>  
> Side Protocol has been thoroughly audited by the industry-leading security research firm @BlockSecTeam.
>  
> 📄 Read the full report here:
> blocksec.com/audit-report/se…

## 147. Jan 10, 2025 · 3:34 AM UTC · 1877559609776640019#m
- 链接：https://twitter.com/BlockSecTeam/status/1877559609776640019#m
- 作者：@BlockSecTeam

> . BunniHub of @alienbasedex on base was attacked with multiple transactions. Though the loss in each transaction is small, it still shows the vulnerabilities existing in the smart contract.
>  
> Subscribe to Phalcon to protect your protocol.
> blocksec.com/phalcon
>  
> Some attack tx:
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…
> app.blocksec.com/explorer/tx…

## 148. Jan 8, 2025 · 9:52 AM UTC · 1876929877489648046#m
- 链接：https://twitter.com/BlockSecTeam/status/1876929877489648046#m
- 作者：@BlockSecTeam

> Being aware of the critical operation of a DeFi protocol is necessary. That's why you must have Phalcon in hand to monitor the *Contract Upgrade*.
>  
> Book a demo with the BlockSec Phalcon team to learn how Phalcon can help mitigate the operation risks.
>  
> calendly.com/blocksec/phalco…

## 149. Dec 30, 2024 · 1:23 PM UTC · 1873721569627033684#m
- 链接：https://twitter.com/BlockSecTeam/status/1873721569627033684#m
- 作者：@BlockSecTeam

> We have released the data from our paper, 'Dissecting Payload-based Transaction Phishing on Ethereum,' which was accepted to NDSS 2025.
>  
> github.com/blocksecteam/PTXP…
>  
> Keep protecting users and make a safe Web3.

## 150. Dec 30, 2024 · 7:33 AM UTC · 1873633529109901810#m
- 链接：https://twitter.com/BlockSecTeam/status/1873633529109901810#m
- 作者：@BlockSecTeam

> RT @Phalcon_xyz: .@FEGtoken was attacked on #Ethereum, #BSC, and #Base last Sunday, resulting in losses exceeding $900K. As the core relaye…

## 151. Dec 26, 2024 · 11:00 AM UTC · 1872235999537066199#m
- 链接：https://twitter.com/BlockSecTeam/status/1872235999537066199#m
- 作者：@BlockSecTeam

> We keep building.

## 152. Dec 26, 2024 · 5:55 AM UTC · 1872159401051828341#m
- 链接：https://twitter.com/BlockSecTeam/status/1872159401051828341#m
- 作者：@BlockSecTeam

> 🔥 Explore BlockSec Phalcon's New Feature: TestKit
>  
> Threat monitoring platforms should be highly flexible, allowing users to tailor monitoring rules for complex scenarios. But how can users ensure their setup meets their expectations? Wait for a threat to strike? Absolutely not!
>  
> With TestKit, users can initiate simulated or historical transactions to test how the system would react. Will the monitor be triggered? Will you receive an alert? Will the response action (e.g., pause the protocol) be automatically initiated? Test your setup with TestKit and rest assured!
>  
> 👀 Learn more about BlockSec Phalcon at blocksec.com/phalcon
> 🤝 Book a demo with our security experts and apply for a free trial today at  calendly.com/blocksec/phalco…

## 153. Dec 24, 2024 · 2:40 PM UTC · 1871566724627390911#m
- 链接：https://twitter.com/BlockSecTeam/status/1871566724627390911#m
- 作者：@BlockSecTeam

> 🎄 Merry Christmas from the BlockSec! 🎄
>  
> This year, we’ve worked hard to protect what matters: your assets, your trust, and the promise of Web3. But none of it would have been possible without YOU—our community.
>  
> Let this season inspire us all to aim higher, dream bigger, and work harder for a safer, brighter Web3 future.
>  
> 🧑‍🎄Merry Christmas
> #MerryChristmas #Web3Security #BlockSec

## 154. Dec 23, 2024 · 5:21 PM UTC · 1871244717184450909#m
- 链接：https://twitter.com/BlockSecTeam/status/1871244717184450909#m
- 作者：@BlockSecTeam

> Merry Christmas, Sleuthers!
>  
> Now, the user manual is available in five more languages: Chinese, German, French, Japanese, and Russian
>  
> Happy sleuthing this holiday season!
>  
> docs.metasleuth.io/

## 155. Dec 21, 2024 · 3:52 PM UTC · 1870497636702409174#m
- 链接：https://twitter.com/BlockSecTeam/status/1870497636702409174#m
- 作者：@BlockSecTeam

> Glad we could assist @TheGemPad by providing real-time attack alerts and helping the team take emergency action to stop ongoing threats.
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 156. Dec 17, 2024 · 8:41 AM UTC · 1868939653929402822#m
- 链接：https://twitter.com/BlockSecTeam/status/1868939653929402822#m
- 作者：@BlockSecTeam

> Mark your calendars! Dec 18, 8 PM (UTC+8)
>  
> 🚀 BlockSec x OKX Chinese @okxchinese are bringing together TOP security experts and protocols to discuss how to safeguard your assets in the BULL MARKET! [Space is in Chinese]
>  
> Don’t miss out on this must-attend event!
>  
> #Blockchain#Security#Web3📷

## 157. Dec 17, 2024 · 8:38 AM UTC · 1868938902415065223#m
- 链接：https://twitter.com/BlockSecTeam/status/1868938902415065223#m
- 作者：@BlockSecTeam

> 🔗 Join us here
>  
> meeting.tencent.com/dm/FtWgh…

## 158. Dec 13, 2024 · 9:07 AM UTC · 1867496590388474307#m
- 链接：https://twitter.com/BlockSecTeam/status/1867496590388474307#m
- 作者：@BlockSecTeam

> 🚀 Exciting news! The MetaSleuth Team Plan, designed specifically for team collaboration, is now live.
>  
> Create your team to experience seamless investigative teamwork now!  metasleuth.io

## 159. Dec 10, 2024 · 9:33 AM UTC · 1866415952180502855#m
- 链接：https://twitter.com/BlockSecTeam/status/1866415952180502855#m
- 作者：@BlockSecTeam

> ALERT! Our system detected an attack on the unknown project named Rebalancer on #Base, resulting in ~133 Ether in losses. It is a typical reentrancy attack. Note that the attacker (0x012fc6) is not a new actor; it has been launching several other attacks for some time.
>  
> Attack TX: app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 160. Dec 10, 2024 · 8:00 AM UTC · 1866392519895847183#m
- 链接：https://twitter.com/BlockSecTeam/status/1866392519895847183#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected suspicious transactions that seem to be initiating governance attacks targeting an unknown protocol on #BSC. Losses exceed $640K. Please take action ASAP!
>  
> As there is no direct method available to contact the respective project, please reach out to us if you have any questions.
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 161. Dec 5, 2024 · 10:01 AM UTC · 1864611113092174226#m
- 链接：https://twitter.com/BlockSecTeam/status/1864611113092174226#m
- 作者：@BlockSecTeam

> Phalcon Explorer ✖️ Core 🎉
>  
> We are excited to announce that Phalcon Explorer now supports @Coredao_Org, allowing users to easily check fund flows & balance changes, and explore transaction traces with just a click.
>  
> 🔥Try it out here: app.blocksec.com/explorer/tx…
>  
> #Core #PhalconExplorer #DeFi #BlockSec

## 162. Dec 3, 2024 · 10:38 AM UTC · 1863895517086003545#m
- 链接：https://twitter.com/BlockSecTeam/status/1863895517086003545#m
- 作者：@BlockSecTeam

> #MetaSleuthTips Edge Drawing Feature
>  
> 🔸 Connect nodes on the canvas to reveal unique relationships, like cross-chain transfers (not supported by automatic tracing) and off-chain connections, and more.
> 🔸 Double-tap to improve clarity by adding notes.

## 163. Nov 28, 2024 · 12:43 PM UTC · 1862115173164798412#m
- 链接：https://twitter.com/BlockSecTeam/status/1862115173164798412#m
- 作者：@BlockSecTeam

> This Thanksgiving, we’re grateful for the trust our clients, partners, and community place in us to secure the blockchain ecosystem. 🎉
>  
> Happy Thanksgiving from all of us at BlockSec! 🧡💙
>  
> #BlockchainSecurity#Thanksgiving2024#BlockSec

## 164. Nov 27, 2024 · 10:04 AM UTC · 1861712730547462383#m
- 链接：https://twitter.com/BlockSecTeam/status/1861712730547462383#m
- 作者：@BlockSecTeam

> Equilibria ✖️ BlockSec Phalcon
>  
> BlockSec Phalcon is actively safeguarding @Equilibriafi by monitoring its mempool and on-chain transactions to detect hacks, operational, interaction and financial risks 24/7. When threats are detected, BlockSec Phalcon will automatically block them to prevent loss before they do any damage to Equilibria.
>  
> This proactive approach establishes robust defenses for Equilibria against evolving threats, securing its platform throughout its entire lifecycle.
>  
> #Equilibria #Phalcon #BlockSec #DeFiSecurity

## 165. Nov 26, 2024 · 11:45 AM UTC · 1861375788211413234#m
- 链接：https://twitter.com/BlockSecTeam/status/1861375788211413234#m
- 作者：@BlockSecTeam

> Thrilled to announce that BlockSec Phalcon now supports Optimism.
>  
> Protocols on @Optimism can now get access to BlockSec Phalcon to continuously monitor and automatically block hacks, operational, interaction, and financial risks, 24/7.
>  
> 👀Interested in experiencing our platform? Book a demo and gain access to a free trial: calendly.com/blocksec/phalco…
>  
> 😎Discover more about what BlockSec Phalcon offers: blocksec.com/phalcon

## 166. Nov 25, 2024 · 3:38 AM UTC · 1860890801909190664#m
- 链接：https://twitter.com/BlockSecTeam/status/1860890801909190664#m
- 作者：@BlockSecTeam

> ALERT! Several hours ago, our system detected an attack transaction targeting an unknown #DCT token on #BSC, leading to losses exceeding $428K. The attacker took advantage of the #DCF token's transfer mechanism, which enforces a forced investment logic. When #DCF tokens were sent to the USDT-DCF liquidity pool address, 5% of the tokens were automatically converted into #USDT and added to the USDT-DCT liquidity pool. This action triggered a swap in the USDT-DCT pool, allowing the attacker to manipulate the #DCT token's price and make profits accordingly.
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 167. Nov 22, 2024 · 3:04 AM UTC · 1859795155131433238#m
- 链接：https://twitter.com/BlockSecTeam/status/1859795155131433238#m
- 作者：@BlockSecTeam

> We’re thrilled to announce our partnership with @BlockSecTeam , a leading security and audit firm driving innovation in Web3 security, led by a team of industry experts.
>  
> BlockSec got involved with our project early on and played a crucial role in ensuring usdx.money is one of the most secure stablecoin protocols in the ecosystem. They’ve just completed a comprehensive audit, solidifying our commitment to transparency, security, and user trust.

## 168. Nov 20, 2024 · 12:53 PM UTC · 1859218441754083364#m
- 链接：https://twitter.com/BlockSecTeam/status/1859218441754083364#m
- 作者：@BlockSecTeam

> BlockSec Phalcon 🛡️ x Arbitrum 🚀
>  
> BlockSec Phalcon now supports @arbitrum , providing unbreakable post-launch security for the Arbitrum Ecosystem.
>  
> Protocols on Arbitrum can now get access to BlockSec Phalcon's robust capabilities to continuously monitor and automatically prevent hacks, operational, interaction, and financial risks, 24/7.
>  
> 👀Interested in experiencing our platform? Book a demo and gain access to a free trial: calendly.com/blocksec/phalco…
> 😎Discover more about what BlockSec Phalcon offers: blocksec.com/phalcon

## 169. Nov 20, 2024 · 11:36 AM UTC · 1859199044343455843#m
- 链接：https://twitter.com/BlockSecTeam/status/1859199044343455843#m
- 作者：@BlockSecTeam

> We launched our attack monitoring and blocking platform, BlockSec Phalcon, last year and are honored to have earned the trust of many esteemed clients, including @0xMantle, @IgnitionFBTC, @Bybit_Official, @MantaNetwork, and @puffer_finance, among others. We sincerely appreciate your trust! ❤️
>  
> We have been continuously refining our platform and, after months of dedicated effort, are excited to announce this major upgrade. The new version covers the vast majority of threats that protocols might face post-launch and greatly optimizes convenience and flexibility in configuration.
>  
> We invite protocols to book a demo with us and start a free trial today at calendly.com/blocksec/phalco….

## 170. Nov 19, 2024 · 12:00 PM UTC · 1858842844041286128#m
- 链接：https://twitter.com/BlockSecTeam/status/1858842844041286128#m
- 作者：@BlockSecTeam

> Phalcon Explorer Integrates Neo X Blockchain for Seamless Transaction Visualization and Analysis 🤝
>  
> Excited to announce that Phalcon Explorer, the supported tool of BlockSec Phalcon, now supports the Neo X chain. This enhancement allows users to easily track fund flows, check balance changes, and delve into transaction traces with just a click.
> 🚀Try it out here: app.blocksec.com/explorer/tx…
>  
> Looking ahead, we are going to deepen our collaboration with @Neo_Blockchain on BlockSec Phalcon to further strengthen the security of the Neo X ecosystem.
> 👀Learn about BlockSec Phalcon at blocksec.com/phalcon
>  
> #NeoX #Phalcon #BlockSec #DeFiSecurity

## 171. Nov 18, 2024 · 11:02 AM UTC · 1858465675762655395#m
- 链接：https://twitter.com/BlockSecTeam/status/1858465675762655395#m
- 作者：@BlockSecTeam

> This week, we’re uncovering how the #Restaking Aggregator was audited and exploring its technical details with the experts at @BlockSecTeam 🔍
>  
> Don’t miss this deep dive — set your reminder!
> localhost:8080/i/spaces/1yNxagkzwqgGj

## 172. Nov 18, 2024 · 9:05 AM UTC · 1858436357674381516#m
- 链接：https://twitter.com/BlockSecTeam/status/1858436357674381516#m
- 作者：@BlockSecTeam

> .@polterfinance on #Fantom suffered a hack, resulting in losses of approximately $8.7M. The incident appears to be a typical price manipulation attack caused by reliance on the spot price of the #BOO token.
>  
> The attacker exploited the calculation of the #BOO token price, which was based on token reserves in the WFTM-BOO liquidity pair. By draining #BOO token reserves through a flash loan, they artificially inflated the calculated price of #BOO. This allowed them to borrow tokens far exceeding the actual value of the collateral (#BOO), resulting in significant profit.

## 173. Nov 18, 2024 · 8:56 AM UTC · 1858434172852822227#m
- 链接：https://twitter.com/BlockSecTeam/status/1858434172852822227#m
- 作者：@BlockSecTeam

> By integrating our key products—including BlockSec's @MetaSleuth, Phalcon Explorer @Phalcon_xyz, and MetaSuites @MetaDockTeam — we aim to provide blockchain developers and users with more comprehensive and secure analysis tools and data services.

## 174. Nov 18, 2024 · 8:56 AM UTC · 1858434170466243035#m
- 链接：https://twitter.com/BlockSecTeam/status/1858434170466243035#m
- 作者：@BlockSecTeam

> 🚀BlockSec and OKX Explorer @OKXExplorer
> have launched a comprehensive partnership focusing on blockchain data interoperability, product integration, and API enhancements.
>  
> blocksec.com/blog/block-sec-…

## 175. Nov 14, 2024 · 10:13 AM UTC · 1857003795420512668#m
- 链接：https://twitter.com/BlockSecTeam/status/1857003795420512668#m
- 作者：@BlockSecTeam

> On October 23, the $SHAR token skyrocketed to $60M in market cap before a rug pull crashed its price from $0.05986 to $0.0013.
>  
> 👉Discover the full story behind this dramatic rise and fall here: metasleuth.io/blog/pump-dump…
>  
> 🕵️Simplify and enhance your on-chain investigations with #MetaSleuth

## 176. Nov 16, 2024 · 7:24 AM UTC · 1857686200414253560#m
- 链接：https://twitter.com/BlockSecTeam/status/1857686200414253560#m
- 作者：@BlockSecTeam

> Dexx @DEXXai_EN was compromised, possibly due to private key leakage. This raised the security concerns of centralized store of users private key in trading bots. A new security architecture of trading bot is needed in #Solana ecosystem, with a real self-custody mechanism.

## 177. Nov 15, 2024 · 1:54 PM UTC · 1857421903004328204#m
- 链接：https://twitter.com/BlockSecTeam/status/1857421903004328204#m
- 作者：@BlockSecTeam

> Enhanced security measures attract more users, increase TVL, and drive ecosystem prosperity, promoting wider adoption. YESSS! Security is CRUCIAL for the growth of L2!
>  
> As a security company, BlockSec offers an effective solution for L2 networks with the Sequencer Threat Overwatch Program (S.T.O.P). Leveraging BlockSec Phalcon's @Phalcon_xyz excellent threat detection capabilities, S.T.O.P brings innovative security solutions to L2.
>  
> Find out more here🔗
>  
> blocksec.com/blog/revolution…
>  
> #BlockchainSecurity #L2Innovation #DevCon2024
>  
> ￼
> ￼

## 178. Nov 15, 2024 · 9:35 AM UTC · 1857356767115477004#m
- 链接：https://twitter.com/BlockSecTeam/status/1857356767115477004#m
- 作者：@BlockSecTeam

> Our investigation on meme coin SHAR Pump and Dump on #Solana
>  
> metasleuth.io/blog/pump-dump…

## 179. Nov 14, 2024 · 10:08 AM UTC · 1857002671124730233#m
- 链接：https://twitter.com/BlockSecTeam/status/1857002671124730233#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected several attacks targeting the #vETH token on #Ethereum, resulting in approximately $450K in losses. This appears to be a price manipulation attack, likely due to reliance on the spot price within the designated factory contract (0x62f2, which is not open-source).
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 180. Nov 13, 2024 · 2:00 AM UTC · 1856517504358895639#m
- 链接：https://twitter.com/BlockSecTeam/status/1856517504358895639#m
- 作者：@BlockSecTeam

> #USDa is secured by top industry experts: @salus_sec  , @BlockSecTeam, @SlowMist_Team, and @fuzzland_ , and has successfully passed their audits. Your assets are protected, and so are we.
>  
> With a zero-tolerance policy for vulnerabilities, #USDa’s smart contracts have been rigorously tested by leading security firms.
>  
> To ensure continuous protection, we partner with @fuzzland_,  a 24/7 on-chain security guard, keeping the Avalon ecosystem and its users safe at all times.

## 181. Nov 11, 2024 · 2:22 AM UTC · 1855798237770170415#m
- 链接：https://twitter.com/BlockSecTeam/status/1855798237770170415#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected an attack targeting the #BGM token on #BSC, resulting in losses exceeding $450K. This is a typical price manipulation attack exploiting reliance on the spot price. Do NOT add liquidity to the pool (0xadc4ec) anymore.
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon
>  
> Attack  TX: app.blocksec.com/explorer/tx…

## 182. Nov 8, 2024 · 9:28 AM UTC · 1854818315845484954#m
- 链接：https://twitter.com/BlockSecTeam/status/1854818315845484954#m
- 作者：@BlockSecTeam

> The new User Manual for #MetaSleuth is now live! 🎉
>  
> 👉 Explore it here: docs.metasleuth.io/user-manu…
>  
> We’ve added sections on "Start by a Simple Search" and "Start by a Shared Link" to help first-time users get up to speed quickly. Dive into our comprehensive resource to maximize your blockchain analysis experience!
>  
> 🕵️Simplify and enhance your on-chain investigations with #MetaSleuth

## 183. Nov 6, 2024 · 12:15 PM UTC · 1854135505602195568#m
- 链接：https://twitter.com/BlockSecTeam/status/1854135505602195568#m
- 作者：@BlockSecTeam

> On August 20, 2024, a phishing attack drained over $54M in DAI from a Gemini-funded vault, and we've just released a detailed analysis of this incident.
>  
> 👉 Check out the full report here: metasleuth.io/blog/illicit-f…
>  
> 🔍 We used #MetaSleuth to trace the stolen funds and uncover the full impact of this phishing scheme.
>  
> 🕵️ Simplify and enhance your on-chain investigations with #MetaSleuth!

## 184. Nov 1, 2024 · 11:47 AM UTC · 1852316637904728372#m
- 链接：https://twitter.com/BlockSecTeam/status/1852316637904728372#m
- 作者：@BlockSecTeam

> Discover the top 4 security incidents in October and stay informed! 🛡️👇
>  
> blocksec.com/blog/monthly-se… #Web3Security

## 185. Oct 31, 2024 · 6:40 AM UTC · 1851876822906212427#m
- 链接：https://twitter.com/BlockSecTeam/status/1851876822906212427#m
- 作者：@BlockSecTeam

> #MetaSleuthTips Cross-Chain Automatic Tracking✍️
>  
> Let's take the address 0x50275...836, marked as KyberSwap Exploiter 1 in MetaSleuth.
> 🔸Cross-Chain Transfers: This address transfers ETH to OP and ARB via OptimismGateway and ArbitrumBridge
> 🔸Visualize Bridges: Cross-chain bridges are displayed in a unique shape on MetaSleuth
> 🔸Track Funds: Click the bridge, then in the InterChain Tracker panel, click Track to automatically follow the fund flows
>  
> 🔍 From the results, we see ETH transferred to OP(3ETH) and ARB(2ETH)
>  
> 🕵️Simplify and enhance your on-chain investigations with #MetaSleuth

## 186. Oct 29, 2024 · 8:43 AM UTC · 1851183171033055473#m
- 链接：https://twitter.com/BlockSecTeam/status/1851183171033055473#m
- 作者：@BlockSecTeam

> #MetaSleuthTips Node Batch Editing is here ✍️
>  
> Simply hold SHIFT + COMMAND to select a group, or use SHIFT to add more nodes.
> Now you can easily move nodes and customize their colors in bulk!
>  
> Simplify and enhance your on-chain investigations with MetaSleuth🕵️

## 187. Oct 25, 2024 · 2:17 AM UTC · 1849636437349527725#m
- 链接：https://twitter.com/BlockSecTeam/status/1849636437349527725#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected a suspicious transaction targeting an unknown project on #Base, resulting in a loss of approximately $1M. The affected project appears to be a #Compound fork, with multiple markets being drained. As the contracts are not open-source, we suspect this may be a classic price manipulation attack caused by reliance on Uniswap's spot price.
>  
> Attack TX: app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 188. Oct 24, 2024 · 8:23 AM UTC · 1849366073579041015#m
- 链接：https://twitter.com/BlockSecTeam/status/1849366073579041015#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected attack transactions targeting @RamsesExchange's contract on #Arbitrum, resulting in a loss of  ~$93K. We have contacted the team, and they have informed us that actions have already been taken.
>  
> The root cause appears to be an unverified input in the getPeriodReward function, where the current timestamp is not properly checked against the specified 'period', allowing the attacker to drain the vulnerable contract.
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 189. Oct 24, 2024 · 11:17 AM UTC · 1849409939007275052#m
- 链接：https://twitter.com/BlockSecTeam/status/1849409939007275052#m
- 作者：@BlockSecTeam

> We've just released a detailed analysis of the @lifiprotocol LiFi Attack, where a vulnerability in the GasZipFacet contract resulted in significant losses.
>  
> 👉 Check out the full report here: metasleuth.io/blog/illicit-f…
> 📊 Explore the MetaSleuth Chart here: metasleuth.io/result/eth/0x1…
>  
> 🔍 We used #MetaSleuth to trace the stolen funds and reveal the full impact of the exploit.
>  
> 🕵️ Simplify and enhance your on-chain investigations with #MetaSleuth

## 190. Oct 17, 2024 · 12:12 PM UTC · 1846887112009744398#m
- 链接：https://twitter.com/BlockSecTeam/status/1846887112009744398#m
- 作者：@BlockSecTeam

> thanks. the repo is public now :)

## 191. Oct 17, 2024 · 11:22 AM UTC · 1846874306745380930#m
- 链接：https://twitter.com/BlockSecTeam/status/1846874306745380930#m
- 作者：@BlockSecTeam

> We have released our dataset on Web3 phishing website detection, containing 26,333 phishing URLs. Feel free to use the dataset for further research and development of better anti-phishing solutions.
>  
> github.com/blocksecteam/TxPh…

## 192. Oct 17, 2024 · 2:20 AM UTC · 1846738049075761561#m
- 链接：https://twitter.com/BlockSecTeam/status/1846738049075761561#m
- 作者：@BlockSecTeam

> 🚀 Excited to announce that BlockSec CEO Prof. Yajin Zhou @yajinzhou spoke at #MEVShanghai, unveiling the innovative Layer 2 security solution: Sequencer Threat Overwatch Program (S.T.O.P) 🛡️.
>  
> Curious to learn more? Check it out! 🔥blocksec.com/blog/revolution…
>  
> Since 2022, BlockSec Phalcon @Phalcon_xyz has safeguarded $20M+ assets through our advanced attack detection and transaction frontrunning. 🔒💪S.T.O.P. utilizes Phalcon's advanced attack detection technology to provide robust security for Layer 2 networks with cutting-edge performance.
>  
> #Blockchain #Layer2 #MEVShanghai

## 193. Oct 8, 2024 · 12:16 PM UTC · 1843626451301773765#m
- 链接：https://twitter.com/BlockSecTeam/status/1843626451301773765#m
- 作者：@BlockSecTeam

> 🚨 On [2024-10-07 11:26], @VitalikButerin  transferred 73 $ETH to #Kanro (Vitalik Charity)! 🚀
>  
> Start tracking today: metasleuth.io/result/eth/0xd…
>  
> 🔍 Simplify your on-chain investigations with #MetaSleuth. Monitor Vitalik's wallet activities and stay up-to-date with just one click! 💻

## 194. Oct 2, 2024 · 2:07 PM UTC · 1841480250942947603#m
- 链接：https://twitter.com/BlockSecTeam/status/1841480250942947603#m
- 作者：@BlockSecTeam

> Thanks for mentioning Phalcon explorer @Phalcon_xyz

## 195. Oct 2, 2024 · 1:23 AM UTC · 1841287935532072997#m
- 链接：https://twitter.com/BlockSecTeam/status/1841287935532072997#m
- 作者：@BlockSecTeam

> My notes for @danielvf interview on @CyfrinAudits
>  
> - Best tools for tracing:
> - Phalcon Explorer app.blocksec.com/explorer/ (by @BlockSecTeam)
> - Tenderly tenderly.co/ (by @TenderlyApp)
> - OpenChain openchain.xyz/trace (by @samczsun)
> - Dedaub for contract decompilation, currently the best app.dedaub.com (by @dedaub)
>  
> - Phalcon usage tips:
> - Disable static calls (reduces noise)
> - Mark contracts with colors
> - Focus on "Gas Used" in Phalcon explorer to identify key calls (higher gas consumption often indicates importance)
> - Asset names are displayed by default instead of just addresses, which makes the trace more readable
>  
> - First 2-3 attempts at tracing a hack may be challenging, but significant improvement occurs afterward (up to 50% faster)
>  
> piped.kavin.rocks/live/PIGEhaMyg1g

## 196. Sep 26, 2024 · 2:41 PM UTC · 1839314277359849484#m
- 链接：https://twitter.com/BlockSecTeam/status/1839314277359849484#m
- 作者：@BlockSecTeam

> .@OnyxDAO was attacked, resulting in a loss of nearly $4M. The root cause was unverified user input during the liquidation process. Specifically, key parameters of the liquidateWithSingleRepay function in the NFTLiquidation contract were controllable by the attacker, allowing manipulation of the extraRepayAmount variable through the repayAmount parameter. By exploiting this, the attacker was able to liquidate all collateral with just one token.
>  
> The key attack steps are summarized as follows:
> 1. The attacker first deposited oETH and borrowed various assets to reach the liquidation threshold. Simultaneously, they created a new contract that, through a donation attack and precision loss (inherent from the Compound V2 fork), reduced the oETH exchange rate, making the attacker's position eligible for liquidation.
> 2. The attacker then performed the liquidation. Due to insufficient parameter validation, the attacker manipulated the extraRepayAmount variable, which was added to the calculation of how many tokens needed to be liquidated. This allowed the attacker to obtain more oETH through liquidation, leading to a profit.
>  
> Attack Tx: app.blocksec.com/explorer/tx…

## 197. Sep 25, 2024 · 4:19 AM UTC · 1838795326112592289#m
- 链接：https://twitter.com/BlockSecTeam/status/1838795326112592289#m
- 作者：@BlockSecTeam

> See the detailed fund flow of address poisoning on BTC network using @MetaSleuth

## 198. Sep 25, 2024 · 3:24 AM UTC · 1838781531868819790#m
- 链接：https://twitter.com/BlockSecTeam/status/1838781531868819790#m
- 作者：@BlockSecTeam

> ALERT! Our system has detected hundreds of suspicious transactions targeting an unknown, non-open-sourced contract on #BSC (0xff2481) over the past few hours, suggesting a possible reentrancy attack. The total loss has reached ~$140K.
>  
> Interestingly, after the first attack transaction (with a profit of ~$78K), the deployer (0x7baa94) invoked the victim contract's 'emergencyWithdrawUSDT' function multiple times, each for a small amount rather than withdrawing all the funds at once. This allowed the attacker to make small, repeated profits, ultimately accumulating to $140K.
>  
> First attack TX: app.blocksec.com/explorer/tx…
>  
> Subscribe to BlockSec Phalcon today to get alerted in realtime and take automatic actions to protect your assets.   blocksec.com/phalcon

## 199. Sep 24, 2024 · 8:54 AM UTC · 1838502376979738707#m
- 链接：https://twitter.com/BlockSecTeam/status/1838502376979738707#m
- 作者：@BlockSecTeam

> address 1: phishing address
> bc1q7jywd9nfjg36skxt4lc0twvgzc3rkjj6lyfjwm
>  
> address 2:
> bc1q7jgulg69frc8zuzy0ng8d5208kae7t0twyfjwm
>  
> address 3:
> bc1q6c3c0t3zvnphce37ufr4yz9veaqvew2wg0shwr

## 200. Sep 24, 2024 · 8:54 AM UTC · 1838502373011943911#m
- 链接：https://twitter.com/BlockSecTeam/status/1838502373011943911#m
- 作者：@BlockSecTeam

> Address poisoning is on BTC now. The following is one concrete case. The phishing address (address 1) is disguising address 2 to send a small amount of BTC to address 3. Since addresses 2 and 3 have historic transactions, the attacker hopes to trick the owner into copying the wrong address.
