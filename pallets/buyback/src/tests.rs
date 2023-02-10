// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use frame_support::assert_ok;

use crate::{mock::*, BuybackMode, Config};

#[test]
fn compile_success() {
	new_test_ext().execute_with(|| {});
}

#[test]
fn create_plan_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		assert_ok!(Buyback::create_plan(
			Origin::signed(creator.clone()),
			1,
			0,
			200,
			100000,
			1,
			10,
			BuybackMode::Burn
		));
	});
}
