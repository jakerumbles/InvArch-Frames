use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use scale_info::prelude::vec::Vec;
use sp_runtime::BoundedVec;

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, Debug, TypeInfo)]
pub struct IpsStakeInfo<Balance, BlockNumber, BVec> {
    pub total_stake: Balance,
    pub block_registered_at: BlockNumber,
    pub stakers: BVec,
}
