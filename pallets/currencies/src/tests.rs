// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{mock::*, Config};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, traits::tokens::fungibles::Inspect};
use sp_core::Hasher;
use sp_runtime::traits::BlakeTwo256;

#[test]
fn compile_success() {
	new_test_ext().execute_with(|| {});
}

#[test]
fn fungibles_inspect_unit_test() {
	new_test_ext().execute_with(|| {
		assert_eq!(Currencies::balance(NativeToken::get(), &ALICE), 10000000000);
		assert_eq!(Currencies::balance(1, &ALICE), 0);
		assert_eq!(Currencies::balance(1, &BOB), 1000);
		assert_eq!(Currencies::total_issuance(NativeToken::get()), 10000000000);
		assert_eq!(Currencies::total_issuance(1), 1000);
		assert_eq!(Currencies::minimum_balance(NativeToken::get()), 10);
		assert_eq!(Currencies::minimum_balance(1), 1);
	});
}

#[test]
fn transfer_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(Currencies::transfer(Origin::signed(ALICE), BOB, NativeToken::get(), 100, true));
		assert_eq!(Currencies::balance(NativeToken::get(), &BOB), 100);
		assert_ok!(Currencies::transfer_native(Origin::signed(ALICE), BOB, 200, true));
		assert_eq!(Currencies::balance(NativeToken::get(), &BOB), 300);

		assert_noop!(
			Currencies::force_transfer(Origin::signed(BOB), ALICE, BOB, NativeToken::get(), 500),
			BadOrigin
		);
		assert_ok!(Currencies::force_transfer(Origin::root(), ALICE, BOB, NativeToken::get(), 500));
		assert_eq!(Currencies::balance(NativeToken::get(), &BOB), 800);

		assert_ok!(Currencies::transfer(Origin::signed(BOB), ALICE, 1, 100, true));
		assert_eq!(Currencies::balance(1, &ALICE), 100);
	});
}

#[test]
fn create_asset_should_work() {
	new_test_ext().execute_with(|| {
		let asset_id_key = BlakeTwo256::hash(b"assetidkey");
		assert_eq!(asset_id_key, <Test as Config>::AssetId::get());
		// println!("asset_id_key: {}", hex::encode(asset_id_key));
		// println!("AssetId: {}", hex::encode(<Test as Config>::AssetId::get()));
		assert_ok!(Currencies::create(
			Origin::signed(ALICE),
			ALICE,
			99,
			"Polkadot".into(),
			"DOT".into(),
			10
		));
		assert_eq!(Assets::minimum_balance(2), 99);
		let next_id = UniqueId::next_object_id(asset_id_key);
		assert_eq!(next_id, 3);
	});
}
