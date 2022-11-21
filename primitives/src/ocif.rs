use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Holds general info about registered IP Sets
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpsStakeInfo<AccountId,Balance, BlockNumber> {
    /// IP Sets accountId
    pub address: AccountId,
    /// Total stake on this IP Set
    pub total_stake: Balance,
    /// New stake during era x that will be added to total_stake at the very beginning of era x+1
    pub next_era_new_stake: Balance,
    /// New unstake during era x that will be subtracted from total_stake at the very beginning of era x+1
    pub next_era_new_unstake: Balance,
    /// Block the IP Set was registered at
    pub block_registered_at: BlockNumber,
}
