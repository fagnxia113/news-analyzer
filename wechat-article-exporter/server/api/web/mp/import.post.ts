/**
 * 添加公众号接口
 * 使用服务端 KV 存储
 */

import { importAccounts } from '~/server/kv/account';

export default defineEventHandler(async event => {
  const body = await readBody(event);
  const { accounts } = body;

  if (!Array.isArray(accounts) || accounts.length === 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid accounts array',
    });
  }

  // 验证每个账号对象
  const validAccounts = [];
  for (const account of accounts) {
    if (!account.fakeid) {
      throw createError({
        statusCode: 400,
        statusMessage: 'Account fakeid is required',
      });
    }
    validAccounts.push({
      fakeid: account.fakeid,
      nickname: account.nickname || '',
      round_head_img: account.round_head_img || '',
      completed: false,
      count: 0,
      articles: 0,
      total_count: 0,
      last_update_time: undefined,
    });
  }

  await importAccounts(validAccounts);

  return {
    base_resp: {
      ret: 0,
      err_msg: '',
    },
    count: validAccounts.length,
  };
});
