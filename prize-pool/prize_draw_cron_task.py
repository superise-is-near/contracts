import os
import re
import sys
import time
from functools import partial


def get_env_or_default(key, default):
    return os.getenv(key) if (os.getenv(key) is not None) else default


get_surprise_contract_name = partial(get_env_or_default, key='SURPRISE_CONTRACT_NAME',
                                     default='prizepool.superise.testnet')

get_prize_draw_account_name = partial(get_env_or_default, key='PRIZE_DRAW_ACCOUNT_NAME', default='prizedraw.testnet')


def view_exist_drawable_pool():
    result = os.popen('near view %s view_exist_drawable_pool' % get_surprise_contract_name()).read()
    print("view_exist_drawable_pool result:", result)
    return "true" in result


def pools_prize_draw():
    result = os.popen(
        'near call %s pools_prize_draw --accountId %s --gas 100000000000000' % (
            get_surprise_contract_name(), get_prize_draw_account_name())).read()
    print("prize_draw_result:", result)


while True:
    try:
        if view_exist_drawable_pool():
            pools_prize_draw()
    except Exception as ex:
        print(ex)

    time.sleep(40)
