use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatAccount {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub vid: String,
    pub token: String,
    pub status: i32,
    pub is_banned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatFeed {
    pub id: String,
    pub mp_name: String,
    pub mp_intro: String,
    pub mp_cover: String,
    pub status: i32,
    pub sync_time: i64,
    pub update_time: i64,
    pub has_history: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatArticle {
    pub id: String,
    pub mp_id: String,
    pub title: String,
    pub url: String,
    pub pic_url: Option<String>,
    pub publish_time: i64,
    pub created_at: String,
    pub updated_at: String,
}

// RSS订阅源数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSSFeed {
    pub id: String,
    pub title: String,
    pub url: String,           // RSS feed URL
    pub website_url: String,   // 网站主页URL
    pub description: Option<String>,
    pub category: Option<String>, // 分类：科技、财经、新闻等
    pub status: i32,           // 状态：1-启用，0-禁用
    pub last_fetched: i64,     // 最后获取时间
    pub created_at: String,
    pub updated_at: String,
}

// RSS文章数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSSArticle {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub publish_time: i64,
    pub created_at: String,
    pub updated_at: String,
}

// 统一的订阅源枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FeedSource {
    WeChat(WeChatFeed),
    RSS(RSSFeed),
}

// 统一的文章枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source_type")]
pub enum Article {
    WeChat(WeChatArticle),
    RSS(RSSArticle),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTask {
    pub id: String,
    pub status: String,
    pub total_articles: i32,
    pub processed_articles: i32,
    pub success_count: i32,
    pub failed_count: i32,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginQRCode {
    pub uuid: String,
    pub scan_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub vid: Option<String>,
    pub token: Option<String>,
    pub username: Option<String>,
    pub message: Option<String>,
}

// 设置相关数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub endpoint: String,
    pub model_id: String,
    pub temperature: f64,
    pub max_tokens: i32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSettings {
    pub llm_configs: Vec<LlmConfig>,
    pub prompt_templates: Vec<PromptTemplate>,
}

// 智能分析相关数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedNews {
    pub id: String,
    pub task_id: String,
    pub article_id: String,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub is_soft_news: bool,
    pub industry_type: String,
    pub news_type: String,
    pub confidence: f64,
    pub keywords: String,
    pub original_url: String,
    pub analyzed_at: String,
    pub created_at: String,
    pub updated_at: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAnalysisResponse {
    pub has_news: bool,
    pub news_list: Vec<AnalyzedNewsItem>,
    pub analysis_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedNewsItem {
    pub title: String,
    pub summary: String,
    pub industry_type: String,
    pub news_type: String,
    pub confidence: f64,
}
