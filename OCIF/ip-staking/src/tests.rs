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
		// ---ERA 0---

		let ips_id = create_ips();
		assert_ok!(register_ips(ips_id));

		// Stake to IP set with 1 above `MinStakingAmount`
		assert_ok!(IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_001));

		let mut ips_total_stake = IpStaking::registered_ips(ips_id).unwrap().total_stake;
		assert_eq!(ips_total_stake, 1_000_000_000_001);

		let stakers_vec = IpStaking::ips_stakers(ips_id);
		assert!(stakers_vec.contains(&BOB));

		// Test staking with no history
		let stakers_by_era = IpStaking::stake_by_era(ips_id, BOB).into_inner();
		let expected_vec1: Vec<(u32, u128)> = vec![(1, 1_000_000_000_001)];
		assert_eq!(stakers_by_era, expected_vec1);

		// Test staking multiple times in the same era
		// Stake a 2nd time in era 0
		assert_ok!(IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_000));

		let stakers_by_era = IpStaking::stake_by_era(ips_id, BOB).into_inner();
		let expected_vec2: Vec<(u32, u128)> = vec![(1, 2_000_000_000_001)];
		assert_eq!(stakers_by_era, expected_vec2);

		// Runtime is set to 1 era = 1 block for ease of testing
		let mut block_number = frame_system::Pallet::<Test>::block_number();

		assert_eq!(block_number, 1);
		run_to_block(2);
		block_number = frame_system::Pallet::<Test>::block_number();
		assert_eq!(block_number, 2);

		// ---Now in era 1---

		// Test staking in a new era, but with a current non-zero stake value from a previous era
		assert_ok!(IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_000));

		let stakers_by_era = IpStaking::stake_by_era(ips_id, BOB).into_inner();
		let expected_vec3: Vec<(u32, u128)> = vec![(1, 2_000_000_000_001), (2, 3_000_000_000_001)];
		assert_eq!(stakers_by_era, expected_vec3);

		assert_eq!(block_number, 2);
		run_to_block(12);
		block_number = frame_system::Pallet::<Test>::block_number();
		assert_eq!(block_number, 12);

		// ---Now in era 11---

		// Test staking in a new era, but with a current non-zero stake value from a previous era
		assert_ok!(IpStaking::stake(Origin::signed(BOB), ips_id, 2_000_000_000_000));

		let stakers_by_era = IpStaking::stake_by_era(ips_id, BOB).into_inner();
		let expected_vec4: Vec<(u32, u128)> = vec![(1, 2_000_000_000_001), (2, 3_000_000_000_001), (12, 5_000_000_000_001)];
		assert_eq!(stakers_by_era, expected_vec4);

		ips_total_stake = IpStaking::registered_ips(ips_id).unwrap().total_stake;
		assert_eq!(ips_total_stake, 5_000_000_000_001);

		// Assert that the NewStake event is being emitted properly
		System::assert_last_event(crate::Event::NewStake { staker: BOB, ips_id: 0, stake_amount: 2_000_000_000_000 }.into());

		// TODO: Add a 2nd staker (ALICE) to this IPS

	});
}

#[test]
fn staking_below_min_amount_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let ips_id = create_ips();
		assert_ok!(register_ips(ips_id));

		// Stake to IP set with 1 below `MinStakingAmount`. `stake` call should return error
		assert_noop!(IpStaking::stake(Origin::signed(BOB), ips_id, 999_999_999_999), Error::<Test>::BelowMinAmount);
	});
}

#[test]
fn staking_to_non_registered_ips_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let ips_id = create_ips();
		// IP set is created, but not registered

		// Stake to IP set with 1 above `MinStakingAmount`. IP set is not registered so `stake` call should return error
		assert_noop!(IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_001), Error::<Test>::IpsNotRegistered);
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