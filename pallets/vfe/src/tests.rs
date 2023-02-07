// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::{
	mock::{Event, *},
	Call,
};
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use p256::{
	ecdsa::{
		signature::{Signature as Sig, Signer, Verifier},
		Signature, SigningKey, VerifyingKey,
	},
	elliptic_curve::{sec1::ToEncodedPoint, PublicKey},
	NistP256,
};
use rand_core::OsRng;
use sha2::Digest;
use sp_core::crypto::Ss58Codec;
use sp_std::convert::TryInto;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

// generate device keypair for testing
fn generate_device_keypair() -> (SigningKey, DeviceKey) {
	let signing_key = SigningKey::random(&mut OsRng);
	let verifying_key = signing_key.verifying_key(); // Serialize with `::to_encoded_point()`
	let public_key: PublicKey<NistP256> = verifying_key.into();
	// let encoded_point = publickey.to_encoded_point(true);
	return (
		signing_key,
		public_key.to_encoded_point(true).as_bytes().try_into().expect("error length"),
	)
}

// produce a device and bind a vfe
fn produce_device_bind_vfe(
	producer: AccountId,
	user: AccountId,
	pub_key: DeviceKey,
	key: SigningKey,
) {
	//set incentive token
	assert_ok!(VFE::set_incentive_token(Origin::root(), 1));
	// register producer
	assert_ok!(VFE::producer_register(Origin::root(), producer.clone()));
	// create vfe brand
	assert_ok!(VFE::create_vfe_brand(
		Origin::signed(CANDY),
		bvec![0u8; 20],
		SportType::JumpRope,
		VFERarity::Common
	));
	assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, Some((0, 10))));
	// register device
	assert_ok!(VFE::register_device(Origin::signed(producer), pub_key, 1, 1));
	let account_nonce = 1u32;
	let account_rip160 = Ripemd::Hash::hash(user.encode().as_ref());
	let mut msg: Vec<u8> = Vec::new();
	msg.extend(account_nonce.to_le_bytes().to_vec());
	msg.extend(account_rip160.to_vec());

	let signature = key.sign(msg.as_ref());

	assert_ok!(VFE::bind_device(
		Origin::none(),
		user.clone(),
		pub_key,
		signature.to_vec().try_into().unwrap(),
		account_nonce,
		None
	));
}

#[test]
fn set_incentive_token_unit_test() {
	new_test_ext().execute_with(|| {
		// wrong origin.
		assert_noop!(VFE::set_incentive_token(Origin::signed(BOB), 1), DispatchError::BadOrigin);
		assert_ok!(VFE::set_incentive_token(Origin::root(), 1));
		System::assert_has_event(Event::VFE(crate::Event::IncentiveTokenSet { asset_id: 1 }));
		let incentive_token = VFE::incentive_token().expect("incentive token not set");
		assert_eq!(incentive_token, 1);
	});
}

#[test]
fn producer_register_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::producer_register(Origin::root(), ALICE));
		System::assert_has_event(Event::VFE(crate::Event::ProducerRegister {
			who: ALICE,
			producer_id: 1,
		}));
		// wrong origin role.
		assert_noop!(VFE::producer_register(Origin::signed(BOB), BOB), DispatchError::BadOrigin);
	});
}

#[test]
fn producer_owner_change_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::producer_register(Origin::root(), ALICE));
		assert_ok!(VFE::producer_owner_change(Origin::signed(ALICE), 1, TOM));
		System::assert_has_event(Event::VFE(crate::Event::ProducerOwnerChanged {
			old_owner: ALICE,
			producer_id: 1,
			new_owner: TOM,
		}));

		// wrong origin role.
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(BOB), 1, ALICE),
			Error::<Test>::OperationIsNotAllowed
		);

		// Operation is not allowed for producer.
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(ALICE), 1, TOM),
			Error::<Test>::OperationIsNotAllowed
		);

		// producer is not exist
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(TOM), 2, ALICE),
			Error::<Test>::ProducerNotExist
		);

		assert_ok!(VFE::producer_owner_change(Origin::signed(TOM), 1, BOB));
		System::assert_has_event(Event::VFE(crate::Event::ProducerOwnerChanged {
			old_owner: TOM,
			producer_id: 1,
			new_owner: BOB,
		}));

		let producer = Producers::<Test>::get(1).expect("can not find producer");
		assert_eq!(producer.id, 1);
		assert_eq!(producer.owner, BOB);
	});
}

