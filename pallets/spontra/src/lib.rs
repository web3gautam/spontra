//! # Spontra Pallet
//!
//! A pallet with minimal functionality to allow sponsored transactions in Substrate chains.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_runtime::{traits::StaticLookup, BoundedVec};
use frame_support::traits::{CallMetadata, GetCallMetadata};
pub mod payment;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub type PalletNameOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;

pub type PalletCallNameOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;

pub type RuntimeCallNameOf<T> = (PalletNameOf<T>, PalletCallNameOf<T>);
pub trait Sponsorship<Call, AccountId> {
	fn get_payer() -> Option<AccountId>;

	fn is_call_sponsored(call: &Call) -> bool;
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxNameLen: Get<u32>;
	}

	#[pallet::storage]
	pub(super) type PayerKey<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type SponsoredCalls<T: Config> =
		StorageMap<_, Blake2_128Concat, RuntimeCallNameOf<T>, (), OptionQuery>;

	/// Events that functions in this pallet can emit.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PayerKeyUpdated {
			/// The old payer key (if one was previously set).
			old: Option<T::AccountId>,
			/// The new payer key (if one was set).
			new: T::AccountId,
		},

		CallSponsored {
			full_name: RuntimeCallNameOf<T>,
		},

		CallUnsponsored {
			full_name: RuntimeCallNameOf<T>,
		},
	}

	/// Errors that can be returned by this pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The value retrieved was `None` as no value was previously set.
		NoneValue,
		/// There was an attempt to increment the value in storage over `u32::MAX`.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn set_payer_key(
			origin: OriginFor<T>,
			payer: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let new = T::Lookup::lookup(payer)?;
			Self::deposit_event(Event::PayerKeyUpdated {
				old: PayerKey::<T>::get(),
				new: new.clone(),
			});
			PayerKey::<T>::put(new);

			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn sponsor_call(
			origin: OriginFor<T>,
			full_name: RuntimeCallNameOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			SponsoredCalls::<T>::insert(&full_name, ());
			Self::deposit_event(Event::CallSponsored { full_name });

			Ok(Pays::No.into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn unsponsor_call(
			origin: OriginFor<T>,
			full_name: RuntimeCallNameOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			SponsoredCalls::<T>::remove(&full_name);
			Self::deposit_event(Event::CallUnsponsored { full_name });

			Ok(Pays::No.into())
		}
	}

	impl <T:Config> Sponsorship<T::RuntimeCall, T::AccountId> for Pallet<T> 
	where
		<T as frame_system::Config>::RuntimeCall: GetCallMetadata,
	{
		fn get_payer() -> Option<T::AccountId> {
			<PayerKey<T>>::get()
		}

		fn is_call_sponsored(call: &<T as frame_system::Config>::RuntimeCall) -> bool {
			let CallMetadata { pallet_name, function_name } = call.get_call_metadata();
			let pallet = PalletNameOf::<T>::try_from(pallet_name.as_bytes().to_vec());
			let call = PalletCallNameOf::<T>::try_from(function_name.as_bytes().to_vec());
			match (pallet, call) {
				(Ok(pallet), Ok(call)) => SponsoredCalls::<T>::contains_key(&(pallet, call)),
				_ => true,
			}
		}
	}
}