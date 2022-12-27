# pallet-vfe

## 概述

本模块是基于substrate实现健身与GameFi结合的去中心化应用。用户根据运动方式购买链上已注册的硬件器材，激活设备后，能在链上将获得一个`虚拟健身器材`，每日使用硬件器材训练同时还赚取代币奖励。

## 业务流程

1. 首先在`Currencies`模块，使用`CreateOrigin`创建激励资产，例如：`FUN`。
1. 在`VFE`模块，使用`Root Origin`调用`set_incentive_token`，选择已存在的AssetId。
1. 在`VFE`模块，使用`BrandOrigin`调用`create_vfe_brand`，创建新的`VFE CollectionId`。
1. 在`VFE`模块，使用`ProducerOrigin`调用`producer_register`，为外部账户创建一个`Producer`。
1. 在`VFE`模块，使用`BrandOrigin`调用`approve_mint`，授权`Producer`为指定的`VFE CollectionId`可以铸造`itemId`，可选`mint_cost`。
1. 每个`Jump Rope`器材，在出售前都会创建唯一的`secp256r1 keypair`，`PrivateKey`存储在芯片中，不暴露。而`PublicKey`，在`VFE`模块，`Producer`调用`register_device`存储到`Device`表，并从`Producer`外部账户中转账一笔`mint_cost`到`ProducerId`账户。
1. 用户购买了`Jump Rope`器材，通过App绑定器材，读取器材的`PublicKey`，在`VFE`模块，调用`bind_device`，链上将铸造一个新的`VFE Item`，并绑定器材的`PublicKey`。每激活一个`itemId`，`Producer`将会支付部分金额给`VFE Brand`拥有者和用户。`bind_device`是一个`unsigned transaction`，所以用户不需要支付手续费。
1. 用户每日使用`Jump Rope`器材训练，在`VFE`模块，调用`upload_training_report`，链上验证数据签名，解析训练报告，转为激励代币`FUN`奖励给用户，同时每次训练都会消耗`VFE Item`的battery和用户的每日能量。
1. 用户在`VFE`模块，调用`restore_power`，消耗`FUN`来充电`VFE Item`。
1. 在`VFE`模块，每过`EnergyRecoveryDuration`都会更新全网的`LastEnergyRecovery`，所有用户的`last_restore_block`小于`LastEnergyRecovery`的话，都可以调用`user_restore`来恢复每日能量。
1. 用户有足够的`FUN`，在`VFE`模块，调用`level_up`来升级`VFE Item`。每升1级都可以获得新的能点数，通过调用`increase_ability`来为`VFE Item`增加能力值，促使`VFE Item`能赚更多的`FUN`。
1. 用户在`VFE`模块，调用`unbind_device`解除`VFE Item`与`Device`的绑定，可选择其他`VFE Item`重新调用`bind_device`绑定`Device`。

## pallet

### Config

- `BrandOrigin`: The origin which who can create collection of VFE.
- `ProducerOrigin`: The origin which may add or remove producer.
- `Currencies`: Multiple Asset hander, which should implement `frame_support::traits::fungibles`.
- `ObjectId`: Unify the value types of ProudcerId, CollectionId, ItemId, AssetId.
- `UniqueId`: UniqueId is used to generate new CollectionId or ItemId.
- `PalletId`: The pallet id.
- `Randomness`: Used to randomly generate VFE base ability value.
- `UniquesInstance`: pallet-uniques instance which used to VFE data management.
- `ProducerId`: The producer-id parent key.
- `VFEBrandId`: The vfe brand-id parent key.
- `UnbindFee`: Fees for unbinding VFE.
- `CostUnit`: Units of Incentive Tokens Rewarded or Costed.
- `EnergyRecoveryDuration`: How long to restore an energy value.
- `DailyEarnedResetDuration`: How long to reset user daily earned value.
- `LevelUpCostFactor`: Level up cost factor.
- `InitEnergy`: Init energy when new user created.
- `InitEarningCap`: Init earning cap of daily when new user created.
- `EnergyRecoveryRatio`: Ratio of each energy recovery.
- `UnixTime`: Used to get real world time.
- `ReportValidityPeriod`: How long is the training report valid, unit: seconds.
- `UserVFEMintedProfitRatio`: Profit ratio of minting fee to VFE owner.

## Core gameplay

**运动类型**

系统将支持3种运动类型: `JumpRope`, `Running`, `Riding`。目前正在开发的是`JumpRope`。

**VFE稀有度**

VFE目前拥有4中稀有度，不同稀有度属性值不同。

- 普通。
- 精英。
- 稀有。
- 史诗。

