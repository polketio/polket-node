// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};

use crate::{
	mock::*, BuybackMode, BuybackPlans, Error, ParticipantInfo, ParticipantRegistrations, PlanInfo,
	PlanStatus, TotalPlansCount,
};

fn create_test_plan(creator: AccountId, amount: u64, mode: BuybackMode, start: u64) {
	assert_ok!(Buyback::create_plan(
		Origin::signed(creator),
		1,
		0,
		200,
		amount,
		100,
		start,
		10,
		mode,
	));
}

#[test]
fn create_plan_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				1,
				2,
				200,
				0,
				100,
				5,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::ValueInvalid
		);

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				1,
				2,
				200,
				10000,
				100,
				5,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::InsufficientBalance
		);

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				2,
				0,
				200,
				10000,
				100,
				5,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::AssetUnavailable
		);

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				1,
				0,
				200,
				10000,
				100,
				1,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::PlanStartGreaterThanCurrent
		);

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				1,
				0,
				200,
				10000000,
				100,
				5,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::InsufficientBalance
		);

		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);

		System::assert_has_event(Event::Buyback(crate::Event::PlanCreated {
			plan_id: 1,
			plan_info: PlanInfo {
				buy_asset_id: 0,
				sell_asset_id: 1,
				creator: ALICE,
				min_sell: 200,
				seller_amount: 0,
				seller_limit: 100,
				total_buy: 10000,
				total_sell: 0,
				start: 5,
				period: 10,
				status: PlanStatus::Upcoming,
				mode: BuybackMode::Burn,
			},
		}));

		let creator_balance = Currencies::balance(0, &creator);
		// println!("creator_balance = {}", creator_balance);
		assert_eq!(creator_balance, 1000000 - 10000);
		let plan_account_balance = Currencies::balance(0, &Buyback::into_account_id(1));
		// println!("plan_account_balance = {}", plan_account_balance);
		assert_eq!(plan_account_balance, 10000);
		assert_eq!(TotalPlansCount::<Test>::get(), 1);

		for _ in 0..19 {
			create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);
		}

		assert_eq!(TotalPlansCount::<Test>::get(), 20);

		assert_noop!(
			Buyback::create_plan(
				Origin::signed(creator.clone()),
				1,
				0,
				200,
				10000,
				100,
				5,
				10,
				BuybackMode::Burn
			),
			Error::<Test>::TotalPlansReachedMax
		);
	});
}

#[test]
fn cancel_plan_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;

		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);

		assert_noop!(
			Buyback::cancel_plan(Origin::signed(creator.clone()), 2),
			Error::<Test>::BuybackPlanNotExisted
		);

		assert_noop!(
			Buyback::cancel_plan(Origin::signed(BOB), 1),
			Error::<Test>::OperationIsNotAllowed
		);

		assert_ok!(Buyback::cancel_plan(Origin::signed(creator.clone()), 1));

		System::assert_has_event(Event::Buyback(crate::Event::PlanCanceled { plan_id: 1 }));

		// plan is not existed
		assert!(!BuybackPlans::<Test>::contains_key(1));
		assert_eq!(TotalPlansCount::<Test>::get(), 0);

		// assets return creator
		let creator_balance = Currencies::balance(0, &creator);
		assert_eq!(creator_balance, 1000000);
		let plan_account_balance = Currencies::balance(0, &Buyback::into_account_id(1));
		assert_eq!(plan_account_balance, 0);

		// create a new plan and start it.
		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);
		run_to_block(6);

		let plan = BuybackPlans::<Test>::get(2).expect("plan is not existed");
		assert_eq!(plan.status, PlanStatus::InProgress);
		assert_noop!(
			Buyback::cancel_plan(Origin::signed(creator.clone()), 2),
			Error::<Test>::OperationIsNotAllowed
		);
		assert_eq!(TotalPlansCount::<Test>::get(), 1);
	});
}

