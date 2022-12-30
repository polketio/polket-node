# pallet-currencies

## Overview

The currencies module provides a mixed currencies system, by configuring a `NativeCurrency` which implements `frame_support::fungible::{Inspect, Mutate, Transfer}`, and `MultiCurrency` which implements `frame_support::fungibles::{Inspect, Mutate, Transfer}`.

At `runtime`, we use `pallet-balances` and `pallat-assets` to configure `NativeCurrency` and `MultiCurrency`.

## License

SPDX-License-Identifier: GPL-3.0-or-later