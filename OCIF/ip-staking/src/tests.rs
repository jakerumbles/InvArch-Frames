use crate::{mock::*, Error};
// use alloc::vec;
use frame_support::{assert_noop, assert_ok};
use inv4::AnyIdOf;
use crate::pallet::Call as IpStakingCall;
// use frame_system::Origin;
use primitives::*;


#[test]
fn ips_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let ips_id = create_ips();
		assert_ok!(register_ips(ips_id));
	});
}

#[test]
fn stake_to_ips() {
	ExtBuilder::default().build().execute_with(|| {
		let ips_id = create_ips();
		assert_ok!(register_ips(ips_id));

		// Stake to IP set with 1 above `MinStakingAmount`
		assert_ok!(IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_001));

		let stakers_vec = IpStaking::ips_stakers(ips_id);
		assert!(stakers_vec.contains(&BOB));
	});
}

fn create_ips() -> u32 {
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

	ips_id
}

fn register_ips(ips_id: u32) -> Result<(), ()> {
	assert_ne!(INV4::ips_storage(ips_id), None);

	// Register IP set for IP staking
	let call = Call::IpStaking(IpStakingCall::register {
		ips_id
	});
	assert_ok!(INV4::operate_multisig(Origin::signed(ALICE), false, (ips_id, None), Box::new(call)));

	assert_ne!(IpStaking::registered_ips(ips_id), None);

	Ok(())
}