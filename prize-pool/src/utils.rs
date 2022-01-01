use std::convert::TryInto;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId, Balance, Gas, env, Timestamp};
use crate::MilliTimeStamp;

pub(crate) type TokenAccountId = AccountId;

pub(crate) const ONE_YOCTO: Balance = 1;
pub(crate) const ONE_NEAR: Balance = 10u128.pow(24);

#[ext_contract(ext_self)]
pub trait RefExchange {
    fn exchange_callback_post_withdraw(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    );
}

pub fn random_number_from_block() -> u64{
    let seed = env::random_seed();
    let mut arr: [u8; 8] = Default::default();
    arr.copy_from_slice(&seed[..8]);

    let seed_num: u64 = u64::from_le_bytes(arr).try_into().unwrap();
    return seed_num;
}

pub fn vec_random<T>(vec: &mut Vec<T>)-> Option<T> {
    let len = vec.len();
    if len==0 {return Option::None};
    let choose_index: usize = (random_number_from_block() % (len as u64)) as usize;
    vec.swap(len-1, choose_index.into());
    vec.pop()
}

/// Attach no deposit.
pub const NO_DEPOSIT: u128 = 0;
/// hotfix_insuffient_gas_for_mft_resolve_transfer, increase from 5T to 20T
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = 20_000_000_000_000;

pub const GAS_FOR_FT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

/// Amount of gas for fungible token transfers, increased to 20T to support AS token contracts.
pub const GAS_FOR_FT_TRANSFER: Gas = 20_000_000_000_000;

/// Fee divisor, allowing to provide fee in bps.
pub const FEE_DIVISOR: u32 = 10_000;

/// Initial shares supply on deposit of liquidity.
pub const INIT_SHARES_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000;

pub fn get_block_milli_time() -> MilliTimeStamp {
    return env::block_timestamp()/1000000;
}