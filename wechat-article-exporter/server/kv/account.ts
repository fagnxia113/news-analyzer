/**
 * 服务端公众号账号存储
 * 使用 KV 存储代替 IndexedDB
 */

export type AccountKVKey = string;

export interface AccountKVValue {
  fakeid: string;
  nickname?: string;
  round_head_img?: string;
  completed: boolean;
  count: number;
  articles: number;
  total_count: number;
  create_time: number;
  update_time: number;
  last_update_time?: number;
}

/**
 * 获取所有公众号
 */
export async function getAllAccounts(): Promise<AccountKVValue[]> {
  const kv = useStorage('kv');
  const keys = (await kv.getKeys('account:')) || [];
  const accounts: AccountKVValue[] = [];

  for (const key of keys) {
    const account = await kv.get<AccountKVValue>(key);
    if (account) {
      accounts.push(account);
    }
  }

  // 按创建时间倒序排列
  return accounts.sort((a, b) => (b.create_time || 0) - (a.create_time || 0));
}

/**
 * 获取单个公众号
 */
export async function getAccount(fakeid: string): Promise<AccountKVValue | null> {
  const kv = useStorage('kv');
  return await kv.get<AccountKVValue>(`account:${fakeid}`);
}

/**
 * 添加/更新公众号
 */
export async function setAccount(account: AccountKVValue): Promise<boolean> {
  const kv = useStorage('kv');
  try {
    const existing = await getAccount(account.fakeid);
    const now = Math.round(Date.now() / 1000);

    const data: AccountKVValue = {
      ...account,
      create_time: existing?.create_time || now,
      update_time: now,
    };

    await kv.set(`account:${account.fakeid}`, data);
    return true;
  } catch (err) {
    console.error('kv.set call failed:', err);
    return false;
  }
}

/**
 * 更新公众号的同步状态
 */
export async function updateAccountSyncStatus(
  fakeid: string,
  updates: Partial<Omit<AccountKVValue, 'fakeid' | 'create_time' | 'account_kv'>>
): Promise<boolean> {
  const account = await getAccount(fakeid);
  if (!account) {
    return false;
  }

  return await setAccount({
    ...account,
    ...updates,
  });
}

/**
 * 批量导入公众号
 */
export async function importAccounts(accounts: Omit<AccountKVValue, 'create_time' | 'update_time'>[]): Promise<void> {
  for (const account of accounts) {
    const existing = await getAccount(account.fakeid);
    const now = Math.round(Date.now() / 1000);

    await setAccount({
      ...account,
      create_time: existing?.create_time || now,
      update_time: now,
    });
  }
}

/**
 * 删除公众号
 */
export async function deleteAccount(fakeid: string): Promise<boolean> {
  const kv = useStorage('kv');
  try {
    await kv.removeItem(`account:${fakeid}`);
    return true;
  } catch (err) {
    console.error('kv.removeItem call failed:', err);
    return false;
  }
}

/**
 * 批量删除公众号
 */
export async function deleteAccounts(fakeids: string[]): Promise<void> {
  for (const fakeid of fakeids) {
    await deleteAccount(fakeid);
  }
}
