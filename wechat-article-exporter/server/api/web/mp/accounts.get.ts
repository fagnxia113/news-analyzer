/**
 * 获取所有公众号接口
 */

import { getAllAccounts } from '~/server/kv/account';

export default defineEventHandler(async event => {
  const accounts = await getAllAccounts();

  return {
    base_resp: {
      ret: 0,
      err_msg: '',
    },
    count: accounts.length,
    list: accounts,
  };
});
