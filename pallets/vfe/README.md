# pallet-vfe

## Overview

This module is a decentralized application that combines `fitness` and `GameFi` based on substrate. Users purchase registered fitness equipment on chain according to their exercise methods. After binding the equipment, they will get a `virtual fitness equipment`(`VFE`) on chain. They can use the fitness equipment to daily train and earn token rewards.

## Business Process

1. First, in the `Currencies` module, use `CreateOrigin` to create incentive assets, for example: `FUN`.
1. In the `VFE` module, use `Root Origin` to call `set_incentive_token`, and select the existing `AssetId` as the training incentive reward.
1. In the `VFE` module, use `BrandOrigin` to call `create_vfe_brand` to create a new `VFE CollectionId`.
1. In the `VFE` module, use `ProducerOrigin` to call `producer_register` to create a `Producer` for the external account.
1. In `VFE` module, use `BrandOrigin` to call `approve_mint`, authorize `Producer` to cast `itemId` for the specified `VFE CollectionId`, optional `mint_cost`.
1. Each `Jump Rope` device will create a unique `secp256r1 keypair` before it is sold, and the `PrivateKey` is stored in the chip and is not exposed. For `PublicKey`, in the `VFE` module, `Producer` calls `register_device` to store in the `Device` table, and transfers a `mint_cost` from the `Producer` external account to the `ProducerId` account.
1. The user purchases a `Jump Rope` device, binds the device through the App, reads the `PublicKey` of the device, calls `bind_device` in the `VFE` module, and a new `VFE Item` will be cast on the chain and bind the device The `PublicKey`. Every time an `itemId` is activated, `Producer` will pay part of the amount to `VFE Brand` owners and users. `bind_device` is an `unsigned transaction`, so the user does not need to pay the transaction fee.
1. The user uses the `Jump Rope` equipment to train every day. In the `VFE` module, call `upload_training_report`, verify the data signature on the chain, analyze the training report, and convert it into an incentive token `FUN` to reward the user. At the same time, each training will consume The battery of `VFE Item` and the user's daily energy.
1. In the `VFE` module, the user calls `restore_power` to consume `FUN` to charge `VFE Item`.
1. In the `VFE` module, the `LastEnergyRecovery` of the network will be updated every `EnergyRecoveryDuration`, and if the `last_restore_block` of all users is less than `LastEnergyRecovery`, `user_restore` can be called to restore daily energy.
1. The user has enough `FUN`, in the `VFE` module, call `level_up` to upgrade `VFE Item`. Every time you level up, you can get new energy points. By calling `increase_ability`, you can increase the ability value of `VFE Item`, so that `VFE Item` can earn more `FUN`.
1. In the `VFE` module, the user calls `unbind_device` to unbind `VFE Item` and `Device`, and can select other `VFE Item` to call `bind_device` to bind `Device` again.

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

> VFE is short for "virtual fitness equipment".

**Sport Type**

The system will support 3 sports types: `JumpRope`, `Running`, `Riding`. Currently under development is `JumpRope`.

**VFE Rarity**

VFE currently has 4 rarities, and the attribute values of different rarities are different.

- Common.
- Elite.
- Rare.
- Epic.

**Initial attribute points**

The ability of VFE consists of four attributes: `Efficiency`, `Skill`, `Luck`, and `Durability`, and the range of initial points varies according to the rarity.

| Quality | Min. Attribute | Max. Attribute |
|---------|----------------|----------------|
| Common  | 2              | 8              |
| Elite   | 6              | 12             |
| Rare    | 10             | 18             |
| Epic    | 20             | 30             |

**Upgrade Growth Points**

Every time a level of VFE is upgraded, growth points are obtained, which can be freely configured by the user to four attributes: `Efficiency`, `Skill`, `Luck`, and `Durability`.

| Quality | Growth Point |
|---------|--------------|
| Common  | 4            |
| Elite   | 4            |
| Rare    | 4            |
| Epic    | 4            |

### Calculation Parameters

