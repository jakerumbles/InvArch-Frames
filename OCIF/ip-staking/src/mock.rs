// use crate as pallet_template;
use codec::{Decode, Encode};
use frame_support::{
    pallet_prelude::*,
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU64, Everything},
    weights::{
        constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
    PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_balances::Account;
use pallet_staking::ConvertCurve;
use smallvec::smallvec;
use sp_core::H256;
use sp_runtime::{
    curve::PiecewiseLinear,
    generic,
    testing::Header,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    AccountId32, MultiSignature, Perbill,
};

use super::*;

/// Import the inv4 pallet.
pub use pallet_inv4 as inv4;

use inv4::ipl::LicenseList;

// pub use pallet_ip_staking as ip_staking;
use crate as ip_staking;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type AccountId = u128;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
///
// pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
// pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
// pub type SignedBlock = generic::SignedBlock<Block>;

// pub type BlockId = generic::BlockId<Block>;

pub type CommonId = u32;

pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

pub const CENTS: Balance = UNIT / 10_000;
pub const MILLICENTS: Balance = CENTS / 1_000;

pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

pub const ALICE: u128 = 1;
pub const BOB: u128 = 2;

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let p = UNIT / 500;
        let q = Balance::from(ExtrinsicBaseWeight::get());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 1200;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// REWARD_CURVE is used to calculate inflation
// pallet_staking_reward_curve::build! {
//     const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
//         min_inflation: 099_999, // 9.9999%
//         max_inflation: 100_000, // 10%
//         ideal_stake: 0_500_000,
//         falloff: 0_050_000,
//         max_piece_count: 100,
//         test_precision: 0_005_000,
//     );
// }

parameter_types! {
    pub const IpsRegisterDeposit: Balance = UNIT;
    pub const OcifIpStakingPalletId: PalletId = PalletId(*b"ia/ipstk");
    pub const MinStakingAmount: Balance = UNIT;
    // pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MillisecondsPerEra: u64 = (DAYS as u64) * MILLISECS_PER_BLOCK;
    pub const BlocksPerEra: u32 = 1;
    pub const BlocksPerYear: u32 = 1 * 365; // BlocksPerEra * 365
    pub const UnbondingPeriod: u32 = 1;
    pub const MaxUniqueStakes: u8 = 10;
    pub const IpStakingInflationRate: Perbill = Perbill::from_percent(10);
    pub const IpsInflationPercentage: Perbill = Perbill::from_percent(60);
    pub const StakerInflationPercentage: Perbill = Perbill::from_percent(40);
}

impl ip_staking::Config for Test {
    type Event = Event;
    type IpsId = CommonId;
    type Currency = Balances;
    type Balance = Balance;
    // type EraPayout = ConvertCurve<RewardCurve>;
    type PalletId = OcifIpStakingPalletId;
    type IpsRegisterDeposit = IpsRegisterDeposit;
    type MinStakingAmount = MinStakingAmount;
    type MillisecondsPerEra = MillisecondsPerEra;
    type BlocksPerEra = BlocksPerEra;
    type BlocksPerYear = BlocksPerYear;
    type UnbondingPeriod = UnbondingPeriod;
    type MaxUniqueStakes = MaxUniqueStakes;
    type IpStakingInflationRate = IpStakingInflationRate;
    type IpsInflationPercentage = IpsInflationPercentage;
    type StakerInflationPercentage = StakerInflationPercentage;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Eq, PartialEq)]