#[test]
fn create_vfe_brand_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::create_vfe_brand(
			Origin::signed(CANDY),
			bvec![0u8; 20],
			SportType::JumpRope,
			VFERarity::Common
		));

		System::assert_has_event(Event::VFE(crate::Event::VFEBrandCreated {
			who: CANDY,
			brand_id: 1,
			sport_type: SportType::JumpRope,
			rarity: VFERarity::Common,
			note: bvec![0u8; 20],
		}));
		let vfe_brand = VFE::vfe_brands(1).expect("can not find vfe brand");
		assert_eq!(vfe_brand.sport_type, SportType::JumpRope);
		assert_eq!(vfe_brand.rarity, VFERarity::Common);

		let vfe_brand_owner = VFEUniques::collection_owner(1).expect("can not find collection");
		assert_eq!(vfe_brand_owner, CANDY);
	});
}

#[test]
fn approve_mint_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::producer_register(Origin::root(), ALICE));
		assert_ok!(VFE::create_vfe_brand(
			Origin::signed(CANDY),
			bvec![0u8; 20],
			SportType::JumpRope,
			VFERarity::Common
		));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, None));
		System::assert_has_event(Event::VFE(crate::Event::ApprovedMint {
			brand_id: 1,
			product_id: 1,
			mint_amount: 10,
			mint_cost: None,
		}));
		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 10);
		assert_eq!(approve.mint_cost, None);

		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 12, None));
		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 22);

		assert_noop!(
			VFE::approve_mint(Origin::signed(CANDY), 1, 1, 12, Some((1, 10))),
			Error::<Test>::RemainingMintAmountIsNotZero
		);
	});
}

#[test]
fn register_device_unit_test() {
	new_test_ext().execute_with(|| {
		// register producer
		assert_ok!(VFE::producer_register(Origin::root(), ALICE));

		// create vfe brand
		assert_ok!(VFE::create_vfe_brand(
			Origin::signed(CANDY),
			bvec![0u8; 20],
			SportType::JumpRope,
			VFERarity::Common
		));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, Some((0, 10))));

		// register device
		// let bytes =
		// 	hex::decode("02e3a9257cf457087eeef75f466d3da31318b046ffcce05d104a0505d9799b47c6")
		// 		.unwrap();
		let bytes = hex!["02e3a9257cf457087eeef75f466d3da31318b046ffcce05d104a0505d9799b47c6"];
		let puk: DeviceKey = DeviceKey::from_raw(bytes);

		assert_ok!(VFE::register_device(Origin::signed(ALICE), puk, 1, 1));
		System::assert_has_event(Event::VFE(crate::Event::DeviceRegistered {
			operator: ALICE,
			producer_id: 1,
			device_key: puk,
			brand_id: 1,
		}));
		let device = Devices::<Test>::get(puk).expect("device is nil");
		assert_eq!(device.brand_id, 1);
		assert_eq!(device.item_id, None);
		assert_eq!(device.producer_id, 1);
		assert_eq!(device.sport_type, SportType::JumpRope);
		assert_eq!(device.status, DeviceStatus::Registered);
		assert_eq!(device.pk, puk);
		assert_eq!(device.nonce, 0);
		assert_eq!(device.mint_cost, Some((0, 10)));

		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 9);
		assert_eq!(approve.registered, 1);
		assert_eq!(approve.locked_of_mint, 10);

		let producer_balance = <Currencies as MultiAssets<AccountId>>::balance(
			0,
			&crate::Pallet::<Test>::into_account_id(1),
		);
		// println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 10);

		//deregister device
		assert_ok!(VFE::deregister_device(Origin::signed(ALICE), puk));
		System::assert_has_event(Event::VFE(crate::Event::DeviceDeregistered {
			operator: ALICE,
			device_key: puk,
		}));

		assert_eq!(Devices::<Test>::get(puk), None);

		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 10);
		assert_eq!(approve.registered, 0);
		assert_eq!(approve.locked_of_mint, 0);

		let producer_balance = <Currencies as MultiAssets<AccountId>>::balance(
			0,
			&crate::Pallet::<Test>::into_account_id(1),
		);
		println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 0);
	});
}

