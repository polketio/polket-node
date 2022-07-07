

// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryInto;
use std::convert::TryInto as TryInto2;
use hex_literal::hex;
macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}


#[test]
fn generate_secp256r1_pk() {
	// Signing
	let x = &hex!["c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721"];
	let signing_key = SigningKey::from_bytes(x).unwrap(); // Serialize with `::to_bytes()`
	let message = b"ECDSA proves knowledge of a secret number in the context of a single message";
	let signature = signing_key.sign(message);

	// println!("signature = {}", hex::encode(signature.to_vec()));

	let sig_t = &signature.to_vec()[..];

	let sig = Signature::from_bytes(sig_t).map_err(|_| Error::<Test>::SigEncodeError);

	let y = &hex!["0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6"];

	// let verifying_key = VerifyingKey::from_sec1_bytes(y).unwrap();
	let verifying_key = signing_key.verifying_key(); // Serialize with `::to_encoded_point()`
	let publickey: PublicKey<NistP256> = verifying_key.into();
	let encoded_point = publickey.to_encoded_point(true);
	println!("pks = {}", hex::encode(encoded_point));
	//0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6
	assert!(verifying_key.verify(message, &signature).is_ok());
}



#[test]
fn generate_secp256r1_pk2() {
	// Signing
	let message = [1u8;24].as_ref();

	// println!("signature = {}", hex::encode(signature.to_vec()));

	let sig_t =  &hex!["000000002ba80211cb75ff1f00000000bcccff1f0500000000000000bcccff1f00000000fcccff1f00000000379b021100000000b0dcff1f0000000013000000"];

	let signature = Signature::from_bytes(sig_t).unwrap();

	let y = &hex!["034ced5c5bd6a31930f6aef5a6af3ea1793f8b1810f98b2e5e915e2692d2de4f9a"];

	let verifying_key = VerifyingKey::from_sec1_bytes(y).unwrap();
	let publickey: PublicKey<NistP256> = verifying_key.into();
	let encoded_point = publickey.to_encoded_point(true);
	println!("pks = {}", hex::encode(encoded_point));
	//0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb6
	assert!(verifying_key.verify(message, &signature).is_ok());
}

#[test]
fn producer_register_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		let account_id = Sport::into_account_id(0);

		System::assert_has_event(Event::Sport(crate::Event::ProducerRegister(ALICE, 0, account_id)));

		// wrong origin role.
		assert_noop!(
			Sport::producer_register(Origin::signed(BOB)),
			DispatchError::BadOrigin
		);
	});
}



#[test]
fn producer_owner_change_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		// Dispatch a signed extrinsic.
		assert_ok!(Sport::producer_owner_change(Origin::signed(ALICE),0,TOM));

		let account_id = Sport::into_account_id(0);

		System::assert_has_event(Event::Sport(crate::Event::ProducerOwnerChanged(ALICE, 0, TOM)));

		// wrong origin role.
		assert_noop!(
			Sport::producer_owner_change(Origin::signed(BOB),0,ALICE),
			DispatchError::BadOrigin
		);

		// RoleInvalid.
		assert_noop!(
			Sport::producer_owner_change(Origin::signed(TOM),0,BOB),
			Error::<Test>::RoleInvalid
		);

		// Operation is not allowed for producer.
		assert_noop!(
			Sport::producer_owner_change(Origin::signed(ALICE),0,TOM),
			Error::<Test>::OperationIsNotAllowedForProducer
		);

		// producer is not exist
		assert_noop!(
			Sport::producer_owner_change(Origin::signed(TOM),1,ALICE),
			Error::<Test>::ProducerNotExist
		);

	});
}



#[test]
fn producer_charge_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		// Dispatch a signed extrinsic.
		assert_ok!(Sport::producer_charge(Origin::signed(ALICE),0,0,100));
		//

		//
		System::assert_has_event(Event::Sport(crate::Event::ProducerCharge(ALICE, 0, 0,100)));

	});
}

#[test]
fn producer_withdraw_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		// Dispatch a signed extrinsic.
		assert_ok!(Sport::producer_charge(Origin::signed(ALICE),0,0,100));
		System::assert_has_event(Event::Sport(crate::Event::ProducerCharge(ALICE, 0, 0,100)));


	});
}


#[test]
fn device_type_create_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));


		assert_ok!(Sport::device_type_create(Origin::signed(ALICE),bvec![0u8; 20],0,SportType::SkippingRope));


		System::assert_has_event(Event::Sport(crate::Event::DeviceTypeCreate(ALICE, 0, 0,SportType::SkippingRope,bvec![0u8; 20])));

		assert_noop!(
			Sport::device_type_create(Origin::signed(ALICE),bvec![0u8; 20],1,SportType::SkippingRope),
			Error::<Test>::ProducerNotExist
		);

	});
}




#[test]
fn register_device_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		assert_ok!(Sport::device_type_create(Origin::signed(ALICE),bvec![0u8; 20],0,SportType::SkippingRope));

		let hash = hex::decode("02e3a9257cf457087eeef75f466d3da31318b046ffcce05d104a0505d9799b47c6").unwrap();
		let puk:[u8;33] = hash[0..33].try_into().expect("error length");
		// In the Normal Case
		assert_ok!(Sport::register_device(Origin::signed(ALICE),puk,0,0));
		System::assert_has_event(Event::Sport(crate::Event::RegisterDevice(ALICE, puk, 0)));

		// Error for OperationIsNotAllowed
		{
			assert_ok!(Sport::producer_register(Origin::signed(TOM)));
			let hash = hex::decode("0360fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb5").unwrap();
			let puk:[u8;33] = hash[0..33].try_into().expect("error length");
			assert_noop!(
				Sport::register_device(Origin::signed(TOM),puk,1,0),
				Error::<Test>::OperationIsNotAllowed
			);
		}

	});
}




#[test]
fn bind_device_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Sport::producer_register(Origin::signed(ALICE)));

		assert_ok!(Sport::device_type_create(Origin::signed(ALICE),bvec![0u8; 20],0,SportType::SkippingRope));

		let hash = hex::decode("034ced5c5bd6a31930f6aef5a6af3ea1793f8b1810f98b2e5e915e2692d2de4f9a").unwrap();
		let puk:[u8;33] = hash[0..33].try_into().expect("error length");

		assert_ok!(Sport::register_device(Origin::signed(ALICE),puk,0,0));
		System::assert_has_event(Event::Sport(crate::Event::RegisterDevice(ALICE, puk, 0)));


		let x:Vec<u8> = hex::decode("000000002ba80211cb75ff1f00000000bcccff1f0500000000000000bcccff1f00000000fcccff1f00000000379b021100000000b0dcff1f0000000013000000").unwrap();

		assert_ok!(Sport::bind_device(Origin::signed(ALICE),puk,x.try_into().unwrap(),1,None));

		System::assert_has_event(Event::Sport(crate::Event::UnBindDevice(ALICE, puk, 0,0)));

	});
}
