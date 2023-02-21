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




#[test]
fn order_create() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFEUniques::create(Origin::signed(ALICE), BOB));
		assert_ok!(VFEUniques::mint(Origin::signed(BOB),0,0, BOB));

		let order_item = OrderItem {
			collection_id: 0,
			item_id: 0,
		};
		let mut order_item_encode: Vec<OrderItem<u32,u32>> = Vec::with_capacity(1);
		order_item_encode.push(order_item);
		

		assert_ok!(VFEorder::submit_order(Origin::signed(BOB),1,10,100,BoundedVec::truncate_from(order_item_encode)));
	
		System::assert_has_event(Event::VFEorder(crate::Event::CreatedOrder{who:BOB,order_id: 1}));


		
	});
}



#[test]
fn order_create_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFEUniques::create(Origin::signed(ALICE), BOB));
		assert_ok!(VFEUniques::mint(Origin::signed(BOB),0,0, BOB));

		let order_item = OrderItem {
			collection_id: 0,
			item_id: 0,
		};
		let mut order_item_encode: Vec<OrderItem<u32,u32>> = Vec::with_capacity(1);
		order_item_encode.push(order_item);
		

		assert_ok!(VFEorder::submit_order(Origin::signed(BOB),1,10,100,BoundedVec::truncate_from(order_item_encode.clone())));
	

		assert_noop!(
			VFEorder::submit_order(Origin::signed(ALICE),1,10,100,BoundedVec::truncate_from(order_item_encode)),
			Error::<Test>::NotBelongToyYou
		);

		System::assert_has_event(Event::VFEorder(crate::Event::CreatedOrder{who:BOB, order_id:1}));


		
	});
}


#[test]
fn order_take_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFEUniques::create(Origin::signed(ALICE), BOB));
		assert_ok!(VFEUniques::mint(Origin::signed(BOB),0,0, BOB));

		let order_item = OrderItem {
			collection_id: 0,
			item_id: 0,
		};
		let mut order_item_encode: Vec<OrderItem<u32,u32>> = Vec::with_capacity(1);
		order_item_encode.push(order_item);
		

		assert_ok!(VFEorder::submit_order(Origin::signed(BOB),1,10,100,BoundedVec::truncate_from(order_item_encode)));
	

		assert_ok!(VFEorder::take_order(Origin::signed(ALICE),1,BOB));
	
		System::assert_has_event(Event::VFEorder(crate::Event::TakenOrder{purchaser:ALICE,order_owner:BOB,order_id:1}));

		
	});
}



#[test]
fn order_remove_test() {
	new_test_ext().execute_with(|| {
		assert_ok!(VFEUniques::create(Origin::signed(ALICE), BOB));
		assert_ok!(VFEUniques::mint(Origin::signed(BOB),0,0, BOB));

		let order_item = OrderItem {
			collection_id: 0,
			item_id: 0,
		};
		let mut order_item_encode: Vec<OrderItem<u32,u32>> = Vec::with_capacity(1);
		order_item_encode.push(order_item);
		

		assert_ok!(VFEorder::submit_order(Origin::signed(BOB),1,10,100,BoundedVec::truncate_from(order_item_encode)));
	

		assert_ok!(VFEorder::remove_order(Origin::signed(BOB),1));
	
		System::assert_has_event(Event::VFEorder(crate::Event::RemovedOrder{who:BOB,order_id:1}));

		
	});
}