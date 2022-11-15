// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryInto;
use std::convert::TryInto as TryInto2;
use frame_support::weights::Pays::No;
use hex_literal::hex;
use rand_core::OsRng;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn generate_device_keypair() -> (SigningKey, PublicKey<NistP256>) {
	let signing_key = SigningKey::random(&mut OsRng);
	let verifying_key = signing_key.verifying_key(); // Serialize with `::to_encoded_point()`
	let public_key: PublicKey<NistP256> = verifying_key.into();
	// let encoded_point = publickey.to_encoded_point(true);
	return (signing_key, public_key);
}

#[test]
fn producer_register_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(VFE::producer_register(Origin::signed(ALICE)));

		System::assert_has_event(Event::VFE(crate::Event::ProducerRegister(ALICE, 1)));

		// wrong origin role.
		assert_noop!(
			VFE::producer_register(Origin::signed(BOB)),
			DispatchError::BadOrigin
		);
	});
}


#[test]
fn producer_owner_change_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::producer_register(Origin::signed(ALICE)));

		// Dispatch a signed extrinsic.
		assert_ok!(VFE::producer_owner_change(Origin::signed(ALICE), 1, TOM));

		System::assert_has_event(Event::VFE(crate::Event::ProducerOwnerChanged(ALICE, 1, TOM)));

		// wrong origin role.
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(BOB), 1, ALICE),
			DispatchError::BadOrigin
		);

		// RoleInvalid.
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(TOM), 1, BOB),
			Error::<Test>::RoleInvalid
		);

		// Operation is not allowed for producer.
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(ALICE), 1, TOM),
			Error::<Test>::OperationIsNotAllowedForProducer
		);

		// producer is not exist
		assert_noop!(
			VFE::producer_owner_change(Origin::signed(TOM), 2, ALICE),
			Error::<Test>::ProducerNotExist
		);
	});
}


// #[test]
// fn producer_charge_should_work() {
// 	new_test_ext().execute_with(|| {
//
// 		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));
//
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(Sport::producer_charge(Origin::signed(ALICE),0,0,100));
// 		//
//
// 		//
// 		System::assert_has_event(Event::Sport(crate::Event::ProducerCharge(ALICE, 0, 0,100)));
//
// 	});
// }
//
// #[test]
// fn producer_withdraw_should_work() {
// 	new_test_ext().execute_with(|| {
//
// 		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));
//
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(Sport::producer_charge(Origin::signed(ALICE),0,0,100));
// 		System::assert_has_event(Event::Sport(crate::Event::ProducerCharge(ALICE, 0, 0,100)));
//
//
// 	});
// }


#[test]
fn create_vfe_brand_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::create_vfe_brand(Origin::signed(CANDY), bvec![0u8; 20], SportType::JumpRope, VFERarity::Common));

		System::assert_has_event(Event::VFE(crate::Event::VFEBrandCreated(
			CANDY, 1,
			SportType::JumpRope, VFERarity::Common, bvec![0u8; 20])));
	});
}

#[test]
fn approve_mint_unit_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFE::producer_register(Origin::signed(ALICE)));
		assert_ok!(VFE::create_vfe_brand(Origin::signed(CANDY), bvec![0u8; 20], SportType::JumpRope, VFERarity::Common));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, None));
		System::assert_has_event(Event::VFE(crate::Event::ApprovedMint(1, 1, 10, None)));
		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 10);
		assert_eq!(approve.mint_cost, None);

		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 12, None));
		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 22);

		assert_noop!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 12, Some((1,10))),
		Error::<Test>::RemainingMintAmountIsNotZero);
	});
}

#[test]
fn register_device_should_work() {
	new_test_ext().execute_with(|| {
		// register producer
		assert_ok!(VFE::producer_register(Origin::signed(ALICE)));

		// create vfe brand
		assert_ok!(VFE::create_vfe_brand(Origin::signed(CANDY), bvec![0u8; 20], SportType::JumpRope, VFERarity::Common));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, Some((0,10))));

		// register device
		let bytes = hex::decode("02e3a9257cf457087eeef75f466d3da31318b046ffcce05d104a0505d9799b47c6").unwrap();
		let puk: [u8; 33] = bytes.try_into().expect("error length");

		assert_ok!(VFE::register_device(Origin::signed(ALICE), puk, 1, 1));
		System::assert_has_event(Event::VFE(crate::Event::DeviceRegistered(ALICE, 1, puk, 1)));
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

		let producer_balance = <Assets as MultiAssets<AccountId>>::balance(0, &crate::Pallet::<Test>::into_account_id(1));
		println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 10);

		//deregister device
		assert_ok!(VFE::deregister_device(Origin::signed(ALICE), puk));
		System::assert_has_event(Event::VFE(crate::Event::DeviceDeregistered(ALICE, puk)));

		assert_eq!(Devices::<Test>::get(puk), None);

		let approve = VFEApprovals::<Test>::get(1, 1).expect("approve is nil");
		assert_eq!(approve.remaining_mint, 10);
		assert_eq!(approve.registered, 0);
		assert_eq!(approve.locked_of_mint, 0);

		let producer_balance = <Assets as MultiAssets<AccountId>>::balance(0, &crate::Pallet::<Test>::into_account_id(1));
		println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 0);
	});
}