#[test]
fn seller_register_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		let seller = BOB;
		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);

		assert_noop!(
			Buyback::seller_register(Origin::signed(seller.clone()), 2, 400),
			Error::<Test>::BuybackPlanNotExisted
		);

		assert_noop!(
			Buyback::seller_register(Origin::signed(seller.clone()), 1, 400),
			Error::<Test>::OperationIsNotAllowed
		);

		run_to_block(6);

		assert_noop!(
			Buyback::seller_register(Origin::signed(seller.clone()), 1, 20),
			Error::<Test>::LockedAmountLessThanMin
		);

		System::assert_has_event(Event::Buyback(crate::Event::PlanStarted { plan_id: 1 }));

		assert_noop!(
			Buyback::seller_register(Origin::signed(seller.clone()), 1, 900000000),
			pallet_assets::Error::<Test>::BalanceLow
		);

		assert_ok!(Buyback::seller_register(Origin::signed(seller.clone()), 1, 400));
		System::assert_has_event(Event::Buyback(crate::Event::SellerRegistered {
			plan_id: 1,
			who: seller.clone(),
			locked: 400,
		}));

		// check asset balance
		let seller_balance = Currencies::balance(1, &seller);
		assert_eq!(seller_balance, 1000000 - 400);
		let plan_account_balance = Currencies::balance(1, &Buyback::into_account_id(1));
		assert_eq!(plan_account_balance, 400);
		let participant = ParticipantRegistrations::<Test>::get(1, &seller);
		assert_eq!(participant.locked, 400);

		assert_ok!(Buyback::seller_register(Origin::signed(seller.clone()), 1, 400));

		// check asset balance
		let seller_balance = Currencies::balance(1, &seller);
		assert_eq!(seller_balance, 1000000 - 800);
		let plan_account_balance = Currencies::balance(1, &Buyback::into_account_id(1));
		assert_eq!(plan_account_balance, 800);
		let participant = ParticipantRegistrations::<Test>::get(1, &seller);
		assert_eq!(participant.locked, 800);
	});
}

#[test]
fn payback_and_burn_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);

		run_to_block(6);

		System::assert_has_event(Event::Buyback(crate::Event::PlanStarted { plan_id: 1 }));

		let bob_locked = 400u64;
		let charlie_locked = 500u64;
		let dave_locked = 600u64;

		assert_ok!(Buyback::seller_register(Origin::signed(BOB), 1, bob_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(CHARLIE), 1, charlie_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(DAVE), 1, dave_locked));

		assert_noop!(
			Buyback::withdraw(Origin::signed(BOB), BOB, 2),
			Error::<Test>::BuybackPlanNotExisted
		);
		assert_noop!(
			Buyback::withdraw(Origin::signed(BOB), BOB, 1),
			Error::<Test>::OperationIsNotAllowed
		);

		run_to_block(16);

		System::assert_has_event(Event::Buyback(crate::Event::PlanCompleted { plan_id: 1 }));
		System::assert_has_event(Event::Buyback(crate::Event::AllPaybacked { plan_id: 1 }));

		// system handle payback, users no longer need to withdraw
		assert_noop!(
			Buyback::withdraw(Origin::signed(BOB), BOB, 1),
			Error::<Test>::OperationIsNotAllowed
		);
		assert_noop!(
			Buyback::withdraw(Origin::signed(CHARLIE), CHARLIE, 1),
			Error::<Test>::OperationIsNotAllowed
		);
		assert_noop!(
			Buyback::withdraw(Origin::signed(CHARLIE), CHARLIE, 1),
			Error::<Test>::OperationIsNotAllowed
		);

		// assert_ok!(Buyback::withdraw(Origin::signed(BOB), BOB, 1));
		// assert_ok!(Buyback::withdraw(Origin::signed(CHARLIE), CHARLIE, 1));
		// assert_ok!(Buyback::withdraw(Origin::signed(CHARLIE), CHARLIE, 1));

		let bob_expect_rewards = 2666u64;
		let charlie_expect_rewards = 3333u64;
		let dave_expect_rewards = 4000u64;

		System::assert_has_event(Event::Buyback(crate::Event::Withdrew {
			who: BOB,
			plan_id: 1,
			rewards: bob_expect_rewards,
		}));
		System::assert_has_event(Event::Buyback(crate::Event::Withdrew {
			who: CHARLIE,
			plan_id: 1,
			rewards: charlie_expect_rewards,
		}));
		System::assert_has_event(Event::Buyback(crate::Event::Withdrew {
			who: DAVE,
			plan_id: 1,
			rewards: dave_expect_rewards,
		}));

		let bob_rewards = Currencies::balance(0, &BOB);
		let charlie_rewards = Currencies::balance(0, &CHARLIE);
		let dave_rewards = Currencies::balance(0, &DAVE);
		let plan_account_rewards = Currencies::balance(0, &Buyback::into_account_id(1));
		let plan_account_locked = Currencies::balance(1, &Buyback::into_account_id(1));

		assert_eq!(bob_rewards, bob_expect_rewards);
		assert_eq!(charlie_rewards, charlie_expect_rewards);
		assert_eq!(dave_rewards, dave_expect_rewards);
		assert_eq!(plan_account_rewards, 0);
		assert_eq!(plan_account_locked, 0); //Burned

		assert_eq!(
			ParticipantRegistrations::<Test>::get(1, BOB),
			ParticipantInfo { locked: bob_locked, rewards: bob_expect_rewards, withdrew: true }
		);
		assert_eq!(
			ParticipantRegistrations::<Test>::get(1, CHARLIE),
			ParticipantInfo {
				locked: charlie_locked,
				rewards: charlie_expect_rewards,
				withdrew: true
			}
		);
		assert_eq!(
			ParticipantRegistrations::<Test>::get(1, DAVE),
			ParticipantInfo { locked: dave_locked, rewards: dave_expect_rewards, withdrew: true }
		);

		System::assert_has_event(Event::Assets(pallet_assets::Event::Burned {
			asset_id: 1,
			owner: Buyback::into_account_id(1),
			balance: bob_locked + charlie_locked + dave_locked,
		}));
	});
}

