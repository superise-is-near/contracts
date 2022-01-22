#!/bin/bash
export ACCOUNT_ID=superise.testnet
export BOB_ACCOUNT_ID=bob.$ACCOUNT_ID
ONE_YOCTO=0.000000000000000000000001
export WHITELIST_ADMIN=whitelist_admin.$CONTRACT_ID

export CONTRACT_ID=prizepool.$ACCOUNT_ID

#创建合约账户
#near create-account $CONTRACT_ID --masterAccount=$ACCOUNT_ID --initialBalance=10

#删除合约账户
#near delete prizepool.superise.testnet superise.testnet

# 删除重新发布一条龙
near delete prizepool.superise.testnet xsb.testnet &&
near create-account $CONTRACT_ID --masterAccount=$ACCOUNT_ID --initialBalance=10 &&
bash ./build.sh && near deploy $CONTRACT_ID out/prize-pool.wasm  new '{"white_list_admin": "'"$WHITELIST_ADMIN"'"}'

# 编译wasm
#bash ./build.sh && near deploy $CONTRACT_ID out/prize-pool.wasm # new '{}'
