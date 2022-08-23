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

use sp_runtime::traits::{AtLeast32BitUnsigned};
use scale_info::prelude::fmt::Display;

use primitives::{Parentage, ocif::IpsStakeInfo, utils::*};

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::{DispatchResultWithPostInfo, DispatchErrorWithPostInfo}, pallet_prelude::{*, StorageMap, ValueQuery}, 
			traits::{Currency, ReservableCurrency}, PalletId, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use core::iter::Sum;

	pub type BalanceOf<T> = <T as pallet::Config>::Balance;
	pub type IpsIdOf<T> = <T as pallet_inv4::Config>::IpId;

	use pallet_inv4 as inv4;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_inv4::Config {
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

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

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
			+ From<u128>;

		#[pallet::constant]
		type PalletId: Get<PalletId>; 

		#[pallet::constant]
		type IpsRegisterDeposit: Get<<Self as pallet::Config>::Balance>;

		#[pallet::constant]
		type MinStakingAmount: Get<<Self as pallet::Config>::Balance>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn registered_ips)]
	pub type RegisteredIps<T> = StorageMap<_, Blake2_128Concat, IpsIdOf<T>, IpsStakeInfo<BalanceOf<T>>>;

	// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IpsRegistered(IpsIdOf<T>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// IP set does not exist
		IpsNotExist,
		/// Account does not have permission to complete this action
		NoPermission,
		/// IP staking is only allowed on top level parent IP sets
		NotParent,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Register an IPS for IP Staking
		#[pallet::weight(1)]
		pub fn register(origin: OriginFor<T>, ips_id: T::IpId) -> DispatchResultWithPostInfo {
			let innovator = ensure_signed(origin)?;

			let ips = inv4::Pallet::<T>::ips_storage(ips_id);

			// Ensure IPS exists
			ensure!(!ips.clone().is_none(), Error::<T>::IpsNotExist);

			// Ensure that `innovator` is the IPS owner. Register can only be called through IPS multisig call
			let derived_address = multi_account_id::<T, T::IpId>(ips_id.clone(), None);
			ensure!(innovator == derived_address, Error::<T>::NoPermission);

			// Ensure IPS is top level i.e. a Parentage::Parent variant
			match ips.unwrap().parentage.clone() {
				Parent => ensure!(false, Error::<T>::NotParent),
				Child => ensure!(false, Error::<T>::NotParent)
			};

			match ips.unwrap().parentage.clone() {
                Parentage::Parent(_) => ensure!(false, Error::<T>::NotParent),
                Parentage::Child(_, _) => ensure!(false, Error::<T>::NotParent)
            };

			Ok(().into())
		}

		/// Unregister an IPS for IP Staking
		#[pallet::weight(1)]
		pub fn unregister(origin: OriginFor<T>, ips_id: T::IpsId) -> DispatchResultWithPostInfo {
			let innovator = ensure_signed(origin)?;

			Ok(().into())
		}

		/// Stake towards an IPS
		#[pallet::weight(1)]
		pub fn stake(origin: OriginFor<T>, ips_id: T::IpsId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let innovator = ensure_signed(origin)?;

			Ok(().into())
		}

		/// Unstake from an IPS
		#[pallet::weight(1)]
		pub fn unstake(origin: OriginFor<T>, ips_id: T::IpsId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let innovator = ensure_signed(origin)?;

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
}
