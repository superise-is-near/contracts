use std::borrow::Borrow;
use std::collections::{BinaryHeap, HashMap, HashSet};
use crate::prize::{NftPrize, Prize};
use crate::prize::FtPrize;
use crate::*;
use crate::{NonFungibleTokenId, FungibleTokenId, Contract, StorageKey};
use near_sdk::{assert_one_yocto, near_bindgen, AccountId, env, Timestamp};
use near_sdk::json_types::{U64, ValidAccountId};
use itertools::{Itertools, join};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{block_timestamp, log};
use near_sdk::serde::{Deserialize, Serialize};
use crate::StorageKey::PrizePools;
use crate::utils::{get_block_milli_time, vec_random};
use std::cmp::Ordering;
use crate::asset::{Asset, Assets, Ft, Nft};

pub type PoolId = u64;

enum PoolEnum {
    TwitterPool(TwitterPool)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizeDrawTime(pub MilliTimeStamp,pub PoolId);

impl Eq for PrizeDrawTime {}

impl PartialEq<Self> for PrizeDrawTime{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)&&self.1.eq(&other.1)
    }
}

impl PartialOrd<Self> for PrizeDrawTime{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrizeDrawTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Record {
    time: MilliTimeStamp,
    ft_prize: FtPrize,
    receiver: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePool {
    pub id: PoolId,
    pub creator_id: AccountId,
    pub ft_prizes: Vec<FtPrize>,
    pub nft_prizes: Vec<NftPrize>,
    pub join_accounts: HashSet<AccountId>,
}

pub trait CountDownDrawPrize {
    fn draw_prize_time(&self)->MilliTimeStamp;
}

pub trait DrawPrize {
   fn draw_prize(&self)-> HashMap<AccountId, Vec<Prize>>;
}

impl PrizePool {
    pub fn new(
        id: PoolId,
        creator_id: &String,
    ) -> Self {
        PrizePool {
            id,
            creator_id: creator_id.into(),
            ft_prizes: vec![],
            nft_prizes: vec![],
            join_accounts: HashSet::new(),
        }
    }
}

pub fn random_distribution_prizes(ft_prizes: &Vec<FtPrize>,
                                  nft_prizes: &Vec<NftPrize>,
                                  mut joiners: Vec<&AccountId>,
                                  creator: &AccountId) -> HashMap<AccountId, Vec<Prize>> {
    let mut indexs = (0..ft_prizes.len() + nft_prizes.len()).collect_vec();
    let len = indexs.len().clone();
    let mut result: HashMap<AccountId, Vec<Prize>> = HashMap::default();
    for _ in 0..len {
        let receiver = vec_random(&mut joiners).unwrap_or(creator);
        let prize_index = vec_random(&mut indexs).unwrap();
        let prize = if prize_index < ft_prizes.len() { Prize::FT_PRIZE(ft_prizes[prize_index].clone()) } else { Prize::NFT_PRIZE(nft_prizes[prize_index].clone()) };
        result.entry(receiver.clone()).or_insert(vec![]).push(prize)
    };
    return result;
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

    #[private]
    fn prize_draw(&mut self, pool_id: PoolId) {
        let mut pool = self.twitter_prize_pools.get(&pool_id).expect("pool id didn't exist");
        // 1. check time
        let time_now = get_block_milli_time();
        assert!(pool.end_time <= time_now, "pool end_time ({}) is before block_timestamp({})", pool.end_time, time_now);
        // 2. internal transfer prize to user
        let user_prize_map = pool.draw_prize();
        user_prize_map.iter()
            .for_each(|(account_id,prizes)|{
                let mut account = self.accounts.get(&account_id).unwrap_or(Account::new(&account_id));
                prizes.iter().for_each(|prize|{
                    match prize {
                        Prize::NFT_PRIZE(nft_prize) => {account.assets.deposit_nft(&nft_prize.nft)}
                        Prize::FT_PRIZE(ft_prize) => {account.assets.deposit_ft(&ft_prize.ft)}
                    }
                });
                self.accounts.insert(&account_id,&account);
            });

        pool.finish = true;
        self.twitter_prize_pools.insert(&pool_id,&pool);
    }

    // pub fn view_prize_pool(&self, pool_id: u64) -> PrizePool {
    //     self.prize_pools.get(&pool_id.into()).expect("nonexistent pool id")
    // }

    pub fn view_prize_pool_queue(&self) -> Vec<PrizeDrawTime> {
        // return self.pool_queue.into_iter().collect_vec();
        return self.pool_queue.iter().map(|e|e.clone()).collect_vec();
    }

    // 访问是否有开奖的奖池
    pub fn touch_pools(&mut self) {
        log!("block time is {}",get_block_milli_time());
        while !self.pool_queue.is_empty() && self.pool_queue.peek().unwrap().0 <= get_block_milli_time() {
            let pool = self.pool_queue.pop().unwrap();
            log!("pool {} start prize_draw at block_time: {}",pool.1, get_block_milli_time());
            // 只有存在的奖池才会继续
            match self.twitter_prize_pools.get(&pool.1) {
                None => {}
                Some(twitter_pool) => { self.prize_draw(pool.1) }
            }
        }
    }
}