#[test]
fn bind_device_should_work() {
	new_test_ext().execute_with(|| {

		// device keypair
		// let bytes = hex::decode("0339d3e6e837d675ce77e85d708caf89ddcdbf53c8e510775c9cb9ec06282475a0").unwrap();
		// let puk: DeviceKey = bytes.try_into().expect("error length");
		let bytes = hex!["0339d3e6e837d675ce77e85d708caf89ddcdbf53c8e510775c9cb9ec06282475a0"];
		let puk: DeviceKey = DeviceKey::from_raw(bytes);

		// register producer
		assert_ok!(VFE::producer_register(Origin::root(), ALICE));
		// create vfe brand
		assert_ok!(VFE::create_vfe_brand(Origin::signed(CANDY), bvec![0u8; 20], SportType::JumpRope, VFERarity::Common));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, Some((0,100))));
		// register device
		assert_ok!(VFE::register_device(Origin::signed(ALICE), puk, 1, 1));
		let producer_balance = <Currencies as MultiAssets<AccountId>>::balance(0, &crate::Pallet::<Test>::into_account_id(1));
		println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 100);

		let user = DANY;
		let account_nonce = 123u32;
		// let account_rip160 = Ripemd::Hash::hash(user.encode().as_ref());
		// println!("account_nonce = {}", hex::encode(account_nonce.to_le_bytes()));
		// println!("account_hex = {:?}", hex::encode(user.encode()));
		// println!("account_rip160 = {:?}", account_rip160);

		let signature = hex::decode("df6e11efe387bec44bc15c3c636dfa51a951a1cda1a96d1d1b32566de948cda6125e873bac098688b4991512ca1dfa68a26862c97b81ad0555a06f1423874d66").unwrap();

		assert_ok!(VFE::bind_device(
			Origin::none(),
			user.clone(),
			puk,
			signature.try_into().unwrap(),
			account_nonce,
			None
		));
		System::assert_has_event(Event::VFE(crate::Event::DeviceBound{owner:user.clone(), device_key:puk, brand_id:1, item_id:1}));
		// assert!(VFEBindDevices::<Test>::contains_key(1, 1));
		let device = Devices::<Test>::get(puk).expect("can not find device");
		assert!(device.item_id.is_some());
		let vfe = VFEDetails::<Test>::get(1, 1).expect("VFEDetail not exist");
		assert_eq!(vfe.device_key, Some(device.pk));
		assert_eq!(vfe.level, 0);

		let user_info = Users::<Test>::get(&user).unwrap();
		assert_eq!(user_info.energy_total, 8);
		assert_eq!(user_info.energy, 8);
		assert_eq!(user_info.earning_cap, 500 * 100000);
		assert_eq!(user_info.earned, 0);

		//check minting fee distribute
		let producer_balance = <Currencies as MultiAssets<AccountId>>::balance(
			0,
			&crate::Pallet::<Test>::into_account_id(1),
		);
		assert_eq!(producer_balance, 0);
		let user_balance = <Currencies as MultiAssets<AccountId>>::balance(
			0,
			&user,
		);
		assert_eq!(user_balance, 30);
		let vfe_brand_owner_balance = <Currencies as MultiAssets<AccountId>>::balance(
			0,
			&CANDY,
		);
		assert_eq!(vfe_brand_owner_balance, 70);

		//unbind_device
		assert_ok!(VFE::unbind_device(
			Origin::signed(user.clone()),
			1,
			1,
		));
		System::assert_has_event(Event::VFE(crate::Event::DeviceUnbound{owner:user.clone(), device_key:puk, brand_id:1, item_id:1}));
		// assert!(!VFEBindDevices::<Test>::contains_key(1, 1));
		let device = Devices::<Test>::get(puk).expect("can not find device");
		assert!(device.item_id.is_none());
		let vfe = VFEDetails::<Test>::get(1, 1).expect("VFEDetail not exist");
		assert!(vfe.device_key.is_none());

	});
}

