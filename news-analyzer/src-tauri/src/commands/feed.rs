use crate::database::models::{WeChatFeed, RSSFeed, RSSArticle};
use crate::state::AppState;
use crate::rss::RSSParser;
use tauri::{State, AppHandle, Emitter};
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct RefreshProgressEvent {
    pub current: usize,
    pub total: usize,
    pub status: String,
    pub log: String,
    pub feed_name: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RefreshLogEvent {
    pub timestamp: String,
    pub level: String, // "info", "warn", "error"
    pub message: String,
    pub feed_name: Option<String>,
}

#[tauri::command]
pub async fn add_feed_from_url(
    url: String,
    account_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("开始添加订阅源: url={}, account_id={}", url, account_id);
    
    // 获取账号信息
    let accounts = state.db.get_all_accounts().map_err(|e| {
        log::error!("获取账号列表失败: {}", e);
        e.to_string()
    })?;
    
    let account = accounts.into_iter()
        .find(|acc| acc.id == account_id)
        .ok_or_else(|| {
            log::error!("账号不存在: {}", account_id);
            "账号不存在".to_string()
        })?;
    
    log::info!("使用账号: {}", account.name);
    
    // 获取公众号信息
    let client = state.weread_client.lock().await;
    let mp_info = client.get_mp_info(&url, &account.vid, &account.token)
        .await
        .map_err(|e| {
            log::error!("获取公众号信息失败: {}", e);
            e.to_string()
        })?;
    
    log::info!("获取到公众号信息: {} ({})", mp_info.name, mp_info.id);
    
    // 保存到数据库
    let feed = WeChatFeed {
        id: mp_info.id.clone(),
        mp_name: mp_info.name.clone(),
        mp_intro: mp_info.intro,
        mp_cover: mp_info.cover,
        status: 1,
        sync_time: 0,
        update_time: mp_info.update_time as i64,
        has_history: 1,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    
    // 添加延迟以避免数据库操作过于频繁
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    state.db.insert_feed(&feed)
        .map_err(|e| {
            log::error!("保存订阅源到数据库失败: {}", e);
            e.to_string()
        })?;
    
    log::info!("订阅源添加成功: {}", feed.mp_name);
    Ok(feed.mp_name) // 返回简单的字符串而不是整个结构
}

#[tauri::command]
pub async fn get_all_feeds(state: State<'_, AppState>) -> Result<Vec<WeChatFeed>, String> {
    state.db.get_all_feeds()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_feed(
    feed_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.delete_feed(&feed_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_feed(
    feed_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    log::info!("开始刷新订阅源: {}", feed_id);
    
    // 先尝试作为微信公众号刷新
    let wechat_feeds = state.db.get_all_feeds().map_err(|e| {
        log::error!("获取微信公众号列表失败: {}", e);
        e.to_string()
    })?;
    
    if let Some(feed) = wechat_feeds.into_iter().find(|f| f.id == feed_id) {
        // 这是微信公众号，使用原有逻辑
        return refresh_wechat_feed(feed, app_handle, state).await;
    }
    
    // 再尝试作为RSS订阅源刷新
    let rss_feeds = state.db.get_all_rss_feeds().map_err(|e| {
        log::error!("获取RSS订阅源列表失败: {}", e);
        e.to_string()
    })?;
    
    if let Some(feed) = rss_feeds.into_iter().find(|f| f.id == feed_id) {
        // 这是RSS订阅源，使用RSS刷新逻辑
        return refresh_rss_feed_by_id(feed, app_handle, state).await;
    }
    
    // 都没找到
    log::error!("订阅源不存在: {}", feed_id);
    Err("订阅源不存在".to_string())
}

// 刷新微信公众号的内部函数
async fn refresh_wechat_feed(
    feed: crate::database::models::WeChatFeed,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    
    let feed_name = feed.mp_name.clone();
    log::info!("刷新订阅源: {}", feed_name);
    
    // 发送开始事件
    let progress_event = RefreshProgressEvent {
        current: 0,
        total: 1,
        status: format!("正在刷新 {}", feed_name),
        log: format!("开始刷新订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("开始刷新订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    // 清理过期的黑名单记录
    if let Err(e) = state.db.cleanup_expired_blacklist() {
        log::warn!("清理过期黑名单失败: {}", e);
    }
    
    // 获取可用账号（自动排除被封禁的账号）
    let available_accounts = state.db.get_available_accounts().map_err(|e| {
        log::error!("获取可用账号列表失败: {}", e);
        e.to_string()
    })?;
    
    if available_accounts.is_empty() {
        return Err("所有账号都不可用，请稍后再试或添加新账号".to_string());
    }
    
    // 随机选择一个可用账号
    let account = available_accounts[fastrand::usize(..) % available_accounts.len()].clone();
    
    log::info!("使用账号: {}", account.name);
    
    // 发送账号选择事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("使用账号: {}", account.name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    // 获取文章列表（分页）
    let mut page = 1;
    let mut total_saved = 0;
    let mut has_history = true;
    
    // 根据刷新类型设置不同的页数限制
    let max_pages = 3; // 单个订阅源获取前3页
    
    while has_history && page <= max_pages {
        log::info!("获取第{}页文章", page);
        
        // 发送页面获取事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: format!("正在获取第{}页文章...", page),
            feed_name: Some(feed_name.clone()),
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
        
        let client = state.weread_client.lock().await;
        match client.get_mp_articles(&feed.id, &account.vid, &account.token, page).await {
            Ok(articles) => {
                log::info!("第{}页获取到{}篇文章", page, articles.len());
                
                // 发送获取成功事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: format!("第{}页获取到{}篇文章", page, articles.len()),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                if articles.is_empty() {
                    has_history = false;
                    log::info!("没有更多历史文章");
                    
                    // 发送完成事件
                    let log_event = RefreshLogEvent {
                        timestamp: Utc::now().to_rfc3339(),
                        level: "info".to_string(),
                        message: "没有更多历史文章".to_string(),
                        feed_name: Some(feed_name.clone()),
                    };
                    
                    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                        log::error!("发送日志事件失败: {}", e);
                    }
                    
                    break;
                }
                
                // 使用批量插入方法（优化性能，保持去重功能）
                let db_articles: Vec<crate::database::models::WeChatArticle> = articles.iter().map(|article| {
                    crate::database::models::WeChatArticle {
                        id: article.id.clone(),
                        mp_id: feed.id.clone(),
                        title: article.title.clone(),
                        url: article.url.clone(),
                        pic_url: Some(article.pic_url.clone().unwrap_or_default()), // 处理可选字段
                        publish_time: article.publish_time as i64,
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                    }
                }).collect();
                
                // 使用新的批量插入方法
                let saved_count = state.db.insert_articles_batch(&db_articles).map_err(|e| {
                    log::error!("批量插入文章失败: {}", e);
                    e.to_string()
                })?;
                
                log::info!("第{}页批量插入完成，实际新增{}篇文章", page, saved_count);
                
                total_saved += saved_count;
                log::info!("第{}页保存了{}篇文章", page, saved_count);
                
                // 发送保存成功事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: format!("第{}页保存了{}篇文章", page, saved_count),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                // 如果文章数量少于默认数量，认为没有更多历史文章
                if articles.len() < 20 { // wewe-rss的默认数量
                    has_history = false;
                    log::info!("文章数量少于默认值，认为没有更多历史文章");
                    
                    // 发送完成事件
                    let log_event = RefreshLogEvent {
                        timestamp: Utc::now().to_rfc3339(),
                        level: "info".to_string(),
                        message: "文章数量少于默认值，认为没有更多历史文章".to_string(),
                        feed_name: Some(feed_name.clone()),
                    };
                    
                    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                        log::error!("发送日志事件失败: {}", e);
                    }
                }
                
                page += 1;
                
                // 添加随机延迟避免请求过于频繁
                let delay_ms = fastrand::u64(2000..5000); // 2-5秒随机延迟
                log::info!("等待 {} 毫秒后获取下一页", delay_ms);
                
                // 发送等待事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: format!("等待 {} 毫秒后获取下一页", delay_ms),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
            Err(e) => {
                log::error!("获取第{}页文章失败: {}", page, e);
                
                // 发送错误事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "error".to_string(),
                    message: format!("获取第{}页文章失败: {}", page, e),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                // 检查是否是账号被封禁的错误
                let error_str = e.to_string();
                if error_str.contains("500") || error_str.contains("Internal Server Error") {
                    log::warn!("检测到账号可能被封禁，将账号加入黑名单: {}", account.name);
                    
                    // 发送警告事件
                    let log_event = RefreshLogEvent {
                        timestamp: Utc::now().to_rfc3339(),
                        level: "warn".to_string(),
                        message: format!("检测到账号可能被封禁，将账号加入黑名单: {}", account.name),
                        feed_name: Some(feed_name.clone()),
                    };
                    
                    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                        log::error!("发送日志事件失败: {}", e);
                    }
                    
                    // 将账号加入黑名单24小时
                    if let Err(blacklist_err) = state.db.add_to_blacklist(&account.id, "API返回500错误，疑似账号被封禁", 24) {
                        log::error!("将账号加入黑名单失败: {}", blacklist_err);
                    } else {
                        log::info!("账号 {} 已被加入黑名单24小时", account.name);
                        
                        // 发送黑名单成功事件
                        let log_event = RefreshLogEvent {
                            timestamp: Utc::now().to_rfc3339(),
                            level: "info".to_string(),
                            message: format!("账号 {} 已被加入黑名单24小时", account.name),
                            feed_name: Some(feed_name.clone()),
                        };
                        
                        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                            log::error!("发送日志事件失败: {}", e);
                        }
                    }
                }
                
                if page == 1 {
                    return Err(e.to_string());
                }
                // 第一页失败则返回错误，后续页失败则继续
                break;
            }
        }
    }
    
    // 更新订阅源的同步时间和历史状态
    let sync_time = Utc::now().timestamp();
    let has_history_flag = if has_history { 1 } else { 0 };
    
    state.db.update_feed_sync_time(&feed.id, sync_time, has_history_flag)
        .map_err(|e| {
            log::error!("更新订阅源状态失败: {}", e);
            e.to_string()
        })?;
    
    log::info!("订阅源刷新完成: {}, 总计保存{}篇文章", feed.mp_name, total_saved);
    
    // 发送完成事件
    let progress_event = RefreshProgressEvent {
        current: 1,
        total: 1,
        status: format!("刷新完成: {}", feed_name),
        log: format!("订阅源刷新完成: {}, 总计保存{}篇文章", feed_name, total_saved),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送完成日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("订阅源刷新完成: {}, 总计保存{}篇文章", feed_name, total_saved),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    Ok(total_saved)
}

// 刷新RSS订阅源的内部函数
async fn refresh_rss_feed_by_id(
    feed: RSSFeed,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let feed_name = feed.title.clone();
    log::info!("刷新RSS订阅源: {}", feed_name);
    
    // 发送开始事件
    let progress_event = RefreshProgressEvent {
        current: 0,
        total: 1,
        status: format!("正在刷新RSS {}", feed_name),
        log: format!("开始刷新RSS订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("开始刷新RSS订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    let parser = RSSParser::new();
    
    // 获取RSS文章
    match parser.fetch_articles(&feed.url, &feed.id).await {
        Ok(articles) => {
            log::info!("RSS订阅源 {} 获取到{}篇文章", feed_name, articles.len());
            
            // 发送获取成功事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源 {} 获取到{}篇文章", feed_name, articles.len()),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            if articles.is_empty() {
                log::info!("RSS订阅源没有文章");
                
                // 发送完成事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: "RSS订阅源没有文章".to_string(),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                return Ok(0);
            }
            
            // 批量插入文章
            let saved_count = state.db.insert_rss_articles_batch(&articles).map_err(|e| {
                log::error!("批量插入RSS文章失败: {}", e);
                e.to_string()
            })?;
            
            log::info!("RSS订阅源 {} 保存了{}篇文章", feed_name, saved_count);
            
            // 更新订阅源的最后获取时间
            let now = Utc::now().timestamp();
            state.db.update_rss_feed_last_fetched(&feed.id, now).map_err(|e| {
                log::error!("更新RSS订阅源最后获取时间失败: {}", e);
                e.to_string()
            })?;
            
            // 发送保存成功事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源 {} 保存了{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            // 发送完成事件
            let progress_event = RefreshProgressEvent {
                current: 1,
                total: 1,
                status: format!("RSS刷新完成: {}", feed_name),
                log: format!("RSS订阅源刷新完成: {}, 保存{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
                log::error!("发送进度事件失败: {}", e);
            }
            
            // 发送完成日志事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源刷新完成: {}, 保存{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            Ok(saved_count)
        }
        Err(e) => {
            log::error!("获取RSS文章失败: {}", e);
            
            // 发送错误事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "error".to_string(),
                message: format!("获取RSS文章失败: {}", e),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn refresh_all_feeds(app_handle: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    log::info!("开始批量刷新所有订阅源");
    
    // 获取所有订阅源
    let feeds = state.db.get_all_feeds().map_err(|e| {
        log::error!("获取订阅源列表失败: {}", e);
        e.to_string()
    })?;
    
    if feeds.is_empty() {
        return Ok("没有订阅源需要刷新".to_string());
    }
    
    let total_feeds = feeds.len();
    
    // 发送开始事件
    let progress_event = RefreshProgressEvent {
        current: 0,
        total: total_feeds,
        status: "开始批量刷新所有订阅源".to_string(),
        log: format!("开始批量刷新 {} 个订阅源", total_feeds),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送开始日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("开始批量刷新 {} 个订阅源", total_feeds),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    // 清理过期的黑名单记录
    if let Err(e) = state.db.cleanup_expired_blacklist() {
        log::warn!("清理过期黑名单失败: {}", e);
        
        // 发送警告事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "warn".to_string(),
            message: format!("清理过期黑名单失败: {}", e),
            feed_name: None,
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
    }
    
    // 获取可用账号
    let _available_accounts = state.db.get_available_accounts().map_err(|e| {
        log::error!("获取可用账号列表失败: {}", e);
        e.to_string()
    })?;
    
    if _available_accounts.is_empty() {
        return Err("所有账号都不可用，请稍后再试或添加新账号".to_string());
    }
    
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut total_articles = 0;
    let mut failed_feeds = Vec::new();
    
    for (index, feed) in feeds.iter().enumerate() {
        // 在开始每个订阅源之前检查中断标志
        if state.refresh_interrupted.load(Ordering::Relaxed) {
            log::info!("刷新任务被用户中断");

            // 发送中断事件
            let progress_event = RefreshProgressEvent {
                current: index,
                total: total_feeds,
                status: "刷新任务已被中断".to_string(),
                log: "用户中断了刷新任务".to_string(),
                feed_name: None,
            };

            let _ = app_handle.emit("refresh-interrupted", &progress_event);

            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "warn".to_string(),
                message: "刷新任务已被用户中断".to_string(),
                feed_name: None,
            };

            let _ = app_handle.emit("refresh-log", &log_event);

            // 重置中断标志
            state.refresh_interrupted.store(false, Ordering::Relaxed);

            break;
        }

        log::info!("刷新订阅源: {} ({})", feed.mp_name, feed.id);
        
        // 发送当前订阅源开始事件
        let progress_event = RefreshProgressEvent {
            current: index,
            total: total_feeds,
            status: format!("正在刷新 {}/{}: {}", index + 1, total_feeds, feed.mp_name),
            log: format!("开始刷新订阅源: {}", feed.mp_name),
            feed_name: Some(feed.mp_name.clone()),
        };
        
        if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
            log::error!("发送进度事件失败: {}", e);
        }
        
        // 发送开始日志事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: format!("开始刷新订阅源: {}", feed.mp_name),
            feed_name: Some(feed.mp_name.clone()),
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
        
        match refresh_feed(feed.id.clone(), app_handle.clone(), state.clone()).await {
            Ok(article_count) => {
                success_count += 1;
                total_articles += article_count;
                log::info!("订阅源 {} 刷新成功，获取{}篇文章", feed.mp_name, article_count);
                
                // 发送成功事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: format!("订阅源 {} 刷新成功，获取{}篇文章", feed.mp_name, article_count),
                    feed_name: Some(feed.mp_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
            }
            Err(e) => {
                failed_count += 1;
                failed_feeds.push(format!("{}: {}", feed.mp_name, e));
                log::error!("订阅源 {} 刷新失败: {}", feed.mp_name, e);
                
                // 发送失败事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "error".to_string(),
                    message: format!("订阅源 {} 刷新失败: {}", feed.mp_name, e),
                    feed_name: Some(feed.mp_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
            }
        }
        
        // 添加随机延迟避免请求过于频繁
        let delay_ms = fastrand::u64(1000..3000); // 1-3秒随机延迟 (优化速度)
        log::info!("等待 {} 毫秒后处理下一个订阅源", delay_ms);
        
        // 发送等待事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: format!("等待 {} 毫秒后处理下一个订阅源", delay_ms),
            feed_name: None,
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
    
    let result = format!(
        "批量刷新完成：成功 {} 个，失败 {} 个，总计获取 {} 篇文章\n失败的订阅源：\n{}",
        success_count,
        failed_count,
        total_articles,
        failed_feeds.join("\n")
    );
    
    log::info!("批量刷新完成: {}", result);
    
    // 发送完成事件
    let progress_event = RefreshProgressEvent {
        current: total_feeds,
        total: total_feeds,
        status: "批量刷新完成".to_string(),
        log: result.clone(),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送完成日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: result.clone(),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn debug_database_info(state: State<'_, AppState>) -> Result<String, String> {
    let mut info = String::new();
    
    // 检查账号数量
    match state.db.get_all_accounts() {
        Ok(accounts) => {
            info.push_str(&format!("账号数量: {}\n", accounts.len()));
            for account in &accounts {
                info.push_str(&format!("  - {} (状态: {}, vid: {})\n", 
                    account.name, account.status, account.vid));
            }
        }
        Err(e) => {
            info.push_str(&format!("获取账号失败: {}\n", e));
        }
    }
    
    // 检查订阅源数量
    match state.db.get_all_feeds() {
        Ok(feeds) => {
            info.push_str(&format!("订阅源数量: {}\n", feeds.len()));
            for feed in &feeds {
                info.push_str(&format!("  - {} (状态: {}, 更新时间: {})\n", 
                    feed.mp_name, feed.status, feed.updated_at));
            }
        }
        Err(e) => {
            info.push_str(&format!("获取订阅源失败: {}\n", e));
        }
    }
    
    // 检查黑名单
    match state.db.cleanup_expired_blacklist() {
        Ok(cleaned) => {
            info.push_str(&format!("清理过期黑名单记录: {}\n", cleaned));
        }
        Err(e) => {
            info.push_str(&format!("清理黑名单失败: {}\n", e));
        }
    }
    
    // 检查可用账号
    match state.db.get_available_accounts() {
        Ok(available) => {
            info.push_str(&format!("可用账号数量: {}\n", available.len()));
            for account in &available {
                info.push_str(&format!("  - {}\n", account.name));
            }
        }
        Err(e) => {
            info.push_str(&format!("获取可用账号失败: {}\n", e));
        }
    }
    
    Ok(info)
}

// RSS相关命令

#[tauri::command]
pub async fn add_rss_feed(
    url: String,
    category: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("开始添加RSS订阅源: url={}", url);
    
    let parser = RSSParser::new();
    
    // 验证RSS URL并获取订阅源信息
    let rss_feed = parser.fetch_feed_info(&url).await.map_err(|e| {
        log::error!("获取RSS订阅源信息失败: {}", e);
        e.to_string()
    })?;
    
    log::info!("获取到RSS订阅源信息: {} ({})", rss_feed.title, rss_feed.id);
    
    // 创建带分类的订阅源
    let mut feed_with_category = rss_feed;
    feed_with_category.category = category;
    
    // 保存到数据库
    state.db.insert_rss_feed(&feed_with_category).map_err(|e| {
        log::error!("保存RSS订阅源到数据库失败: {}", e);
        e.to_string()
    })?;
    
    log::info!("RSS订阅源添加成功: {}", feed_with_category.title);
    Ok(feed_with_category.title)
}

#[tauri::command]
pub async fn get_all_rss_feeds(state: State<'_, AppState>) -> Result<Vec<RSSFeed>, String> {
    state.db.get_all_rss_feeds()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_rss_feed(
    feed_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.delete_rss_feed(&feed_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_rss_feed(
    feed_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    log::info!("开始刷新RSS订阅源: {}", feed_id);
    
    // 获取RSS订阅源信息
    let feeds = state.db.get_all_rss_feeds().map_err(|e| {
        log::error!("获取RSS订阅源列表失败: {}", e);
        e.to_string()
    })?;
    
    let feed = feeds.into_iter()
        .find(|f| f.id == feed_id)
        .ok_or_else(|| {
            log::error!("RSS订阅源不存在: {}", feed_id);
            "RSS订阅源不存在".to_string()
        })?;
    
    let feed_name = feed.title.clone();
    log::info!("刷新RSS订阅源: {}", feed_name);
    
    // 发送开始事件
    let progress_event = RefreshProgressEvent {
        current: 0,
        total: 1,
        status: format!("正在刷新RSS {}", feed_name),
        log: format!("开始刷新RSS订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("开始刷新RSS订阅源: {}", feed_name),
        feed_name: Some(feed_name.clone()),
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    let parser = RSSParser::new();
    
    // 获取RSS文章
    match parser.fetch_articles(&feed.url, &feed.id).await {
        Ok(articles) => {
            log::info!("RSS订阅源 {} 获取到{}篇文章", feed_name, articles.len());
            
            // 发送获取成功事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源 {} 获取到{}篇文章", feed_name, articles.len()),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            if articles.is_empty() {
                log::info!("RSS订阅源没有文章");
                
                // 发送完成事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: "RSS订阅源没有文章".to_string(),
                    feed_name: Some(feed_name.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
                
                return Ok(0);
            }
            
            // 批量插入文章
            let saved_count = state.db.insert_rss_articles_batch(&articles).map_err(|e| {
                log::error!("批量插入RSS文章失败: {}", e);
                e.to_string()
            })?;
            
            log::info!("RSS订阅源 {} 保存了{}篇文章", feed_name, saved_count);
            
            // 更新订阅源的最后获取时间
            let now = Utc::now().timestamp();
            state.db.update_rss_feed_last_fetched(&feed.id, now).map_err(|e| {
                log::error!("更新RSS订阅源最后获取时间失败: {}", e);
                e.to_string()
            })?;
            
            // 发送保存成功事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源 {} 保存了{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            // 发送完成事件
            let progress_event = RefreshProgressEvent {
                current: 1,
                total: 1,
                status: format!("RSS刷新完成: {}", feed_name),
                log: format!("RSS订阅源刷新完成: {}, 保存{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
                log::error!("发送进度事件失败: {}", e);
            }
            
            // 发送完成日志事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("RSS订阅源刷新完成: {}, 保存{}篇文章", feed_name, saved_count),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            Ok(saved_count)
        }
        Err(e) => {
            log::error!("获取RSS文章失败: {}", e);
            
            // 发送错误事件
            let log_event = RefreshLogEvent {
                timestamp: Utc::now().to_rfc3339(),
                level: "error".to_string(),
                message: format!("获取RSS文章失败: {}", e),
                feed_name: Some(feed_name.clone()),
            };
            
            if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                log::error!("发送日志事件失败: {}", e);
            }
            
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn refresh_all_rss_feeds(app_handle: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    log::info!("开始批量刷新所有RSS订阅源");
    
    // 获取所有RSS订阅源
    let feeds = state.db.get_all_rss_feeds().map_err(|e| {
        log::error!("获取RSS订阅源列表失败: {}", e);
        e.to_string()
    })?;
    
    if feeds.is_empty() {
        return Ok("没有RSS订阅源需要刷新".to_string());
    }
    
    let total_feeds = feeds.len();
    
    // 发送开始事件
    let progress_event = RefreshProgressEvent {
        current: 0,
        total: total_feeds,
        status: "开始批量刷新所有RSS订阅源".to_string(),
        log: format!("开始批量刷新 {} 个RSS订阅源", total_feeds),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送开始日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: format!("开始批量刷新 {} 个RSS订阅源", total_feeds),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut total_articles = 0;
    let mut failed_feeds = Vec::new();
    
    for (index, feed) in feeds.iter().enumerate() {
        log::info!("刷新RSS订阅源: {} ({})", feed.title, feed.id);
        
        // 发送当前订阅源开始事件
        let progress_event = RefreshProgressEvent {
            current: index,
            total: total_feeds,
            status: format!("正在刷新RSS {}/{}: {}", index + 1, total_feeds, feed.title),
            log: format!("开始刷新RSS订阅源: {}", feed.title),
            feed_name: Some(feed.title.clone()),
        };
        
        if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
            log::error!("发送进度事件失败: {}", e);
        }
        
        // 发送开始日志事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: format!("开始刷新RSS订阅源: {}", feed.title),
            feed_name: Some(feed.title.clone()),
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
        
        match refresh_rss_feed(feed.id.clone(), app_handle.clone(), state.clone()).await {
            Ok(article_count) => {
                success_count += 1;
                total_articles += article_count;
                log::info!("RSS订阅源 {} 刷新成功，获取{}篇文章", feed.title, article_count);
                
                // 发送成功事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "info".to_string(),
                    message: format!("RSS订阅源 {} 刷新成功，获取{}篇文章", feed.title, article_count),
                    feed_name: Some(feed.title.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
            }
            Err(e) => {
                failed_count += 1;
                failed_feeds.push(format!("{}: {}", feed.title, e));
                log::error!("RSS订阅源 {} 刷新失败: {}", feed.title, e);
                
                // 发送失败事件
                let log_event = RefreshLogEvent {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "error".to_string(),
                    message: format!("RSS订阅源 {} 刷新失败: {}", feed.title, e),
                    feed_name: Some(feed.title.clone()),
                };
                
                if let Err(e) = app_handle.emit("refresh-log", &log_event) {
                    log::error!("发送日志事件失败: {}", e);
                }
            }
        }
        
        // 添加随机延迟避免请求过于频繁
        let delay_ms = fastrand::u64(2000..5000); // 2-5秒随机延迟
        log::info!("等待 {} 毫秒后处理下一个RSS订阅源", delay_ms);
        
        // 发送等待事件
        let log_event = RefreshLogEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: format!("等待 {} 毫秒后处理下一个RSS订阅源", delay_ms),
            feed_name: None,
        };
        
        if let Err(e) = app_handle.emit("refresh-log", &log_event) {
            log::error!("发送日志事件失败: {}", e);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
    
    let result = format!(
        "RSS批量刷新完成：成功 {} 个，失败 {} 个，总计获取 {} 篇文章\n失败的RSS订阅源：\n{}",
        success_count,
        failed_count,
        total_articles,
        failed_feeds.join("\n")
    );
    
    log::info!("RSS批量刷新完成: {}", result);
    
    // 发送完成事件
    let progress_event = RefreshProgressEvent {
        current: total_feeds,
        total: total_feeds,
        status: "RSS批量刷新完成".to_string(),
        log: result.clone(),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-progress", &progress_event) {
        log::error!("发送进度事件失败: {}", e);
    }
    
    // 发送完成日志事件
    let log_event = RefreshLogEvent {
        timestamp: Utc::now().to_rfc3339(),
        level: "info".to_string(),
        message: result.clone(),
        feed_name: None,
    };
    
    if let Err(e) = app_handle.emit("refresh-log", &log_event) {
        log::error!("发送日志事件失败: {}", e);
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn interrupt_refresh_refresh(state: State<'_, AppState>) -> Result<String, String> {
    log::info!("用户请求中断刷新任务");

    // 设置中断标志
    state.refresh_interrupted.store(true, Ordering::Relaxed);

    Ok("刷新中断请求已发送".to_string())
}

#[tauri::command]
pub async fn validate_rss_url(url: String) -> Result<bool, String> {
    let parser = RSSParser::new();
    parser.validate_rss_url(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_database_info() -> Result<String, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| e.to_string())?;

    let app_data_dir = current_exe
        .parent()
        .ok_or("无法获取应用父目录")?
        .join("data");

    let db_path = app_data_dir.join("news_analyzer.db");
    let old_db_path = std::env::temp_dir().join("news-analyzer-mvp").join("news_analyzer.db");

    let info = format!(
        "数据库信息:\n\
        当前数据库路径: {}\n\
        数据库存在: {}\n\
        旧数据库路径: {}\n\
        旧数据库存在: {}\n\
        应用可执行文件路径: {}",
        db_path.display(),
        db_path.exists(),
        old_db_path.display(),
        old_db_path.exists(),
        current_exe.display()
    );

    Ok(info)
}
