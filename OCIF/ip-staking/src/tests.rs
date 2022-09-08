use crate::{mock::*, Error};
// use alloc::vec;
use frame_support::{assert_noop, assert_ok};
use inv4::AnyIdOf;
use crate::pallet::Call as IpStakingCall;
// use frame_system::Origin;
use primitives::*;

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

// #[test]
// fn jake_test() {
// 	new_test_ext().execute_with(|| {
// 		// assert_eq(1 == 2, Error::<Test>::Overflow);
// 		assert_eq!(1, 2);
// 	});
// }

// #[test]
// fn ips_registered_old() {
// 	new_test_ext().execute_with(|| {
// 		let id = INV4::next_ips_id();
// 		assert_eq!(id, 0);
// 		// Create an IP set
// 		let metadata: Vec<u8> = vec![1u8, 2u8, 3u8];
// 		let assets: Vec<AnyIdOf<Test>> = vec![];

// 		assert_ok!(INV4::create_ips(
// 			Origin::signed(1),
// 			metadata,
// 			assets,
// 			false,
// 			InvArchLicenses::Apache2,
// 			OneOrPercent::One,
// 			OneOrPercent::One,
// 			false
// 		));
		
// 		assert_ne!(INV4::ips_storage(0), None);

// 	});
// }

#[test]
fn ips_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let ips_id = INV4::next_ips_id();
		assert_eq!(ips_id, 0);

		// Create an IP set
		let metadata: Vec<u8> = vec![1u8, 2u8, 3u8];
		let assets: Vec<AnyIdOf<Test>> = vec![];

		assert_ok!(INV4::create_ips(
			Origin::signed(ALICE),
			metadata,
			assets,
			false,
			InvArchLicenses::Apache2,
			OneOrPercent::One,
			OneOrPercent::One,
			false
		));
		
		assert_ne!(INV4::ips_storage(ips_id), None);

		// Register IP set for IP staking
		let call = Call::IpStaking(IpStakingCall::register {
			ips_id
		});
		assert_ok!(INV4::operate_multisig(Origin::signed(ALICE), false, (ips_id, None), Box::new(call)));

		assert_ne!(IpStaking::registered_ips(ips_id), None);
	});
}