//!
//! Runtime extrinsic fee logic
//!
use crate::{Call, Fees, Fee, Runtime, AccountId};
use fees::{AssetOf, CheckCallFee, Trait};
use runtime_primitives::traits::{As, Applyable, Block as BlockT, Checkable, Zero};
use support::{
	dispatch::Result,
	additional_traits::{ChargeExtrinsicFee, ChargeFee},
};

/// A type that does fee calculation and payment for extrinsics
pub struct ExtrinsicFeeCharger<Block, Context, T>(rstd::marker::PhantomData<(Block, Context, T)>);

impl<Block, Context, T> ChargeExtrinsicFee<AccountId, <Block::Extrinsic as Checkable<Context>>::Checked> for ExtrinsicFeeCharger<Block, Context, T>
where
	T: Trait<Fee = Fee>,
	Context: Default,
	Block: BlockT<Header = T::Header, Hash = T::Hash>,
	Block::Extrinsic: Checkable<Context>,
	<Block::Extrinsic as Checkable<Context>>::Checked: Applyable<Index=T::Index, AccountId=T::AccountId, Call=Call>,
{
	/// Calculate and charge a fee to `transactor` for the given `extrinsic`
	/// The fee is calculated as: 'base fee + (byte fee * encoded length)'
	fn charge_extrinsic_fee(transactor: &AccountId, encoded_len: usize, extrinsic: &<Block::Extrinsic as Checkable<Context>>::Checked) -> Result {
		let bytes_fee = Fees::fee_registry(Fee::fees(fees::Fee::Bytes))
			.checked_mul(As::sa(encoded_len))
			.ok_or_else(|| "extrinsic fee overflow (bytes)")?;

		let call_fee = Runtime::check_call_fee(extrinsic.call());

		let total_fee = Fees::fee_registry(Fee::fees(fees::Fee::Base))
			.checked_add(bytes_fee)
			.ok_or_else(|| "extrinsic fee overflow (base + bytes)")?
			.checked_add(call_fee)
			.ok_or_else(|| "extrinsic fee overflow (base + bytes + call)")?;

		Fees::charge_fee(transactor, total_fee)
	}
}

/// Check the call fee for the given runtime call
impl CheckCallFee<AssetOf<Self>, Call> for Runtime {
	/// Return the associated fee for the given runtime `call`
	/// This ties a fee to a public runtime call method
	fn check_call_fee(module_call: &Call) -> AssetOf<Self> {
		// Match by module variant and then method
		match module_call {
			Call::GenericAsset(method) => match method {
				generic_asset::Call::<Self>::transfer(_, _, _) => {
					return Fees::fee_registry(Fee::generic_asset(generic_asset::Fee::Transfer))
				}
				_ => Zero::zero(),
			},
			_ => Zero::zero(),
		}
	}
}
