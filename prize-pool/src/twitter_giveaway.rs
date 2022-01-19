use std::collections::{HashMap, HashSet};
use itertools::join;
use near_sdk::{assert_one_yocto, env};
use near_sdk::json_types::ValidAccountId;
use crate::{AccountId, Assets, Contract, CountDownDrawPrize, DrawPrize, MilliTimeStamp, PoolId, PrizePool};
use crate::prize::{FtPrize, NftPrize, Prize};
use crate::prize_pool::{random_distribution_prizes, RandomDistributePrize};
use crate::StorageKey::TwitterPools;

type TwitterAccount = String;

pub struct TwitterPoolWhiteListParam {
    pub pool_id: PoolId,
    pub account: ValidAccountId,
    pub twitter_account: TwitterAccount,
}

pub struct TwitterPool {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub prize_pool: PrizePool,
    pub finish: bool,
    pub end_time: MilliTimeStamp,
    pub white_list: HashSet<AccountId>,
    pub requirement: Vec<String>,
    pub twitter_near_bind: HashMap<AccountId, TwitterAccount>,
}

pub struct TwitterPoolCreateParam {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub end_time: MilliTimeStamp,
    pub white_list: Option<Vec<AccountId>>,
    pub requirement: Option<Vec<String>>,
    pub ft_prizes: Option<Vec<FtPrize>>,
    pub nft_prizes: Option<Vec<NftPrize>>,
    pub join_accounts: Option<Vec<AccountId>>,
}

impl TwitterPool {
    pub fn new_by_near_call(param: TwitterPoolCreateParam, pool_id: PoolId, creator_id: AccountId) -> Self {
        Self {
            name: param.name,
            describe: param.describe,
            cover: param.cover,
            prize_pool: PrizePool {
                id: pool_id,
                creator_id,
                ft_prizes: param.ft_prizes.unwrap_or(vec![]),
                nft_prizes: param.nft_prizes.unwrap_or(vec![]),
                join_accounts: param.join_accounts.unwrap_or(vec![]).into_iter().collect(),
            },
            finish: false,
            end_time: 0,
            white_list: param.white_list.unwrap_or(vec![]).into_iter().collect(),
            requirement: param.requirement.unwrap_or(vec![]),
            twitter_near_bind: Default::default(),
        }
    }
}

impl DrawPrize for TwitterPool {
    fn draw_prize(&self) -> HashMap<AccountId, Vec<Prize>> {
        return random_distribution_prizes(&self.prize_pool.ft_prizes,
                                          &self.prize_pool.nft_prizes,
                                          self.prize_poolpool.join_accounts.iter().collect_vec(),
                                          &self.prize_pool.creator_id);
    }
}

impl CountDownDrawPrize for TwitterPool {
    fn draw_prize_time(&self) -> MilliTimeStamp {
        return self.end_time;
    }
}

impl RandomDistributePrize for TwitterPool {
    fn random_distribute_prize(self: Self) -> HashMap<AccountId, Vec<Prize>> {}
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_twitter_pool(&mut self, param: TwitterPoolCreateParam) -> TwitterPool {
        assert_one_yocto();
        let creator_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&creator_id).expect(format!("{} never deposit before", &creator_id).as_ref());
        param.ft_prizes.unwrap_or(vec![]).iter().for_each(|x| account.assets.withdraw_ft(&x.ft));
        param.nft_prizes.unwrap_or(vec![]).iter().for_each(|x| account.assets.withdraw_nft(&x.nft));
        let pool_id = self.next_id();
        let pool = TwitterPool::new_by_near_call(param, pool_id, creator_id);
        self.twitter_prize_pools.insert(&pool_id, &pool);
        return pool;
    }

    pub fn join_twitter_pool(&mut self, pool_id: u64) {
        let mut pool = self.twitter_prize_pools.get(&pool_id).expect(&format!("no such pool,id:{}", pool_id));
        let joiner = env::predecessor_account_id();
        assert!(pool.white_list.contains(&joiner), "you are not in whitelist");
        pool.prize_pool.join_accounts.insert(joiner);
    }

    //todo unjoin twitter pool
    pub fn unjoin_twitter_pool(&mut self, pool_id: PoolId) {}

    pub fn view_twitter_prize_pool(&self, pool_id: PoolId) -> TwitterPool {
        return self.twitter_prize_pools.get(&pool_id).expect("inexistent pool id");
    }

    pub fn update_whitelist(&mut self) {}

    pub fn add_user_into_whitelist(&mut self, param: TwitterPoolWhiteListParam) {
        let mut pool = self.twitter_prize_pools.get(&param.pool_id).expect("inexistent pool id");
        let signer = env::predecessor_account_id();
        // check authority
        assert!(signer == pool.prize_pool.creator_id || signer == self.white_list_admin, "no authority change whitelist");
        assert_eq!(pool.twitter_near_bind.values().find(|&e| e.eq(&param.twitter_account)).is_none(),
                   format!("this twitter account {} has been used!", param.twitter_account));
        pool.white_list.insert(account.into());
    }

    // pub fn view_twitter_prize_pool_list(&self) -> Vec<TwitterPoolDisplay> {
    //     return self.twitter_prize_pools.get(&pool_id).expect("inexistent pool id");
    // }
}
