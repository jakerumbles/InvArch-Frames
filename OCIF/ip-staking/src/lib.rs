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

use sp_runtime::{print, traits::{AtLeast32BitUnsigned, Zero, One, CheckedAdd, AccountIdConversion}};
use scale_info::prelude::fmt::Display;

use primitives::{Parentage, ocif::IpsStakeInfo, utils::*};

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
	use frame_support::{dispatch::{DispatchResultWithPostInfo, DispatchErrorWithPostInfo}, pallet_prelude::{*, StorageMap, StorageDoubleMap, ValueQuery, OptionQuery, StorageValue, DispatchResult}, 
			traits::{Currency, ReservableCurrency, LockableCurrency, fungible::{Mutate, Inspect}}, PalletId, Blake2_128Concat, BoundedVec};
	use frame_system::pallet_prelude::*;
	use core::iter::Sum;
	use pallet_staking::{EraPayout};
	use sp_std::vec::Vec;

	// pub type BalanceOf<T> = <T as pallet::Config>::Balance;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type IpsIdOf<T> = <T as pallet_inv4::Config>::IpId;
	pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
	pub type IpsStakeInfoOf<T> = IpsStakeInfo<BalanceOf<T>, BlockNumberOf<T>>;
	pub type Era = u32;

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
		type Currency: Currency<Self::AccountId> + LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId> + Mutate<Self::AccountId> + frame_support::traits::fungible::Inspect<Self::AccountId>;

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
			+ From<<<Self as pallet::Config>::Currency as Currency<<Self as frame_system::Config>::AccountId>>::Balance>;

		/// Provides access to the `era_payout()` function which determines how much should be paid out to stakers per era
		type EraPayout: EraPayout<<<Self as Config>::Currency as Currency<<Self as frame_system::Config>::AccountId>>::Balance>;
		// type EraPayout: EraPayout<<Self as Config>::Balance>;

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
		// type MinStakingAmount: Get<<Self as pallet::Config>::Balance>;

		/// Used for the EraPayout::era_payout() function which determines the # of tokens to mint per era (inflation)
		#[pallet::constant]
		type MillisecondsPerEra: Get<u64>;

		/// The number of blocks per era. The lower the #, the more chain storage and computation will increase per a given time period
		#[pallet::constant]
		type BlocksPerEra: Get<u32>;

		/// The number of eras before an account gets its tokens back after calling unstake
		#[pallet::constant]
		type UnbondingPeriod: Get<Era>;

		/// This is the maximum number of accounts that can stake towards an IP set in any given era
		#[pallet::constant]
		type MaxStakersPerIps: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub trait Store)]
	pub struct Pallet<T>(_);

	/// The number of tokens staked in the system. In other words, the sum of the tokens staked to each IP set, for all IP sets.
	#[pallet::storage]
	#[pallet::getter(fn total_staked)]
	pub type TotalStaked<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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

	/// Keeps track of which accounts are staking towards a given IP set
	#[pallet::storage]
	#[pallet::getter(fn ips_stakers)]
	pub type IpsStakers<T: Config> = StorageMap<_, Blake2_128Concat, IpsIdOf<T>, BoundedVec<AccountIdOf<T>, T::MaxStakersPerIps>, ValueQuery>;

	/// Keeps track of how much, if any, a given account is staking towards a given IP set, per era
	#[pallet::storage]
	#[pallet::getter(fn stake_by_era)]
	pub type StakeByEra<T: Config> = StorageDoubleMap<_, Blake2_128Concat, IpsIdOf<T>, Blake2_128Concat, AccountIdOf<T>, BoundedVec<(Era, BalanceOf<T>), T::MaxStakersPerIps>, ValueQuery>;


	// Set up initial storage values when chain starts up the first time
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub total_staked: BalanceOf<T>,
		pub current_era: Era,
		pub last_payout_block: BlockNumberOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { 
				total_staked: Default::default(), 
				current_era: Default::default(), 
				last_payout_block: Default::default() 
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<TotalStaked<T>>::put(&self.total_staked);
			<CurrentEra<T>>::put(&self.current_era);
			<LastPayoutBlock<T>>::put(&self.last_payout_block);
			// for (a, b) in &self.account_map {
			// 	<AccountMap<T>>::insert(a, b);
			// }
		}
	}

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IpsRegistered { ips_id: IpsIdOf<T> },
		InflationEvent { inflation_pot: AccountIdOf<T>, inflation_amount: BalanceOf<T> },
		NewStake { staker: AccountIdOf<T>, ips_id: IpsIdOf<T>, stake_amount: BalanceOf<T> },
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
		BelowMinAmount,
		/// Error adding account as staker to IP set
		FailedAddingStaker,
		/// IP set is not registered for staking
		IpsNotRegistered,
		/// Calling account does not have enough free balance
		NotEnoughFreeBalance
	}

	#[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
			let blocks_per_era: T::BlockNumber = <T as Config>::BlocksPerEra::get().into();

			// Activates once at first block of each era
            if n != Zero::zero() && (n % blocks_per_era).is_zero() {
				// Get previous era
				let prev_era = Self::increment_era();
				
				// Compute inflation for previous era
				let total_staked: BalanceOf<T> = Self::total_staked();
				let total_issuance: BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<T::AccountId>>::total_issuance();
				let (to_mint, extra) =
					<T as Config>::EraPayout::era_payout(total_staked, total_issuance, <T as Config>::MillisecondsPerEra::get());
				let total = to_mint + extra;

				// Mint tokens (inflation) to inflation pot
				let inflation_pot: T::AccountId = Self::account_id();
				<T as Config>::Currency::deposit_creating(&inflation_pot, total.clone());

				Self::deposit_event(Event::<T>::InflationEvent{ inflation_pot, inflation_amount: total });

				// Update storage
				let current_block_number = frame_system::Pallet::<T>::block_number();
				LastPayoutBlock::<T>::put(current_block_number);
			}

			// to get rid of error for now
			// TODO: Add weight
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
				total_stake: BalanceOf::<T>::from(0u32),
				block_registered_at: current_block_number,
			};
			
			// Update storage
			RegisteredIps::<T>::insert(ips_id, registered_ips);

			Self::deposit_event(Event::<T>::IpsRegistered{ ips_id });

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
		pub fn stake(origin: OriginFor<T>, ips_id: T::IpId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			// Ensure IPS is registered for staking
			ensure!(Self::registered_ips(ips_id).is_some(), Error::<T>::IpsNotRegistered);

			// Ensure account has enough funds to stake this amount
			let free_balance = <<T as pallet::Config>::Currency as Currency<T::AccountId>>::free_balance(&staker);
			ensure!(free_balance > amount, Error::<T>::NotEnoughFreeBalance);

			// Ensure account is staking above set minimum
			ensure!(amount >= <T as Config>::MinStakingAmount::get(), Error::<T>::BelowMinAmount);

			// Update storage
			IpsStakers::<T>::try_mutate(ips_id, |bvec| {
				bvec.try_push(staker.clone())
			}).map_err(|_| Error::<T>::FailedAddingStaker)?;

			// 1. If account has no stake record on IPS
			// 		- push on tuple (current era + 1, stake)
			// 2. Else If account already has stake record (even 0 stake) on IPS
			//  	- pop off tuple from StakeByEra and update staking value (+ for stake, - for unstake)
			//		- Push on tuple (current era + 1, updated stake)

			let current_era = Self::current_era();

			// Update staking info
			match Self::stake_by_era(ips_id, staker.clone()).pop() {
				// Account has staked to this IPS before
				Some(v) => {
					let era = v.0;
					let existing_stake = v.1;

					// Push on new record
					if era <= current_era {
						let new_staking_record = (current_era + 1, existing_stake + amount);
						StakeByEra::<T>::try_mutate(ips_id, staker.clone(), |bvec| -> DispatchResult {
							bvec.try_push(new_staking_record).unwrap();
							Ok(())
						})?;
					}
					// Pop off latest record, update, and push back on
					else if era == current_era + 1 {
						StakeByEra::<T>::try_mutate(ips_id, staker.clone(), |bvec| -> DispatchResult {
							match bvec.pop() {
								Some(v) => {
									let updated_staking_record = (v.0, v.1 + amount);
									bvec.try_push(updated_staking_record).unwrap();
								}
								// Should always be Some
								None => { return Err(Error::<T>::FailedAddingStaker.into()); }
							}
							Ok(())
						})?;
					}
					else {
						return Err(Error::<T>::FailedAddingStaker.into());
					}	
				}
				// Account has never staked to this IPS before
				None => {
					let new_stake = (current_era + 1, amount);

					StakeByEra::<T>::try_mutate(ips_id, staker.clone(), |bvec| -> DispatchResult {
						// TODO: Don'know if error handling is correct here
						bvec.try_push(new_stake).unwrap();
						Ok(())
					})?;
				}
			}
			
			// Update IpsStakeInfo struct with correct total_stake
			RegisteredIps::<T>::try_mutate(ips_id, |ips| -> DispatchResult {
				let mut ips_obj = ips.take().ok_or(Error::<T>::IpsNotRegistered)?;
				let updated_amount = ips_obj.total_stake + amount;
				ips_obj.total_stake = updated_amount;
				*ips = Some(ips_obj);
				Ok(())
			})?;

			// Reserve accounts tokens they are staking
			<T as Config>::Currency::reserve(&staker, amount)?;

			Self::deposit_event(Event::<T>::NewStake{staker: staker, ips_id: ips_id, stake_amount: amount});

			Ok(().into())
		}

		/// Unstake from an IPS
		#[pallet::weight(1)]
		pub fn unstake(origin: OriginFor<T>, ips_id: T::IpId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let innovator = ensure_signed(origin)?;

			Ok(().into())
		}

		/// Claim tokens earned from IP staking
		#[pallet::weight(1)]
		pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let claimer = ensure_signed(origin)?;

			Ok(().into())
		}





		// /// An example dispatchable that takes a singles value as a parameter, writes the value to
		// /// storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://docs.substrate.io/v3/runtime/origins
		// 	let who = ensure_signed(origin)?;

		// 	// Update storage.
		// 	<Something<T>>::put(something);

		// 	// Emit an event.
		// 	Self::deposit_event(Event::SomethingStored(something, who));
		// 	// Return a successful DispatchResultWithPostInfo
		// 	Ok(().into())
		// }

		// /// An example dispatchable that may throw a custom error.
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
		// 	let _who = ensure_signed(origin)?;

		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => Err(Error::<T>::NoneValue)?,
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(().into())
		// 		},
		// 	}
		// }
	}

	impl<T: Config> Pallet<T> {
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
	}
}