#[test]
fn bind_device_should_work() {
	new_test_ext().execute_with(|| {

		// device keypair
		let bytes = hex::decode("0339d3e6e837d675ce77e85d708caf89ddcdbf53c8e510775c9cb9ec06282475a0").unwrap();
		let puk: [u8; 33] = bytes.try_into().expect("error length");


		// register producer
		assert_ok!(VFE::producer_register(Origin::signed(ALICE)));
		// create vfe brand
		assert_ok!(VFE::create_vfe_brand(Origin::signed(CANDY), bvec![0u8; 20], SportType::JumpRope, VFERarity::Common));
		assert_ok!(VFE::approve_mint(Origin::signed(CANDY), 1, 1, 10, Some((0,10))));
		// register device
		assert_ok!(VFE::register_device(Origin::signed(ALICE), puk, 1, 1));
		let producer_balance = <Assets as MultiAssets<AccountId>>::balance(0, &crate::Pallet::<Test>::into_account_id(1));
		println!("producer_balance: {}", producer_balance);
		assert_eq!(producer_balance, 10);

		let user = Dany;
		let account_nonce = 123u32;
		let account_rip160 = Ripemd::Hash::hash(user.encode().as_ref());
		println!("account_nonce = {}", hex::encode(account_nonce.to_le_bytes()));
		println!("account_hex = {:?}", hex::encode(user.encode()));
		println!("account_rip160 = {:?}", account_rip160);

		let signature = hex::decode("df6e11efe387bec44bc15c3c636dfa51a951a1cda1a96d1d1b32566de948cda6125e873bac098688b4991512ca1dfa68a26862c97b81ad0555a06f1423874d66").unwrap();

		assert_ok!(VFE::bind_device(
			Origin::signed(user.clone()),
			puk,
			signature.try_into().unwrap(),
			account_nonce,
			None
		));
		System::assert_has_event(Event::VFE(crate::Event::DeviceBound(user, puk, 1, 1)));

		let vfe = VFEDetails::<Test>::get(1, 1).expect("VFEDetail not exist");
		assert_eq!(vfe.level, 0);
		println!("vfe.base.efficiency: {}", vfe.base_ability.efficiency);
		println!("vfe.base.skill: {}", vfe.base_ability.skill);
		println!("vfe.base.luck: {}", vfe.base_ability.luck);
		println!("vfe.base.durable: {}", vfe.base_ability.durable);
	});
}


// #[test]
// fn sport_upload_should_work() {
// 	new_test_ext().execute_with(|| {
//
// 		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));
//
// 		assert_ok!(Sport::device_type_create(Origin::signed(ALICE),bvec![0u8; 20],0,SportType::SkippingRope));
//
// 		let hash = hex::decode("02721aacc27b73f67f417856f183e83986f7dee7a1a16ce39b202ba988c890b1d2").unwrap();
// 		let puk:[u8;33] = hash[0..33].try_into().expect("error length");
//
// 		assert_ok!(Sport::register_device(Origin::signed(ALICE),puk,0,0));
// 		System::assert_has_event(Event::Sport(crate::Event::DeviceRegistered(ALICE, puk, 0)));
//
// 		let  nonce_account = 1u32.to_be_bytes();
//
// 		println!("nonce_account 1 = {:?}", nonce_account);
//
// 		// println!("nonce_account = {}", hex::encode(nonce_account));
//
// 		let account_rip160 = Ripemd::Hash::hash(ALICE.encode().as_ref());
//
// 		println!("account_rip160 = {:?}", account_rip160);
//
// 		let x:Vec<u8> = hex::decode("63bb64a3bdffa7f8dc0a6723c56294a97a0012765f4b35b118338ffe36cf6dededcb5b11f8ce279b59dabbe391a1a1975179cb80e10b4197c12399df00b8de5e").unwrap();
//
// 		assert_ok!(Sport::bind_device(Origin::signed(ALICE),puk,x.try_into().unwrap(),1,None));
//
// 		let msg = hex::decode("c5968238060023005e015e012300000600").unwrap();
//
// 		let final_req_sig = hex::decode("2b1984438448ace394b3c0a15195f830e9bd8bc6df88a51db218dc25e18bc9e43a867493dcc98edc38d1c15f621bb5440cd0c9cd01e5011d89ebef7dd976a734").unwrap();
//
//
// 		assert_ok!(Sport::sport_upload(Origin::signed(ALICE),puk,final_req_sig.try_into().unwrap(),msg.try_into().unwrap()));
// 		// System::assert_has_event(Event::Sport(crate::Event::UnBindDevice(ALICE, puk, 0,0)));
//
// 	});
// }
