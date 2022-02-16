use std::collections::{HashMap, HashSet};
use crate::prize::{FtPrize, NftPrize};
use crate::prize_pool::{PoolId};
use crate::utils::{ONE_YOCTO, TokenAccountId};
use crate::utils::{GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER};
use crate::{Contract, StorageKey};
use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, to_string};
use asset::*;

use near_sdk::collections::{TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{ext_contract, assert_one_yocto, env, near_bindgen, AccountId, Balance, Promise, PromiseResult, Timestamp, log};
use itertools::Itertools;
use std::panic::catch_unwind;
use near_sdk::env::log;
use crate::asset::{ContractId, NftId};
use crate::twitter_giveaway::TwitterPoolDisplay;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountPrizePoolHistory {
    pool: TwitterPoolDisplay,
    records: Vec<Record>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VAccount {
    Current(Account),
}


impl VAccount {
    /// Upgrades from other versions to the currently used version.
    pub fn into_current(self) -> Account {
        match self {
            VAccount::Current(account) => account,
        }
    }
}

impl From<Account> for VAccount {
    fn from(account: Account) -> Self {
        VAccount::Current(account)
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    pub assets: Assets,
    pub pools: HashSet<PoolId>,
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Account {
            assets: Assets::default(),
            pools: HashSet::default(),
        }
    }

    // /// Deposit amount to the balance of given token.
    // pub(crate) fn deposit_ft(&mut self, token: &AccountId, amount: &Balance) {
    //     self.assets.deposit_ft(token,amount);
    // }

    // pub(crate) fn deposit_nft(&mut self, contract_id: &ContractId, nft_id: &NftId) {
    //     self.assets.deposit_nft(contract_id,nft_id);
    // }

    // pub(crate) fn withdraw_ft(&mut self, token: &AccountId, amount: &Balance) {
    //     let balance = self.fts.get(token).expect("unregistered token");
    //     assert!(balance >= *amount, "token balance not enough");
    //     self.fts.insert(&token, &(balance - (*amount)));
    // }

    // pub(crate) fn withdraw_nft(&mut self, token: &AccountId, amount: &Balance) {
    //     let balance = self.fts.get(token).expect("unregistered token");
    //     assert!(balance >= *amount, "token balance not enough");
    //     self.fts.insert(&token, &(balance - (*amount)));
    // }
}

#[ext_contract(ext_self)]
pub trait ContractMethod {
    fn withdraw_ft_callback(
        &mut self,
        sender_id: AccountId,
        token_id: AccountId,
        amount: U128,
    );
    fn withdraw_nft_callback(
        &mut self,
        sender_id: AccountId,
        contract_id: AccountId,
        nft_id: AccountId,
    );
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn withdraw_nft_callback(
        &mut self,
        sender_id: AccountId,
        contract_id: AccountId,
        nft_id: AccountId,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "withdraw_nft_callback_invalid"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                self.internal_deposit_nft(&sender_id, &contract_id, &nft_id);
            }
        }
    }

    #[private]
    pub fn withdraw_ft_callback(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "withdraw_ft_callback_invalid"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                // If account doesn't exit, deposits to the owner's account as lostfound.
                self.internal_deposit_ft(&sender_id, &token_id, &amount.into());
            }
        }
    }

    // transfer ft to other account
    #[private]
    pub(crate) fn external_send_ft(
        &self,
        received_id: &AccountId,
        token_id: &AccountId,
        amount: &U128,
    ) -> Promise {
        ext_fungible_token::ft_transfer(
            received_id.clone(),
            amount.clone(),
            None,
            &token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        )
            .then(ext_self::withdraw_ft_callback(
                received_id.clone(),
                token_id.clone(),
                amount.clone(),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER,
            ))
    }

    // transfer ft to other account
    #[private]
    pub(crate) fn external_send_nft(
        &self,
        received_id: &AccountId,
        contract_id: &AccountId,
        nft_id: &NftId,
    ) -> Promise {
        return Promise::new(contract_id.clone())
            .function_call(
                b"nft_transfer".to_vec(),
                json!({
                    "receiver_id": received_id.clone(),
                    "token_id": nft_id.clone()
                }).to_string().into_bytes(),
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER)
            .then(ext_self::withdraw_nft_callback(
                received_id.clone(),
                contract_id.clone(),
                nft_id.clone(),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER,
            ));
        // receiver_id: string,
        // token_id: string,
        // approval_id: number|null,
        // memo: string|null,
    }

    // todo add storage manager
    pub fn internal_get_account(&self, account_id: &AccountId) -> Account {
        return self.accounts
            .get(account_id)
            .map(VAccount::into_current)
            .unwrap_or(Account::new(&account_id));
    }

    pub fn internal_save_account(&mut self, account_id: &AccountId, account: Account) {
        self.accounts.insert(account_id, &account.into());
    }

    pub fn view_account_balance(&self, account_id: ValidAccountId) -> HashMap<ContractId, U128> {
        return self.internal_get_account(account_id.as_ref())
            .assets
            .fts
            .iter()
            .map(|(contract_id, balance)| (contract_id.clone(), U128::from(balance.clone())))
            .collect();
        // .map(|(contract_id,balance)|{})
    }
    pub fn view_account_assets(&self, account_id: ValidAccountId) -> AssetsDTO {
        return self.internal_get_account(account_id.as_ref())
            .assets.into();
    }

    pub fn view_account_prizepool_history(&self, account_id: ValidAccountId)->Vec<AccountPrizePoolHistory> {
        let account = self.internal_get_account(account_id.as_ref());
        account.pools.iter()
            .map(|pool_id|{
                let pool = self.internal_get_twitter_pool(pool_id);
                AccountPrizePoolHistory{
                    records: pool.records.iter()
                        .filter(|&record|record.receiver.eq(account_id.as_ref()))
                        .map(|e|e.clone())
                        .collect_vec(),
                    pool: pool.into(),
                }
            }).collect_vec()
    }

    pub(crate) fn internal_use_account<F>(&mut self, account_id: &AccountId, mut f: F)
    where F: FnMut(&mut Account) {
        let mut account = self.internal_get_account(&account_id);
        f(&mut account);
        self.internal_save_account(&account_id, account);
    }

    #[private]
    pub fn internal_deposit_ft(&mut self, account_id: &AccountId, contract_id: &ContractId, amount: &U128) {
        let mut account = self.internal_get_account(&account_id);
        account.assets.deposit_contract_amount(&contract_id, &amount.0);
        self.internal_save_account(account_id, account);
    }

    #[private]
    pub fn internal_deposit_nft(&mut self, account_id: &AccountId, contract_id: &ContractId, nft_id: &NftId) {
        let mut account = self.internal_get_account(&account_id);
        account.assets.deposit_contract_nft_id(&contract_id, &nft_id);
        self.internal_save_account(account_id, account);
    }


    // //todo 实现withdraw
    #[payable]
    pub fn withdraw_ft(&mut self, token_id: ValidAccountId, amount: U128) -> Promise {
        assert_one_yocto();

        //1. 使用account
        self.internal_use_account(&env::predecessor_account_id(), |account| {
            // withdraw
            account.assets.withdraw_contract_amount(token_id.as_ref(), &amount.0);
        });

        //3. 外部合约transfer
        self.external_send_ft(&env::predecessor_account_id(), token_id.as_ref(), &amount)
    }

    // //todo 实现withdraw
    #[payable]
    pub fn withdraw_nft(&mut self, contract_id: ValidAccountId, nft_id: NftId) -> Promise {
        assert_one_yocto();

        self.internal_use_account(
            &env::predecessor_account_id(),
            |account| {
                account.assets.withdraw_contract_nft_id(contract_id.as_ref(), &nft_id);
            },
        );
        //3. 调外部合约transfer nft
        self.external_send_nft(&env::predecessor_account_id(), contract_id.as_ref(), &nft_id)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_account {
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::log;
    use crate::*;
    use crate::asset::Ft;
    use crate::TwitterPool;
    use crate::tests::setup_contract;
    use crate::twitter_giveaway::TwitterPoolCreateParam;

    #[test]
    fn test_create() {
        const CREATE_PARAM_RAW: &str = r#"{
    "name": "1",
    "requirements": "[]",
    "twitter_link": "123",
    "white_list": [],
    "cover": "https://justplayproducts.com/wp-content/uploads/2020/06/78550_78551-Ryans-Mystery-Playdate-Mini-Mystery-Boxes-Call-Out-2-scaled-470x470.jpg",
    "describe": "1",
    "end_time": 1642919340000,
    "ft_prizes": [
      {
        "ft": {
          "contract_id": "NEAR",
          "balance": "00000000000000000000000000"
        }
      }
    ],
    "join_accounts": null,
    "nft_prizes": []
}"#;

        // tests::setup_contract()
        let (mut context, mut contract) = setup_contract();
        let param = near_sdk::serde_json::from_str(CREATE_PARAM_RAW).unwrap();

        let pool = contract.create_twitter_pool(param);
        println!("{:?}", pool)
    }
}