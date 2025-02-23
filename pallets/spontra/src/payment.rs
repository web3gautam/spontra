use crate::{Config, Sponsorship};
use core::marker::PhantomData;
use pallet_transaction_payment::OnChargeTransaction;
use sp_std::prelude::*;

use sp_runtime::{
	traits::{DispatchInfoOf, PostDispatchInfoOf, Zero},
	transaction_validity::InvalidTransaction, Saturating,
};

use frame_support::{
	traits::{
		fungible::{Balanced, Credit, Debt, Inspect},
		tokens::{Precision, WithdrawConsequence},
		Imbalance, OnUnbalanced},
	unsigned::TransactionValidityError,
};

/// Implements transaction payment for a pallet implementing the [`frame_support::traits::fungible`]
/// trait (eg. pallet_balances) using an unbalance handler (implementing
/// [`OnUnbalanced`]).
///
/// The unbalance handler is given 2 unbalanceds in [`OnUnbalanced::on_unbalanceds`]: `fee` and
/// then `tip`.
pub struct SponsoredFungibleAdapter<F, OU, S>(PhantomData<(F, OU, S)>);

impl<T, F, OU, S> OnChargeTransaction<T> for SponsoredFungibleAdapter<F, OU, S>
where
	T: Config,
	F: Balanced<T::AccountId>,
	OU: OnUnbalanced<Credit<T::AccountId, F>>,
	S: Sponsorship<T::RuntimeCall, T::AccountId>
{
	type LiquidityInfo = Option<Credit<T::AccountId, F>>;
	type Balance = <F as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	fn withdraw_fee(
		who: &<T>::AccountId,
		call: &<T>::RuntimeCall,
		_dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
		fee: Self::Balance,
		_tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		if fee.is_zero() {
			return Ok(None)
		}

		let mut who_pays: T::AccountId = who.clone();

		if S::is_call_sponsored(call) {
			if let Some(payer) = S::get_payer() {
				who_pays = payer;
			} else {
				// Call is sponsored but no payer found -> Error
				return Err(InvalidTransaction::Payment.into())
			}
		}

		match F::withdraw(
			&who_pays,
			fee,
			Precision::Exact,
			frame_support::traits::tokens::Preservation::Preserve,
			frame_support::traits::tokens::Fortitude::Polite,
		) {
			Ok(imbalance) => Ok(Some(imbalance)),
			Err(_) => Err(InvalidTransaction::Payment.into()),
		}
	}

	fn can_withdraw_fee(
		who: &T::AccountId,
		_call: &T::RuntimeCall,
		_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		fee: Self::Balance,
		_tip: Self::Balance,
	) -> Result<(), TransactionValidityError> {
		if fee.is_zero() {
			return Ok(())
		}

		match F::can_withdraw(who, fee) {
			WithdrawConsequence::Success => Ok(()),
			_ => Err(InvalidTransaction::Payment.into()),
		}
	}

	fn correct_and_deposit_fee(
		who: &<T>::AccountId,
		_dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
		_post_info: &PostDispatchInfoOf<<T>::RuntimeCall>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		if let Some(paid) = already_withdrawn {
			// Calculate how much refund we should return
			let refund_amount = paid.peek().saturating_sub(corrected_fee);
			// refund to the the account that paid the fees if it exists. otherwise, don't refind
			// anything.
			let refund_imbalance = if F::total_balance(who) > F::Balance::zero() {
				F::deposit(who, refund_amount, Precision::BestEffort)
					.unwrap_or_else(|_| Debt::<T::AccountId, F>::zero())
			} else {
				Debt::<T::AccountId, F>::zero()
			};
			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid: Credit<T::AccountId, F> = paid
				.offset(refund_imbalance)
				.same()
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
			// Call someone else to handle the imbalance (fee and tip separately)
			let (tip, fee) = adjusted_paid.split(tip);
			OU::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));
		}

		Ok(())
	}
}
