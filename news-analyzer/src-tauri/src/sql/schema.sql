-- 微信账号表
CREATE TABLE IF NOT EXISTS wechat_accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    avatar TEXT NOT NULL,
    vid TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_wechat_accounts_status ON wechat_accounts(status);
CREATE INDEX IF NOT EXISTS idx_wechat_accounts_vid ON wechat_accounts(vid);

-- 微信订阅源表
CREATE TABLE IF NOT EXISTS wechat_feeds (
    id TEXT PRIMARY KEY,
    mp_name TEXT NOT NULL,
    mp_intro TEXT NOT NULL,
    mp_cover TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    sync_time INTEGER NOT NULL DEFAULT 0,
    update_time INTEGER NOT NULL DEFAULT 0,
    has_history INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_wechat_feeds_status ON wechat_feeds(status);
CREATE INDEX IF NOT EXISTS idx_wechat_feeds_update_time ON wechat_feeds(update_time);

-- 微信文章表
CREATE TABLE IF NOT EXISTS wechat_articles (
    id TEXT PRIMARY KEY,
    mp_id TEXT NOT NULL,
    title TEXT NOT NULL,
    pic_url TEXT NOT NULL,
    publish_time INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (mp_id) REFERENCES wechat_feeds(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_wechat_articles_mp_id ON wechat_articles(mp_id);
CREATE INDEX IF NOT EXISTS idx_wechat_articles_publish_time ON wechat_articles(publish_time);

-- 账号黑名单表（小黑屋）
CREATE TABLE IF NOT EXISTS account_blacklist (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    banned_until INTEGER NOT NULL, -- 封禁到期时间（时间戳）
    created_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES wechat_accounts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_account_blacklist_account_id ON account_blacklist(account_id);
CREATE INDEX IF NOT EXISTS idx_account_blacklist_banned_until ON account_blacklist(banned_until);

-- LLM 配置表
CREATE TABLE IF NOT EXISTS llm_configs (
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
);

CREATE INDEX IF NOT EXISTS idx_llm_configs_enabled ON llm_configs(enabled);

-- 提示词模板表
CREATE TABLE IF NOT EXISTS prompt_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    template TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompt_templates_name ON prompt_templates(name);
CREATE INDEX IF NOT EXISTS idx_prompt_templates_is_default ON prompt_templates(is_default);

-- 智能分析任务表
CREATE TABLE IF NOT EXISTS analysis_tasks (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, running, completed, failed
    total_articles INTEGER NOT NULL DEFAULT 0,
    processed_articles INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    error_message TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_analysis_tasks_status ON analysis_tasks(status);
CREATE INDEX IF NOT EXISTS idx_analysis_tasks_created_at ON analysis_tasks(created_at);

-- 分析结果表
CREATE TABLE IF NOT EXISTS analyzed_news (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    article_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    summary TEXT NOT NULL,
    is_soft_news BOOLEAN NOT NULL DEFAULT 0,
    industry_type TEXT NOT NULL,
    news_type TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.0,
    keywords TEXT,
    original_url TEXT NOT NULL,
    analyzed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES analysis_tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_analyzed_news_task_id ON analyzed_news(task_id);
CREATE INDEX IF NOT EXISTS idx_analyzed_news_article_id ON analyzed_news(article_id);
CREATE INDEX IF NOT EXISTS idx_analyzed_news_created_at ON analyzed_news(created_at);

-- RSS订阅源表
CREATE TABLE IF NOT EXISTS rss_feeds (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    website_url TEXT NOT NULL,
    description TEXT,
    category TEXT,
    status INTEGER NOT NULL DEFAULT 1,
    last_fetched INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rss_feeds_status ON rss_feeds(status);
CREATE INDEX IF NOT EXISTS idx_rss_feeds_category ON rss_feeds(category);
CREATE INDEX IF NOT EXISTS idx_rss_feeds_last_fetched ON rss_feeds(last_fetched);

-- RSS文章表
CREATE TABLE IF NOT EXISTS rss_articles (
    id TEXT PRIMARY KEY,
    feed_id TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    content TEXT,
    summary TEXT,
    author TEXT,
    publish_time INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (feed_id) REFERENCES rss_feeds(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rss_articles_feed_id ON rss_articles(feed_id);
CREATE INDEX IF NOT EXISTS idx_rss_articles_publish_time ON rss_articles(publish_time);
CREATE UNIQUE INDEX IF NOT EXISTS idx_rss_articles_url ON rss_articles(url);