#[test]
fn bind_device_should_not_work() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());

		let account_nonce = 1u32;
		let account_rip160 = Ripemd::Hash::hash(user.encode().as_ref());
		let mut msg: Vec<u8> = Vec::new();
		msg.extend(account_nonce.to_le_bytes().to_vec());
		msg.extend(account_rip160.to_vec());

		let signature = key.sign(msg.as_ref());

		assert_noop!(
			<VFE as ValidateUnsigned>::validate_unsigned(
				TransactionSource::Local,
				&Call::bind_device {
					from: user.clone(),
					puk: pub_key,
					signature: signature.to_vec().try_into().unwrap(),
					nonce: account_nonce,
					bind_item: Some(1u32),
				}
			),
			dispatch_error_to_invalid(Error::<Test>::NonceMustGreatThanBefore.into())
		);

		assert_noop!(
			VFE::bind_device(
				Origin::none(),
				user.clone(),
				pub_key,
				signature.to_vec().try_into().unwrap(),
				account_nonce,
				Some(1),
			),
			Error::<Test>::NonceMustGreatThanBefore
		);

		let account_nonce = 2u32;
		let mut msg: Vec<u8> = Vec::new();
		msg.extend(account_nonce.to_le_bytes().to_vec());
		msg.extend(account_rip160.to_vec());

		let signature = key.sign(msg.as_ref());

		assert_noop!(
			VFE::bind_device(
				Origin::none(),
				user.clone(),
				pub_key,
				signature.to_vec().try_into().unwrap(),
				account_nonce,
				Some(1),
			),
			Error::<Test>::DeviceBond
		);

		//unbind_device
		assert_ok!(VFE::unbind_device(Origin::signed(user.clone()), 1, 1,));

		assert_noop!(
			VFE::bind_device(
				Origin::none(),
				user.clone(),
				pub_key,
				signature.to_vec().try_into().unwrap(),
				account_nonce,
				None,
			),
			Error::<Test>::DeviceBond
		);

		assert_noop!(
			VFE::bind_device(
				Origin::none(),
				user.clone(),
				pub_key,
				signature.to_vec().try_into().unwrap(),
				account_nonce,
				Some(2),
			),
			Error::<Test>::VFENotExist
		);
	});
}

#[test]
fn upload_training_report_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());
		Timestamp::set_timestamp(1668694716000);
		// first report
		let report = JumpRopeTrainingReport {
			timestamp: 1668676716,
			training_duration: 183,
			total_jump_rope_count: 738,
			average_speed: 140,
			max_speed: 230,
			max_jump_rope_count: 738,
			interruptions: 0,
			jump_rope_duration: 183,
		};
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_ok!(VFE::upload_training_report(
			Origin::none(),
			pub_key,
			BoundedVec::truncate_from(report_sig.to_vec()),
			BoundedVec::truncate_from(report.into()),
		));
		System::assert_has_event(Event::VFE(crate::Event::TrainingReportsAndRewards {
			owner: DANY,
			brand_id: 1,
			item_id: 1,
			sport_type: SportType::JumpRope,
			training_time: report.timestamp,
			training_duration: report.training_duration,
			training_count: report.total_jump_rope_count,
			energy_used: 6,
			asset_id: 1,
			rewards: 9000000,
		}));
		System::assert_has_event(Event::Assets(pallet_assets::Event::Issued {
			asset_id: 1,
			owner: user.clone(),
			total_supply: 9000000,
		}));
		let user_balance = <Currencies as fungibles::Inspect<AccountId>>::balance(1, &user);
		assert_eq!(user_balance, 9000000);
		let user_data = Users::<Test>::get(&user).expect("cannot find user");
		// println!("user data: {:?}", user_data);
		assert_eq!(user_data.energy, 2);
		assert_eq!(user_data.earned, 9000000);
		let vfe_data = VFEDetails::<Test>::get(1, 1).expect("cannot find vfe detail");
		assert_eq!(vfe_data.remaining_battery, 94);

		// can not report same timestamp
		assert_noop!(
			VFE::upload_training_report(
				Origin::none(),
				pub_key,
				BoundedVec::truncate_from(report_sig.to_vec()),
				BoundedVec::truncate_from(report.into()),
			),
			Error::<Test>::ValueInvalid
		);
		Timestamp::set_timestamp(1668847349000);
		// second report
		let mut report = JumpRopeTrainingReport {
			timestamp: 1668827349,
			training_duration: 1,
			total_jump_rope_count: 738,
			average_speed: 140,
			max_speed: 230,
			max_jump_rope_count: 738,
			interruptions: 0,
			jump_rope_duration: 1,
		};

		// user insufficient training
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_noop!(
			VFE::upload_training_report(
				Origin::none(),
				pub_key,
				BoundedVec::truncate_from(report_sig.to_vec()),
				BoundedVec::truncate_from(report.into()),
			),
			Error::<Test>::InsufficientTraining
		);

		// user training report time is expired
		Timestamp::set_timestamp(1668914749000);
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_noop!(
			VFE::upload_training_report(
				Origin::none(),
				pub_key,
				BoundedVec::truncate_from(report_sig.to_vec()),
				BoundedVec::truncate_from(report.into()),
			),
			Error::<Test>::TrainingReportTimeExpired
		);

		// sufficient training
		report.training_duration = 183;
		report.jump_rope_duration = 183;
		report.timestamp = 1668904749;
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_ok!(VFE::upload_training_report(
			Origin::none(),
			pub_key,
			BoundedVec::truncate_from(report_sig.to_vec()),
			BoundedVec::truncate_from(report.into()),
		));
		System::assert_has_event(Event::VFE(crate::Event::TrainingReportsAndRewards {
			owner: DANY,
			brand_id: 1,
			item_id: 1,
			sport_type: SportType::JumpRope,
			training_time: report.timestamp,
			training_duration: report.training_duration,
			training_count: report.total_jump_rope_count,
			energy_used: 2,
			asset_id: 1,
			rewards: 4200000,
		}));
		System::assert_has_event(Event::Assets(pallet_assets::Event::Issued {
			asset_id: 1,
			owner: user.clone(),
			total_supply: 4200000,
		}));

		// if user no energy, can not report training
		report.timestamp = 1668905749;
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_noop!(
			VFE::upload_training_report(
				Origin::none(),
				pub_key,
				BoundedVec::truncate_from(report_sig.to_vec()),
				BoundedVec::truncate_from(report.into()),
			),
			Error::<Test>::EnergyExhausted
		);
	});
}