**初始属性点数**

VFE的能力由效率、技巧、运气、耐久四项属性组成，根据稀有度初始点数范围不同。

| Quality | Min. Attribute | Max. Attribute |
|---------|----------------|----------------|
| 普通    | 2              | 8              |
| 精英    | 6              | 12             |
| 稀有    | 10             | 18             |
| 史诗    | 20             | 30             |

**升级成长点数**

VFE每升1级，获得成长点数，由用户自由配置到效率、技巧、运气、耐久四项属性。

| Quality | Growth Point |
|---------|--------------|
| 普通    | 4            |
| 精英    | 4            |
| 稀有    | 4            |
| 史诗    | 4            |

### 计算参数

| Parameter  | Type | Description                                               |
|------------|------|-----------------------------------------------------------|
| Lv         | u16  | 等级数                                                    |
| R          | u64  | 链上随机数                                                |
| E          | u16  | 当前效率值                                                |
| S          | u16  | 当前技巧值                                                |
| L          | u16  | 当前运气值                                                |
| D          | u16  | 当前耐久值                                                |
| $E_{base}$ | u16  | 初始效率值                                                |
| $S_{base}$ | u16  | 初始技巧值                                                |
| $L_{base}$ | u16  | 初始运气值                                                |
| $D_{base}$ | u16  | 初始耐久值                                                |
| $J_{max}$  | u16  | 最多连跳数                                                |
| $J_{avg}$  | u16  | 每分钟平均跳数                                            |
| G          | u16  | 升级后，获得的成长点数，参考**升级成长点数**                |
| T          | u16  | 绊绳次数                                                  |
| N          | u16  | 能量值，`Jump Rope`每运动满30秒消耗1点能量，少于30秒不消耗。 |
| K          | u16  | 考核频次常量，系统值=120次/分。                             |
| FUN        | u128 | 奖励代币，小数精度为12。1 FUN=$10^12$                       |

**计算随机运气分数**

$L_{Rnd} = R \pmod L + 1$

通过链上随机数计算实际运气分数。

**计算本次训练的技巧分数**

$S' = \lfloor \frac {S*J_{max}}{(T+1)*K} \rfloor $

$\Delta S = R \pmod {\vert S - S'\vert}$
$
S''=
\begin{cases}
S-\Delta S, & S' \lt S \\
S+\Delta S, & S' \ge S
\end{cases}
$

一次有效训练中，最大连跳数和绊绳次数是影响技巧得分的关键。
训练者对跳绳技巧的掌握程度，会给技巧分数带来倍增倍减的效果。

**计算有效频次因素**

$
F=
\begin{cases}
0, & J_{avg} \notin [80, 400] \\
1, & J_{avg} \in [80, 400]
\end{cases}
$

有效频次因素：范围在80次/分钟 ~ 400次/分钟内，值为1，否则为0。

**FUN奖励基数**

$$B = \frac {FUN} {10}$$

**FUN奖励公式**

$$FUN_{reward} = (E+S''+ 2*L_{rnd})*N*F*B$$

**能量**

初始每日能量上限8点，每运动30秒消耗1能量点，所以初始每日最多运动4分钟。
VFE每升2级上限增加4点能量，最大能量上限为68点。
每6小时恢复能量上限的25%。

能量每日上限公式：

$$N_{cap} = \lfloor\frac {Lv} 2 \rfloor*4+8$$

**电量**

电量范围在0%~100%，每消耗一个能量，扣1%电量，当电量为0%时，VFE将损坏无法充电修复。
VFE耐久度和等级影响充电成本，消耗每点能量的充电FUN成本公式如下：

$$FUN_{cost} = (\frac {E_{base}+S_{base}+L_{base}+D_{base}} 2+(\frac {E+S+L+D} {4*D})^2*Lv)*N*B$$

**VFE 升级成本**

VFE升级需要消耗FUN，按此公式计算升级成本：

$$LevelCost(Lv+1)=T_{cost}*N_{cap}*B*[Lv*(G-1)+\frac {E_{base}+S_{base}+L_{base}-D_{base}} 2]$$

- $T_{cost}$为常数，系统值为7
- $G$是VFE成长点数，根据稀有度不同而值不同，数值参考**升级成长属性点数**。

**VFE升级时间(Under Development)**

VFE升级消耗时间，以分钟为单位，公式如下：
$$Time(Lv)=Lv*60$$

若要即时完成升级，则根据剩余时间计算FUN成本，公式如下：

$$f(x)=\frac {LevelCost} {Time} * x$$

**奖励兑换率(Under Development)**

奖励兑换率受等级影响，由系统调整。