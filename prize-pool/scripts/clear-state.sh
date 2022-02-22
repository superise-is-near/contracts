#!/bin/bash
set -e

source ./variables.sh

# reference: https://github.com/near/core-contracts/tree/master/state-cleanup
# 1. deploy state_cleanup wasm
near deploy $PRIZE_POOL_CONTRACT_ID out/state_cleanup.wasm --node_url $NODE_URL &&
# 2. cleanup state
python3 state-clean.py prizepool.superise.testnet xsb.testnet &&
# 3. redeploy my contract
bash ./build.sh && near deploy $PRIZE_POOL_CONTRACT_ID out/prize-pool.wasm new '{"white_list_admin": "'$WHITELIST_ADMIN'"}' --node_url $NODE_URL
