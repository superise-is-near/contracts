use crate::{NonFungibleTokenId, FungibleTokenId, CreateId, Contract};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::ValidAccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
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
    pub prize_id: PrizeId,
    pub contract: FungibleTokenId,
    pub sum: u64,
}

impl Prize {
    pub fn default() -> Self {
        // let x = PrizeToken::FT { id: "".to_string(), sum: 1 };
        return Prize{prize_id: 1,token: PrizeToken::NFT {contract: "paras".to_string(),id: "xxx".to_string()}};
    }
}

impl Contract{

    // #[payable]
    // pub fn add_ft_prize(&self, pool_id: u64, ft: PrizeToken::FT ) {
    //     assert_at_least_one_yocto();
    //     self.prize_pools.get(&pool_id).expect("no such pool").add_ft_prize(ft)
    // }

    // #[payable]
    // pub fn add_nft_prize(&self, pool_id: u64, nft: PrizeToken::NFT ) {
    //     assert_at_least_one_yocto();
    //     self.prize_pools.get(&pool_id).expect("no such pool").add_nft_prize(nft)
    // }

    pub fn view_ft_prizes(&self,pool_id: u64)-> Vec<FtPrize> {
        self.prize_pools.get(&pool_id).expect("no such pool").ft_prizes
    }
}