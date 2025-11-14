use crate::database::Database;
use crate::weread::{WeReadClient, AccountManager};
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Arc<Database>,
    pub weread_client: Arc<Mutex<WeReadClient>>,
    pub account_manager: Arc<Mutex<AccountManager>>,
    // 新增中断标志
    pub refresh_interrupted: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let db = Arc::new(Database::new(db_path)?);

        // 执行数据库迁移
        db.migrate_analyzed_news_table()?;

        let weread_client = Arc::new(Mutex::new(WeReadClient::new("https://weread.111965.xyz")));
        let account_manager = Arc::new(Mutex::new(AccountManager::new()));

        Ok(Self {
            db,
            weread_client,
            account_manager,
            refresh_interrupted: Arc::new(AtomicBool::new(false)),
        })
    }
}
