use crate::{mock::*, Error};
// use alloc::vec;
use crate::pallet::Call as IpStakingCall;
use frame_support::BoundedVec;
use frame_support::{assert_noop, assert_ok, PalletId, traits::{LockIdentifier}};
use sp_runtime::traits::AccountIdConversion;
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

        assert_eq!(IpStaking::total_staked(), (0, 0, 0));

        // BOB has never staked before
        assert!(IpStaking::ips_stakers(BOB).is_none());

        // Stake to IP set with 1 above `MinStakingAmount`
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_001
        ));

        // New stake of 1_000_000_000_001 will apply to next era, but current total stake is still 0. 0 unstake as well
        assert_eq!(IpStaking::total_staked(), (0, 1_000_000_000_001, 0));

        // Account has 0 active stake. 1_000_000_000_001 new stake will be added to active stake at beginning of next era
        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 1_000_000_000_001, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 1_000_000_000_001);
        assert_eq!(new_unstake, 0);

        // Test staking with no history
        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple1 = Some((None, Some(1_000_000_000_001), None));
        assert_eq!(stakers_by_era, expected_tuple1);

        // Test staking multiple times in the same era
        // Stake a 2nd time in era 0
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        let stakers_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple2 = Some((None, Some(2_000_000_000_001), None));
        assert_eq!(stakers_by_era, expected_tuple2);

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 2_000_000_000_001, 0));

        // New stake of 1_000_000_000_00 will apply to next era, but current total stake is still 0. 0 unstake as well
        assert_eq!(IpStaking::total_staked(), (0, 2_000_000_000_001, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 2_000_000_000_001);
        assert_eq!(new_unstake, 0);

        // Runtime is set to 1 era = 1 block for ease of testing
        let mut block_number = frame_system::Pallet::<Test>::block_number();

        assert_eq!(block_number, 1);
        run_to_block(2);
        block_number = frame_system::Pallet::<Test>::block_number();
        assert_eq!(block_number, 2);

        // // ---Now in era 1---

        // New stake should be added over to total stake
        assert_eq!(IpStaking::total_staked(), (2_000_000_000_001, 0, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 2_000_000_000_001);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        let staker_by_era = IpStaking::stake_by_era(BOB, ips_id);
        // Stake should have been shifted over to first spot in tuple
        let expected_tuple3 = Some((Some((1, 2_000_000_000_001)), None, None));
        assert_eq!(staker_by_era, expected_tuple3);

        // New stake should be shifted over in tuple
        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (2_000_000_000_001, 0, 0));

        // Test staking in a new era, but with a current non-zero stake value from a previous era
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        // 1_000_000_000_000 should be added on to new stake
        assert_eq!(IpStaking::total_staked(), (2_000_000_000_001, 1_000_000_000_000, 0));

        // 1_000_000_000_000 should be added on to new stake
        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 2_000_000_000_001);
        assert_eq!(new_stake, 1_000_000_000_000);
        assert_eq!(new_unstake, 0);

        let staker_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple4 = Some((Some((1, 2_000_000_000_001)), Some(1_000_000_000_000), None));
        assert_eq!(staker_by_era, expected_tuple4);

        assert_eq!(block_number, 2);
        run_to_block(12);
        block_number = frame_system::Pallet::<Test>::block_number();
        assert_eq!(block_number, 12);

        // ---Now in era 11---

        // New stake should be added over to total stake
        assert_eq!(IpStaking::total_staked(), (3_000_000_000_001, 0, 0));

        // New stake should be added over to total stake
        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 3_000_000_000_001);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        // Stake should have been shifted over to first spot in tuple
        let staker_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple5 = Some((Some((2, 3_000_000_000_001)), None, None));
        assert_eq!(staker_by_era, expected_tuple5);

        // Test staking in a new era, but with a current non-zero stake value from a previous era
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            2_000_000_000_000
        ));

        // 2_000_000_000_000 should be added on to new stake
        assert_eq!(IpStaking::total_staked(), (3_000_000_000_001, 2_000_000_000_000, 0));

        // 2_000_000_000_000 should be added on to new stake
        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 3_000_000_000_001);
        assert_eq!(new_stake, 2_000_000_000_000);
        assert_eq!(new_unstake, 0);

        let staker_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let expected_tuple6 = Some((Some((2, 3_000_000_000_001)), Some(2_000_000_000_000), None));
        assert_eq!(staker_by_era, expected_tuple6);

        assert_eq!(IpStaking::total_staked(), (3_000_000_000_001, 2_000_000_000_000, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (3_000_000_000_001, 2_000_000_000_000, 0));

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
fn staking_lock_correct() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));

        // Stake 10 tokens to IP set
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            10_000_000_000_000
        ));

        const STAKER_LOCK_ID: LockIdentifier = *b"stakelok";

        let mut lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 10_000_000_000_000);

        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            15_000_000_000_000
        ));

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 25_000_000_000_000);

        run_to_block(2);

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 25_000_000_000_000);

        assert_ok!(IpStaking::unstake_amount(
            Origin::signed(BOB),
            ips_id,
            5_000_000_000_000
        ));

        let mut unlock_era = IpStaking::current_era() + <Test as crate::Config>::UnbondingPeriod::get() + 1;
        assert!(IpStaking::unstake_queue(unlock_era).contains(&BOB));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (25_000_000_000_000, 0, 5_000_000_000_000));

        run_to_block(3);

        // Lock amount should still be the same. UnbondingPeriod is not over yet.
        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 25_000_000_000_000);

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (20_000_000_000_000, 0, 0));

        run_to_block(4);

        assert!(!IpStaking::unstake_queue(unlock_era).contains(&BOB));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (20_000_000_000_000, 0, 0));

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 20_000_000_000_000);

        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            80_000_000_000_000
        ));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (20_000_000_000_000, 80_000_000_000_000, 0));

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 100_000_000_000_000);

        run_to_block(5);

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (100_000_000_000_000, 0, 0));

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 100_000_000_000_000);

        assert_ok!(IpStaking::unstake_all(
            Origin::signed(BOB),
            ips_id
        ));

        unlock_era = IpStaking::current_era() + <Test as crate::Config>::UnbondingPeriod::get() + 1;
        assert!(IpStaking::unstake_queue(unlock_era).contains(&BOB));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (100_000_000_000_000, 0, 100_000_000_000_000));

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 100_000_000_000_000);

        run_to_block(6);

        assert_eq!(IpStaking::ips_stakers(BOB), None);

        lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID).unwrap().amount;
        assert_eq!(lock_amount, 100_000_000_000_000);

        run_to_block(7);
        
        assert!(!IpStaking::unstake_queue(unlock_era).contains(&BOB));

        let final_lock_amount = pallet_balances::Locks::<Test>::get(BOB).into_iter().find(|lock| lock.id == STAKER_LOCK_ID);
        assert_eq!(final_lock_amount, None);


    });
}

