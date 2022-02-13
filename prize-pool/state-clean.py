import os
import re
import sys

# cmd like: python state-clean.py your_contract_name.testnet your_account.testnet
contract_name = sys.argv[1]
account_name = sys.argv[2]

states = os.popen('near view-state prizepool.superise.testnet --finality final').read()
print("states:", states)

pattern = re.compile("key: '([^']+)'", )
state_keys = pattern.findall(states.read())  # ['AAsAAAB4c2IudGVzdG5ldA==', ... ,'U1RBVEU=']
state_keys_arg = "[" + ",".join(list(map(lambda e: "\"%s\"" % e, state_keys))) + "]" # "[\"AAsAAAB4c2IudGVzdG5ldA==\",\"AmkBAAAAAAAAAA==\",\"AmsAAAAAAAAAAA==\",\"AnYAAAAAAAAAAA==\",\"U1RBVEU=\"]"
print("states_key_arg:", state_keys_arg)

cmd = "near call %s clean \'{\"keys\": %s}\' --accountId %s" % (contract_name, state_keys_arg, account_name)
print("execute: "+cmd)

result = os.popen(cmd)
print(result.read())
