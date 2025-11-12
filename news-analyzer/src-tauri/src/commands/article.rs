use crate::database::models::{WeChatArticle, Article};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_feed_articles(
    feed_id: String,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let limit = limit.unwrap_or(0); // 0 表示返回所有文章
    state.db.get_feed_articles_unified(&feed_id, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_articles(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let limit = limit.unwrap_or(0); // 0 表示返回所有文章
    state.db.get_all_articles_unified(limit)
        .map_err(|e| e.to_string())
}

// 保持向后兼容性的旧版本命令（只返回微信公众号文章）
#[tauri::command]
pub async fn get_wechat_feed_articles(
    feed_id: String,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<WeChatArticle>, String> {
    let limit = limit.unwrap_or(0); // 0 表示返回所有文章
    state.db.get_feed_articles(&feed_id, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wechat_all_articles(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<WeChatArticle>, String> {
    let limit = limit.unwrap_or(0); // 0 表示返回所有文章
    state.db.get_all_articles(limit)
        .map_err(|e| e.to_string())
}

// 调试命令：获取RSS文章
#[tauri::command]
pub async fn get_rss_articles_debug(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::models::RSSArticle>, String> {
    let limit = limit.unwrap_or(0); // 0 表示返回所有文章
    state.db.get_all_rss_articles(limit)
        .map_err(|e| e.to_string())
}

// 调试命令：获取统一文章并打印详细信息
#[tauri::command]
pub async fn debug_articles(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let limit = limit.unwrap_or(10); // 默认限制10条
    match state.db.get_all_articles_unified(limit) {
        Ok(articles) => {
            let mut debug_info = String::new();
            debug_info.push_str(&format!("总共找到 {} 篇文章:\n\n", articles.len()));
            
            for (index, article) in articles.iter().enumerate() {
                match article {
                    crate::database::models::Article::WeChat(wa) => {
                        debug_info.push_str(&format!(
                            "{}. [微信] {} - {}\n",
                            index + 1,
                            wa.title,
                            wa.url
                        ));
                    }
                    crate::database::models::Article::RSS(ra) => {
                        debug_info.push_str(&format!(
                            "{}. [RSS] {} - {}\n",
                            index + 1,
                            ra.title,
                            ra.url
                        ));
                    }
                }
            }
            
            Ok(debug_info)
        }
        Err(e) => Err(format!("获取文章失败: {}", e))
    }
}