| Parameter  | Type | Description                                                                                                                               |
|------------|------|-------------------------------------------------------------------------------------------------------------------------------------------|
| Lv         | u16  | Level                                                                                                                                     |
| R          | u64  | On-chain Random Seed                                                                                                                      |
| E          | u16  | Current efficiency value                                                                                                                  |
| S          | u16  | Current skill value                                                                                                                       |
| L          | u16  | Current luck value                                                                                                                        |
| D          | u16  | Current durability value                                                                                                                  |
| $E_{base}$ | u16  | Basic efficiency value                                                                                                                    |
| $S_{base}$ | u16  | Basic skill value                                                                                                                         |
| $L_{base}$ | u16  | Basic luck value                                                                                                                          |
| $D_{base}$ | u16  | Basic durability value                                                                                                                    |
| $J_{max}$  | u16  | Maximum number of jumps                                                                                                                   |
| $J_{avg}$  | u16  | Average jumps per minute                                                                                                                  |
| G          | u16  | After upgrading, the growth points obtained, refer to **Upgrade Growth Points**                                                           |
| T          | u16  | Trip rope times                                                                                                                           |
| N          | u16  | Energy value, `Jump Rope` consumes 1 point of energy for every 30 seconds of exercise, and no energy is consumed for less than 30 seconds |
| K          | u16  | Assessment frequency constant, system value = 120 times/minute.                                                                           |
| FUN        | u128 | Reward tokens with a decimal precision of 12. 1 FUN=10^12 unit                                                                            |

**Calculate random luck score**

$$L_{Rnd} = R \pmod L + 1$$

The actual luck score is calculated via on-chain random number.

**Compute the skill score for this training**

$$S' = \lfloor \frac {S*J_{max}}{(T+1)*K} \rfloor $$

$$\Delta S = R \pmod {\vert S - S'\vert}$$

$$
S''=
\begin{cases}
S-\Delta S, & S' \lt S \\
S+\Delta S, & S' \ge S
\end{cases}
$$

In a training session, the maximum number of consecutive jumps and the number of trip ropes are the key factors affecting skill scores.
The trainer's mastery of jumping rope skills will double the skill scores.

**Calculate Jump Frequency Factor**

$$
F=
\begin{cases}
0, & J_{avg} \notin [80, 400] \\
1, & J_{avg} \in [80, 400]
\end{cases}
$$

Valid frequency factor: the value is 1 within the range of 80 times/minute to 400 times/minute, otherwise it is 0.

**FUN reward base**

$$B = \frac {FUN} {10}$$

**FUN reward formula**

$$FUN_{reward} = (E+S'' + 2 * L_{rnd} ) * N * F * B$$

**Energy**

The initial daily energy limit is 8 points, and every 30 seconds of training consumes 1 energy point, so the initial maximum daily exercise is 4 minutes.
The upper limit of VFE increases by 4 points for every 2 levels, and the maximum energy limit is 68 points.
Restores 25% of maximum energy every 6 hours.

Energy Daily Limit Formula：

$$N_{cap} = \lfloor\frac {Lv} 2 \rfloor*4+8$$

**Battery**

The power range is from 0% to 100%. For every energy consumed, 1% of the power will be deducted. When the power is 0%, the VFE will be damaged and cannot be repaired by charging.
The durability and level of VFE affect the charging cost, and the charging `FUN` cost formula for each point of energy consumed is as follows:

$$FUN_{cost} = (\frac {E_{base}+S_{base}+L_{base}+D_{base}} 2+(\frac {E+S+L+D} {4*D})^2*Lv)*N*B$$

**VFE upgrade cost**

VFE upgrade needs to consume `FUN`, and the upgrade cost is calculated according to this formula:

$$LevelCost(Lv+1)=T_{cost}*N_{cap}*B*[Lv*(G-1)+\frac {E_{base}+S_{base}+L_{base}-D_{base}} 2]$$

- $T_{cost}$is a constant, the system value is 7.
- $G$ is the VFE growth point, the value is different according to the `Rarity`, the value refers to **Upgrade Growth Attribute Points**.

**VFE upgrade time(Under Development)**

VFE upgrade consumption time, in minutes, the formula is as follows:
$$Time(Lv)=Lv*60$$

To complete the upgrade immediately, calculate the `FUN` cost based on the remaining time, the formula is as follows:

$$f(x)=\frac {LevelCost} {Time} * x$$


## License

SPDX-License-Identifier: GPL-3.0-or-later