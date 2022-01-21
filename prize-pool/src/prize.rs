use crate::*;
use crate::asset::*;
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Balance, env, near_bindgen};
use near_sdk::serde::{Serialize,Deserialize};

pub type PrizeId = u64;

pub enum Prize {
    NFT_PRIZE(NftPrize),FT_PRIZE(FtPrize)
}

#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone,Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftPrize{
    pub prize_id: PrizeId,
    pub nft: Nft,
}

#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone,Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtPrize{
    pub prize_id: PrizeId,
    pub ft: Ft,
}