#[test]
fn unstaking_from_ips() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));

        // Stake 10 tokens to IP set
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            10_000_000_000_000
        ));

        let mut stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        let mut expected_tuple = Some((None, Some(10_000_000_000_000), None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (0, 10_000_000_000_000, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 10_000_000_000_000, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 10_000_000_000_000);
        assert_eq!(new_unstake, 0);

        run_to_block(2);

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((1, 10_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (10_000_000_000_000, 0, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (10_000_000_000_000, 0, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 10_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        // Unstake 5 tokens from IP set
        assert_ok!(IpStaking::unstake_amount(
            Origin::signed(BOB),
            ips_id,
            5_000_000_000_000
        ));

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((1, 10_000_000_000_000)), None, Some(5_000_000_000_000)));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (10_000_000_000_000, 0, 5_000_000_000_000));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (10_000_000_000_000, 0, 5_000_000_000_000));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 10_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 5_000_000_000_000);

        run_to_block(3);

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((2, 5_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (5_000_000_000_000, 0, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (5_000_000_000_000, 0, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 5_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        assert_ok!(IpStaking::unstake_amount(
            Origin::signed(BOB),
            ips_id,
            1_000_000_000_000
        ));

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((2, 5_000_000_000_000)), Some(1_000_000_000_000), Some(1_000_000_000_000)));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (5_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (5_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 5_000_000_000_000);
        assert_eq!(new_stake, 1_000_000_000_000);
        assert_eq!(new_unstake, 1_000_000_000_000);

        run_to_block(4);

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((3, 5_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (5_000_000_000_000, 0, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (5_000_000_000_000, 0, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 5_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        assert_ok!(IpStaking::unstake_all(
            Origin::signed(BOB),
            ips_id
        ));

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((3, 5_000_000_000_000)), None, Some(5_000_000_000_000)));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (5_000_000_000_000, 0, 5_000_000_000_000));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (5_000_000_000_000, 0, 5_000_000_000_000));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 5_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 5_000_000_000_000);

        run_to_block(5);

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = None;
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (0, 0, 0));

        assert_eq!(IpStaking::ips_stakers(BOB), None);

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        let ips_id_2 = create_ips();
        assert_ok!(register_ips(ips_id_2));

        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            6_000_000_000_000
        ));

        assert_ok!(IpStaking::stake(
            Origin::signed(ALICE),
            ips_id_2,
            10_000_000_000_000
        ));

        assert_ok!(IpStaking::stake(
            Origin::signed(CHARLIE),
            ips_id_2,
            3_000_000_000_000
        ));
        

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((None, Some(6_000_000_000_000), None));
        assert_eq!(stake_by_era, expected_tuple);

        stake_by_era = IpStaking::stake_by_era(ALICE, ips_id_2);
        expected_tuple = Some((None, Some(10_000_000_000_000), None));
        assert_eq!(stake_by_era, expected_tuple);

        stake_by_era = IpStaking::stake_by_era(CHARLIE, ips_id_2);
        expected_tuple = Some((None, Some(3_000_000_000_000), None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (0, 19_000_000_000_000, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (0, 6_000_000_000_000, 0));
        assert_eq!(IpStaking::ips_stakers(ALICE).unwrap(), (0, 10_000_000_000_000, 0));
        assert_eq!(IpStaking::ips_stakers(CHARLIE).unwrap(), (0, 3_000_000_000_000, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 6_000_000_000_000);
        assert_eq!(new_unstake, 0);

        let registerd_ips_obj_2 = IpStaking::registered_ips(ips_id_2).unwrap();
        let total_stake = registerd_ips_obj_2.total_stake;
        let new_stake = registerd_ips_obj_2.next_era_new_stake;
        let new_unstake = registerd_ips_obj_2.next_era_new_unstake;
        assert_eq!(total_stake, 0);
        assert_eq!(new_stake, 13_000_000_000_000);
        assert_eq!(new_unstake, 0);

        run_to_block(6);

        stake_by_era = IpStaking::stake_by_era(BOB, ips_id);
        expected_tuple = Some((Some((5, 6_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        stake_by_era = IpStaking::stake_by_era(ALICE, ips_id_2);
        expected_tuple = Some((Some((5, 10_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        stake_by_era = IpStaking::stake_by_era(CHARLIE, ips_id_2);
        expected_tuple = Some((Some((5, 3_000_000_000_000)), None, None));
        assert_eq!(stake_by_era, expected_tuple);

        assert_eq!(IpStaking::total_staked(), (19_000_000_000_000, 0, 0));

        assert_eq!(IpStaking::ips_stakers(BOB).unwrap(), (6_000_000_000_000, 0, 0));
        assert_eq!(IpStaking::ips_stakers(ALICE).unwrap(), (10_000_000_000_000, 0, 0));
        assert_eq!(IpStaking::ips_stakers(CHARLIE).unwrap(), (3_000_000_000_000, 0, 0));

        let registerd_ips_obj = IpStaking::registered_ips(ips_id).unwrap();
        let total_stake = registerd_ips_obj.total_stake;
        let new_stake = registerd_ips_obj.next_era_new_stake;
        let new_unstake = registerd_ips_obj.next_era_new_unstake;
        assert_eq!(total_stake, 6_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);

        let registerd_ips_obj_2 = IpStaking::registered_ips(ips_id_2).unwrap();
        let total_stake = registerd_ips_obj_2.total_stake;
        let new_stake = registerd_ips_obj_2.next_era_new_stake;
        let new_unstake = registerd_ips_obj_2.next_era_new_unstake;
        assert_eq!(total_stake, 13_000_000_000_000);
        assert_eq!(new_stake, 0);
        assert_eq!(new_unstake, 0);
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
        assert_noop!(IpStaking::unstake_amount(Origin::signed(BOB), ips_id, 999_999_999_999), Error::<Test>::BelowMinUnstakingAmount);
    });
}

#[test]
fn unstake_unregistered_ips() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));

        // Stake 10 tokens to IP set
        assert_ok!(IpStaking::stake(
            Origin::signed(BOB),
            ips_id,
            10_000_000_000_000
        ));

        run_to_block(10);

        

        // TODO: finish!

        // Unstaking less than the MinStakingAmount should fail
        assert_noop!(IpStaking::unstake_amount(Origin::signed(BOB), ips_id, 999_999_999_999), Error::<Test>::BelowMinUnstakingAmount);
    });
}

#[test]
fn unstaking_with_no_stake_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let ips_id = create_ips();
        assert_ok!(register_ips(ips_id));
        assert_noop!(IpStaking::unstake_amount(Origin::signed(BOB), ips_id, 1_000_000_000_000), Error::<Test>::AccountHasNoStake);
        assert_noop!(IpStaking::unstake_all(Origin::signed(BOB), ips_id), Error::<Test>::AccountHasNoStake);
    });
}