#[test]
fn global_energy_recovery_and_daily_earned_reset_unit_test() {
	new_test_ext().execute_with(|| {
		assert_eq!(LastEnergyRecovery::<Test>::get(), 0);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), 0);
		run_to_block(5);
		assert_eq!(LastEnergyRecovery::<Test>::get(), 0);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), 0);
		run_to_block(9);
		assert_eq!(LastEnergyRecovery::<Test>::get(), 8);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), 0);
		System::assert_has_event(Event::VFE(crate::Event::GlobalEnergyRecoveryOccurred {
			block_number: 8,
		}));
		run_to_block(17);
		assert_eq!(LastEnergyRecovery::<Test>::get(), 16);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), 0);
		System::assert_has_event(Event::VFE(crate::Event::GlobalEnergyRecoveryOccurred {
			block_number: 16,
		}));
		run_to_block(20);
		assert_eq!(LastEnergyRecovery::<Test>::get(), 16);
		run_to_block(25);
		assert_eq!(LastEnergyRecovery::<Test>::get(), 24);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), 24);
		System::assert_has_event(Event::VFE(crate::Event::GlobalDailyEarnedResetOccurred {
			block_number: 24,
		}));
		run_to_block(9889);
		let last_update =
			9889u64.saturating_div(EnergyRecoveryDuration::get()) * EnergyRecoveryDuration::get();
		assert_eq!(LastEnergyRecovery::<Test>::get(), last_update);
		assert_eq!(LastDailyEarnedReset::<Test>::get(), last_update);
		System::assert_has_event(Event::VFE(crate::Event::GlobalEnergyRecoveryOccurred {
			block_number: 9888,
		}));
		System::assert_has_event(Event::VFE(crate::Event::GlobalDailyEarnedResetOccurred {
			block_number: 9888,
		}));
	});
}

