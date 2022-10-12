use crate::{mock::*, Error};
// use alloc::vec;
use crate::pallet::Call as IpStakingCall;
use frame_support::{assert_noop, assert_ok};
use inv4::AnyIdOf;
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

        assert_eq!(IpStaking::total_staked(), 0);

        // BOB has never staked before
        assert!(IpStaking::ips_stakers(BOB).is_none());

        // Stake to IP set with 1 above `MinStakingAmount`
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_001
        ));

        assert_eq!(IpStaking::total_staked(), 1_000_000_000_001);

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 1_000_000_000_001));

        let ips_total_stake = IpStaking::registered_ips(ips_id).unwrap().total_stake;
        assert_eq!(ips_total_stake, 1_000_000_000_001);

        // Test staking with no history
        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple1 = Some((None, Some((1, 1_000_000_000_001))));
        assert_eq!(stakers_by_era, expected_tuple1);

        // Test staking multiple times in the same era
        // Stake a 2nd time in era 0
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple2 = Some((None, Some((1, 2_000_000_000_001))));
        assert_eq!(stakers_by_era, expected_tuple2);

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 2_000_000_000_001));

        // Runtime is set to 1 era = 1 block for ease of testing
        let mut block_number = frame_system::Pallet::<Test>::block_number();

        assert_eq!(block_number, 1);
        run_to_block(2);
        block_number = frame_system::Pallet::<Test>::block_number();
        assert_eq!(block_number, 2);

        // // ---Now in era 1---

        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        // Stake should have been shifted over to first spot in tuple
        let expected_tuple3 = Some((Some((1, 2_000_000_000_001)), None));
        assert_eq!(stakers_by_era, expected_tuple3);

        // New stake should be shifted over in tuple
        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (2_000_000_000_001, 0));

        // // Test staking in a new era, but with a current non-zero stake value from a previous era
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple3 = Some((Some((1, 2_000_000_000_001)), Some((2, 3_000_000_000_001))));
        assert_eq!(stakers_by_era, expected_tuple3);

        assert_eq!(block_number, 2);
        run_to_block(12);
        block_number = frame_system::Pallet::<Test>::block_number();
        assert_eq!(block_number, 12);

        // // ---Now in era 11---

        // Stake should have been shifted over to first spot in tuple
        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple4 = Some((Some((2, 3_000_000_000_001)), None));
        assert_eq!(stakers_by_era, expected_tuple4);

        // // Test staking in a new era, but with a current non-zero stake value from a previous era
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            2_000_000_000_000
        ));

        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple5 = Some((Some((2, 3_000_000_000_001)), Some((12, 5_000_000_000_001))));
        assert_eq!(stakers_by_era, expected_tuple5);

        let ips_total_stake = IpStaking::registered_ips(ips_id).unwrap().total_stake;
        assert_eq!(ips_total_stake, 5_000_000_000_001);

        // // Assert that the NewStake event is being emitted properly
        System::assert_last_event(
            crate::Event::NewStake {
                staker: BOB,
                ips_id: 0,
                stake_amount: 2_000_000_000_000,
            }
            .into(),
        );

        // TODO: Add a 2nd staker (ALICE) to this IPS
    });
}

#[test]
fn staking_below_min_amount_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));

        // Stake to IP set with 1 below `MinStakingAmount`. `stake` call should return error
        assert_noop!(
            IpStaking::stake(Origin::signed(BOB), ips_id, 999_999_999_999),
            Error::<Test>::BelowMinStakingAmount
        );
    });
}

#[test]
fn staking_to_non_registered_ips_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        // IP set is created, but not registered

        // Stake to IP set with 1 above `MinStakingAmount`. IP set is not registered so `stake` call should return error
        assert_noop!(
            IpStaking::stake(Origin::signed(BOB), ips_id, 1_000_000_000_001),
            Error::<Test>::IpsNotRegistered
        );
    });
}

#[test]
fn unstaking_below_min_amount_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));

        // Stake to IP set with MinStakingAmount
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        // Unstaking less than the MinStakingAmount should fail
        // assert_noop!(IpStaking::unstake_amount(Origin::signed(BOB), ips_id, 999_999_999_999), Error::<Test>::BelowMinUnstakingAmount);
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
    let call = Call::IpStaking(IpStakingCall::register { ips_id });
    assert_ok!(INV4::operate_multisig(
        Origin::signed(ALICE),
        false,
        (ips_id, None),
        Box::new(call)
    ));

    assert_ne!(IpStaking::registered_ips(ips_id), None);

    Ok(())
}
