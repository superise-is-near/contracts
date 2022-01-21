use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use itertools::{Itertools, join};
use near_sdk::{assert_one_yocto, env, near_bindgen};
use near_sdk::json_types::ValidAccountId;
use crate::{Account, AccountId, Assets, Contract, CountDownDrawPrize, DrawPrize, MilliTimeStamp, PoolId, PrizeDrawTime, PrizePool};
use crate::prize::{FtPrize, NftPrize, Prize};
use crate::prize_pool::{random_distribution_prizes};
use crate::StorageKey::TwitterPools;
use crate::ContractContract;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};


type TwitterAccount = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone,Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolWhiteListParam {
    pub pool_id: PoolId,
    pub account: ValidAccountId,
    pub twitter_account: TwitterAccount,
}
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone,Debug)]
#[serde(crate = "near_sdk::serde")]
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
    pub twitter_link: String
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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
    pub twitter_link: String
}

impl TwitterPool {
    pub fn new_by_near_call(param: &TwitterPoolCreateParam, pool_id: PoolId, creator_id: AccountId) -> Self {
        Self {
            name: param.name.clone(),
            describe: param.describe.clone(),
            cover: param.cover.clone(),
            prize_pool: PrizePool {
                id: pool_id,
                creator_id,
                ft_prizes: param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e|e.clone()).collect_vec(),
                nft_prizes: param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e|e.clone()).collect_vec(),
                join_accounts: HashSet::from_iter(param.join_accounts.as_ref().unwrap_or(&vec![]).iter().map(|e|e.clone())),
            },
            finish: false,
            end_time: 0,
            white_list: param.white_list.as_ref().unwrap_or(&vec![]).iter().map(|e|e.clone()).collect(),
            requirement: param.requirement.as_ref().unwrap_or(&vec![]).clone(),
            twitter_near_bind: Default::default(),
            twitter_link: param.twitter_link.clone()
        }
    }
}

impl DrawPrize for TwitterPool {
    fn draw_prize(&self) -> HashMap<AccountId, Vec<Prize>> {
        return random_distribution_prizes(&self.prize_pool.ft_prizes,
                                          &self.prize_pool.nft_prizes,
                                          self.prize_pool.join_accounts.iter().collect_vec(),
                                          &self.prize_pool.creator_id);
    }
}

impl From<&TwitterPool> for PrizeDrawTime {
    fn from(pool: &TwitterPool) -> Self {
        return PrizeDrawTime(pool.end_time.clone(),pool.prize_pool.id.clone());
    }
}

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn create_twitter_pool(&mut self, param: TwitterPoolCreateParam) -> TwitterPool {
        assert_one_yocto();
        let creator_id = env::predecessor_account_id();
        // todo user should register first
        let mut account = self.accounts.get(&creator_id).unwrap_or(Account::new(&creator_id));

        param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_ft(&x.ft));
        param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_nft(&x.nft));
        let pool_id = self.next_id();
        let pool = TwitterPool::new_by_near_call(&param, pool_id, creator_id);
        self.twitter_prize_pools.insert(&pool_id, &pool);
        self.pool_queue.push((&pool).into());
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
        assert!(pool.twitter_near_bind.values().find(|&e| e.eq(&param.twitter_account)).is_none(),
                   format!("this twitter account {} has been used!", param.twitter_account));
        pool.white_list.insert(param.account.into());
    }

    // pub fn view_twitter_prize_pool_list(&self) -> Vec<TwitterPoolDisplay> {
    //     return self.twitter_prize_pools.get(&pool_id).expect("inexistent pool id");
    // }
}
mod test_twitter {
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::log;
    use crate::{tests, TwitterPool};
    use crate::tests::setup_contract;
    use crate::twitter_giveaway::TwitterPoolCreateParam;

    #[test]
    fn test() {
        // tests::setup_contract()
        let (mut context, mut contract) = setup_contract();
        let pool = contract.create_twitter_pool(TwitterPoolCreateParam {
            name: "test".to_string(),
            describe: "test".to_string(),
            cover: "test".to_string(),
            end_time: 0,
            white_list: None,
            requirement: None,
            ft_prizes: None,
            nft_prizes: None,
            join_accounts: None,
            twitter_link: "test".to_string()
        });
        println!("{:?}",pool)
    }

}