#[test]
fn user_restore_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());

		Timestamp::set_timestamp(1668686716000);

		let report = JumpRopeTrainingReport {
			timestamp: 1668676716,
			training_duration: 183,
			total_jump_rope_count: 738,
			average_speed: 140,
			max_speed: 230,
			max_jump_rope_count: 738,
			interruptions: 0,
			jump_rope_duration: 183,
		};

		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_ok!(VFE::upload_training_report(
			Origin::none(),
			pub_key,
			report_sig.to_vec().try_into().unwrap(),
			report_encode.try_into().unwrap()
		));

		//after global energy recovery occurred
		run_to_block(9);

		// let call = Call::user_restore { who: user.clone() };
		// assert_ok!(<VFE as ValidateUnsigned>::validate_unsigned(TransactionSource::Local,
		// &call));
		assert_ok!(VFE::user_restore(Origin::signed(user.clone())));
		System::assert_has_event(Event::VFE(crate::Event::UserEnergyRestored {
			who: user.clone(),
			restored_amount: 2,
		}));
		let user_data = Users::<Test>::get(&user).expect("cannot find user");
		// println!("user data: {:?}", user_data);
		assert_eq!(user_data.energy, 4);
		assert!(user_data.earned != 0);

		//after repeatedly global energy recovery occurred
		run_to_block(229);
		assert_ok!(VFE::user_restore(Origin::signed(user.clone())));
		System::assert_has_event(Event::VFE(crate::Event::UserEnergyRestored {
			who: user.clone(),
			restored_amount: 4,
		}));
		let user_data = Users::<Test>::get(&user).expect("cannot find user");
		assert_eq!(user_data.energy, 8);
		assert!(user_data.earned == 0);
	});
}

#[test]
fn restore_power_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());

		Timestamp::set_timestamp(1668686716000);

		let report = JumpRopeTrainingReport {
			timestamp: 1668676716,
			training_duration: 183,
			total_jump_rope_count: 738,
			average_speed: 140,
			max_speed: 230,
			max_jump_rope_count: 738,
			interruptions: 0,
			jump_rope_duration: 183,
		};
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_ok!(VFE::upload_training_report(
			Origin::none(),
			pub_key,
			report_sig.to_vec().try_into().unwrap(),
			report_encode.try_into().unwrap()
		));

		assert_ok!(VFE::restore_power(Origin::signed(user.clone()), 1, 1, 3));
		System::assert_has_event(Event::VFE(crate::Event::PowerRestored {
			owner: user.clone(),
			charge_amount: 3,
			use_amount: 2100000,
			brand_id: 1,
			item_id: 1,
		}));

		let vfe_data = VFEDetails::<Test>::get(1, 1).expect("cannot find vfe detail");
		assert_eq!(vfe_data.remaining_battery, 97);

		assert_noop!(
			VFE::restore_power(Origin::signed(user.clone()), 1, 1, 4),
			Error::<Test>::ValueOverflow
		);

		// user balance of asset is no enough to restore pow
		assert_ok!(Currencies::transfer(Origin::signed(user.clone()), BOB, 1, 6400000, false));
		assert_noop!(
			VFE::restore_power(Origin::signed(user.clone()), 1, 1, 3),
			pallet_assets::Error::<Test>::BalanceLow
		);

		// after transfer user balance of asset is enough to restore pow
		assert_ok!(Currencies::transfer(Origin::signed(BOB), user.clone(), 1, 6400000, false));
		assert_ok!(VFE::restore_power(Origin::signed(user.clone()), 1, 1, 3));
		assert_noop!(
			VFE::restore_power(Origin::signed(user.clone()), 1, 1, 3),
			Error::<Test>::VFEFullyCharged
		);
	});
}

