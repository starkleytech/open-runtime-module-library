#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::pallet_prelude::*;
use xcm::v0::{MultiAsset, MultiLocation};

use orml_xcm_support::UnknownAsset;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event {
		/// Deposit success. [asset, to]
		Deposited(MultiAsset, MultiLocation),
		/// Deposit failed. [asset, to, error]
		DepositFailed(MultiAsset, MultiLocation, DispatchError),
		/// Withdraw success. [asset, from]
		Withdrawn(MultiAsset, MultiLocation),
		/// Withdraw failed. [asset, from, error]
		WithdrawFailed(MultiAsset, MultiLocation, DispatchError),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The balance is too low.
		BalanceTooLow,
		/// The operation will cause balance to overflow.
		BalanceOverflow,
		/// Unhandled asset.
		UnhandledAsset,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	/// Concrete fungible balances under a given location and a concrete
	/// fungible id.
	#[pallet::storage]
	#[pallet::getter(fn concrete_fungible)]
	pub(crate) type ConcreteFungibleBalances<T> =
		StorageDoubleMap<_, Blake2_128Concat, MultiLocation, Blake2_128Concat, MultiLocation, u128, ValueQuery>;

	/// Abstract fungible balances under a given location and a abstract
	/// fungible id.
	#[pallet::storage]
	#[pallet::getter(fn abstract_fungible)]
	pub(crate) type AbstractFungibleBalances<T> =
		StorageDoubleMap<_, Blake2_128Concat, MultiLocation, Blake2_128Concat, Vec<u8>, u128, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> UnknownAsset for Pallet<T> {
	fn deposit(asset: &MultiAsset, to: &MultiLocation) -> DispatchResult {
		let result = match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				ConcreteFungibleBalances::<T>::try_mutate(to, id, |b| -> DispatchResult {
					*b = b
						.checked_add(*amount)
						.ok_or_else::<DispatchError, _>(|| Error::<T>::BalanceOverflow.into())?;
					Ok(())
				})
			}
			MultiAsset::AbstractFungible { id, amount } => {
				AbstractFungibleBalances::<T>::try_mutate(to, id, |b| -> DispatchResult {
					*b = b
						.checked_add(*amount)
						.ok_or_else::<DispatchError, _>(|| Error::<T>::BalanceOverflow.into())?;
					Ok(())
				})
			}
			_ => Err(Error::<T>::UnhandledAsset.into()),
		};

		if let Err(err) = result {
			Self::deposit_event(Event::DepositFailed(asset.clone(), to.clone(), err));
		} else {
			Self::deposit_event(Event::Deposited(asset.clone(), to.clone()));
		}

		result
	}
	fn withdraw(asset: &MultiAsset, from: &MultiLocation) -> DispatchResult {
		let result = match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				ConcreteFungibleBalances::<T>::try_mutate(from, id, |b| -> DispatchResult {
					*b = b
						.checked_sub(*amount)
						.ok_or_else::<DispatchError, _>(|| Error::<T>::BalanceTooLow.into())?;
					Ok(())
				})
			}
			MultiAsset::AbstractFungible { id, amount } => {
				AbstractFungibleBalances::<T>::try_mutate(from, id, |b| -> DispatchResult {
					*b = b
						.checked_sub(*amount)
						.ok_or_else::<DispatchError, _>(|| Error::<T>::BalanceTooLow.into())?;
					Ok(())
				})
			}
			_ => Err(Error::<T>::UnhandledAsset.into()),
		};

		if let Err(err) = result {
			Self::deposit_event(Event::WithdrawFailed(asset.clone(), from.clone(), err));
		} else {
			Self::deposit_event(Event::Withdrawn(asset.clone(), from.clone()));
		}

		result
	}
}