pub enum InvArchLicenses {
    /// Apache License 2.0 | https://choosealicense.com/licenses/apache-2.0/
    Apache2,
    /// GNU General Public License v3.0 | https://choosealicense.com/licenses/gpl-3.0/
    GPLv3,
    /// GNU General Public License v2.0 | https://choosealicense.com/licenses/gpl-2.0/
    GPLv2,
    /// GNU Affero General Public License v3.0 | https://choosealicense.com/licenses/agpl-3.0/
    AGPLv3,
    /// GNU Lesser General Public License v3.0 | https://choosealicense.com/licenses/lgpl-3.0/
    LGPLv3,
    /// MIT License | https://choosealicense.com/licenses/mit/
    MIT,
    /// ISC License | https://choosealicense.com/licenses/isc/
    ISC,
    /// Mozilla Public License 2.0 | https://choosealicense.com/licenses/mpl-2.0/
    MPLv2,
    /// Boost Software License 1.0 | https://choosealicense.com/licenses/bsl-1.0/
    BSLv1,
    /// The Unlicense | https://choosealicense.com/licenses/unlicense/
    Unlicense,
    /// Creative Commons Zero v1.0 Universal | https://choosealicense.com/licenses/cc0-1.0/
    CC0_1,
    /// Creative Commons Attribution 4.0 International | https://choosealicense.com/licenses/cc-by-4.0/
    CC_BY_4,
    /// Creative Commons Attribution Share Alike 4.0 International | https://choosealicense.com/licenses/cc-by-sa-4.0/
    CC_BY_SA_4,
    /// Creative Commons Attribution-NoDerivatives 4.0 International | https://creativecommons.org/licenses/by-nd/4.0/
    CC_BY_ND_4,
    /// Creative Commons Attribution-NonCommercial 4.0 International | http://creativecommons.org/licenses/by-nc/4.0/
    CC_BY_NC_4,
    /// Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International | http://creativecommons.org/licenses/by-nc-sa/4.0/
    CC_BY_NC_SA_4,
    /// Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International | http://creativecommons.org/licenses/by-nc-nd/4.0/
    CC_BY_NC_ND_4,
    /// SIL Open Font License 1.1 | https://choosealicense.com/licenses/ofl-1.1/
    OFL_1_1,
    /// Dapper Labs' NFT License Version 2.0 | https://www.nftlicense.org/
    NFT_License_2,
    Custom(
        BoundedVec<u8, <Test as inv4::Config>::MaxMetadata>,
        <Test as frame_system::Config>::Hash,
    ),
}

