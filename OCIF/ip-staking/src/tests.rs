use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
// use frame_system::Origin;
use primitives::*;
use sp_runtime::AccountId32;

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
// 	});
// }

#[test]
fn jake_test() {
	new_test_ext().execute_with(|| {
		// assert_eq(1 == 2, Error::<Test>::Overflow);
		assert_eq!(1, 2);
	});
}

#[test]
fn ips_registered() {
	new_test_ext().execute_with(|| {
		// Create an IP set
		INV4::create_ips(Origin::signed(1), vec![], vec![], false, InvArchLicenses::Apache2, OneOrPercent::One, OneOrPercent::One, true);
	});
}