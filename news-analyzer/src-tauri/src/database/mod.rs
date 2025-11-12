pub mod models;
pub mod operations;

use rusqlite::{Connection, Result, params};
use std::sync::Mutex;
use std::time::Duration;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let mut conn = Connection::open(db_path)?;
        
        // 启用外键约束 - 使用 query_row 而不是 execute
        let _: String = conn.query_row("PRAGMA foreign_keys = ON", [], |row| row.get(0)).unwrap_or_default();
        
        // 执行初始化脚本
        conn.execute_batch(include_str!("../sql/schema.sql"))?;
        
        // 使用DELETE模式而不是WAL模式，避免临时文件问题 - 使用 query_row 而不是 execute
        let _: String = conn.query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0)).unwrap_or_default();
        
        // 检查并添加缺失的字段（用于数据库迁移）
        Self::migrate_database(&mut conn)?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// 数据库迁移：添加新字段
    fn migrate_database(conn: &mut Connection) -> Result<()> {
        // 检查 wechat_feeds 表是否有 has_history 字段
        {
            let mut stmt = conn.prepare("PRAGMA table_info(wechat_feeds)")?;
            let columns: Vec<String> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(1)?)
            })?.collect::<Result<Vec<_>, _>>()?;
            
            if !columns.contains(&"has_history".to_string()) {
                log::info!("检测到旧版本数据库，添加 has_history 字段");
                conn.execute(
                    "ALTER TABLE wechat_feeds ADD COLUMN has_history INTEGER NOT NULL DEFAULT 1",
                    [],
                )?;
            }
        } // stmt 在这里被 drop
        
        // 检查 account_blacklist 表是否存在
        {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='account_blacklist'")?;
            let blacklist_exists: Vec<String> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect::<Result<Vec<_>, _>>()?;
            
            if blacklist_exists.is_empty() {
                log::info!("创建 account_blacklist 表");
                conn.execute(
                    "CREATE TABLE account_blacklist (
                        id TEXT PRIMARY KEY,
                        account_id TEXT NOT NULL,
                        reason TEXT NOT NULL,
                        banned_until INTEGER NOT NULL,
                        created_at TEXT NOT NULL,
                        FOREIGN KEY (account_id) REFERENCES wechat_accounts(id) ON DELETE CASCADE
                    )",
                    [],
                )?;
                
                // 创建索引
                conn.execute("CREATE INDEX idx_account_blacklist_account_id ON account_blacklist(account_id)", [])?;
                conn.execute("CREATE INDEX idx_account_blacklist_banned_until ON account_blacklist(banned_until)", [])?;
            }
        } // stmt 在这里被 drop
        
        // 检查并创建设置相关的表
        Self::migrate_settings_tables(conn)?;
        
        // 检查并创建提示词模板表
        Self::migrate_prompt_templates_table(conn)?;
        
        // 初始化默认数据
        Self::init_default_data(conn)?;
        
        Ok(())
    }
    
    /// 迁移设置相关的表
    fn migrate_settings_tables(conn: &mut Connection) -> Result<()> {
        // 检查 llm_configs 表是否存在
        let llm_configs_exists = {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='llm_configs'")?;
            let results: Vec<String> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect::<Result<Vec<_>, _>>()?;
            !results.is_empty()
        };
        
        if llm_configs_exists {
            // 检查表结构并迁移
            let columns = {
                let mut stmt = conn.prepare("PRAGMA table_info(llm_configs)")?;
                let x = stmt.query_map([], |row| {
                    Ok(row.get::<_, String>(1)?)
                })?.collect::<Result<Vec<_>, _>>()?;
                x
            };
            
            // 如果有旧的 provider 字段，需要迁移
            if columns.contains(&"provider".to_string()) {
                log::info!("检测到旧版本 llm_configs 表，执行数据库迁移：移除 provider 字段");
                
                // 执行真正的表重建迁移
                Self::force_recreate_llm_configs_table(conn)?;
                
            } else if !columns.contains(&"model_id".to_string()) {
                log::info!("检测到旧版本 llm_configs 表，添加 model_id 字段");
                conn.execute(
                    "ALTER TABLE llm_configs ADD COLUMN model_id TEXT NOT NULL DEFAULT ''",
                    [],
                )?;
            }
        } else {
            log::info!("创建 llm_configs 表");
            conn.execute(
                "CREATE TABLE llm_configs (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    api_key TEXT NOT NULL,
                    endpoint TEXT NOT NULL,
                    model_id TEXT NOT NULL,
                    temperature REAL NOT NULL DEFAULT 0.7,
                    max_tokens INTEGER NOT NULL DEFAULT 2000,
                    enabled BOOLEAN NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            // 创建索引
            conn.execute("CREATE INDEX idx_llm_configs_enabled ON llm_configs(enabled)", [])?;
        }
        
        // 检查 industry_types 表是否存在
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='industry_types'")?;
        let industry_types_exists: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?.collect::<Result<Vec<_>, _>>()?;
        
        if industry_types_exists.is_empty() {
            log::info!("创建 industry_types 表");
            conn.execute(
                "CREATE TABLE industry_types (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    icon TEXT NOT NULL DEFAULT '🏭',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            // 创建索引
            conn.execute("CREATE INDEX idx_industry_types_name ON industry_types(name)", [])?;
        }
        
        // 检查 news_types 表是否存在
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='news_types'")?;
        let news_types_exists: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?.collect::<Result<Vec<_>, _>>()?;
        
        if news_types_exists.is_empty() {
            log::info!("创建 news_types 表");
            conn.execute(
                "CREATE TABLE news_types (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    icon TEXT NOT NULL DEFAULT '📰',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            // 创建索引
            conn.execute("CREATE INDEX idx_news_types_name ON news_types(name)", [])?;
        }
        
        Ok(())
    }
    
    /// 强制重建 llm_configs 表（用于删除旧字段）
    fn force_recreate_llm_configs_table(conn: &mut Connection) -> Result<()> {
        log::info!("强制重建 llm_configs 表以移除 provider 字段");
        
        // 开始事务
        let tx = conn.transaction()?;
        
        // 重命名旧表
        tx.execute("ALTER TABLE llm_configs RENAME TO llm_configs_old", [])?;
        
        // 创建新表
        tx.execute(
            "CREATE TABLE llm_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                api_key TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                model_id TEXT NOT NULL,
                temperature REAL NOT NULL DEFAULT 0.7,
                max_tokens INTEGER NOT NULL DEFAULT 2000,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // 创建索引
        tx.execute("CREATE INDEX idx_llm_configs_enabled ON llm_configs(enabled)", [])?;
        
        // 复制数据（排除旧字段，处理可能的 NULL 值）
        tx.execute(
            "INSERT INTO llm_configs (id, name, api_key, endpoint, model_id, temperature, max_tokens, enabled, created_at, updated_at)
             SELECT id, name, api_key, endpoint, 
                    COALESCE(model_id, COALESCE(provider, '')) as model_id,
                    temperature, max_tokens, enabled, created_at, updated_at
             FROM llm_configs_old",
            [],
        )?;
        
        // 删除旧表
        tx.execute("DROP TABLE llm_configs_old", [])?;
        
        // 提交事务
        tx.commit()?;
        
        Ok(())
    }
    
    /// 检查并创建提示词模板表
    fn migrate_prompt_templates_table(conn: &mut Connection) -> Result<()> {
        // 检查 prompt_templates 表是否存在
        let prompt_templates_exists = {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prompt_templates'")?;
            let results: Vec<String> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect::<Result<Vec<_>, _>>()?;
            !results.is_empty()
        };
        
        if !prompt_templates_exists {
            log::info!("创建 prompt_templates 表");
            conn.execute(
                "CREATE TABLE prompt_templates (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    template TEXT NOT NULL,
                    is_default BOOLEAN NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            
            // 创建索引
            conn.execute("CREATE INDEX IF NOT EXISTS idx_prompt_templates_name ON prompt_templates(name)", [])?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_prompt_templates_is_default ON prompt_templates(is_default)", [])?;
        } else {
            // 检查表结构是否正确
            let columns = {
                let mut stmt = conn.prepare("PRAGMA table_info(prompt_templates)")?;
                let rows = stmt.query_map([], |row| {
                    Ok(row.get::<_, String>(1)?)
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            
            // 如果缺少 template 字段，需要重建表
            if !columns.contains(&"template".to_string()) {
                log::info!("检测到旧版本 prompt_templates 表，重建表结构");
                
                // 开始事务
                let tx = conn.transaction()?;
                
                // 备份数据
                let backup_data = if columns.contains(&"id".to_string()) && columns.contains(&"name".to_string()) {
                    let mut stmt = tx.prepare("SELECT id, name FROM prompt_templates")?;
                    let results: Vec<(String, String)> = stmt.query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })?.collect::<Result<Vec<_>, _>>()?;
                    results
                } else {
                    Vec::new()
                };
                
                // 删除旧表
                tx.execute("DROP TABLE prompt_templates", [])?;
                
                // 创建新表
                tx.execute(
                    "CREATE TABLE prompt_templates (
                        id TEXT PRIMARY KEY,
                        name TEXT NOT NULL UNIQUE,
                        template TEXT NOT NULL,
                        is_default BOOLEAN NOT NULL DEFAULT 0,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL
                    )",
                    [],
                )?;
                
                // 创建索引
                tx.execute("CREATE INDEX idx_prompt_templates_name ON prompt_templates(name)", [])?;
                tx.execute("CREATE INDEX idx_prompt_templates_is_default ON prompt_templates(is_default)", [])?;
                
                // 恢复数据（如果有）
                for (id, name) in backup_data {
                    tx.execute(
                        "INSERT INTO prompt_templates (id, name, template, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                        params![id, name, "", 0],
                    )?;
                }
                
                // 提交事务
                tx.commit()?;
            }
        }
        
        Ok(())
    }
    
    /// 初始化默认数据
    fn init_default_data(conn: &mut Connection) -> Result<()> {
        // 检查 industry_types 表是否存在并有数据
        let industry_count: i64 = {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM industry_types")?;
            stmt.query_row([], |row| row.get(0)).unwrap_or(0)
        };
        
        if industry_count == 0 {
            log::info!("初始化默认行业类型数据");
            let default_industries = vec![
                ("21a8c321-5b22-4b4a-bd88-6d1c946aad0f", "数据中心", "数据中心建设、运营、基础设施相关", "🏢"),
                ("ce566a54-bbe7-4d9d-9d54-a83efac31887", "能源电力", "电力供应、能源基础设施、新能源相关", "⚡"),
                ("0da05043-aa47-42d8-893b-e413821490c8", "云计算", "云服务、算力租赁、云基础设施相关", "☁️"),
                ("d413ac3c-aa5a-4e7a-894c-2e224f4f892d", "半导体", "芯片制造、半导体设备、电子元件相关", "💾"),
            ];
            
            for (id, name, description, icon) in default_industries {
                conn.execute(
                    "INSERT INTO industry_types (id, name, description, icon, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    params![id, name, description, icon],
                )?;
            }
        }
        
        // 检查 news_types 表是否存在并有数据
        let news_count: i64 = {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM news_types")?;
            stmt.query_row([], |row| row.get(0)).unwrap_or(0)
        };
        
        if news_count == 0 {
            log::info!("初始化默认新闻类型数据");
            let default_news_types = vec![
                ("f7a40872-a1bf-477d-8d83-0726d70f3179", "投资合作", "企业投资、并购、战略合作相关", "🤝"),
                ("7f08f57b-5d06-42c3-b021-f66b9b5981ae", "项目建设", "新项目建设、扩建、启用相关", "🏗️"),
                ("215689ec-2f17-4d84-8f1f-139c43f694f6", "技术创新", "技术研发、产品创新、技术突破相关", "🔬"),
                ("308fa6d9-d258-466a-8661-1b7a83792de9", "市场扩张", "市场进入、业务扩张、国际化相关", "🌍"),
                ("403dd18e-9958-4b21-8cb8-d962ad2c7a83", "资产交易", "资产买卖、公司收购、股权转让相关", "💰"),
                ("daf0c2cb-bd7e-412a-9795-f78de35715a0", "产品发布", "新产品发布、服务升级、功能更新相关", "📱"),
                ("201c04f3-a26b-4c52-beb4-8f2b16736f32", "政策法规", "政策变化、法规更新、合规要求相关", "⚖️"),
                ("01e0c2a1-cfa6-4a16-b9cb-6e7575620c72", "设备供应", "设备制造、供应链、硬件产品相关", "🔧"),
                ("1f33bd00-666a-4b42-af8b-6c7c03d0f41c", "运营管理", "企业运营、管理优化、效率提升相关", "📊"),
            ];
            
            for (id, name, description, icon) in default_news_types {
                conn.execute(
                    "INSERT INTO news_types (id, name, description, icon, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    params![id, name, description, icon],
                )?;
            }
        }
        
        Ok(())
    }
    
    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
    
    /// 带超时的数据库连接获取，防止死锁
    pub fn get_connection_with_timeout(&self, timeout_ms: u64) -> Result<std::sync::MutexGuard<'_, Connection>> {
        use std::time::Instant;
        
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        
        loop {
            match self.conn.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(_) => {
                    if start.elapsed() > timeout {
                        return Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                            Some("数据库连接超时，请稍后重试".to_string())
                        ));
                    }
                    // 短暂等待后重试
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}
