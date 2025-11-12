use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use crate::database::Database;
use crate::database::models::WeChatAccount;

/// 账号状态枚举（参考 wewe-rss）
#[derive(Debug, Clone, PartialEq)]
pub enum AccountStatus {
    Invalid = 0,  // 失效
    Enable = 1,   // 启用
    Disable = 2,  // 禁用
}

/// 账号管理器（参考 wewe-rss 的 blockedAccountsMap）
pub struct AccountManager {
    /// 当日被封禁的账号缓存（按日期分组）
    blocked_accounts: HashMap<String, Vec<String>>,
    /// 最后清理时间
    last_cleanup: u64,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            blocked_accounts: HashMap::new(),
            last_cleanup: Self::current_timestamp(),
        }
    }
    
    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// 获取今日日期字符串（Asia/Shanghai 时区）
    fn get_today_date() -> String {
        // 简化版本，实际应该使用 chrono 库处理时区
        let timestamp = Self::current_timestamp();
        let days_since_epoch = timestamp / 86400;
        let base_date = chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00+00:00")
            .unwrap()
            .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap());
        let today = base_date + chrono::Duration::days(days_since_epoch as i64);
        today.format("%Y-%m-%d").to_string()
    }
    
    /// 清理过期的封禁记录（每天清理一次）
    fn cleanup_expired_blocks(&mut self) {
        let now = Self::current_timestamp();
        let today = Self::get_today_date();
        
        // 如果距离上次清理超过24小时，执行清理
        if now - self.last_cleanup > 86400 {
            // 只保留今天的记录
            self.blocked_accounts.retain(|date, _| date == &today);
            self.last_cleanup = now;
            log::info!("清理过期的账号封禁记录");
        }
    }
    
    /// 将账号添加到当日黑名单
    pub fn block_account(&mut self, account_id: &str, reason: &str) {
        self.cleanup_expired_blocks();
        
        let today = Self::get_today_date();
        let blocked = self.blocked_accounts.entry(today).or_insert_with(Vec::new);
        
        if !blocked.contains(&account_id.to_string()) {
            blocked.push(account_id.to_string());
            log::warn!("账号 {} 已被加入黑名单，原因: {}", account_id, reason);
        }
    }
    
    /// 从黑名单中移除账号（解封）
    pub fn unblock_account(&mut self, account_id: &str) {
        let today = Self::get_today_date();
        if let Some(blocked) = self.blocked_accounts.get_mut(&today) {
            blocked.retain(|id| id != account_id);
            log::info!("账号 {} 已从黑名单中移除", account_id);
        }
    }
    
    /// 检查账号是否被封禁
    pub fn is_account_blocked(&self, account_id: &str) -> bool {
        let today = Self::get_today_date();
        if let Some(blocked) = self.blocked_accounts.get(&today) {
            blocked.contains(&account_id.to_string())
        } else {
            false
        }
    }
    
    /// 获取所有被封禁的账号ID
    pub fn get_blocked_account_ids(&self) -> Vec<String> {
        let today = Self::get_today_date();
        self.blocked_accounts
            .get(&today)
            .map(|blocked| blocked.clone())
            .unwrap_or_default()
    }
    
    /// 获取可用账号（参考 wewe-rss 的 getAvailableAccount）
    pub async fn get_available_account(&self, db: &Database) -> Result<WeChatAccount> {
        let blocked_ids = self.get_blocked_account_ids();
        
        // 获取所有启用的账号，排除被封禁的
        let all_accounts = db.get_all_accounts()?;
        let total_count = all_accounts.len();
        let available_accounts: Vec<WeChatAccount> = all_accounts
            .into_iter()
            .filter(|account| {
                account.status == AccountStatus::Enable as i32 
                && !blocked_ids.contains(&account.id)
            })
            .collect();
        
        if available_accounts.is_empty() {
            return Err(anyhow::anyhow!(
                "暂无可用读书账号！总账号数: {}, 被封禁数: {}", 
                total_count,
                blocked_ids.len()
            ));
        }
        
        // 随机选择一个可用账号（参考 wewe-rss）
        let selected_index = (Self::current_timestamp() as usize) % available_accounts.len();
        Ok(available_accounts[selected_index].clone())
    }
    
    /// 处理 API 错误，自动封禁账号（参考 wewe-rss 的错误处理逻辑）
    pub fn handle_api_error(&mut self, account_id: &str, error_message: &str) {
        if error_message.contains("WeReadError401") {
            // 账号失效
            self.block_account(account_id, "登录失效 (401)");
            log::error!("账号（{}）登录失效，已禁用", account_id);
        } else if error_message.contains("WeReadError429") {
            // 请求频繁
            self.block_account(account_id, "请求频繁 (429)");
            log::error!("账号（{}）请求频繁，打入小黑屋", account_id);
        } else if error_message.contains("WeReadError400") {
            // 参数错误，暂时不封禁，记录日志
            log::error!("账号（{}）处理请求参数出错: {}", account_id, error_message);
        } else if error_message.contains("500") || error_message.contains("Internal Server Error") {
            // 服务器错误，暂时封禁
            self.block_account(account_id, "服务器错误 (500)");
            log::error!("账号（{}）遇到服务器错误，暂时封禁", account_id);
        } else {
            log::error!("账号（{}）未知错误: {}", account_id, error_message);
        }
    }
    
    /// 获取账号状态统计
    pub fn get_account_stats(&self, db: &Database) -> Result<AccountStats> {
        let all_accounts = db.get_all_accounts()?;
        let blocked_ids = self.get_blocked_account_ids();
        
        let mut stats = AccountStats::default();
        
        for account in all_accounts {
            match account.status {
                0 => stats.invalid += 1,
                1 => {
                    if blocked_ids.contains(&account.id) {
                        stats.blocked += 1;
                    } else {
                        stats.available += 1;
                    }
                },
                2 => stats.disabled += 1,
                _ => {}
            }
        }
        
        Ok(stats)
    }
}

/// 账号状态统计
#[derive(Debug, Default)]
pub struct AccountStats {
    pub total: usize,
    pub available: usize,
    pub blocked: usize,
    pub invalid: usize,
    pub disabled: usize,
}

impl AccountStats {
    pub fn new(total: usize, available: usize, blocked: usize, invalid: usize, disabled: usize) -> Self {
        Self {
            total,
            available,
            blocked,
            invalid,
            disabled,
        }
    }
}
