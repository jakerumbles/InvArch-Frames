use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpsStakeInfo<Balance, BlockNumber> {
    pub total_stake: Balance,
    pub block_registered_at: BlockNumber,
}
