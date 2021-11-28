use std::collections::{HashMap, HashSet};
use crate::prize::NftPrize;
use crate::prize::FtPrize;
use crate::*;
use crate::{NonFungibleTokenId, FungibleTokenId, CreateId, Contract, next_id, StorageKey};
use crate::prize::{Prize, PrizeToken};
use near_sdk::{assert_one_yocto, near_bindgen, AccountId, env, Timestamp};
use near_sdk::json_types::{U64, ValidAccountId};
use itertools::{Itertools, join};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

pub type PoolId = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePoolDisplay {
    pub id: PoolId,
    pub owner_id: CreateId,
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub begin_time: Timestamp,
    pub end_time: Timestamp,
    pub joiner_sum: U64,
}

impl From<PrizePool> for PrizePoolDisplay {
    fn from(prize_pool: PrizePool) -> Self {
        return PrizePoolDisplay {
            id: prize_pool.id,
            owner_id: prize_pool.owner_id,
            name: prize_pool.name,
            describe: prize_pool.describe,
            cover: prize_pool.cover,
            begin_time: prize_pool.begin_time,
            end_time: prize_pool.end_time,
            joiner_sum: prize_pool.join_accounts.len().into(),
        };
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePool {
    pub id: PoolId,
    pub owner_id: CreateId,
    pub name: String,
    pub describe: String,
    pub cover: String,
    // pub prizes: Vec<Prize>,
    // the fucking typescript is hard to deserialize rust enum type
    pub ft_prizes: Vec<FtPrize>,
    pub nft_prizes: Vec<NftPrize>,
    pub ticket_price: u128,
    pub join_accounts: HashSet<AccountId>,
    pub begin_time: Timestamp,
    pub end_time: Timestamp,
    pub winner_list: HashMap<AccountId, U64>,
}

impl PrizePool {
    pub fn new(
        id: PoolId,
        owner_id: &String,
        name: String,
        describe: String,
        cover: String,
        ticket_prize: U128,
        ft_prizes: Vec<FtPrize>,
        nft_prizes: Vec<NftPrize>,
    ) -> Self {
        PrizePool {
            id,
            owner_id: owner_id.into(),
            name,
            describe,
            cover,
            ft_prizes,
            nft_prizes,
            ticket_price: ticket_prize.into(),
            join_accounts: HashSet::new(),
            begin_time: 0,
            end_time: 0,
            winner_list: HashMap::new(),
        }
    }

    //region private

    //endregion

    // pub fn add_ft_prize(&mut self, prize: FtPrize) {
    //     self.ft_prizes.push(prize);
    // }

    // pub fn add_nft_prize(&mut self, prize: NftPrize) {
    //     self.nft_prizes.push(prize);
    // }

    // pub fn delete_ft_prize(&mut self, index: usize) {
    //     self.prizes.remove(index);
    // }

    // pub fn view_prizes(self) -> Vec<Prize> {
    //     self.prizes
    // }
}

pub struct CreatePrizePoolParam {
    name: String,
    describe: String,
    cover: String,
    fts: Vec<FtPrize>,
    nfts: Vec<NftPrize>,
    begin_time: U64,
    end_time: U64,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_prize_pool(&mut self,
                             id: U64,
                             name: String,
                             describe: String,
                             cover: String,
                             ticket_prize: U128,
                             fts: Option<Vec<FtPrize>>,
                             nfts: Option<Vec<NftPrize>>) {
        assert_one_yocto();
        let prizes: Vec<PrizeToken> = vec![];
        let account_id = env::predecessor_account_id();
        // self.prize_pools.insert(&next_id(), PrizePool::new())
        let mut pool = self.prize_pools.insert(
            id.into(),
            &PrizePool::new(
                id.into(),
                &account_id,
                name,
                describe,
                cover,
                ticket_prize,
                fts.unwrap_or(vec![]),
                nfts.unwrap_or(vec![]),
            )).expect("create prize fail");
        // fts.unwrap_or(vec![]).iter() //foreach(|e| pool.add_ft_prize(e));
        // nfts.unwrap_or(vec![]). //foreach(|e| pool.add_nft_prize(e));
        // 更新用户的拥有记录
        let mut set = self.own_pools.get(&account_id)
            .unwrap_or(UnorderedSet::new(StorageKey::OwnPools { account_id }));
        set.insert(&pool.id);
        self.own_pools.insert(&account_id,&set);
    }

    #[payable]
    pub fn delete_prize_pool(&mut self, pool_id: U64) {
        assert_one_yocto();
        self.prize_pools.remove(&pool_id.into());
    }

    pub fn view_prize_pool(&self, pool_id: U64) -> PrizePool {
        let x = self.prize_pools.get(&pool_id.into()).expect("nonexistent pool id");
        let prizePool: PrizePool = *x;
        return (*x)
    }

    pub fn view_prize_pool_list(&self) -> vec<PrizePoolDisplay> {
        return self.prize_pools.iter()
            .filter_map(|(k, v)| {
                if env::block_timestamp() < v.end_time {
                    Option::from(v.into())
                } else {
                    Option::None
                }
            }).collect_vec();
    }

    #[payable]
    pub fn join(&mut self, pool_id: U64 ) {
        let pay = env::attached_deposit();
        let mut pool = self.prize_pools.get(&pool_id.into()).expect(&format!("no such pool,id:{}", pool_id.into()).to_string());
        // 支付的金额需要超过门票价
        assert!(pay>=pool.ticket_price,"The ticket price {} exceeds attahed deposit {}",pool.ticket_price ,pay);
        let joiner_id = env::predecessor_account_id();
        pool.join_accounts.push(joiner_id);
        self.accounts.get(&joiner_id).get_or_insert()
        let account = self.accounts.get(&joiner_id)
            .unwrap_or(Account::new(&joiner_id));
        // account.fts.get(&WRAP_NEAR.to_string()).
    }


    #[payable]
    pub fn quit(&mut self, pool_id: U64 ) {

    }
}