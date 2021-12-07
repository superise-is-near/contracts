use crate::{NonFungibleTokenId, FungibleTokenId, Contract};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{Balance, env, near_bindgen};
use near_sdk::serde::{Serialize,Deserialize};

pub type PrizeId = u64;

#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum PrizeToken {
    NFT { contract: NonFungibleTokenId,id: String },
    FT { contract: FungibleTokenId, sum: u64 },
    // (a: FungibleTokenId,)
}

// 奖品
#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Prize {
    pub prize_id: PrizeId,
    pub token: PrizeToken,
}

#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NftPrize{
    pub prize_id: PrizeId,
    pub contract: NonFungibleTokenId,
    pub id: String
}

#[derive(BorshDeserialize, BorshSerialize,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FtPrize{
    pub token_id: FungibleTokenId,
    pub amount: U128,
}

impl Prize {
    pub fn default() -> Self {
        // let x = PrizeToken::FT { id: "".to_string(), sum: 1 };
        return Prize{prize_id: 1,token: PrizeToken::NFT {contract: "paras".to_string(),id: "xxx".to_string()}};
    }
}