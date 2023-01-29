// This file is part of Polket.
// Copyright (C) 2021-2022 Polket.
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn generate_object_id_should_work() {
	new_test_ext().execute_with(|| {
		let brand_id = 0u32;
		assert_ok!(crate::Pallet::<Test>::generate_object_id(brand_id), 1u32);
		assert_eq!(UniqueId::next_object_id(brand_id), 2u32);

		assert_ok!(crate::Pallet::<Test>::generate_object_id(brand_id), 2u32);
		assert_eq!(UniqueId::next_object_id(brand_id), 3u32);

		let producer_id = 1u32;
		assert_ok!(crate::Pallet::<Test>::generate_object_id(producer_id), 1u32);
		assert_eq!(UniqueId::next_object_id(producer_id), 2u32);
	});
}

#[test]
fn generate_over_half_of_max_value_should_not_work() {
	new_test_ext().execute_with(|| {
		let brand_id = 0u32;
		for i in 0..101 {
			// println!("id = {}", i);
			if i >= 100 {
				assert_noop!(crate::Pallet::<Test>::generate_object_id(brand_id), Error::<Test>::ValueOverflow);
			} else {
				assert_ok!(crate::Pallet::<Test>::generate_object_id(brand_id), i +1);
				assert_eq!(UniqueId::next_object_id(brand_id), i + 2);
			}

		}
	});
}

#[test]
fn encode_test() {
	let prefix_id = 1u64;
	let parent_id = 1u64;
	let encode_id = (prefix_id, parent_id).encode();
	println!("(1, 2)): {}", hex::encode(encode_id));

	let prefix_id = 2u64;
	let parent_id = 1u64;
	let encode_id = (prefix_id, parent_id).encode();
	println!("(2, 1)): {}", hex::encode(encode_id));
}