// #[test]
// fn claiming() {
//     ExtBuilder::default().build().execute_with(|| {
//         let ips_id = create_ips();
//         assert_ok!(register_ips(ips_id));

//         // Inital supply should be 11.7 million tokens
//         assert_eq!(Balances::total_issuance(), 11_700_000_000_000_000_000); 

//         // BOB stakes 500
//         assert_ok!(IpStaking::stake(
//             Origin::signed(BOB),
//             ips_id,
//             500_000_000_000_000
//         ));

//         // ALICE stakes 2
//         assert_ok!(IpStaking::stake(
//             Origin::signed(ALICE),
//             ips_id,
//             2_000_000_000_000
//         ));

//         let bob_remaining_balance = Balances::free_balance(&BOB);
//         assert_eq!(bob_remaining_balance,  11_699_493_000_000_000_000);

//         run_to_block(3);

//         // ---Now in Era 2 where rewards for Era 1 were just computed---

//         // Claim reward tokens from staking for an entire year
//         assert_ok!(IpStaking::claim(Origin::signed(BOB)));

//         System::assert_last_event(
//             crate::Event::RewardsClaimed {
//                 claimer: BOB,
//                 reward_amount:  1_596_115_537_848_610,
//             }
//             .into(),
//         );

//         let bob_new_balance = Balances::free_balance(&BOB);
//         let bob_staking_reward = bob_new_balance - bob_remaining_balance;

