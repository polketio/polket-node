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
