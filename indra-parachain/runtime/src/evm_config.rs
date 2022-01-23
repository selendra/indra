use super::{
	precompiles::SelendraPrecompiles, AccountId, Aura, Balance, Balances, Call, Event, Runtime,
	Signature, MILLICENTS, NANO,
};
use sp_core::{crypto::Public, H160, U256};
use sp_runtime::{
	traits::{BlakeTwo256, Verify},
	transaction_validity::TransactionPriority,
	FixedPointNumber, Permill, Perquintill,
};

use frame_support::{
	pallet_prelude::PhantomData,
	parameter_types,
	traits::FindAuthor,
	weights::{constants::WEIGHT_PER_SECOND, Weight},
	ConsensusEngineId,
};
use pallet_evm::{EnsureAddressTruncated, FeeCalculator, HashedAddressMapping};
use pallet_transaction_payment::Multiplier;

use parachains_common::{MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO};

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

pub struct SelendraGasWeightMapping;

impl pallet_evm::GasWeightMapping for SelendraGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		(5 * NANO).into()
	}
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Aura::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]))
		}
		None
	}
}

parameter_types! {
	pub const ChainId: u64 = 222;
	pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly. This low value causes changes to occur slowly over time.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero` in integration_tests.rs.
	/// This value is currently only used by pallet-transaction-payment as an assertion that the
	/// next multiplier is always > min value.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
	pub PrecompilesValue: SelendraPrecompiles<Runtime> = SelendraPrecompiles::<_>::new();
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = SelendraGasWeightMapping;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = SelendraPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = ();
	type FindAuthor = FindAuthorTruncated<Aura>;
}

impl pallet_ethereum::Config for Runtime {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

parameter_types! {
	// Tells `pallet_base_fee` whether to calculate a new BaseFee `on_finalize` or not.
	pub IsActive: bool = false;
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type Event = Event;
	type Threshold = BaseFeeThreshold;
	type IsActive = IsActive;
}

parameter_types! {
	pub const EcdsaUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const CallFee: Balance = MILLICENTS / 10;
	pub const CallMagicNumber: u16 = 0xff51;
}

impl pallet_custom_signatures::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Signature = pallet_custom_signatures::ethereum::EthereumSignature;
	type Signer = <Signature as Verify>::Signer;
	type CallMagicNumber = CallMagicNumber;
	type Currency = Balances;
	type CallFee = CallFee;
	type OnChargeTransaction = ();
	type UnsignedPriority = EcdsaUnsignedPriority;
}
