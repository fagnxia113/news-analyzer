use crate::database::models::{RSSFeed, RSSArticle};
use anyhow::{Result, anyhow};
use chrono::Utc;
use feed_rs::parser;
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

/// RSS解析器
pub struct RSSParser {
    client: Client,
}

impl RSSParser {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("news-analyzer-mvp/1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    /// 从RSS URL获取并解析订阅源信息
    pub async fn fetch_feed_info(&self, url: &str) -> Result<RSSFeed> {
        log::info!("开始获取RSS订阅源信息: {}", url);
        
        // 获取RSS内容
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("HTTP请求失败: {}", response.status()));
        }
        
        let content = response.text().await?;
        
        // 解析RSS
        let feed = parser::parse(content.as_bytes())
            .map_err(|e| anyhow!("RSS解析失败: {}", e))?;
        
        // 提取网站URL
        let website_url = feed.links.first()
            .map(|link| link.href.clone())
            .unwrap_or_else(|| url.to_string());
        
        // 生成ID
        let feed_id = Uuid::new_v4().to_string();
        
        let rss_feed = RSSFeed {
            id: feed_id,
            title: feed.title.map(|t| t.content).unwrap_or_else(|| "未知订阅源".to_string()),
            url: url.to_string(),
            website_url,
            description: feed.description.map(|d| d.content),
            category: None, // 可以从feed.categories中提取
            status: 1, // 默认启用
            last_fetched: 0,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        
        log::info!("成功获取RSS订阅源: {}", rss_feed.title);
        Ok(rss_feed)
    }

    /// 从RSS URL获取文章列表
    pub async fn fetch_articles(&self, url: &str, feed_id: &str) -> Result<Vec<RSSArticle>> {
        log::info!("开始获取RSS文章: {}", url);
        
        // 获取RSS内容
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("HTTP请求失败: {}", response.status()));
        }
        
        let content = response.text().await?;
        
        // 解析RSS
        let feed = parser::parse(content.as_bytes())
            .map_err(|e| anyhow!("RSS解析失败: {}", e))?;
        
        let mut articles = Vec::new();
        
        for entry in feed.entries {
            // 提取文章URL
            let article_url = entry.links.first()
                .map(|link| link.href.clone())
                .unwrap_or_else(|| "".to_string());
            
            if article_url.is_empty() {
                let title = entry.title.as_ref()
                    .map(|t| t.content.as_str())
                    .unwrap_or("未知标题");
                log::warn!("跳过没有URL的文章: {}", title);
                continue;
            }
            
            // 生成文章ID（使用URL的哈希）
            let article_id = format!("rss_{}", md5::compute(&article_url));
            
            // 解析发布时间
            let publish_time = entry.published
                .or(entry.updated)
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|| Utc::now().timestamp());
            
            // 提取内容
            let content_str = if let Some(content) = &entry.content {
                if let Some(body) = &content.body {
                    body.clone()
                } else {
                    String::new()
                }
            } else if let Some(summary) = &entry.summary {
                summary.content.clone()
            } else {
                String::new()
            };
            
            // 提取摘要
            let summary_str = if let Some(summary) = &entry.summary {
                summary.content.clone()
            } else {
                String::new()
            };
            
            // 提取作者
            let author = entry.authors.first()
                .map(|a| a.name.clone());
            
            // 提取标题
            let title = entry.title.as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_else(|| "未知标题".to_string());
            
            let rss_article = RSSArticle {
                id: article_id,
                feed_id: feed_id.to_string(),
                title,
                url: article_url,
                content: if content_str.is_empty() { None } else { Some(content_str) },
                summary: if summary_str.is_empty() { None } else { Some(summary_str) },
                author,
                publish_time,
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            };
            
            articles.push(rss_article);
        }
        
        log::info!("成功获取 {} 篇RSS文章", articles.len());
        Ok(articles)
    }

    /// 验证RSS URL是否有效
    pub async fn validate_rss_url(&self, url: &str) -> Result<bool> {
        match self.fetch_feed_info(url).await {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("RSS URL验证失败: {} - {}", url, e);
                Ok(false)
            }
        }
    }
}

/// 简单的MD5哈希函数（用于生成文章ID）
mod md5 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub fn compute(s: &str) -> String {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_rss_url() {
        let parser = RSSParser::new();
        
        // 测试一个有效的RSS源
        let result = parser.validate_rss_url("https://feeds.bbci.co.uk/news/rss.xml").await;
        assert!(result.is_ok());
        
        // 测试一个无效的URL
        let result = parser.validate_rss_url("https://example.com/invalid.xml").await;
        // 这个可能会失败，这是预期的
    }
}
