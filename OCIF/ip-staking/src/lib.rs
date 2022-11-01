//! # IP Staking FRAME Pallet.

//! Intellectual Property Staking
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//! This pallet demonstrates how to stake and unstake IP.
//!
//! ### Pallet Functions
//!
//! - `register` -
//! - `unregister` -
//! - `bond_and_stake` -
//! - `unbond_and_unstake` -
//! - `withdraw_unbonded` -
//! - `claim` -
//! - `force_new_era` -

#![cfg_attr(not(feature = "std"), no_std)]

use scale_info::prelude::fmt::Display;
use sp_runtime::{
    print,
    traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One, Zero},
    Perbill,
};

use primitives::{ocif::IpsStakeInfo, utils::*, Parentage};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use core::iter::Sum;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
        pallet_prelude::{
            DispatchResult, OptionQuery, StorageDoubleMap, StorageMap, StorageValue, ValueQuery, *,
        },
        traits::{
            fungible::{Inspect, Mutate},
            Currency, LockableCurrency, ReservableCurrency,
            ExistenceRequirement::AllowDeath
        },
        Blake2_128Concat, BoundedBTreeSet, BoundedVec, PalletId,
    };
    use frame_system::{pallet_prelude::*, Origin};
    use pallet_staking::EraPayout;


    // pub type BalanceOf<T> = <T as pallet::Config>::Balance;
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type IpsIdOf<T> = <T as pallet_inv4::Config>::IpId;
    pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
    pub type IpsStakeInfoOf<T> = IpsStakeInfo<AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>>;
    pub type Era = u32;

    // Index 1: Current era active stake
    // Index 2: New stake to be added next era
    // Index 3: New unstake to be subtracted next era
    pub type EraStake<T> = (Option<(Era, BalanceOf<T>)>, Option<BalanceOf<T>>, Option<BalanceOf<T>>);

    use pallet_inv4::{self as inv4};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_inv4::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The IPS ID type
        type IpsId: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + Display
            + MaxEncodedLen
            + Clone;

        /// Get access to the balances pallet
        type Currency: Currency<Self::AccountId>
            + LockableCurrency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + Mutate<Self::AccountId>
            + frame_support::traits::fungible::Inspect<Self::AccountId>;

        type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Sum<<Self as pallet::Config>::Balance>
            + IsType<<Self as pallet_balances::Config>::Balance>
            + From<u128>
            + From<
                <<Self as pallet::Config>::Currency as Currency<
                    <Self as frame_system::Config>::AccountId,
                >>::Balance,
            >;

        /// Provides access to the `era_payout()` function which determines how much should be paid out to stakers per era
        // type EraPayout: EraPayout<<<Self as Config>::Currency as Currency<<Self as frame_system::Config>::AccountId>>::Balance>;

        /// The IP Staking pallet id, used for deriving its sovereign account ID.
        /// Tokens from inflation will be minted to here before they are claimed by members of the staking system.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// To deter the waste of chain storage, require a reasonable deposit to register an IPS
        #[pallet::constant]
        type IpsRegisterDeposit: Get<<Self as pallet::Config>::Balance>;

        /// To deter the waste of chain storage, set a reasonable minimum staking amount
        #[pallet::constant]
        type MinStakingAmount: Get<BalanceOf<Self>>;

        /// Used for the EraPayout::era_payout() function which determines the # of tokens to mint per era (inflation)
        // #[pallet::constant]
        // type MillisecondsPerEra: Get<u64>;

        /// The number of blocks per era. The lower the #, the more chain storage and computation will increase per a given time period
        #[pallet::constant]
        type BlocksPerEra: Get<u32>;

        /// The number of blocks per year.
        #[pallet::constant]
        type BlocksPerYear: Get<u32>;

        /// The number of eras before an account gets its tokens back after calling unstake
        #[pallet::constant]
        type UnbondingPeriod: Get<Era>;

        /// Max # of IP sets that an account can be staked to at once. Prevents state and computation bloat.
        #[pallet::constant]
        type MaxUniqueStakes: Get<u8>;

        /// Inflation rate for the whole IP Staking system
        #[pallet::constant]
        type IpStakingInflationRate: Get<Perbill>;

        /// The percentage of inflation that is allocated for registered IP sets
        #[pallet::constant]
        type IpsInflationPercentage: Get<Perbill>;

        /// The percentage of inflation that is allocated for accounts staking on IP sets.
        #[pallet::constant]
        type StakerInflationPercentage: Get<Perbill>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub trait Store)]
    pub struct Pallet<T>(_);

    /// The number of tokens staked in the system. In other words, the sum of the tokens staked to each IP set, for all IP sets.
    /// 1st balance: Total system stake during current era
    /// 2nd balance: New stake during this era to add to 1st balance (total system stake) at beginning of next era
    /// 3rd balance: New unstake during this era to subtract from 1st balance (total system stake) at beginning of next era
    #[pallet::storage]
    #[pallet::getter(fn total_staked)]
    pub type TotalStaked<T> = StorageValue<_, (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>), ValueQuery>;

    /// Keeps track of the current era. Staking rewards are calculated per era.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, u32, ValueQuery>;

    /// Keeps track of the most recent block where staking rewards were calculated
    #[pallet::storage]
    #[pallet::getter(fn last_payout_block)]
    pub type LastPayoutBlock<T> = StorageValue<_, BlockNumberOf<T>, ValueQuery>;

    /// Keeps track of which IP sets are registered for the IP staking system
    #[pallet::storage]
    #[pallet::getter(fn registered_ips)]
    pub type RegisteredIps<T> = StorageMap<_, Blake2_128Concat, IpsIdOf<T>, IpsStakeInfoOf<T>>;

    /// Keeps track of which accounts are staking in the IP staking system and what the accounts total stake is across all IP sets they are staked to.
    /// 1st balance of the tuple is the accounts total stake in the system that the rewards will be calculated from at the beginning of the next era.
    /// 2nd balance of the tuple is new stake, if any, and will be added to the accounts total stake (1st half of the tuple) after rewards for
    /// era x are calculated at the very beginning of era x+1.
    /// 3rd balance: Keep track of new unstakes similar to new stake
    #[pallet::storage]
    #[pallet::getter(fn ips_stakers)]
    pub type IpsStakers<T: Config> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>), OptionQuery>;

    /// Keeps track of how much, if any, a given account is staking towards a given IP set, per era
    #[pallet::storage]
    #[pallet::getter(fn stake_by_era)]
    pub type StakeByEra<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AccountIdOf<T>,
        Blake2_128Concat,
        IpsIdOf<T>,
        EraStake<T>,
        OptionQuery,
    >;

    /// Keeps track of staking rewards earned by a given account. Balance is reset to 0 after a user calls claim()
    #[pallet::storage]
    #[pallet::getter(fn rewards_claimable)]
    pub type RewardsClaimable<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;

    /// Balance is the inflation to be minted at the beginning of every era during a year. 
    /// This balance is recalculated every T::BlocksPerYear blocks
    #[pallet::storage]
    #[pallet::getter(fn inflation_per_era)]
    pub type InflationPerEra<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The last block where InflationPerEra was recalculated
    #[pallet::storage]
    #[pallet::getter(fn last_inflation_recalc_block)]
    pub type LastInflationRecalcBlock<T> = StorageValue<_, BlockNumberOf<T>, ValueQuery>;

    // Set up initial storage values when chain starts up the first time
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub total_staked: (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
        pub current_era: Era,
        pub last_payout_block: BlockNumberOf<T>,
        pub inital_inflation_per_era: BalanceOf<T>,
        pub last_inflation_recalc_block: BlockNumberOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                total_staked: (Default::default(), Default::default(), Default::default()),
                current_era: Default::default(),
                last_payout_block: Default::default(),
                inital_inflation_per_era: Default::default(),
                last_inflation_recalc_block: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <TotalStaked<T>>::put(&self.total_staked);
            <CurrentEra<T>>::put(&self.current_era);
            <LastPayoutBlock<T>>::put(&self.last_payout_block);
            <InflationPerEra<T>>::put(&self.inital_inflation_per_era);
            <LastInflationRecalcBlock<T>>::put(frame_system::Pallet::<T>::block_number());

            // for (a, b) in &self.account_map {
            // 	<AccountMap<T>>::insert(a, b);
            // }
        }
    }

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        IpsRegistered {
            ips_id: IpsIdOf<T>,
        },
        InflationEvent {
            era: Era,
            inflation_pot: AccountIdOf<T>,
            inflation_amount: BalanceOf<T>,
        },
        NewStake {
            staker: AccountIdOf<T>,
            ips_id: IpsIdOf<T>,
            stake_amount: BalanceOf<T>,
        },
        Unstake {
            staker: AccountIdOf<T>,
            ips_id: IpsIdOf<T>,
            unstake_amount: BalanceOf<T>,
        },
        RewardsClaimed {
            claimer: AccountIdOf<T>,
            reward_amount: BalanceOf<T>,
        },
        NewDailyInflationRate {
            amount: BalanceOf<T>,
        }
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// IP set does not exist
        IpDoesntExist,
        /// Account does not have permission to complete this action
        NoPermission,
        /// IP staking is only allowed on top level parent IP sets
        NotParent,
        /// Datatype holding era has maxed out. Should never happen.
        NoAvailableEra,
        /// Data type overflow
        Overflow,
        /// Value below minimum stake amount
        BelowMinStakingAmount,
        /// Error adding account as staker to IP set
        FailedAddingStaker,
        /// IP set is not registered for staking
        IpsNotRegistered,
        /// Calling account does not have enough free balance
        NotEnoughFreeBalance,
        /// Account has less tokens staked than it is trying to unstake
        UnstakeValueGreaterThanStakedAmount,
        /// Account has no stake on this IP set or any IP set
        AccountHasNoStake,
        /// Cannot unstake less than `MinStakingAmount`
        BelowMinUnstakingAmount,
        /// Unstake amount will result in accounts staked balance going below `MinStakingAmount`
        StakingAmountTooLow,
        /// StakeByEra record should have been deleted and was not for some reason
        RecordNotDeleted,
        /// Account is already staked to the max allowed # of IP sets (MaxUniqueStakes)
        MaxStakesAlreadyReached,
        /// Account tries to claim rewards, but doesn't have any rewards to claim
        AccountHasNoClaim,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Runs before any extrinsics in block `n`
        fn on_initialize(n: T::BlockNumber) -> Weight {
            let blocks_per_era: T::BlockNumber = <T as Config>::BlocksPerEra::get().into();

            // Activates once at first block of each era
            if n != Zero::zero() && (n % blocks_per_era).is_zero() {
                // Get previous era
                let prev_era = Self::increment_era();
                let total_system_stake = Self::total_staked();

                let (ips_era_inflation, staker_era_inflation) = Self::mint_inflation(prev_era, total_system_stake.0);

                Self::calculate_ips_rewards(ips_era_inflation, total_system_stake.0);
                Self::calculate_staker_rewards(staker_era_inflation, total_system_stake.0);
                Self::update_total_system_stake(total_system_stake);
            }

            // Calculate the inflation per era for the new year (block based, not calendar based)
            if n - Self::last_inflation_recalc_block() == T::BlocksPerYear::get().into() {
                let total_supply: BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<T::AccountId>>::total_issuance();

                // Inflation for the next year
                let one_year_inflation = T::IpStakingInflationRate::get() * total_supply;

                // A single Eras inflation
                let one_day_inflation = one_year_inflation / T::BlocksPerYear::get().into();

                // Update storage
                InflationPerEra::<T>::put(one_day_inflation);
                LastInflationRecalcBlock::<T>::put(n);

                Self::deposit_event(Event::<T>::NewDailyInflationRate {
                    amount: one_day_inflation,
                });
            }

            // to get rid of error for now
            // TODO: Add weight

            // TODO: Add check for inflation to make sure it doesn't go higher than supposed to. Talk to Gabe

            100
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register an IPS for IP Staking
        #[pallet::weight(1)]
        pub fn register(origin: OriginFor<T>, ips_id: T::IpId) -> DispatchResultWithPostInfo {
            let innovator = ensure_signed(origin)?;

            // Ensure IPS exists
            let ips = inv4::Pallet::<T>::ips_storage(ips_id).ok_or(Error::<T>::IpDoesntExist);

            // Ensure that `innovator` is the IPS owner. Register can only be called through IPS multisig call
            let derived_address = multi_account_id::<T, T::IpId>(ips_id.clone(), None);
            ensure!(innovator == derived_address, Error::<T>::NoPermission);

            // TODO: Finish checks. Gabe said parentage check is not needed

            // Ensure IPS is top level i.e. a Parentage::Parent variant
            // match ips.unwrap().parentage.clone() {
            // 	Parent => ensure!(false, Error::<T>::NotParent),
            // 	Child => ensure!(false, Error::<T>::NotParent)
            // };

            // match ips.unwrap().parentage.clone() {
            //     Parentage::<<T as frame_system::Config>::AccountId, <T as pallet_inv4::Config>::IpId>::Parent(_) => ensure!(false, Error::<T>::NotParent),
            //     Parentage::Child(_, _) => ensure!(false, Error::<T>::NotParent)
            // };

            let current_block_number = frame_system::Pallet::<T>::block_number();

            // Register IPS
            let registered_ips: IpsStakeInfoOf<T> = IpsStakeInfo {
                address: derived_address,
                total_stake: Zero::zero(),
                next_era_new_stake: Zero::zero(),
                next_era_new_unstake: Zero::zero(),
                block_registered_at: current_block_number,
            };

            // TODO: Add the register deposit

            // Update storage
            RegisteredIps::<T>::insert(ips_id, registered_ips);

            Self::deposit_event(Event::<T>::IpsRegistered { ips_id });

            Ok(().into())
        }

        /// Unregister an IPS for IP Staking
        #[pallet::weight(1)]
        pub fn unregister(origin: OriginFor<T>, ips_id: T::IpId) -> DispatchResultWithPostInfo {
            let innovator = ensure_signed(origin)?;

            Ok(().into())
        }

        /// Stake towards an IPS. Staking of new funds will begin at the current era + 1
        #[pallet::weight(1)]
        pub fn stake(
            origin: OriginFor<T>,
            ips_id: T::IpId,
            stake_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            // Ensure IPS is registered for staking
            ensure!(
                Self::registered_ips(ips_id).is_some(),
                Error::<T>::IpsNotRegistered
            );

            // Ensure account has enough funds to stake this stake_amount
            let free_balance =
                <<T as pallet::Config>::Currency as Currency<T::AccountId>>::free_balance(&staker);
            ensure!(free_balance > stake_amount, Error::<T>::NotEnoughFreeBalance);

            // Ensure account is staking above set minimum
            ensure!(
                stake_amount >= <T as Config>::MinStakingAmount::get(),
                Error::<T>::BelowMinStakingAmount
            );

            // Ensure account is not already staked to the max # of IP sets (MaxUniqueStakes)
            let count_ips_staked_to = StakeByEra::<T>::iter_prefix_values(staker.clone()).count();
            ensure!(
                count_ips_staked_to <= usize::from(<T as Config>::MaxUniqueStakes::get()),
                Error::<T>::MaxStakesAlreadyReached
            );

            // Update storage
            // IpsStakers::<T>::try_mutate(ips_id, |set| {
            // 	set.try_insert(staker.clone())
            // }).map_err(|_| Error::<T>::FailedAddingStaker)?;

            // 1. If account has no stake record on IPS
            // 		- push on tuple (current era + 1, stake)
            // 2. Else If account already has stake record (even 0 stake) on IPS
            //  	- pop off tuple from StakeByEra and update staking value (+ for stake, - for unstake)
            //		- Push on tuple (current era + 1, updated stake)

            let current_era = Self::current_era();

            // Update staking info
            StakeByEra::<T>::try_mutate(staker.clone(), ips_id, |era_stake| -> DispatchResult {
                match era_stake.take() {
                    // Account has stake on this IPS
                    Some(tuple) => {
                        let current_era_stake = tuple.0;
                        let next_era_stake = tuple.1;
                        let next_era_unstake = tuple.2;

                        match next_era_stake {
                            // Account has already called stake during `current_era`. Update stake amount for next era
                            Some(existing_new_stake) => {
                                let updated_next_era_stake = existing_new_stake + stake_amount;
                                *era_stake =
                                    Some((current_era_stake, Some(updated_next_era_stake), next_era_unstake));
                            }
                            // Account has not called stake during `current_era`
                            None => {
                                *era_stake =
                                    Some((current_era_stake, Some(stake_amount), next_era_unstake));
                            }
                        }
                    }
                    // Account has no stake on this IPS
                    None => {
                        *era_stake = Some((None, Some(stake_amount), None));
                    }
                }

                Ok(())
            })?;

            // Update accounts total system stake
            if Self::ips_stakers(staker.clone()).is_some() {
                IpsStakers::<T>::try_mutate(staker.clone(), |total_stake| -> DispatchResult {
                    let mut updated_stake =
                        total_stake.take().ok_or(Error::<T>::AccountHasNoStake)?;
                    updated_stake.1 = updated_stake.1 + stake_amount;
                    // let updated_stake = old_stake.checked_add(&stake_amount).ok_or(Error::<T>::Overflow)?;
                    *total_stake = Some(updated_stake);
                    Ok(())
                })?;
            } else {
                let zero: BalanceOf<T> = Zero::zero();
                IpsStakers::<T>::insert(staker.clone(), (zero, stake_amount, zero)); //.ok_or(Error::<T>::FailedAddingStaker)?;
            }

            // Update IpsStakeInfo struct with correct total_stake
            RegisteredIps::<T>::try_mutate(ips_id, |ips| -> DispatchResult {
                let mut ips_obj = ips.take().ok_or(Error::<T>::IpsNotRegistered)?;      
                let updated_add = ips_obj.next_era_new_stake.checked_add(&stake_amount).ok_or(Error::<T>::Overflow)?;
                ips_obj.next_era_new_stake = updated_add;
                *ips = Some(ips_obj);
                Ok(())
            })?;

            // Update the amount of new stake to add to the total system stake at beginning of next era
            let mut new_total_system_stake = TotalStaked::<T>::get();
            new_total_system_stake.1 = new_total_system_stake.1
                .checked_add(&stake_amount)
                .ok_or(Error::<T>::Overflow)?;
            TotalStaked::<T>::put(new_total_system_stake);

            // TODO: Change to lock instead of reserve
            // Reserve accounts tokens they are staking
            <T as Config>::Currency::reserve(&staker, stake_amount)?;

            Self::deposit_event(Event::<T>::NewStake {
                staker: staker,
                ips_id: ips_id,
                stake_amount: stake_amount,
            });

            Ok(().into())
        }

        /// Unstake a specific amount of tokens from an IPS for a given account
        #[pallet::weight(1)]
        pub fn unstake_amount(origin: OriginFor<T>, ips_id: T::IpId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
        	let staker = ensure_signed(origin)?;

        	Self::unstake(staker, ips_id, amount)
        }

        // Unstake all tokens from an IPS for a given account
        #[pallet::weight(1)]
        pub fn unstake_all(origin: OriginFor<T>, ips_id: T::IpId) -> DispatchResultWithPostInfo {
        	let staker = ensure_signed(origin)?;

        	// Get stakers total stake. If no stake then error
        	let staked_amount = match Self::stake_by_era(staker.clone(), ips_id) {
        		Some(era_stake) => {
                    match era_stake.0 {
                        Some(active_stake_tuple) => {
                            // Return accounts current stake
                            active_stake_tuple.1
                        }
                        // Shouldn't happen
                        None => {
                            return Err(Error::<T>::AccountHasNoStake.into());
                        }
                    }
        		}
        		// Account has no stake so return 0
        		None => {
        			return Err(Error::<T>::AccountHasNoStake.into());
        		}
        	};

        	Self::unstake(staker, ips_id, staked_amount)
        }

        /// Claim tokens earned from IP staking
        #[pallet::weight(1)]
        pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let claimer = ensure_signed(origin)?;

            let reward_amount = Self::rewards_claimable(claimer.clone());

            if reward_amount > Zero::zero() {
                RewardsClaimable::<T>::try_mutate(claimer.clone(), |reward| -> DispatchResult {
                    let claimed_reward = *reward;
                    *reward = Zero::zero();

                    <T as Config>::Currency::transfer(&Self::account_id(), &claimer.clone(), claimed_reward, AllowDeath)?;

                    Ok(())
                })?;
            }
            else {
                return Err(Error::<T>::AccountHasNoClaim.into());
            }

            Self::deposit_event(Event::<T>::RewardsClaimed{
                claimer,
                reward_amount,
            });

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn unstake(staker: AccountIdOf<T>, ips_id: T::IpId, unstake_amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
        	// Ensure IPS is registered for staking
        	ensure!(Self::registered_ips(ips_id).is_some(), Error::<T>::IpsNotRegistered);

        	// Ensure `unstake_amount` is at least `MinStakingAmount`
        	ensure!(unstake_amount >= <T as Config>::MinStakingAmount::get(), Error::<T>::BelowMinUnstakingAmount);

        	// Ensure account has enough tokens staked on given IPS to unstake that much
        	let staked_amount = match Self::stake_by_era(staker.clone(), ips_id) {
        		Some(era_stake) => {
                    match era_stake.0 {
                        Some(active_stake_tuple) => {
                            // Return accounts current stake
                            active_stake_tuple.1
                        }
                        // Shouldn't happen
                        None => {
                            return Err(Error::<T>::AccountHasNoStake.into());
                        }
                    }
        		}
        		// Account has no stake so return 0
        		None => {
        			return Err(Error::<T>::AccountHasNoStake.into());
        		}
        	};
        	ensure!(unstake_amount <= staked_amount, Error::<T>::UnstakeValueGreaterThanStakedAmount);

        	// Ensure staking amount stays in valid range. Must either be above MinStakingAmount or be 0
        	ensure!(staked_amount - unstake_amount >= <T as Config>::MinStakingAmount::get() || staked_amount - unstake_amount == Zero::zero(), Error::<T>::StakingAmountTooLow);




            let current_era = Self::current_era();

            // Update staking info
            StakeByEra::<T>::try_mutate(staker.clone(), ips_id, |era_stake| -> DispatchResult {
                match era_stake.take() {
                    // Account has stake on this IPS
                    Some(tuple) => {
                        let current_era_stake = tuple.0;
                        let next_era_stake = tuple.1;
                        let next_era_unstake = tuple.2;

                        match next_era_unstake {
                            // Account has already called unstake during `current_era`. Update unstake amount for next era
                            Some(existing_new_unstake) => {
                                let updated_next_era_unstake = existing_new_unstake + unstake_amount;
                                *era_stake =
                                    Some((current_era_stake, next_era_stake, Some(updated_next_era_unstake)));
                            }
                            // Account has not called unstake during `current_era`
                            None => {
                                *era_stake = Some((current_era_stake, next_era_stake, Some(unstake_amount)));
                            }
                        }
                    }
                    // Account has never staked to this IPS before
                    None => {
                        return Err(Error::<T>::AccountHasNoStake.into());
                    }
                }

                Ok(())
            })?;

            // Update accounts total system stake
            IpsStakers::<T>::try_mutate(staker.clone(), |total_stake| -> DispatchResult {
                let mut updated_stake =
                    total_stake.take().ok_or(Error::<T>::AccountHasNoStake)?;
                updated_stake.2 = updated_stake.2 + unstake_amount;
                // let updated_stake = old_stake.checked_add(&stake_amount).ok_or(Error::<T>::Overflow)?;
                *total_stake = Some(updated_stake);
                Ok(())
            })?;

            // Update IpsStakeInfo struct with correct total_stake
            RegisteredIps::<T>::try_mutate(ips_id, |ips| -> DispatchResult {
                let mut ips_obj = ips.take().ok_or(Error::<T>::IpsNotRegistered)?;      
                let updated_add = ips_obj.next_era_new_unstake.checked_add(&unstake_amount).ok_or(Error::<T>::Overflow)?;
                ips_obj.next_era_new_unstake = updated_add;
                *ips = Some(ips_obj);
                Ok(())
            })?;

            // Update the amount of new stake to add to the total system stake at beginning of next era
            let mut new_total_system_stake = TotalStaked::<T>::get();
            new_total_system_stake.2 = new_total_system_stake.2
                .checked_add(&unstake_amount)
                .ok_or(Error::<T>::Overflow)?;
            TotalStaked::<T>::put(new_total_system_stake);

            // TODO: Change to lock instead of reserve
            // Reserve accounts tokens they are staking
            // <T as Config>::Currency::reserve(&staker, stake_amount)?;

            Self::deposit_event(Event::<T>::Unstake {
                staker: staker,
                ips_id: ips_id,
                unstake_amount: unstake_amount,
            });

        	Ok(().into())
        }

        /// Return the current era #, and then increment it by 1
        fn increment_era() -> Era {
            let mut old_era = 0;
            let _ = CurrentEra::<T>::try_mutate(|era| -> DispatchResult {
                old_era = *era;
                *era = era
                    .checked_add(One::one())
                    .ok_or(Error::<T>::NoAvailableEra)?;

                Ok(())
            });

            old_era
        }

        /// Get the IP staking pallet pot account ID
        fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        // Calculate inflation and mint it to the IP staking pallet pot
        fn mint_inflation(era: Era, total_system_stake: BalanceOf<T>) -> (BalanceOf<T>, BalanceOf<T>) {
            // Compute inflation for previous era
            
            
            // ---Old method with variable inflation---
            // let (to_mint, extra) = <T as Config>::EraPayout::era_payout(
            //     total_system_stake,
            //     total_issuance,
            //     <T as Config>::MillisecondsPerEra::get(),
            // );
            // let total = to_mint + extra;

            let total = Self::inflation_per_era();

            // Mint tokens (inflation) to inflation pot
            let inflation_pot: T::AccountId = Self::account_id();
            <T as Config>::Currency::deposit_creating(&inflation_pot, total.clone());

            Self::deposit_event(Event::<T>::InflationEvent {
                era,
                inflation_pot,
                inflation_amount: total,
            });

            // Update storage
            let current_block_number = frame_system::Pallet::<T>::block_number();
            LastPayoutBlock::<T>::put(current_block_number);

            // Calculate token inflation breakdown for IP sets and IP stakers
            let ips_era_inflation = T::IpsInflationPercentage::get() * total;
            let staker_era_inflation = T::StakerInflationPercentage::get() * total;

            (ips_era_inflation, staker_era_inflation)
        }

        fn calculate_ips_rewards(
            ips_era_inflation: BalanceOf<T>,
            total_system_stake: BalanceOf<T>,
        ) -> DispatchResult {
            // Calculate rewards for registered IP sets and then update IPS total stakes for current era
            let registered_ips_iter = RegisteredIps::<T>::iter();
            for (_, (ips_id, ips_stake_info)) in registered_ips_iter.enumerate() {
                let ips_total_stake = ips_stake_info.total_stake;
                let account = ips_stake_info.address;

                // Only earn rewards if a non-zero amount of tokens have been staked to this IP Set
                if ips_total_stake > Zero::zero() {
                    // IPS percentage of total system stake (TotalStaked)
                    let ips_percentage = Perbill::from_rational(ips_total_stake, total_system_stake);
                    
                    // Actual token rewards that will be added to the IP Set accounts claimable rewards
                    let era_rewards = ips_percentage * ips_era_inflation;

                    // Update storage
                    let new_claimable_balance = Self::rewards_claimable(account.clone())
                        .checked_add(&era_rewards)
                        .ok_or(Error::<T>::Overflow)?;
                    RewardsClaimable::<T>::set(account.clone(), new_claimable_balance);
                }

                // ---- TAKE NOTE: REWARDS MUST BE CALCULATED BEFORE NEW STAKES/UNSTAKES ARE ADDED/SUBTRACTED FROM TOTALS ----

                // Update IPS total stake
                RegisteredIps::<T>::try_mutate(ips_id, |ips| -> DispatchResult {
                    let mut ips_obj = ips.take().ok_or(Error::<T>::IpsNotRegistered)?;
                    
                    // Add new stake to total_stake
                    let updated_stake_add = ips_obj.total_stake.checked_add(&ips_obj.next_era_new_stake).ok_or(Error::<T>::Overflow)?;

                    // Subtract unstake from total_stake
                    ips_obj.total_stake = updated_stake_add - ips_obj.next_era_new_unstake;

                    // Reset values
                    ips_obj.next_era_new_stake = Zero::zero();
                    ips_obj.next_era_new_unstake = Zero::zero();

                    *ips = Some(ips_obj);
                    Ok(())
                })?;
            }

            Ok(())
        }

        fn calculate_staker_rewards(staker_era_inflation: BalanceOf<T>, total_system_stake: BalanceOf<T>) -> DispatchResult {
            // Calculate rewards for IP stakers and then update account total stakes for current era
            let ips_stakers_iter = IpsStakers::<T>::iter();
            for (_, (account, (account_total_stake, new_stake, new_unstake))) in ips_stakers_iter.enumerate() {
                // Calculate accounts claimable rewards and update storage
                // Accounts percentage of total system stake (TotalStaked)
                let stakers_percentage = Perbill::from_rational(account_total_stake, total_system_stake);
                
                // Actual token rewards that will be added to the users claimable rewards
                let era_rewards = stakers_percentage * staker_era_inflation;

                // Update storage
                let new_claimable_balance = Self::rewards_claimable(account.clone())
                    .checked_add(&era_rewards)
                    .ok_or(Error::<T>::Overflow)?;
                RewardsClaimable::<T>::set(account.clone(), new_claimable_balance);

                // ---- TAKE NOTE: REWARDS MUST BE CALCULATED BEFORE NEW STAKES/UNSTAKES ARE ADDED/SUBTRACTED FROM TOTALS ----

                // Set updated total stake for account
                let updated_total_stake = account_total_stake + new_stake - new_unstake;
                let zero: BalanceOf<T> = Zero::zero();
                IpsStakers::<T>::set(account, Some((updated_total_stake, zero, zero)));
            }

            // Update StakeByEra records for IP Stakers
            let era = Self::current_era();
            let stake_by_era_iter = StakeByEra::<T>::iter_keys();
            for (_, (account, ips_id)) in stake_by_era_iter.enumerate() {
                StakeByEra::<T>::try_mutate(account, ips_id, |era_stake| -> DispatchResult {
                    // let old_era_stake = era_stake.take().ok_or(Error::<T>::AccountHasNoStake)?;

                    if let Some(old_era_stake) = era_stake.take() {
                        if old_era_stake.1.is_some() || old_era_stake.2.is_some() {
                            let mut old_active_stake = Zero::zero();
                            let mut new_stake = Zero::zero();
                            let mut new_unstake = Zero::zero();

                            if let Some(tuple) = old_era_stake.0 {
                                old_active_stake = tuple.1;
                            };

                            if let Some(stake_amount) = old_era_stake.1 {
                                new_stake = stake_amount;
                            };

                            if let Some(unstake_amount) = old_era_stake.2 {
                                new_unstake = unstake_amount;
                            };

                            let new_stake_amount = old_active_stake + new_stake - new_unstake;
                            let new_active_stake = Some((era, new_stake_amount));
                            let new_era_stake: EraStake<T> = (new_active_stake, None, None);
                            *era_stake = Some(new_era_stake);
                        }
                        else {
                            *era_stake = Some(old_era_stake);
                        }
                    };
                    // pub type EraStake<T> = (Option<(Era, BalanceOf<T>)>, Option<BalanceOf<T>>, Option<BalanceOf<T>>);
                    
                    Ok(())
                })?;
            }

            Ok(())
        }

        fn update_total_system_stake(total_system_stake: (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>)) {
            let (old_total_stake, new_stake, new_unstake) = total_system_stake;
            let new_total_stake = old_total_stake + new_stake - new_unstake;
            let zero: BalanceOf<T> = Zero::zero();
            TotalStaked::<T>::put((new_total_stake, zero, zero));
        }
    }
}
