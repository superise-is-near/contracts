use std::convert::TryFrom;
use near_sdk::AccountId;
use near_sdk::json_types::ValidAccountId;

pub fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}