#[test]
fn payback_and_transfer_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		create_test_plan(creator.clone(), 10000, BuybackMode::Transfer, 5);

		run_to_block(6);

		let bob_locked = 400u64;
		let charlie_locked = 500u64;
		let dave_locked = 600u64;

		assert_ok!(Buyback::seller_register(Origin::signed(BOB), 1, bob_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(CHARLIE), 1, charlie_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(DAVE), 1, dave_locked));

		run_to_block(16);

		let creator_returned = Currencies::balance(1, &creator);
		let plan_account_rewards = Currencies::balance(0, &Buyback::into_account_id(1));
		let plan_account_locked = Currencies::balance(1, &Buyback::into_account_id(1));

		assert_eq!(creator_returned, bob_locked + charlie_locked + dave_locked);
		assert_eq!(plan_account_rewards, 0);
		assert_eq!(plan_account_locked, 0); //transferred

		System::assert_has_event(Event::Assets(pallet_assets::Event::Transferred {
			asset_id: 1,
			from: Buyback::into_account_id(1),
			to: creator,
			amount: bob_locked + charlie_locked + dave_locked,
		}));
	});
}

#[test]
fn payback_no_participant_should_work() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, 5);
		let creator_remaining_amount = Currencies::balance(0, &creator);
		// 1000000 - 10000 = 990000
		assert_eq!(creator_remaining_amount, 990000);

		run_to_block(16);

		// let plan = BuybackPlans::<Test>::get(1);
		// println!("plan = {:?}", plan);
		System::assert_has_event(Event::Buyback(crate::Event::PlanStarted { plan_id: 1 }));
		System::assert_has_event(Event::Buyback(crate::Event::PlanCompleted { plan_id: 1 }));
		System::assert_has_event(Event::Buyback(crate::Event::AllPaybacked { plan_id: 1 }));

		let creator_remaining_amount = Currencies::balance(0, &creator);
		// 990000 + 10000 = 1000000
		assert_eq!(creator_remaining_amount, 1000000);
	});
}

#[test]
fn buyback_plan_clear_unit_test() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		// create 19 plans
		let start = 5u64;
		for i in 0u64..19 {
			create_test_plan(creator.clone(), 10000, BuybackMode::Burn, start + i);
		}

		run_to_block(6);

		let bob_locked = 400u64;
		let charlie_locked = 500u64;
		let dave_locked = 600u64;

		assert_ok!(Buyback::seller_register(Origin::signed(BOB), 1, bob_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(CHARLIE), 1, charlie_locked));
		assert_ok!(Buyback::seller_register(Origin::signed(DAVE), 1, dave_locked));

		run_to_block(22);

		// create 20th plan reach max
		create_test_plan(creator.clone(), 10000, BuybackMode::Burn, start + 20);
		assert_eq!(TotalPlansCount::<Test>::get(), 20);

		run_to_block(23);
		// 7 plans cleared
		for i in 1u32..=7 {
			System::assert_has_event(Event::Buyback(crate::Event::PlanStarted { plan_id: i }));
			System::assert_has_event(Event::Buyback(crate::Event::PlanCompleted { plan_id: i }));
			System::assert_has_event(Event::Buyback(crate::Event::AllPaybacked { plan_id: i }));
			System::assert_has_event(Event::Buyback(crate::Event::PlanCleared { plan_id: i }));
		}

		assert!(!BuybackPlans::<Test>::contains_key(1));
		assert_eq!(ParticipantRegistrations::<Test>::get(1, BOB), ParticipantInfo::default());
	});
}