#[test]
fn level_up_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());

		assert_noop!(VFE::level_up(Origin::signed(user.clone()), 1, 2), Error::<Test>::VFENotExist);
		assert_noop!(
			VFE::level_up(Origin::signed(BOB), 1, 1),
			Error::<Test>::OperationIsNotAllowed
		);

		let vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		let user_info = Users::<Test>::get(&user).unwrap();
		let level_up_costs = VFE::calculate_level_up_costs(&vfe, &user_info);
		// println!("level_up_costs: {}", level_up_costs);

		//issue some asset to user, then level up VFE
		assert_ok!(Currencies::mint_into(1, &user, 180000000));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		System::assert_has_event(Event::VFE(crate::Event::VFELevelUp {
			brand_id: 1,
			item_id: 1,
			level_up: 1,
			cost: level_up_costs,
		}));
		let vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		assert_eq!(vfe.level, 1);
		assert_eq!(vfe.available_points, 4);

		let user_info = Users::<Test>::get(&user).unwrap();
		assert_eq!(user_info.energy_total, 8);
		assert_eq!(user_info.earning_cap, 1000 * 100000);

		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));

		let vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		let user_info = Users::<Test>::get(&user).unwrap();
		let level_up_costs = VFE::calculate_level_up_costs(&vfe, &user_info);
		// let user_balance = Currencies::balance(1, &user);
		// println!("user_balance = {}, level_up_costs = {}", user_balance, level_up_costs);
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		System::assert_has_event(Event::VFE(crate::Event::VFELevelUp {
			brand_id: 1,
			item_id: 1,
			level_up: 4,
			cost: level_up_costs,
		}));
		let vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		assert_eq!(vfe.level, 4);
		assert_eq!(vfe.available_points, 16);

		let user_info = Users::<Test>::get(&user).unwrap();
		assert_eq!(user_info.energy_total, 16);
		assert_eq!(user_info.earning_cap, 2500 * 100000);

		// let level_up_costs = VFE::calculate_level_up_costs(&vfe, &user_info);
		// let user_balance = Currencies::balance(1, &user);
		// println!("user_balance = {}, level_up_costs = {}", user_balance, level_up_costs);
		assert_noop!(
			VFE::level_up(Origin::signed(user.clone()), 1, 1),
			pallet_assets::Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn increase_ability_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer, user.clone(), pub_key, key.clone());

		assert_noop!(VFE::level_up(Origin::signed(user.clone()), 1, 2), Error::<Test>::VFENotExist);
		assert_noop!(
			VFE::level_up(Origin::signed(BOB), 1, 1),
			Error::<Test>::OperationIsNotAllowed
		);

		//issue some asset to user, then level up VFE
		assert_ok!(Currencies::mint_into(1, &user, 180000000));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		assert_ok!(VFE::level_up(Origin::signed(user.clone()), 1, 1));
		let origin_vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		assert_eq!(origin_vfe.level, 4);
		assert_eq!(origin_vfe.available_points, 16);

		let add_point = VFEAbility { efficiency: 10, skill: 10, luck: 10, durable: 10 };
		assert_noop!(
			VFE::increase_ability(Origin::signed(user.clone()), 1, 1, add_point),
			Error::<Test>::ValueInvalid
		);

		let add_point = VFEAbility { efficiency: 3, skill: 4, luck: 3, durable: 4 };
		assert_ok!(VFE::increase_ability(Origin::signed(user.clone()), 1, 1, add_point));
		System::assert_has_event(Event::VFE(crate::Event::VFEAbilityIncreased {
			brand_id: 1,
			item_id: 1,
		}));

		let vfe = VFEDetails::<Test>::get(1, 1).unwrap();
		assert_eq!(vfe.level, 4);
		assert_eq!(vfe.available_points, 2);
		assert_eq!(
			vfe.current_ability.efficiency,
			origin_vfe.current_ability.efficiency + add_point.efficiency
		);
		assert_eq!(vfe.current_ability.skill, origin_vfe.current_ability.skill + add_point.skill);
		assert_eq!(vfe.current_ability.luck, origin_vfe.current_ability.luck + add_point.luck);
		assert_eq!(
			vfe.current_ability.durable,
			origin_vfe.current_ability.durable + add_point.durable
		);
	});
}

#[test]
fn verify_bind_device_message_unit_test() {
	new_test_ext().execute_with(|| {
		// verify_bind_device_message
		let x = hex!["0339d3e6e837d675ce77e85d708caf89ddcdbf53c8e510775c9cb9ec06282475a0"];
		// let pubkey = VerifyingKey::from_sec1_bytes(x).unwrap();
		let sig = Signature::from_bytes(&hex!["d851b2fcd63bf78a52008e043cb28c47523c2ee2c3c9425c8b0a005e2f6f53b101690e030e0b2c8c6ae99c3a40c02f58dffd1fded34f84953d0e6ea671d83930"]).unwrap();
		let nonce = 0u32;
		let account_id = AccountId::new(hex!["764f41f48346146c043c5d0c7948f4b24a7877c649d5b72973850ccbe0f86840"]) ;
		let account_rip160 = Ripemd::Hash::hash(account_id.encode().as_ref());
		//13a7c41c6fa23d80f586051c6ccce5eb60192a20
		//13a7c41c6fa23d80f586051c6ccce5eb60192a20
		println!("ripemd160: {}", hex::encode(account_rip160));

		assert_ok!(VFE::verify_bind_device_message(account_id, nonce, DeviceKey::from_raw(x), sig.as_bytes()), true);
	});
}

