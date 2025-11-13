use super::Database;
use super::models::*;
use rusqlite::params;
use rusqlite::OptionalExtension;
use anyhow::Result;

impl Database {
    // ========== 账号操作 ==========
    
    pub fn insert_account(&self, account: &WeChatAccount) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT INTO wechat_accounts (id, name, avatar, vid, token, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                account.id,
                account.name,
                account.avatar,
                account.vid,
                account.token,
                account.status,
                account.created_at,
                account.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_accounts(&self) -> Result<Vec<WeChatAccount>> {
        let conn = self.get_connection();
        let now = chrono::Utc::now().timestamp();
        
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.avatar, a.vid, a.token, a.status, a.created_at, a.updated_at,
                    CASE WHEN b.id IS NOT NULL AND b.banned_until > ?1 THEN 1 ELSE 0 END as is_banned
             FROM wechat_accounts a
             LEFT JOIN account_blacklist b ON a.id = b.account_id AND b.banned_until > ?1
             ORDER BY a.created_at DESC"
        )?;
        
        let accounts = stmt.query_map(params![now], |row| {
            Ok(WeChatAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar: row.get(2)?,
                vid: row.get(3)?,
                token: row.get(4)?,
                status: row.get(5)?,
                is_banned: row.get(8)?, // 从查询结果获取黑名单状态
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(accounts)
    }
    
    pub fn update_account_status(&self, id: &str, status: i32) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE wechat_accounts SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }
    
    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.get_connection();
        conn.execute("DELETE FROM wechat_accounts WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    // ========== 订阅源操作 ==========
    
    pub fn insert_feed(&self, feed: &WeChatFeed) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT OR REPLACE INTO wechat_feeds (id, mp_name, mp_intro, mp_cover, status, sync_time, update_time, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                feed.id,
                feed.mp_name,
                feed.mp_intro,
                feed.mp_cover,
                feed.status,
                feed.sync_time,
                feed.update_time,
                feed.created_at,
                feed.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_feeds(&self) -> Result<Vec<WeChatFeed>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, mp_name, mp_intro, mp_cover, status, sync_time, update_time, has_history, created_at, updated_at
             FROM wechat_feeds ORDER BY update_time DESC"
        )?;
        
        let feeds = stmt.query_map([], |row| {
            Ok(WeChatFeed {
                id: row.get(0)?,
                mp_name: row.get(1)?,
                mp_intro: row.get(2)?,
                mp_cover: row.get(3)?,
                status: row.get(4)?,
                sync_time: row.get(5)?,
                update_time: row.get(6)?,
                has_history: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(feeds)
    }
    
    pub fn delete_feed(&self, id: &str) -> Result<()> {
        let mut conn = self.get_connection();
        // 使用事务确保数据一致性
        let tx = conn.transaction()?;
        
        // 先删除该订阅源的所有文章
        tx.execute("DELETE FROM wechat_articles WHERE mp_id = ?1", params![id])?;
        
        // 再删除订阅源
        tx.execute("DELETE FROM wechat_feeds WHERE id = ?1", params![id])?;
        
        tx.commit()?;
        Ok(())
    }
    
    pub fn update_feed_sync_time(&self, id: &str, sync_time: i64, has_history: i32) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE wechat_feeds SET sync_time = ?1, has_history = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![sync_time, has_history, id],
        )?;
        Ok(())
    }
    
    // ========== 文章操作 ==========
    
    pub fn insert_article(&self, article: &WeChatArticle) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT OR IGNORE INTO wechat_articles (id, mp_id, title, pic_url, publish_time, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                article.id,
                article.mp_id,
                article.title,
                article.pic_url,
                article.publish_time,
                article.created_at,
                article.updated_at,
            ],
        )?;
        Ok(())
    }
    
    /// 批量插入文章（优化性能，保持去重功能）
    pub fn insert_articles_batch(&self, articles: &[WeChatArticle]) -> Result<usize> {
        let mut conn = self.get_connection();
        let tx = conn.transaction()?;
        
        let mut saved_count = 0;
        for article in articles {
            let rows_affected = tx.execute(
                "INSERT OR IGNORE INTO wechat_articles (id, mp_id, title, pic_url, publish_time, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    article.id,
                    article.mp_id,
                    article.title,
                    article.pic_url,
                    article.publish_time,
                    article.created_at,
                    article.updated_at,
                ],
            )?;
            
            // rows_affected == 1 表示真正插入了新文章
            // rows_affected == 0 表示文章已存在，被忽略
            saved_count += rows_affected;
        }
        
        tx.commit()?;
        Ok(saved_count)
    }
    
    pub fn get_feed_articles(&self, feed_id: &str, limit: i32) -> Result<Vec<WeChatArticle>> {
        let conn = self.get_connection();

        if limit > 0 {
            let mut stmt = conn.prepare(
                "SELECT id, mp_id, title, pic_url, publish_time, created_at, updated_at
                 FROM wechat_articles WHERE mp_id = ?1 ORDER BY publish_time DESC LIMIT ?2"
            )?;

            let articles = stmt.query_map(params![feed_id, limit], |row| {
                let article_id: String = row.get(0)?;
                Ok(WeChatArticle {
                    id: row.get(0)?,
                    mp_id: row.get(1)?,
                    title: row.get(2)?,
                    url: format!("https://mp.weixin.qq.com/s/{}", article_id),
                    pic_url: row.get(3)?,
                    publish_time: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, mp_id, title, pic_url, publish_time, created_at, updated_at
                 FROM wechat_articles WHERE mp_id = ?1 ORDER BY publish_time DESC"
            )?;

            let articles = stmt.query_map(params![feed_id], |row| {
                let article_id: String = row.get(0)?;
                Ok(WeChatArticle {
                    id: row.get(0)?,
                    mp_id: row.get(1)?,
                    title: row.get(2)?,
                    url: format!("https://mp.weixin.qq.com/s/{}", article_id),
                    pic_url: row.get(3)?,
                    publish_time: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        }
    }
    
    pub fn get_all_articles(&self, limit: i32) -> Result<Vec<WeChatArticle>> {
        let conn = self.get_connection();

        if limit > 0 {
            let mut stmt = conn.prepare(
                "SELECT id, mp_id, title, pic_url, publish_time, created_at, updated_at
                 FROM wechat_articles ORDER BY publish_time DESC LIMIT ?1"
            )?;

            let articles = stmt.query_map(params![limit], |row| {
                let article_id: String = row.get(0)?;
                Ok(WeChatArticle {
                    id: row.get(0)?,
                    mp_id: row.get(1)?,
                    title: row.get(2)?,
                    url: format!("https://mp.weixin.qq.com/s/{}", article_id),
                    pic_url: row.get(3)?,
                    publish_time: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, mp_id, title, pic_url, publish_time, created_at, updated_at
                 FROM wechat_articles ORDER BY publish_time DESC"
            )?;

            let articles = stmt.query_map([], |row| {
                let article_id: String = row.get(0)?;
                Ok(WeChatArticle {
                    id: row.get(0)?,
                    mp_id: row.get(1)?,
                    title: row.get(2)?,
                    url: format!("https://mp.weixin.qq.com/s/{}", article_id),
                    pic_url: row.get(3)?,
                    publish_time: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        }
    }
    
    // ========== 黑名单操作 ==========
    
    pub fn add_to_blacklist(&self, account_id: &str, reason: &str, banned_hours: i32) -> Result<()> {
        let mut conn = self.get_connection();
        
        // 使用事务来避免死锁
        let tx = conn.transaction()?;
        
        let banned_until = chrono::Utc::now().timestamp() + (banned_hours as i64 * 3600);
        let id = uuid::Uuid::new_v4().to_string();
        
        // 先检查是否已经在黑名单中
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM account_blacklist WHERE account_id = ?1",
            params![account_id],
            |row| row.get(0),
        )?;
        
        if count == 0 {
            tx.execute(
                "INSERT INTO account_blacklist (id, account_id, reason, banned_until, created_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                params![id, account_id, reason, banned_until],
            )?;
        }
        
        // 在同一个事务中禁用账号，避免重复获取连接
        tx.execute(
            "UPDATE wechat_accounts SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![0, account_id],
        )?;
        
        // 提交事务
        tx.commit()?;
        
        Ok(())
    }
    
    pub fn is_account_banned(&self, account_id: &str) -> Result<bool> {
        let conn = self.get_connection();
        let now = chrono::Utc::now().timestamp();
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM account_blacklist 
             WHERE account_id = ?1 AND banned_until > ?2",
            params![account_id, now],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
    
    pub fn get_available_accounts(&self) -> Result<Vec<WeChatAccount>> {
        let conn = self.get_connection();
        let now = chrono::Utc::now().timestamp();
        
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.avatar, a.vid, a.token, a.status, a.created_at, a.updated_at
             FROM wechat_accounts a
             LEFT JOIN account_blacklist b ON a.id = b.account_id AND b.banned_until > ?1
             WHERE a.status = 1 AND b.id IS NULL
             ORDER BY a.created_at DESC"
        )?;
        
        let accounts = stmt.query_map(params![now], |row| {
            Ok(WeChatAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar: row.get(2)?,
                vid: row.get(3)?,
                token: row.get(4)?,
                status: row.get(5)?,
                is_banned: false, // 可用账号不在黑名单中
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(accounts)
    }
    
    pub fn cleanup_expired_blacklist(&self) -> Result<usize> {
        let conn = self.get_connection();
        let now = chrono::Utc::now().timestamp();
        
        let rows_affected = conn.execute(
            "DELETE FROM account_blacklist WHERE banned_until <= ?1",
            params![now],
        )?;
        
        Ok(rows_affected)
    }
    
    // ========== 设置相关操作 ==========
    
    // LLM 配置操作
    pub fn insert_llm_config(&self, config: &LlmConfig) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT INTO llm_configs (id, name, api_key, endpoint, model_id, temperature, max_tokens, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                config.id,
                config.name,
                config.api_key,
                config.endpoint,
                config.model_id,
                config.temperature,
                config.max_tokens,
                config.enabled,
                config.created_at,
                config.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_llm_configs(&self) -> Result<Vec<LlmConfig>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, name, api_key, endpoint, model_id, temperature, max_tokens, enabled, created_at, updated_at
             FROM llm_configs ORDER BY created_at DESC"
        )?;
        
        let configs = stmt.query_map([], |row| {
            Ok(LlmConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                api_key: row.get(2)?,
                endpoint: row.get(3)?,
                model_id: row.get(4)?,
                temperature: row.get(5)?,
                max_tokens: row.get(6)?,
                enabled: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(configs)
    }
    
    pub fn update_llm_config(&self, id: &str, config: &LlmConfig) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE llm_configs SET name = ?1, api_key = ?2, endpoint = ?3, model_id = ?4, 
             temperature = ?5, max_tokens = ?6, enabled = ?7, updated_at = datetime('now') WHERE id = ?8",
            params![
                config.name,
                config.api_key,
                config.endpoint,
                config.model_id,
                config.temperature,
                config.max_tokens,
                config.enabled,
                id,
            ],
        )?;
        Ok(())
    }
    
    pub fn delete_llm_config(&self, id: &str) -> Result<()> {
        let conn = self.get_connection();
        conn.execute("DELETE FROM llm_configs WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    pub fn toggle_llm_config(&self, id: &str) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE llm_configs SET enabled = NOT enabled, updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
    
    pub fn get_enabled_llm_config(&self) -> Result<Option<LlmConfig>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, name, api_key, endpoint, model_id, temperature, max_tokens, enabled, created_at, updated_at
             FROM llm_configs WHERE enabled = 1 ORDER BY updated_at DESC LIMIT 1"
        )?;
        
        let config = stmt.query_row([], |row| {
            Ok(LlmConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                api_key: row.get(2)?,
                endpoint: row.get(3)?,
                model_id: row.get(4)?,
                temperature: row.get(5)?,
                max_tokens: row.get(6)?,
                enabled: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }).optional()?;
        
        Ok(config)
    }
    
    // 提示词模板操作
    pub fn insert_prompt_template(&self, template: &PromptTemplate) -> Result<()> {
        let conn = self.get_connection();
        
        // 如果设置为默认，先将其他模板设为非默认
        if template.is_default {
            conn.execute("UPDATE prompt_templates SET is_default = 0", [])?;
        }
        
        conn.execute(
            "INSERT INTO prompt_templates (id, name, template, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                template.id,
                template.name,
                template.template,
                template.is_default,
                template.created_at,
                template.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_prompt_templates(&self) -> Result<Vec<PromptTemplate>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, name, template, is_default, created_at, updated_at
             FROM prompt_templates ORDER BY is_default DESC, created_at DESC"
        )?;
        
        let templates = stmt.query_map([], |row| {
            Ok(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template: row.get(2)?,
                is_default: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(templates)
    }
    
    pub fn update_prompt_template(&self, id: &str, template: &PromptTemplate) -> Result<()> {
        let conn = self.get_connection();
        
        // 如果设置为默认，先将其他模板设为非默认
        if template.is_default {
            conn.execute("UPDATE prompt_templates SET is_default = 0 WHERE id != ?1", params![id])?;
        }
        
        conn.execute(
            "UPDATE prompt_templates SET name = ?1, template = ?2, is_default = ?3, updated_at = datetime('now') WHERE id = ?4",
            params![
                template.name,
                template.template,
                template.is_default,
                id,
            ],
        )?;
        Ok(())
    }
    
    pub fn delete_prompt_template(&self, id: &str) -> Result<()> {
        let conn = self.get_connection();
        conn.execute("DELETE FROM prompt_templates WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    pub fn get_default_prompt_template(&self) -> Result<Option<PromptTemplate>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, name, template, is_default, created_at, updated_at
             FROM prompt_templates WHERE is_default = 1 LIMIT 1"
        )?;
        
        let template = stmt.query_row([], |row| {
            Ok(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template: row.get(2)?,
                is_default: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).optional()?;
        
        Ok(template)
    }
    
    // 获取所有设置
    pub fn get_all_settings(&self) -> Result<AllSettings> {
        let llm_configs = self.get_all_llm_configs()?;
        let prompt_templates = self.get_all_prompt_templates()?;
        
        Ok(AllSettings {
            llm_configs,
            prompt_templates,
        })
    }
    
    // ========== 智能分析操作 ==========
    
    // 分析任务操作
    pub fn insert_analysis_task(&self, task: &AnalysisTask) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT INTO analysis_tasks (id, status, total_articles, processed_articles, 
             success_count, failed_count, start_time, end_time, error_message, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task.id,
                task.status,
                task.total_articles,
                task.processed_articles,
                task.success_count,
                task.failed_count,
                task.start_time,
                task.end_time,
                task.error_message,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_analysis_task(&self, task_id: &str) -> Result<Option<AnalysisTask>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, status, total_articles, processed_articles, success_count, failed_count,
             start_time, end_time, error_message, created_at, updated_at
             FROM analysis_tasks WHERE id = ?1"
        )?;
        
        let task = stmt.query_row(params![task_id], |row| {
            Ok(AnalysisTask {
                id: row.get(0)?,
                status: row.get(1)?,
                total_articles: row.get(2)?,
                processed_articles: row.get(3)?,
                success_count: row.get(4)?,
                failed_count: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                error_message: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).optional()?;
        
        Ok(task)
    }
    
    pub fn update_analysis_task_status(&self, task_id: &str, status: &str, processed_articles: i32, 
                                     success_count: i32, failed_count: i32, error_message: Option<String>) -> Result<()> {
        let conn = self.get_connection();
        let end_time = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().timestamp())
        } else {
            None
        };
        
        conn.execute(
            "UPDATE analysis_tasks SET status = ?1, processed_articles = ?2, 
             success_count = ?3, failed_count = ?4, error_message = ?5, 
             end_time = ?6, updated_at = datetime('now') WHERE id = ?7",
            params![
                status,
                processed_articles,
                success_count,
                failed_count,
                error_message,
                end_time,
                task_id,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_analysis_tasks(&self) -> Result<Vec<AnalysisTask>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, status, total_articles, processed_articles, success_count, failed_count,
             start_time, end_time, error_message, created_at, updated_at
             FROM analysis_tasks ORDER BY created_at DESC"
        )?;
        
        let tasks = stmt.query_map([], |row| {
            Ok(AnalysisTask {
                id: row.get(0)?,
                status: row.get(1)?,
                total_articles: row.get(2)?,
                processed_articles: row.get(3)?,
                success_count: row.get(4)?,
                failed_count: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                error_message: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(tasks)
    }
    
    pub fn get_latest_analysis_task(&self) -> Result<Option<AnalysisTask>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, status, total_articles, processed_articles, success_count, failed_count,
             start_time, end_time, error_message, created_at, updated_at
             FROM analysis_tasks ORDER BY created_at DESC LIMIT 1"
        )?;
        
        let task = stmt.query_row([], |row| {
            Ok(AnalysisTask {
                id: row.get(0)?,
                status: row.get(1)?,
                total_articles: row.get(2)?,
                processed_articles: row.get(3)?,
                success_count: row.get(4)?,
                failed_count: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                error_message: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).optional()?;
        
        Ok(task)
    }
    
    // 分析结果操作
    pub fn insert_analyzed_news(&self, news: &AnalyzedNews) -> Result<()> {
        let conn = self.get_connection();
        
        conn.execute(
            "INSERT INTO analyzed_news (id, task_id, article_id, title, content, summary, 
             is_soft_news, industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                news.id,
                news.task_id,
                news.article_id,
                news.title,
                news.content,
                news.summary,
                news.is_soft_news,
                news.industry_type,
                news.news_type,
                news.confidence,
                news.keywords,
                news.original_url,
                news.analyzed_at,
                news.created_at,
                news.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_analyzed_news(&self, task_id: &str, limit: Option<i32>) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news WHERE task_id = ?1 ORDER BY created_at DESC{}",
            limit_clause
        );
        
        let mut stmt = conn.prepare(&sql)?;
        let news_list = stmt.query_map(params![task_id], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(news_list)
    }
    
    pub fn clear_analyzed_news(&self, task_id: &str) -> Result<usize> {
        let conn = self.get_connection();
        let rows_affected = conn.execute(
            "DELETE FROM analyzed_news WHERE task_id = ?1",
            params![task_id],
        )?;
        Ok(rows_affected)
    }
    
    // 数据库迁移方法
    pub fn migrate_analyzed_news_table(&self) -> Result<()> {
        let conn = self.get_connection();
        
        // 检查所有字段是否存在
        let mut stmt = conn.prepare("PRAGMA table_info(analyzed_news)")?;
        let columns = stmt.query_map([], |row| {
            let column_name: String = row.get(1)?;
            Ok(column_name)
        })?;
        
        // 收集所有列名
        let column_names: Result<Vec<String>, _> = columns.collect();
        let column_names = column_names?;
        
        // 检查并添加 original_url 字段
        if !column_names.iter().any(|col| col == "original_url") {
            log::info!("正在添加 original_url 字段到 analyzed_news 表");
            conn.execute("ALTER TABLE analyzed_news ADD COLUMN original_url TEXT NOT NULL DEFAULT ''", [])?;
        }
        
        // 检查并添加 analyzed_at 字段
        if !column_names.iter().any(|col| col == "analyzed_at") {
            log::info!("正在添加 analyzed_at 字段到 analyzed_news 表");
            conn.execute("ALTER TABLE analyzed_news ADD COLUMN analyzed_at TEXT NOT NULL DEFAULT ''", [])?;
        }
        
        Ok(())
    }
    
    pub fn get_analyzed_news_by_filter(&self, task_id: &str, is_soft_news: Option<bool>, 
                                     industry_type: Option<String>, news_type: Option<String>) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        
        let mut sql = String::from(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news WHERE task_id = ?1"
        );
        
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(task_id.to_string())];
        
        if let Some(is_soft) = is_soft_news {
            sql.push_str(" AND is_soft_news = ?");
            param_values.push(Box::new(if is_soft { 1 } else { 0 }));
        }
        
        if let Some(industry) = &industry_type {
            sql.push_str(" AND industry_type = ?");
            param_values.push(Box::new(industry.clone()));
        }
        
        if let Some(news_type_filter) = &news_type {
            sql.push_str(" AND news_type = ?");
            param_values.push(Box::new(news_type_filter.clone()));
        }
        
        sql.push_str(" ORDER BY created_at DESC");
        
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        
        let news_list = stmt.query_map(&param_refs[..], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(news_list)
    }
    
    // 获取所有分析结果（用于数据修复）
    pub fn get_all_analyzed_news(&self) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news ORDER BY created_at DESC"
        )?;
        
        let news_list = stmt.query_map([], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(news_list)
    }
    
    // 获取所有分析结果（带限制，用于前端显示）
    pub fn get_all_analyzed_news_with_limit(&self, limit: Option<i32>) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        
        log::info!("开始查询所有分析结果...");
        
        // 查询所有必要字段
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news ORDER BY created_at DESC{}",
            limit_clause
        );
        
        log::info!("执行SQL: {}", sql);
        
        let mut stmt = conn.prepare(&sql)?;
        let news_list = stmt.query_map([], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        log::info!("查询完成，返回 {} 条记录", news_list.len());
        
        Ok(news_list)
    }
    
    // 更新分析结果（用于数据修复）
    pub fn update_analyzed_news(&self, news: &AnalyzedNews) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE analyzed_news SET title = ?1, content = ?2, summary = ?3, is_soft_news = ?4, 
             industry_type = ?5, news_type = ?6, confidence = ?7, keywords = ?8, original_url = ?9, 
             analyzed_at = ?10, updated_at = datetime('now') WHERE id = ?11",
            params![
                news.title,
                news.content,
                news.summary,
                news.is_soft_news,
                news.industry_type,
                news.news_type,
                news.confidence,
                news.keywords,
                news.original_url,
                news.analyzed_at,
                news.id,
            ],
        )?;
        Ok(())
    }
    
    // 删除单个分析结果
    pub fn delete_analyzed_news(&self, news_id: &str) -> Result<()> {
        let conn = self.get_connection();
        conn.execute("DELETE FROM analyzed_news WHERE id = ?1", params![news_id])?;
        Ok(())
    }
    
    
    // 获取近一个月的分析结果统计
    pub fn get_recent_month_stats(&self) -> Result<(i64, i64)> {
        let conn = self.get_connection();
        
        // 计算一个月前的时间戳
        let one_month_ago = chrono::Utc::now().timestamp() - (30 * 24 * 60 * 60);
        
        // 统计近一个月的任务数量和新闻数量
        let task_count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT task_id) FROM analysis_tasks WHERE created_at >= ?1",
            params![one_month_ago],
            |row| row.get(0),
        )?;
        
        let news_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM analyzed_news WHERE created_at >= ?1",
            params![one_month_ago],
            |row| row.get(0),
        )?;
        
        Ok((task_count, news_count))
    }
    
    // ========== 去重相关操作 ==========
    
    // 获取指定天数内的分析结果（用于去重检查）
    pub fn get_analyzed_news_since(&self, days: i64) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);
        
        let mut stmt = conn.prepare(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news WHERE created_at >= ?1 ORDER BY created_at DESC"
        )?;
        
        let news_list = stmt.query_map(params![cutoff_time], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(news_list)
    }
    
    // 检查是否存在完全相同的新闻
    pub fn check_exact_duplicate(&self, title: &str, summary: &str, days: i64) -> Result<bool> {
        let conn = self.get_connection();
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM analyzed_news 
             WHERE title = ?1 AND summary = ?2 AND created_at >= ?3",
            params![title, summary, cutoff_time],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
    
    // 检查标题相似的新闻
    pub fn check_similar_title(&self, title: &str, similarity_threshold: f64, days: i64) -> Result<Vec<AnalyzedNews>> {
        let conn = self.get_connection();
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);
        
        let mut stmt = conn.prepare(
            "SELECT id, task_id, article_id, title, content, summary, is_soft_news, 
             industry_type, news_type, confidence, keywords, original_url, analyzed_at, created_at, updated_at
             FROM analyzed_news WHERE created_at >= ?1 ORDER BY created_at DESC"
        )?;
        
        let news_list = stmt.query_map(params![cutoff_time], |row| {
            Ok(AnalyzedNews {
                id: row.get(0)?,
                task_id: row.get(1)?,
                article_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                is_soft_news: row.get(6)?,
                industry_type: row.get(7)?,
                news_type: row.get(8)?,
                confidence: row.get(9)?,
                keywords: row.get(10)?,
                original_url: row.get(11)?,
                analyzed_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        // 筛选出标题相似的新闻
        let similar_news: Vec<AnalyzedNews> = news_list.into_iter()
            .filter(|existing| {
                let similarity = Self::calculate_text_similarity_db(title, &existing.title);
                similarity >= similarity_threshold
            })
            .collect();
        
        Ok(similar_news)
    }
    
    // ========== 辅助函数 ==========
    
    // 计算文本相似度（数据库模块版本）
    fn calculate_text_similarity_db(text1: &str, text2: &str) -> f64 {
        if text1.is_empty() && text2.is_empty() {
            return 1.0;
        }
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }
        
        let distance = Self::levenshtein_distance_db(text1, text2);
        let max_len = text1.len().max(text2.len());
        
        if max_len == 0 {
            return 1.0;
        }
        
        1.0 - (distance as f64 / max_len as f64)
    }
    
    // 计算编辑距离（数据库模块版本）
    fn levenshtein_distance_db(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // 初始化第一行和第一列
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        // 填充矩阵
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = *[
                    &matrix[i - 1][j] + 1,      // 删除
                    &matrix[i][j - 1] + 1,      // 插入
                    &matrix[i - 1][j - 1] + cost // 替换
                ].iter().min().unwrap();
            }
        }
        
        matrix[len1][len2]
    }

    // ========== RSS订阅源操作 ==========
    
    pub fn insert_rss_feed(&self, feed: &RSSFeed) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT OR REPLACE INTO rss_feeds (id, title, url, website_url, description, category, status, last_fetched, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                feed.id,
                feed.title,
                feed.url,
                feed.website_url,
                feed.description,
                feed.category,
                feed.status,
                feed.last_fetched,
                feed.created_at,
                feed.updated_at,
            ],
        )?;
        Ok(())
    }
    
    pub fn get_all_rss_feeds(&self) -> Result<Vec<RSSFeed>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare(
            "SELECT id, title, url, website_url, description, category, status, last_fetched, created_at, updated_at
             FROM rss_feeds ORDER BY updated_at DESC"
        )?;
        
        let feeds = stmt.query_map([], |row| {
            Ok(RSSFeed {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                website_url: row.get(3)?,
                description: row.get(4)?,
                category: row.get(5)?,
                status: row.get(6)?,
                last_fetched: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        
        Ok(feeds)
    }
    
    pub fn delete_rss_feed(&self, id: &str) -> Result<()> {
        let mut conn = self.get_connection();
        // 使用事务确保数据一致性
        let tx = conn.transaction()?;
        
        // 先删除该RSS订阅源的所有文章
        tx.execute("DELETE FROM rss_articles WHERE feed_id = ?1", params![id])?;
        
        // 再删除RSS订阅源
        tx.execute("DELETE FROM rss_feeds WHERE id = ?1", params![id])?;
        
        tx.commit()?;
        Ok(())
    }
    
    pub fn update_rss_feed_status(&self, id: &str, status: i32) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE rss_feeds SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }
    
    pub fn update_rss_feed_last_fetched(&self, id: &str, last_fetched: i64) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE rss_feeds SET last_fetched = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![last_fetched, id],
        )?;
        Ok(())
    }
    
    // ========== RSS文章操作 ==========
    
    pub fn insert_rss_article(&self, article: &RSSArticle) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT OR IGNORE INTO rss_articles (id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                article.id,
                article.feed_id,
                article.title,
                article.url,
                article.content,
                article.summary,
                article.author,
                article.publish_time,
                article.created_at,
                article.updated_at,
            ],
        )?;
        Ok(())
    }
    
    /// 批量插入RSS文章（优化性能，保持去重功能）
    pub fn insert_rss_articles_batch(&self, articles: &[RSSArticle]) -> Result<usize> {
        let mut conn = self.get_connection();
        let tx = conn.transaction()?;
        
        let mut saved_count = 0;
        for article in articles {
            let rows_affected = tx.execute(
                "INSERT OR IGNORE INTO rss_articles (id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    article.id,
                    article.feed_id,
                    article.title,
                    article.url,
                    article.content,
                    article.summary,
                    article.author,
                    article.publish_time,
                    article.created_at,
                    article.updated_at,
                ],
            )?;
            
            // rows_affected == 1 表示真正插入了新文章
            // rows_affected == 0 表示文章已存在，被忽略
            saved_count += rows_affected;
        }
        
        tx.commit()?;
        Ok(saved_count)
    }
    
    pub fn get_rss_feed_articles(&self, feed_id: &str, limit: i32) -> Result<Vec<RSSArticle>> {
        let conn = self.get_connection();

        if limit > 0 {
            let mut stmt = conn.prepare(
                "SELECT id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at
                 FROM rss_articles WHERE feed_id = ?1 ORDER BY publish_time DESC LIMIT ?2"
            )?;

            let articles = stmt.query_map(params![feed_id, limit], |row| {
                Ok(RSSArticle {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    publish_time: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at
                 FROM rss_articles WHERE feed_id = ?1 ORDER BY publish_time DESC"
            )?;

            let articles = stmt.query_map(params![feed_id], |row| {
                Ok(RSSArticle {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    publish_time: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        }
    }
    
    pub fn get_all_rss_articles(&self, limit: i32) -> Result<Vec<RSSArticle>> {
        let conn = self.get_connection();

        if limit > 0 {
            let mut stmt = conn.prepare(
                "SELECT id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at
                 FROM rss_articles ORDER BY publish_time DESC LIMIT ?1"
            )?;

            let articles = stmt.query_map(params![limit], |row| {
                Ok(RSSArticle {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    publish_time: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, feed_id, title, url, content, summary, author, publish_time, created_at, updated_at
                 FROM rss_articles ORDER BY publish_time DESC"
            )?;

            let articles = stmt.query_map([], |row| {
                Ok(RSSArticle {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    publish_time: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(articles)
        }
    }
    
    // ========== 统一文章操作 ==========
    
    /// 获取指定订阅源的所有文章（支持微信公众号和RSS）
    pub fn get_feed_articles_unified(&self, feed_id: &str, limit: i32) -> Result<Vec<Article>> {
        let mut all_articles = Vec::new();
        
        // 先尝试获取微信公众号文章
        if let Ok(wechat_articles) = self.get_feed_articles(feed_id, 0) {
            for article in wechat_articles {
                all_articles.push(Article::WeChat(article));
            }
        }
        
        // 再尝试获取RSS文章
        if let Ok(rss_articles) = self.get_rss_feed_articles(feed_id, 0) {
            for article in rss_articles {
                all_articles.push(Article::RSS(article));
            }
        }
        
        // 按发布时间排序
        all_articles.sort_by(|a, b| {
            let time_a = match a {
                Article::WeChat(wa) => wa.publish_time,
                Article::RSS(ra) => ra.publish_time,
            };
            let time_b = match b {
                Article::WeChat(wb) => wb.publish_time,
                Article::RSS(rb) => rb.publish_time,
            };
            time_b.cmp(&time_a) // 降序排列
        });
        
        // 应用限制
        if limit > 0 {
            all_articles.truncate(limit as usize);
        }
        
        Ok(all_articles)
    }
    
    /// 获取所有文章（支持微信公众号和RSS）
    pub fn get_all_articles_unified(&self, limit: i32) -> Result<Vec<Article>> {
        let mut all_articles = Vec::new();
        
        // 获取所有微信公众号文章
        if let Ok(wechat_articles) = self.get_all_articles(0) {
            for article in wechat_articles {
                all_articles.push(Article::WeChat(article));
            }
        }
        
        // 获取所有RSS文章
        if let Ok(rss_articles) = self.get_all_rss_articles(0) {
            for article in rss_articles {
                all_articles.push(Article::RSS(article));
            }
        }
        
        // 按发布时间排序
        all_articles.sort_by(|a, b| {
            let time_a = match a {
                Article::WeChat(wa) => wa.publish_time,
                Article::RSS(ra) => ra.publish_time,
            };
            let time_b = match b {
                Article::WeChat(wb) => wb.publish_time,
                Article::RSS(rb) => rb.publish_time,
            };
            time_b.cmp(&time_a) // 降序排列
        });
        
        // 应用限制
        if limit > 0 {
            all_articles.truncate(limit as usize);
        }
        
        Ok(all_articles)
    }
}
