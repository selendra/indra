// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Auxillary struct/enums for parachain runtimes.

use frame_support::traits::{
	fungibles::{self, Balanced, CreditOf},
	Contains, Currency, Get, Imbalance, OnUnbalanced,
};
use pallet_asset_tx_payment::HandleCredit;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;
use xcm::latest::{AssetId, Fungibility::Fungible, MultiAsset, MultiLocation};
use xcm_executor::traits::FilterAssetLocation;

/// Type alias to conveniently refer to the `Currency::NegativeImbalance` associated type.
pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

/// Implementation of `OnUnbalanced` that deposits the fees into a staking pot for later payout.
pub struct ToStakingPot<R>(PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToStakingPot<R>
where
	R: pallet_balances::Config + pallet_collator_selection::Config,
	AccountIdOf<R>:
		From<selendra_primitives::v1::AccountId> + Into<selendra_primitives::v1::AccountId>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let numeric_amount = amount.peek();
		let staking_pot = <pallet_collator_selection::Pallet<R>>::account_id();
		<pallet_balances::Pallet<R>>::resolve_creating(&staking_pot, amount);
		<frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit {
			who: staking_pot,
			amount: numeric_amount,
		});
	}
}

/// Implementation of `OnUnbalanced` that deals with the fees by combining tip and fee and passing
/// the result on to `ToStakingPot`.
pub struct DealWithFees<R>(PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_collator_selection::Config,
	AccountIdOf<R>:
		From<selendra_primitives::v1::AccountId> + Into<selendra_primitives::v1::AccountId>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			<ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
		}
	}
}

/// A `HandleCredit` implementation that naively transfers the fees to the block author.
/// Will drop and burn the assets in case the transfer fails.
pub struct AssetsToBlockAuthor<R>(PhantomData<R>);
impl<R> HandleCredit<AccountIdOf<R>, pallet_assets::Pallet<R>> for AssetsToBlockAuthor<R>
where
	R: pallet_authorship::Config + pallet_assets::Config,
	AccountIdOf<R>:
		From<selendra_primitives::v1::AccountId> + Into<selendra_primitives::v1::AccountId>,
{
	fn handle_credit(credit: CreditOf<AccountIdOf<R>, pallet_assets::Pallet<R>>) {
		let author = pallet_authorship::Pallet::<R>::author();
		// In case of error: Will drop the result triggering the `OnDrop` of the imbalance.
		let _ = pallet_assets::Pallet::<R>::resolve(&author, credit);
	}
}

/// Allow checking in assets that have issuance > 0.
pub struct NonZeroIssuance<AccountId, Assets>(PhantomData<(AccountId, Assets)>);
impl<AccountId, Assets> Contains<<Assets as fungibles::Inspect<AccountId>>::AssetId>
	for NonZeroIssuance<AccountId, Assets>
where
	Assets: fungibles::Inspect<AccountId>,
{
	fn contains(id: &<Assets as fungibles::Inspect<AccountId>>::AssetId) -> bool {
		!Assets::total_issuance(*id).is_zero()
	}
}

/// Asset filter that allows all assets from a certain location.
pub struct AssetsFrom<T>(PhantomData<T>);
impl<T: Get<MultiLocation>> FilterAssetLocation for AssetsFrom<T> {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		let loc = T::get();
		&loc == origin &&
			matches!(asset, MultiAsset { id: AssetId::Concrete(asset_loc), fun: Fungible(_a) }
			if asset_loc.match_and_split(&loc).is_some())
	}
}