#[test]
fn generate_bind_device_message_unit_test() {
	new_test_ext().execute_with(|| {
		let (key, pub_key) = generate_device_keypair();
		let nonce = 1u32;
		//5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
		let account_id =
			AccountId::from_string("5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y").unwrap();
		let account_rip160 = Ripemd::Hash::hash(account_id.encode().as_ref());
		let mut msg: Vec<u8> = Vec::new();
		msg.extend(nonce.to_le_bytes().to_vec());
		msg.extend(account_rip160.to_vec());
		let sig = key.sign(&msg);
		println!("account_id: {}", hex::encode(account_id.clone()));
		println!("device key: {}", hex::encode(pub_key));
		println!("signature: {}", hex::encode(sig));

		assert_ok!(
			VFE::verify_bind_device_message(account_id, nonce, pub_key, sig.as_bytes()),
			true
		);
	});
}

#[test]
fn verify_training_data_signature() {
	let x = &hex!["0339d3e6e837d675ce77e85d708caf89ddcdbf53c8e510775c9cb9ec06282475a0"];
	let pubkey = VerifyingKey::from_sec1_bytes(x).unwrap();

	let training_data = &hex!["68db756303001200680168011200000300"];
	// let training_hash = Sha256::new_with_prefix(training_data);
	let training_hash = sha2::Sha256::digest(training_data);
	println!("training_hash = {}", hex::encode(training_hash));

	let sig = Signature::from_bytes(&hex!["33bc624ff2ce52dfe013d0e8d8f2839ad350cfe56f5000f192c886b72ce0cd5864f7ab233d69e01568a988998967ee87e611b8455c0439dc69abf9b6eafad20d"]).unwrap();

	assert!(pubkey.verify(training_data.as_ref(), &sig).is_ok());
	// assert!(pubkey.verify_digest(training_hash, &sig).is_ok());

	let training_report =
		JumpRopeTrainingReport::try_from(training_data.to_vec()).expect("convert failed");
	println!("training_report = {:?}", training_report);
}

#[test]
fn training_report_encode_unit_test() {
	let report = JumpRopeTrainingReport {
		timestamp: 1668676716,
		training_duration: 183,
		total_jump_rope_count: 738,
		average_speed: 140,
		max_speed: 230,
		max_jump_rope_count: 738,
		interruptions: 0,
		jump_rope_duration: 183,
	};
	let encode: Vec<u8> = report.into();
	println!("encode = {}", hex::encode(&encode[..]));

	let decode_report = JumpRopeTrainingReport::try_from(encode).expect("convert failed");
	println!("decode = {:?}", decode_report);
}

#[test]
fn transfer_unit_test() {
	new_test_ext().execute_with(|| {
		let producer = ALICE;
		let user = DANY;
		let to = BOB;
		let (key, pub_key) = generate_device_keypair();
		produce_device_bind_vfe(producer.clone(), user.clone(), pub_key, key.clone());

		assert_noop!(
			VFE::transfer(Origin::signed(user.clone()), 1, 2, to.clone()),
			Error::<Test>::VFENotExist
		);
		assert_noop!(
			VFE::transfer(Origin::signed(producer.clone()), 1, 1, to.clone()),
			Error::<Test>::OperationIsNotAllowed
		);

		Timestamp::set_timestamp(1668686716000);

		let report = JumpRopeTrainingReport {
			timestamp: 1668676716,
			training_duration: 183,
			total_jump_rope_count: 738,
			average_speed: 140,
			max_speed: 230,
			max_jump_rope_count: 738,
			interruptions: 0,
			jump_rope_duration: 183,
		};
		let report_encode: Vec<u8> = report.into();
		let report_sig = key.sign(&report_encode);
		assert_ok!(VFE::upload_training_report(
			Origin::none(),
			pub_key,
			report_sig.to_vec().try_into().unwrap(),
			report_encode.try_into().unwrap()
		));

		assert_noop!(
			VFE::transfer(Origin::signed(user.clone()), 1, 1, to.clone()),
			Error::<Test>::VFENotFullyCharged
		);
		assert_ok!(VFE::restore_power(Origin::signed(user.clone()), 1, 1, 6));
		assert_ok!(VFE::transfer(Origin::signed(user.clone()), 1, 1, to.clone()));
		System::assert_has_event(Event::VFE(crate::Event::Transferred {
			brand_id: 1,
			item_id: 1,
			from: user.clone(),
			to: to.clone(),
		}));

		let vfe_owner = <VFE as Inspect<AccountId>>::owner(&1u32, &1u32).expect("vfe can not find");
		assert_eq!(vfe_owner, to);

	});
}
