use near_sdk::AccountId;
use near_sdk_sim::{deploy, init_simulator, to_yocto};

fn dai() -> AccountId {
    "dai".to_string()
}

fn eth() -> AccountId {
    "eth".to_string()
}

fn swap() -> AccountId {
    "swap".to_string()
}

fn prizepool() -> AccountId{"prize-pool".to_string()}

fn setup() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let prizePool = deploy!(
        contract: PrizePool,
        contract_id: prizepool(),
        bytes: &PRIZPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );
}