//         // assert_eq!(bob_staking_reward, 0);

//         assert_eq!(Balances::total_issuance(), 1105382683747058);
//     });
// }

// #[test]
// fn inflation_recalculated() {
//     ExtBuilder::default().build().execute_with(|| {
//         assert_eq!(IpStaking::inflation_per_era(), 3_205_000_000_000_000);

//         run_to_block(365);

//         assert_eq!(IpStaking::inflation_per_era(), 3_526_027_397_260_270);
       
//     });
// }

#[test]
fn inflation_minting_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let inflation_acc = PalletId(*b"ia/ipstk").into_account_truncating();
        assert_eq!(Balances::free_balance(&inflation_acc), 0);

        run_to_block(2);
        assert_eq!(Balances::free_balance(&inflation_acc), 3_205_000_000_000_000);

        run_to_block(3);
        assert_eq!(Balances::free_balance(&inflation_acc), 6_410_000_000_000_000);
    });
}


// #[test]
// fn claiming_should_fail() {
//     ExtBuilder::default().build().execute_with(|| {
//         let ips_id = create_ips();
//         assert_ok!(register_ips(ips_id));

//         assert_noop!(IpStaking::claim(Origin::signed(BOB)), Error::<Test>::AccountHasNoClaim);

//         // Stake to IP set
//         assert_ok!(IpStaking::stake(
//             Origin::signed(BOB),
//             ips_id,
//             500_000_000_000_000
//         ));

//         assert_noop!(IpStaking::claim(Origin::signed(BOB)), Error::<Test>::AccountHasNoClaim);

//         run_to_block(2);

//         IpStaking::claim(Origin::signed(BOB));

//         System::assert_last_event(
//             crate::Event::RewardsClaimed {
//                 claimer: BOB,
//                 reward_amount: 0,
//             }
//             .into(),
//         );
//         // assert_noop!(IpStaking::claim(Origin::signed(BOB)), Error::<Test>::AccountHasNoClaim);

//         run_to_block(3);

//         assert_ok!(IpStaking::claim(Origin::signed(BOB)));
//     });
// }


fn create_ips() -> u32 {
    let ips_id = INV4::next_ips_id();
    // assert_eq!(ips_id, 0);

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

fn unregister_ips(ips_id: u32) -> Result<(), ()> {
    assert_ne!(INV4::ips_storage(ips_id), None);

    // Register IP set for IP staking
    let call = Call::IpStaking(IpStakingCall::unregister { ips_id });
    assert_ok!(INV4::operate_multisig(
        Origin::signed(ALICE),
        false,
        (ips_id, None),
        Box::new(call)
    ));

    assert_eq!(IpStaking::registered_ips(ips_id), None);

    Ok(())
}