impl LicenseList<Test> for InvArchLicenses {
    /// Returns the license name as bytes and the IPFS hash of the licence on IPFS
    fn get_hash_and_metadata(
        &self,
    ) -> (
        BoundedVec<u8, <Test as inv4::Config>::MaxMetadata>,
        <Test as frame_system::Config>::Hash,
    ) {
        match self {
            InvArchLicenses::Apache2 => (
                vec![
                    65, 112, 97, 99, 104, 101, 32, 76, 105, 99, 101, 110, 115, 101, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    7, 57, 92, 251, 234, 183, 217, 144, 220, 196, 201, 132, 176, 249, 18, 224, 237,
                    201, 2, 113, 146, 78, 111, 152, 92, 71, 16, 228, 87, 39, 81, 142,
                ]
                .into(),
            ),
            InvArchLicenses::GPLv3 => (
                vec![
                    71, 78, 85, 32, 71, 101, 110, 101, 114, 97, 108, 32, 80, 117, 98, 108, 105, 99,
                    32, 76, 105, 99, 101, 110, 115, 101, 32, 118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    72, 7, 169, 24, 30, 7, 200, 69, 232, 27, 10, 138, 130, 253, 91, 158, 210, 95,
                    127, 37, 85, 41, 106, 136, 66, 116, 64, 35, 252, 195, 69, 253,
                ]
                .into(),
            ),
            InvArchLicenses::GPLv2 => (
                vec![
                    71, 78, 85, 32, 71, 101, 110, 101, 114, 97, 108, 32, 80, 117, 98, 108, 105, 99,
                    32, 76, 105, 99, 101, 110, 115, 101, 32, 118, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    83, 11, 214, 48, 75, 23, 172, 31, 175, 110, 63, 110, 178, 73, 2, 178, 184, 21,
                    246, 188, 76, 84, 217, 226, 18, 136, 59, 165, 230, 221, 238, 176,
                ]
                .into(),
            ),
            InvArchLicenses::AGPLv3 => (
                vec![
                    71, 78, 85, 32, 65, 102, 102, 101, 114, 111, 32, 71, 101, 110, 101, 114, 97,
                    108, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105, 99, 101, 110, 115, 101, 32,
                    118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    16, 157, 152, 89, 106, 226, 188, 217, 72, 112, 106, 206, 65, 165, 183, 196, 92,
                    139, 38, 166, 5, 26, 115, 178, 28, 146, 161, 129, 62, 94, 35, 237,
                ]
                .into(),
            ),
            InvArchLicenses::LGPLv3 => (
                vec![
                    71, 78, 85, 32, 76, 101, 115, 115, 101, 114, 32, 71, 101, 110, 101, 114, 97,
                    108, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105, 99, 101, 110, 115, 101, 32,
                    118, 51, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    41, 113, 123, 121, 57, 73, 217, 57, 239, 157, 246, 130, 231, 72, 190, 228, 200,
                    196, 32, 236, 163, 234, 84, 132, 137, 143, 25, 250, 176, 138, 20, 72,
                ]
                .into(),
            ),
            InvArchLicenses::MIT => (
                vec![77, 73, 84, 32, 76, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    30, 110, 34, 127, 171, 16, 29, 6, 239, 45, 145, 39, 222, 102, 84, 140, 102,
                    230, 120, 249, 189, 170, 34, 83, 199, 156, 9, 49, 150, 152, 11, 200,
                ]
                .into(),
            ),
            InvArchLicenses::ISC => (
                vec![73, 83, 67, 32, 76, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    119, 124, 140, 27, 203, 222, 251, 174, 95, 70, 118, 187, 129, 69, 225, 96, 227,
                    232, 195, 7, 229, 132, 185, 27, 190, 77, 151, 87, 106, 54, 147, 44,
                ]
                .into(),
            ),
            InvArchLicenses::MPLv2 => (
                vec![
                    77, 111, 122, 105, 108, 108, 97, 32, 80, 117, 98, 108, 105, 99, 32, 76, 105,
                    99, 101, 110, 115, 101, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    22, 230, 111, 228, 166, 207, 221, 50, 16, 229, 13, 232, 100, 77, 102, 184, 158,
                    79, 129, 211, 209, 102, 176, 109, 87, 105, 70, 160, 64, 123, 111, 125,
                ]
                .into(),
            ),
            InvArchLicenses::BSLv1 => (
                vec![
                    66, 111, 111, 115, 116, 32, 83, 111, 102, 116, 119, 97, 114, 101, 32, 76, 105,
                    99, 101, 110, 115, 101, 32, 49, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    174, 124, 16, 124, 106, 249, 123, 122, 241, 56, 223, 75, 59, 68, 65, 204, 73,
                    69, 88, 196, 145, 163, 233, 220, 238, 63, 99, 237, 91, 2, 44, 204,
                ]
                .into(),
            ),
            InvArchLicenses::Unlicense => (
                vec![84, 104, 101, 32, 85, 110, 108, 105, 99, 101, 110, 115, 101]
                    .try_into()
                    .unwrap(),
                [
                    208, 213, 16, 2, 240, 247, 235, 52, 119, 223, 47, 248, 137, 215, 165, 255, 76,
                    216, 178, 1, 189, 80, 159, 6, 76, 219, 36, 87, 18, 95, 66, 69,
                ]
                .into(),
            ),
            InvArchLicenses::CC0_1 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    90, 101, 114, 111, 32, 118, 49, 46, 48, 32, 85, 110, 105, 118, 101, 114, 115,
                    97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    157, 190, 198, 99, 94, 106, 166, 7, 57, 110, 33, 230, 148, 72, 5, 109, 159,
                    142, 83, 41, 164, 67, 188, 195, 189, 191, 36, 11, 61, 171, 27, 20,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 32, 52, 46, 48, 32, 73,
                    110, 116, 101, 114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    40, 210, 60, 93, 221, 27, 242, 205, 66, 90, 61, 65, 117, 72, 161, 102, 0, 242,
                    255, 168, 0, 82, 46, 245, 187, 126, 239, 220, 22, 231, 141, 195,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_SA_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 32, 83, 104, 97, 114, 101,
                    32, 65, 108, 105, 107, 101, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110,
                    97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    250, 189, 246, 254, 64, 139, 178, 19, 24, 92, 176, 241, 128, 91, 98, 105, 205,
                    149, 22, 98, 175, 178, 74, 187, 181, 189, 44, 158, 64, 117, 224, 61,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_ND_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 68, 101, 114,
                    105, 118, 97, 116, 105, 118, 101, 115, 32, 52, 46, 48, 32, 73, 110, 116, 101,
                    114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    50, 75, 4, 246, 125, 55, 242, 42, 183, 14, 224, 101, 36, 251, 72, 169, 71, 35,
                    92, 129, 50, 38, 165, 223, 90, 240, 205, 149, 113, 56, 115, 85,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 32, 52, 46, 48, 32, 73, 110, 116, 101,
                    114, 110, 97, 116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    30, 62, 213, 3, 26, 115, 233, 140, 111, 241, 54, 179, 119, 44, 203, 198, 240,
                    172, 227, 68, 101, 15, 57, 156, 29, 234, 167, 155, 66, 200, 219, 146,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_SA_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 45, 83, 104, 97, 114, 101, 65, 108, 105,
                    107, 101, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110, 97, 116, 105, 111,
                    110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    52, 186, 173, 229, 107, 225, 22, 146, 198, 254, 191, 247, 180, 34, 43, 39, 219,
                    40, 4, 143, 186, 8, 23, 44, 210, 224, 186, 201, 166, 41, 158, 121,
                ]
                .into(),
            ),
            InvArchLicenses::CC_BY_NC_ND_4 => (
                vec![
                    67, 114, 101, 97, 116, 105, 118, 101, 32, 67, 111, 109, 109, 111, 110, 115, 32,
                    65, 116, 116, 114, 105, 98, 117, 116, 105, 111, 110, 45, 78, 111, 110, 67, 111,
                    109, 109, 101, 114, 99, 105, 97, 108, 45, 78, 111, 68, 101, 114, 105, 118, 97,
                    116, 105, 118, 101, 115, 32, 52, 46, 48, 32, 73, 110, 116, 101, 114, 110, 97,
                    116, 105, 111, 110, 97, 108,
                ]
                .try_into()
                .unwrap(),
                [
                    127, 207, 189, 44, 174, 24, 37, 236, 169, 209, 80, 31, 171, 44, 32, 63, 200,
                    40, 59, 177, 185, 27, 199, 7, 96, 93, 98, 43, 219, 226, 216, 52,
                ]
                .into(),
            ),
            InvArchLicenses::OFL_1_1 => (
                vec![
                    83, 73, 76, 32, 79, 112, 101, 110, 32, 70, 111, 110, 116, 32, 76, 105, 99, 101,
                    110, 115, 101, 32, 49, 46, 49,
                ]
                .try_into()
                .unwrap(),
                [
                    44, 228, 173, 234, 177, 180, 217, 203, 36, 28, 127, 255, 113, 162, 181, 151,
                    240, 101, 203, 142, 246, 219, 177, 3, 77, 139, 82, 210, 87, 200, 140, 196,
                ]
                .into(),
            ),
            InvArchLicenses::NFT_License_2 => (
                vec![
                    78, 70, 84, 32, 76, 105, 99, 101, 110, 115, 101, 32, 86, 101, 114, 115, 105,
                    111, 110, 32, 50, 46, 48,
                ]
                .try_into()
                .unwrap(),
                [
                    126, 111, 159, 224, 78, 176, 72, 197, 201, 197, 30, 50, 31, 166, 61, 182, 81,
                    131, 149, 233, 202, 149, 92, 62, 241, 34, 86, 196, 64, 243, 112, 152,
                ]
                .into(),
            ),
            InvArchLicenses::Custom(metadata, hash) => (metadata.clone(), *hash),
        }
    }
}

parameter_types! {
    pub const MaxMetadata: u32 = 10000;
    pub const MaxCallers: u32 = 10000;
    pub const MaxLicenseMetadata: u32 = 10000;
}

impl inv4::Config for Test {
    // The maximum size of an IPS's metadata
    type MaxMetadata = MaxMetadata;
    // The IPS ID type
    type IpId = CommonId;
    // The IPS Pallet Events
    type Event = Event;
    // Currency
    type Currency = Balances;
    // The ExistentialDeposit
    type ExistentialDeposit = ExistentialDeposit;

    type Balance = Balance;

    type Call = Call;
    type MaxCallers = MaxCallers;
    type WeightToFee = WeightToFee;
    type MaxSubAssets = MaxCallers;
    type Licenses = InvArchLicenses;
    type MaxWasmPermissionBytes = MaxCallers;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const MaxRecursions: u32 = 10;
    pub const ResourceSymbolLimit: u32 = 10;
    pub const PartsLimit: u32 = 25;
    pub const MaxPriorities: u32 = 25;
    pub const CollectionSymbolLimit: u32 = 100;
    pub const MaxResourcesOnMint: u32 = 100;
}

impl pallet_rmrk_core::Config for Test {
    type Event = Event;
    type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxRecursions = MaxRecursions;
    type ResourceSymbolLimit = ResourceSymbolLimit;
    type PartsLimit = PartsLimit;
    type MaxPriorities = MaxPriorities;
    type CollectionSymbolLimit = CollectionSymbolLimit;
    type MaxResourcesOnMint = MaxResourcesOnMint;
}

parameter_types! {
    pub const CollectionDeposit: Balance = 10 * MILLIUNIT;
    pub const ItemDeposit: Balance = UNIT;
    pub const KeyLimit: u32 = 32;
    pub const ValueLimit: u32 = 256;
    pub const UniquesMetadataDepositBase: Balance = 10 * MILLIUNIT;
    pub const AttributeDepositBase: Balance = 10 * MILLIUNIT;
    pub const DepositPerByte: Balance = MILLIUNIT;
    pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Test {
    type Event = Event;
    type CollectionId = CommonId;
    type ItemId = CommonId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type Locker = pallet_rmrk_core::Pallet<Test>;
    type CollectionDeposit = CollectionDeposit;
    type ItemDeposit = ItemDeposit;
    type MetadataDepositBase = UniquesMetadataDepositBase;
    type AttributeDepositBase = AttributeDepositBase;
    type DepositPerByte = DepositPerByte;
    type StringLimit = UniquesStringLimit;
    type KeyLimit = KeyLimit;
    type ValueLimit = ValueLimit;
    type WeightInfo = ();
}

parameter_types! {
    // The maximum size of an IPF's metadata
    pub const MaxIpfMetadata: u32 = 10000;
}

impl pallet_ipf::Config for Test {
    // The maximum size of an IPF's metadata
    type MaxIpfMetadata = MaxIpfMetadata;
    // The IPF ID type
    type IpfId = u64;
    // Th IPF pallet events
    type Event = Event;
}

parameter_types! {
    pub const MaxPropertiesPerTheme: u32 = 100;
    pub const MaxCollectionsEquippablePerPart: u32 = 100;
}

impl pallet_rmrk_equip::Config for Test {
    type Event = Event;
    type MaxPropertiesPerTheme = MaxPropertiesPerTheme;
    type MaxCollectionsEquippablePerPart = MaxCollectionsEquippablePerPart;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        // TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
        Ipf: pallet_ipf::{Pallet, Call, Storage, Event<T>},
        INV4: inv4::{Pallet, Call, Storage, Event<T>},
        IpStaking: ip_staking::{Pallet, Call, Storage, Event<T>},

        Uniques: pallet_uniques::{Pallet, Storage, Event<T>},
        RmrkCore: pallet_rmrk_core::{Pallet, Call, Event<T>, Storage},
        RmrkEquip: pallet_rmrk_equip::{Pallet, Call, Event<T>, Storage},

    }
);

// // Build genesis storage according to the mock runtime.
// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
// }

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        // Give accounts 10 tokens each
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 3_000_000_000_000),
                (2, 11_699_993_000_000_000_000),
                (3, 1_000_000_000_000),
                (4, 1_000_000_000_000),
                (5, 1_000_000_000_000),
                (6, 1_000_000_000_000),
            ],
            // balances: vec![
            //     (1, 100_000_000_000_000),
            //     (2, 500_000_000_000_001),
            //     (3, 100_000_000_000_000),
            //     (4, 100_000_000_000_000),
            //     (5, 100_000_000_000_000),
            //     (6, 100_000_000_000_000),
            // ],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        crate::GenesisConfig::<Test> {
            total_staked: (0, 0, 0),
            current_era: 0,
            last_payout_block: 0,
            inital_inflation_per_era: 3_205_000_000_000_000,
            last_inflation_recalc_block: 0,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        // pub total_staked: (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
        // pub current_era: Era,
        // pub last_payout_block: BlockNumberOf<T>,
        // pub inital_yearly_inflation: BalanceOf<T>,
        // pub last_inflation_recalc_block: BlockNumberOf<T>,

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub const INIT_TIMESTAMP: u64 = 30_000;
// pub const BLOCK_TIME: u64 = 1000;

/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    IpStaking::on_finalize(System::block_number());
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
        // Session::on_initialize(b);
        <IpStaking as Hooks<BlockNumber>>::on_initialize(b);
        Timestamp::set_timestamp(
            System::block_number() as u64 * MILLISECS_PER_BLOCK + INIT_TIMESTAMP,
        );
        if b != n {
            IpStaking::on_finalize(System::block_number());
        }
    }
}
