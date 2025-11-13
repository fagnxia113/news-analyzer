use crate::database::models::*;
use crate::state::AppState;
use tauri::State;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use regex::Regex;
use once_cell::sync::Lazy;

// 开始分析
#[tauri::command]
pub async fn start_analysis(
    article_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.db.clone();
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_id_clone = task_id.clone();
    
    log::info!("创建新的分析任务: {}, 文章数量: {}", task_id, article_ids.len());
    
    // 创建分析任务
    let task = AnalysisTask {
        id: task_id.clone(),
        status: "pending".to_string(),
        total_articles: article_ids.len() as i32,
        processed_articles: 0,
        success_count: 0,
        failed_count: 0,
        start_time: chrono::Utc::now().timestamp(),
        end_time: None,
        error_message: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    if let Err(e) = db.insert_analysis_task(&task) {
        let error_msg = format!("创建分析任务失败: {}", e);
        log::error!("{}", error_msg);
        return Err(error_msg);
    }
    
    log::info!("分析任务创建成功: {}", task_id);
    
    // 启动异步分析任务
    let db_clone = db.clone();
    let article_ids_clone = article_ids.clone();
    
    tokio::spawn(async move {
        log::info!("启动异步分析任务: {}", task_id_clone);
        if let Err(e) = run_analysis(&db_clone, &task_id_clone, &article_ids_clone, &[], &[]).await {
            let error_msg = format!("分析失败: {}", e);
            log::error!("异步分析任务失败: {} - {}", task_id_clone, error_msg);
            let _ = db_clone.update_analysis_task_status(
                &task_id_clone, 
                "failed", 
                0, 
                0, 
                1, 
                Some(error_msg)
            );
        } else {
            log::info!("异步分析任务完成: {}", task_id_clone);
        }
    });
    
    Ok(task_id)
}

// 获取分析任务状态
#[tauri::command]
pub async fn get_analysis_task(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisTask, String> {
    let db = &state.db;
    match db.get_analysis_task(&task_id) {
        Ok(Some(task)) => Ok(task),
        Ok(None) => Err("分析任务不存在".to_string()),
        Err(e) => Err(format!("获取分析任务失败: {}", e)),
    }
}

// 获取所有分析任务
#[tauri::command]
pub async fn get_analysis_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<AnalysisTask>, String> {
    let db = &state.db;
    db.get_all_analysis_tasks()
        .map_err(|e| format!("获取分析任务列表失败: {}", e))
}

// 获取分析结果
#[tauri::command]
pub async fn get_analyzed_news(
    task_id: String,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<AnalyzedNews>, String> {
    let db = &state.db;
    db.get_analyzed_news(&task_id, limit)
        .map_err(|e| format!("获取分析结果失败: {}", e))
}

// 获取所有分析结果（跨任务）
#[tauri::command]
pub async fn get_all_analyzed_news(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<AnalyzedNews>, String> {
    let db = &state.db;
    db.get_all_analyzed_news_with_limit(limit)
        .map_err(|e| format!("获取所有分析结果失败: {}", e))
}


// 清空所有分析结果
#[tauri::command]
pub async fn clear_all_analyzed_news(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    // 获取最新的分析任务
    match db.get_latest_analysis_task() {
        Ok(Some(task)) => {
            db.clear_analyzed_news(&task.id)
                .map_err(|e| format!("清空分析结果失败: {}", e))?;
            Ok(())
        }
        Ok(None) => Err("没有找到分析任务".to_string()),
        Err(e) => Err(format!("获取分析任务失败: {}", e)),
    }
}

// 删除单个分析结果
#[tauri::command]
pub async fn delete_analyzed_news(
    news_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = &state.db;
    db.delete_analyzed_news(&news_id)
        .map_err(|e| format!("删除分析结果失败: {}", e))
}

// 批量删除分析结果
#[tauri::command]
pub async fn delete_multiple_analyzed_news(
    news_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = &state.db;
    let mut success_count = 0;
    let mut error_count = 0;
    
    for news_id in news_ids {
        match db.delete_analyzed_news(&news_id) {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }
    
    Ok(format!("批量删除完成！成功删除 {} 条，失败 {} 条", success_count, error_count))
}

// 获取近一个月的分析统计信息
#[tauri::command]
pub async fn get_recent_month_stats(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = &state.db;
    
    match db.get_recent_month_stats() {
        Ok((task_count, news_count)) => {
            let message = format!("近一个月统计：{} 个分析任务，{} 条新闻", task_count, news_count);
            log::info!("{}", message);
            Ok(message)
        }
        Err(e) => {
            let error_msg = format!("获取近一个月统计失败: {}", e);
            log::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// 修复分析结果中的类型ID问题（已废弃，因为不再使用行业类型和新闻类型表）
#[tauri::command]
pub async fn fix_analyzed_news_type_ids(
    _state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("修复分析结果中的类型ID功能已废弃，现在使用提示词模板直接定义分类标准");
    Ok("修复功能已废弃，现在使用提示词模板直接定义分类标准".to_string())
}

// 运行分析任务
async fn run_analysis(
    db: &Arc<crate::database::Database>,
    task_id: &str,
    article_ids: &[String],
    _industry_types: &[String], // 不再使用，保留参数兼容性
    _news_types: &[String], // 不再使用，保留参数兼容性
) -> Result<()> {
    log::info!("开始运行分析任务: {}, 文章数量: {}", task_id, article_ids.len());

    // 记录开始日志
    let _ = add_log(db, task_id, "info", &format!("开始执行分析任务，共 {} 篇文章", article_ids.len()));

    // 更新任务状态为运行中
    if let Err(e) = db.update_analysis_task_status(task_id, "running", 0, 0, 0, None) {
        let _ = add_log(db, task_id, "error", &format!("更新任务状态失败: {}", e));
        return Err(anyhow::anyhow!("更新任务状态失败: {}", e));
    }

    let _ = add_log(db, task_id, "info", "分析任务状态已更新为运行中");

    let _total_articles = article_ids.len();
    let mut processed_articles = 0;
    let mut success_count = 0;
    let mut failed_count = 0;

    // 获取所有文章
    let articles = db.get_all_articles(1000)?;
    let _ = add_log(db, task_id, "info", &format!("从数据库获取到 {} 篇文章", articles.len()));

    for (index, article_id) in article_ids.iter().enumerate() {
        let _ = add_log(db, task_id, "info", &format!("开始处理第 {}/{} 篇文章: {}", index + 1, article_ids.len(), article_id));

        // 获取文章信息
        let article = articles.iter().find(|a| &a.id == article_id);

        if let Some(article) = article {
            let _ = add_log(db, task_id, "info", &format!("找到文章: {} ({})", article.title, article.url));

            // 爬取网页内容
            let _ = add_log(db, task_id, "info", &format!("开始爬取网页内容: {}", article.url));

            match fetch_web_content(&article.url).await {
                Ok(content) => {
                    let _ = add_log(db, task_id, "info", &format!("成功爬取网页内容，长度: {} 字符", content.len()));

                    // 调用LLM分析（使用提示词模板）
                    let _ = add_log(db, task_id, "info", "开始调用LLM进行分析");

                    match analyze_with_llm(db, &content, &[], &[], &article.url).await {
                        Ok(response) => {
                            let _ = add_log(db, task_id, "info", &format!("LLM分析成功，提取到 {} 条新闻", response.news_list.len()));

                            // 保存分析结果，进行去重检查
                            for (news_index, news_item) in response.news_list.iter().enumerate() {
                                let _ = add_log(db, task_id, "info", &format!("处理第 {} 条新闻: {}", news_index + 1, news_item.title));

                                // 检查是否为重复新闻
                                match check_news_duplicate(
                                    db,
                                    &news_item.title,
                                    &news_item.summary,
                                    &news_item.industry_type,
                                    &news_item.news_type,
                                ).await {
                                    Ok(is_duplicate) => {
                                        if is_duplicate {
                                            let _ = add_log(db, task_id, "info", &format!("跳过重复新闻: {}", news_item.title));
                                            continue;
                                        }

                                        let analyzed_news = AnalyzedNews {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            task_id: task_id.to_string(),
                                            article_id: article_id.clone(),
                                            title: news_item.title.clone(),
                                            content: content.clone(),
                                            summary: news_item.summary.clone(),
                                            is_soft_news: false, // 这里可以根据LLM响应设置
                                            industry_type: news_item.industry_type.clone(), // 直接使用名称
                                            news_type: news_item.news_type.clone(), // 直接使用名称
                                            confidence: news_item.confidence,
                                            keywords: "".to_string(), // 可以从LLM响应中提取
                                            original_url: article.url.clone(),
                                            analyzed_at: chrono::Utc::now().to_rfc3339(),
                                            created_at: chrono::Utc::now().to_rfc3339(),
                                            updated_at: chrono::Utc::now().to_rfc3339(),
                                        };

                                        match db.insert_analyzed_news(&analyzed_news) {
                                            Ok(_) => {
                                                let _ = add_log(db, task_id, "info",
                                                    &format!("成功保存分析结果: {} (行业: {}, 新闻: {})",
                                                        news_item.title, news_item.industry_type, news_item.news_type));
                                            }
                                            Err(e) => {
                                                let _ = add_log(db, task_id, "error",
                                                    &format!("保存分析结果失败: {}, 标题: {}", e, news_item.title));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = add_log(db, task_id, "warning", &format!("检查重复新闻时出错: {}, 继续保存", e));
                                    }
                                }
                            }
                            success_count += 1;
                            let _ = add_log(db, task_id, "info",
                                &format!("文章 {} 分析完成，成功保存 {} 条结果", article_id, response.news_list.len()));
                        }
                        Err(e) => {
                            let _ = add_log(db, task_id, "error", &format!("LLM分析失败: {}", e));
                            failed_count += 1;
                        }
                    }
                }
                Err(e) => {
                    let _ = add_log(db, task_id, "error", &format!("爬取网页内容失败: {}", e));
                    failed_count += 1;
                }
            }
        } else {
            let _ = add_log(db, task_id, "error", &format!("找不到文章: {}", article_id));
            failed_count += 1;
        }

        processed_articles += 1;

        // 更新进度
        if let Err(e) = db.update_analysis_task_status(
            task_id,
            "running",
            processed_articles as i32,
            success_count,
            failed_count,
            None
        ) {
            let _ = add_log(db, task_id, "warning", &format!("更新任务进度失败: {}", e));
        }

        let _ = add_log(db, task_id, "info",
            &format!("任务进度: {}/{} (成功: {}, 失败: {})",
                processed_articles, article_ids.len(), success_count, failed_count));

        // 添加随机延迟避免请求过于频繁（大约1分钟抓取1条）
        let delay_ms = fastrand::u64(45000..75000); // 45-75秒随机延迟
        let _ = add_log(db, task_id, "info", &format!("等待 {} 秒后处理下一篇文章", delay_ms / 1000));
        sleep(Duration::from_millis(delay_ms)).await;
    }

    // 完成分析任务
    if let Err(e) = db.update_analysis_task_status(
        task_id,
        "completed",
        processed_articles as i32,
        success_count,
        failed_count,
        None
    ) {
        let _ = add_log(db, task_id, "error", &format!("更新任务完成状态失败: {}", e));
        return Err(anyhow::anyhow!("更新任务完成状态失败: {}", e));
    }

    let _ = add_log(db, task_id, "info",
        &format!("分析任务完成！总计处理 {} 篇文章，成功 {} 篇，失败 {} 篇",
            processed_articles, success_count, failed_count));

    Ok(())
}

// 内部日志记录函数
fn add_log(db: &Arc<crate::database::Database>, task_id: &str, level: &str, message: &str) -> Result<()> {
    let log_entry = crate::database::models::AnalysisLog {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: task_id.to_string(),
        level: level.to_string(),
        message: message.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db.insert_analysis_log(&log_entry)?;
    Ok(())
}

// 爬取网页内容
async fn fetch_web_content(url: &str) -> Result<String> {
    log::info!("开始爬取网页内容: {}", url);
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("创建HTTP客户端失败: {}", e);
            anyhow::anyhow!("创建HTTP客户端失败: {}", e)
        })?;
    
    // 重试机制
    let max_retries = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        log::info!("第 {} 次尝试爬取网页: {}", attempt, url);
        
        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status();
                log::info!("网页响应状态: {}", status);
                
                if status.is_success() {
                    match response.text().await {
                        Ok(html) => {
                            log::info!("成功获取网页内容，长度: {} 字符", html.len());
                            
                            // 简单的HTML内容提取
                            let content = extract_text_from_html(&html);
                            log::info!("提取文本内容，长度: {} 字符", content.len());
                            
                            if content.trim().is_empty() {
                                log::warn!("提取的文本内容为空");
                            }
                            
                            return Ok(content);
                        }
                        Err(e) => {
                            log::error!("读取网页内容失败: {}", e);
                            last_error = Some(anyhow::anyhow!("读取网页内容失败: {}", e));
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    log::error!("网页响应错误: {} - {}", status, error_text);
                    last_error = Some(anyhow::anyhow!("网页响应错误: {} - {}", status, error_text));
                }
            }
            Err(e) => {
                log::error!("发送网页请求失败: {}", e);
                last_error = Some(anyhow::anyhow!("发送网页请求失败: {}", e));
            }
        }
        
        // 如果不是最后一次尝试，等待一段时间再重试
        if attempt < max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    // 所有重试都失败了
    let error = last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误"));
    log::error!("爬取网页内容失败，已重试 {} 次", max_retries);
    Err(error)
}

// 从HTML中提取文本内容
fn extract_text_from_html(html: &str) -> String {
    use regex::Regex;
    
    // 移除script和style标签
    let re_script = Regex::new(r"(?s)<script.*?</script>").unwrap();
    let re_style = Regex::new(r"(?s)<style.*?</style>").unwrap();
    let re_comments = Regex::new(r"(?s)<!--.*?-->").unwrap();
    
    let clean_html = re_script.replace_all(html, "");
    let clean_html = re_style.replace_all(&clean_html, "");
    let clean_html = re_comments.replace_all(&clean_html, "");
    
    // 移除HTML标签
    let re_tags = Regex::new(r"<[^>]*>").unwrap();
    let text = re_tags.replace_all(&clean_html, "");
    
    // 清理空白字符
    let re_whitespace = Regex::new(r"\s+").unwrap();
    let clean_text = re_whitespace.replace_all(&text, " ");
    
    clean_text.trim().to_string()
}


// 调用LLM进行分析
async fn analyze_with_llm(
    db: &Arc<crate::database::Database>,
    content: &str,
    _industry_types: &[String],
    _news_types: &[String],
    _original_url: &str,
) -> Result<LlmAnalysisResponse> {
    // 获取默认提示词模板
    let default_template = db.get_default_prompt_template()
        .map_err(|e| anyhow::anyhow!("获取默认提示词模板失败: {}", e))?;
    
    let template = match default_template {
        Some(t) => t,
        None => {
            // 如果没有默认模板，创建一个
            log::info!("没有找到默认提示词模板，创建默认模板");
            if let Err(e) = create_default_prompt_template_internal(db).await {
                log::error!("创建默认模板失败: {}", e);
                return Err(anyhow::anyhow!("没有找到提示词模板且创建失败"));
            }
            
            // 再次获取
            db.get_default_prompt_template()
                .map_err(|e| anyhow::anyhow!("获取默认提示词模板失败: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("无法获取默认提示词模板"))?
        }
    };
    
    // 构建提示词
    let prompt = template.template.replace("{content}", content);
    
    // 调用真实的LLM API，传入数据库实例
    call_llm_api(db, &prompt).await
}

// 内部创建默认模板的函数
async fn create_default_prompt_template_internal(
    db: &Arc<crate::database::Database>,
) -> Result<()> {
    use crate::database::models::PromptTemplate;
    use uuid::Uuid;
    use chrono;
    
    // 检查是否已有模板
    let existing_templates = db.get_all_prompt_templates()?;
    if !existing_templates.is_empty() {
        return Ok(());
    }
    
    // 创建默认模板
    let default_template = PromptTemplate {
        id: Uuid::new_v4().to_string(),
        name: "标准新闻分析模板".to_string(),
        template: r#"你是专业的新闻分析专家。分析以下文章内容，提取符合目标类型的新闻信息。

文章内容：
{content}

目标行业类型：
- 科技：包括人工智能、互联网、软件、硬件、电子、通信等技术相关领域
- 金融：包括银行、保险、证券、投资、支付、区块链等金融相关领域
- 医疗：包括医药、医疗器械、医疗服务、生物技术、健康管理等相关领域
- 教育：包括在线教育、培训、学校、教育科技等相关领域
- 房地产：包括房地产开发、建筑、物业、家居等相关领域
- 零售：包括电商、实体店、消费品、物流等相关领域
- 制造业：包括工业制造、机械、材料、能源等相关领域
- 交通：包括汽车、航空、铁路、物流、智慧交通等相关领域
- 文娱：包括影视、音乐、游戏、体育、旅游等相关领域
- 餐饮：包括食品、饮料、餐饮服务、外卖等相关领域
- 其他：不属于以上分类的其他行业

目标新闻类型：
- 产品发布：新产品、新服务的发布和推出
- 融资投资：公司的融资、投资、并购等资本活动
- 政策法规：政府政策、法律法规的发布和变化
- 市场动态：市场趋势、竞争格局、行业变化等
- 技术突破：技术创新、研发成果、专利等
- 人事变动：高管任命、离职、组织架构调整等
- 财务报告：公司财报、业绩发布、财务数据等
- 合作伙伴：战略合作、业务合作、联盟等
- 行业活动：展会、峰会、奖项、评选等
- 负面新闻：公司危机、产品问题、法律纠纷等
- 其他：不属于以上分类的其他新闻类型

要求：
- 识别所有独立的新闻事件
- 为每条新闻选择最匹配的类型
- 提取准确的标题和详细摘要
- 评估信息可信度

直接返回JSON格式结果，不要包含任何其他文字说明：
{{
  "has_news": true,
  "news_list": [
    {{
      "title": "新闻标题",
      "summary": "200字左右的详细摘要，包含事件背景、关键细节、重要数据、影响范围等",
      "industry_type": "行业类型名称",
      "news_type": "新闻类型名称",
      "confidence": 0.8
    }}
  ],
  "analysis_summary": "分析完成"
}}

注意：
- 只返回JSON，不要添加解释性文字
- 如果没有符合条件的新闻，has_news设为false，news_list为空数组
- 确保JSON格式正确"#.to_string(),
        is_default: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    db.insert_prompt_template(&default_template)?;
    log::info!("默认提示词模板已创建");
    
    Ok(())
}

// 调用LLM API
async fn call_llm_api(db: &Arc<crate::database::Database>, prompt: &str) -> Result<LlmAnalysisResponse> {
    // 从数据库获取启用的LLM配置
    let (api_key, endpoint, model_id, temperature, max_tokens) = get_enabled_llm_config(db)?;
    
    log::info!("LLM配置 - API Key: {}, Endpoint: {}, Model: {}, Temperature: {}, Max Tokens: {}", 
              &api_key[..8], endpoint, model_id, temperature, max_tokens);
    
    // 直接使用用户设置的max_tokens，不做任何限制
    let adjusted_max_tokens = max_tokens;
    
    log::info!("调整后的max_tokens: {} (模型: {})", adjusted_max_tokens, model_id);
    
    // 构建请求体
    let request_body = serde_json::json!({
        "model": model_id,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": temperature,
        "max_tokens": adjusted_max_tokens
    });
    
    log::info!("LLM请求体: {}", serde_json::to_string_pretty(&request_body)?);
    
    // 创建HTTP客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 增加到300秒（5分钟）
        .build()?;
    
    // 发送请求
    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let status = response.status();
    log::info!("LLM响应状态: {}", status);
    
    // 检查响应状态
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        log::error!("LLM API 错误响应: {}", error_text);
        return Err(anyhow::anyhow!("LLM API 请求失败: {} - {}", status, error_text));
    }
    
    // 解析响应
    let response_text = response.text().await?;
    log::info!("LLM原始响应文本: {}", response_text);
    
    let api_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            log::error!("解析 LLM 响应 JSON 失败: {}, 原始文本: {}", e, response_text);
            anyhow::anyhow!("解析 LLM 响应失败: {}", e)
        })?;
    
    log::info!("LLM响应JSON: {}", serde_json::to_string_pretty(&api_response)?);
    
    // 提取内容 - 支持智谱API格式
    let content = if let Some(reasoning_content) = api_response
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("reasoning_content"))
        .and_then(|content| content.as_str()) {
        // 智谱API特殊格式：优先使用reasoning_content
        log::info!("使用智谱API的reasoning_content字段");
        reasoning_content
    } else if let Some(content) = api_response
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str()) {
        content
    } else if let Some(content) = api_response.get("content") {
        content.as_str().unwrap_or("")
    } else if let Some(reasoning_content) = api_response.get("reasoning_content") {
        reasoning_content.as_str().unwrap_or("")
    } else {
        ""
    };
    
    log::info!("提取的内容: '{}'", content);
    
    if content.is_empty() {
        log::error!("LLM返回的内容为空，可能是API配置问题或模型不支持");
        return Err(anyhow::anyhow!("LLM返回的内容为空"));
    }
    
    // 解析LLM返回的JSON
    parse_llm_response(content)
}

// 新的健壮解析器 - 采用"两步+归一化"方法
fn parse_llm_response(content: &str) -> Result<LlmAnalysisResponse> {
    log::info!("开始使用新的健壮解析器");
    log::info!("LLM原始响应长度: {}", content.len());
    
    match parse_payload_from_content(content) {
        Ok(payload) => {
            log::info!("解析成功: has_news={}, news_list_len={}, summary={}", 
                      payload.has_news, payload.news_list.len(), payload.analysis_summary);
            
            // 转换为我们的结构
            let news_list: Vec<AnalyzedNewsItem> = payload.news_list.into_iter().map(|item| {
                AnalyzedNewsItem {
                    title: item.title,
                    summary: item.summary,
                    industry_type: item.industry_tags.join(", "), // 转换回逗号分隔字符串
                    news_type: item.type_tags.join(", "), // 转换回逗号分隔字符串
                    confidence: item.confidence.unwrap_or(0.0),
                }
            }).collect();
            
            Ok(LlmAnalysisResponse {
                has_news: payload.has_news,
                news_list,
                analysis_summary: payload.analysis_summary,
            })
        }
        Err(e) => {
            log::error!("健壮解析器失败: {}", e);
            Err(anyhow::anyhow!("解析 LLM JSON 响应失败: {}", e))
        }
    }
}

// 解析错误类型
#[derive(Debug, thiserror::Error)]
enum ParseError {
    #[error("no content found")]
    NoContent,
    #[error("json parse error: {0}")]
    Json(String),
    #[error("extraction error: {0}")]
    Extraction(String),
}

// ChatCompletion响应结构
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

// LLM返回的原始结构（容忍字段变化）
#[derive(Debug, Deserialize)]
struct RawPayload {
    has_news: Option<bool>,
    news_list: Option<Vec<RawItem>>,
    analysis_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawItem {
    title: String,
    summary: String,
    #[serde(default)]
    industry_type: Option<String>,
    #[serde(default)]
    news_type: Option<String>,
    #[serde(default)]
    industries: Option<Vec<String>>,
    #[serde(default)]
    types: Option<Vec<String>>,
    #[serde(default)]
    confidence: Option<f64>,
}

// 规范化结构
#[derive(Debug, Serialize)]
struct CanonicalItem {
    title: String,
    summary: String,
    industry_tags: Vec<String>,
    type_tags: Vec<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
struct CanonicalPayload {
    has_news: bool,
    news_list: Vec<CanonicalItem>,
    analysis_summary: String,
}

// 标签分隔符正则表达式
static TAG_SEPARATOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[,\uFF0C\u3001|\uFF5C/\\]\s*"#).unwrap()
});

// 分割标签字符串
fn split_tags(s: &str) -> Vec<String> {
    TAG_SEPARATOR.split(s)
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

// 归一化单个新闻项
fn normalize_item(raw: RawItem) -> CanonicalItem {
    let mut industry_tags = Vec::new();
    let mut type_tags = Vec::new();

    // 收集所有可能的行业标签
    if let Some(v) = raw.industries.as_ref() {
        industry_tags.extend(v.iter().filter(|s| !s.is_empty()).cloned());
    }
    if let Some(s) = raw.industry_type.as_ref() {
        industry_tags.extend(split_tags(s));
    }

    // 收集所有可能的新闻类型标签
    if let Some(v) = raw.types.as_ref() {
        type_tags.extend(v.iter().filter(|s| !s.is_empty()).cloned());
    }
    if let Some(s) = raw.news_type.as_ref() {
        type_tags.extend(split_tags(s));
    }

    // 去重并排序
    let uniq = |mut v: Vec<String>| {
        v.sort();
        v.dedup();
        v
    };

    CanonicalItem {
        title: raw.title,
        summary: raw.summary,
        industry_tags: uniq(industry_tags),
        type_tags: uniq(type_tags),
        confidence: raw.confidence,
    }
}

// 归一化整个payload
fn normalize_payload(raw: RawPayload) -> CanonicalPayload {
    let items = raw.news_list.unwrap_or_default()
        .into_iter()
        .map(normalize_item)
        .filter(|it| !it.title.is_empty() && !it.summary.is_empty()) // 过滤空标题/摘要
        .collect::<Vec<_>>();

    CanonicalPayload {
        has_news: raw.has_news.unwrap_or(!items.is_empty()),
        news_list: items,
        analysis_summary: raw.analysis_summary.unwrap_or_else(|| "分析完成".to_string()),
    }
}

// 去除代码围栏和噪声
fn strip_fences(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix("```json").unwrap_or(t);
    let t = t.strip_prefix("```").unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t);
    t.trim().to_string()
}

// 从字符串中提取根JSON对象或数组
fn extract_root_json(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut in_str = false;
    let mut esc = false;
    let mut depth = 0i32;
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
                continue;
            }
            match b {
                b'\\' => esc = true,
                b'"' => in_str = false,
                _ => {}
            }
            continue;
        } else {
            match b {
                b'"' => in_str = true,
                b'{' | b'[' => {
                    if depth == 0 { start = Some(i); }
                    depth += 1;
                }
                b'}' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        let st = start?;
                        let end = i + 1;
                        return Some(&s[st..end]);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

// 宽松JSON解析
fn parse_loose_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, ParseError> {
    // 先尝试标准JSON解析
    match serde_json::from_str::<T>(s) {
        Ok(v) => return Ok(v),
        Err(e) => {
            log::info!("标准JSON解析失败，尝试JSON5: {}", e);
        }
    }
    
    // 再尝试JSON5解析（容忍尾逗号等）
    match json5::from_str::<T>(s) {
        Ok(v) => Ok(v),
        Err(e2) => {
            Err(ParseError::Json(format!("serde_json failed; json5 failed: {}", e2)))
        }
    }
}

// 主入口：从content解析出CanonicalPayload
fn parse_payload_from_content(content: &str) -> Result<CanonicalPayload, ParseError> {
    log::info!("开始解析content，长度: {}", content.len());
    
    // 如果content看起来像完整的ChatCompletion响应，先解析外层
    let cleaned_content = if content.trim_start().starts_with('{') && content.contains("\"choices\"") {
        log::info!("检测到ChatCompletion响应格式");
        match serde_json::from_str::<ChatResponse>(content) {
            Ok(chat) => {
                let content = chat.choices.first()
                    .ok_or(ParseError::NoContent)?
                    .message.content.clone();
                log::info!("提取到内层content，长度: {}", content.len());
                content
            }
            Err(e) => {
                log::info!("不是有效的ChatCompletion响应，直接处理content: {}", e);
                content.to_string()
            }
        }
    } else {
        content.to_string()
    };

    let cleaned = strip_fences(&cleaned_content);
    log::info!("去除围栏后长度: {}", cleaned.len());
    
    let root = extract_root_json(&cleaned)
        .ok_or_else(|| ParseError::Extraction("no root json found".into()))?;
    
    log::info!("提取到根JSON，长度: {}", root.len());
    
    let raw: RawPayload = parse_loose_json(root)?;
    log::info!("JSON解析成功，开始归一化");
    
    Ok(normalize_payload(raw))
}

// 智能提取完整的JSON结构
fn extract_complete_json(content: &str, start: usize) -> &str {
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = content[start..].chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => {
                brace_count += 1;
            }
            '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                    return &content[start..start + i + 1];
                }
            }
            _ => {}
        }
    }
    
    // 如果没有找到完整的JSON，返回到末尾
    &content[start..]
}

// 修复常见的JSON问题
fn fix_common_json_issues(json_str: &str) -> String {
    let mut fixed = json_str.to_string();
    
    // 修复截断的JSON - 尝试补全结构
    if !fixed.trim_end().ends_with('}') {
        // 计算未闭合的大括号数量
        let open_braces = fixed.matches('{').count();
        let close_braces = fixed.matches('}').count();
        let missing_braces = open_braces - close_braces;
        
        // 添加缺失的闭合括号
        for _ in 0..missing_braces {
            fixed.push('}');
        }
        
        // 如果news_list被截断，尝试补全
        if fixed.contains("\"news_list\"") && !fixed.contains("]") {
            if fixed.ends_with(',') {
                fixed.pop(); // 移除最后的逗号
            }
            fixed.push_str("]");
        }
        
        // 确保对象结构完整
        if !fixed.contains("\"analysis_summary\"") {
            if fixed.ends_with(',') {
                fixed.pop();
            }
            fixed.push_str(",\"analysis_summary\":\"分析完成\"");
        }
    }
    
    fixed
}

// 多种JSON解析策略
fn try_parse_json_strategies(json_str: &str) -> Result<serde_json::Value> {
    // 策略1: 直接解析
    log::info!("策略1: 尝试直接解析JSON");
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(parsed) => {
            log::info!("策略1成功");
            return Ok(parsed);
        }
        Err(e) => {
            log::info!("策略1失败: {}", e);
        }
    }
    
    // 策略2: 标准化JSON格式后解析
    log::info!("策略2: 标准化JSON格式后解析");
    let normalized = normalize_json_format(json_str);
    match serde_json::from_str::<serde_json::Value>(&normalized) {
        Ok(parsed) => {
            log::info!("策略2成功");
            return Ok(parsed);
        }
        Err(e) => {
            log::info!("策略2失败: {}", e);
        }
    }
    
    // 策略3: 使用json5解析器（更宽松的JSON格式）
    log::info!("策略3: 使用宽松JSON解析");
    match parse_lenient_json(json_str) {
        Ok(parsed) => {
            log::info!("策略3成功");
            return Ok(parsed);
        }
        Err(e) => {
            log::info!("策略3失败: {}", e);
        }
    }
    
    // 策略4: 手动构建JSON对象
    log::info!("策略4: 手动构建JSON对象");
    match manual_json_parse(json_str) {
        Ok(parsed) => {
            log::info!("策略4成功");
            return Ok(parsed);
        }
        Err(e) => {
            log::info!("策略4失败: {}", e);
        }
    }
    
    // 策略5: 尝试从部分JSON中提取有用信息
    log::info!("策略5: 尝试从部分JSON中提取信息");
    let partial_result = try_partial_json_parse(json_str);
    log::info!("策略5: 部分解析完成");
    Ok(partial_result)
}

// 标准化JSON格式
fn normalize_json_format(json_str: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    
    for ch in json_str.chars() {
        if escape_next {
            normalized.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => {
                normalized.push(ch);
                escape_next = true;
            }
            '"' => {
                normalized.push(ch);
                in_string = !in_string;
            }
            c if c.is_whitespace() && !in_string => {
                // 在字符串外，将所有空白字符转换为单个空格
                if !normalized.ends_with(' ') && !normalized.ends_with('{') && !normalized.ends_with('[') {
                    normalized.push(' ');
                }
            }
            _ => {
                normalized.push(ch);
            }
        }
    }
    
    // 清理多余的空格
    normalized = normalized.replace("  ", " ");
    normalized = normalized.replace(" ,", ",");
    normalized = normalized.replace(" :", ":");
    normalized = normalized.replace("{ ", "{");
    normalized = normalized.replace(" }", "}");
    normalized = normalized.replace("[ ", "[");
    normalized = normalized.replace(" ]", "]");
    
    normalized
}

// 宽松的JSON解析
fn parse_lenient_json(json_str: &str) -> Result<serde_json::Value> {
    // 移除所有注释（如果有的话）
    let without_comments = remove_json_comments(json_str);
    
    // 移除尾随逗号
    let without_trailing_commas = remove_trailing_commas(&without_comments);
    
    // 尝试解析
    serde_json::from_str::<serde_json::Value>(&without_trailing_commas)
        .map_err(|e| anyhow::anyhow!("宽松JSON解析失败: {}", e))
}

// 移除JSON注释
fn remove_json_comments(json_str: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    
    let chars: Vec<char> = json_str.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        if escape_next {
            result.push(ch);
            escape_next = false;
            i += 1;
            continue;
        }
        
        match ch {
            '\\' => {
                if in_string {
                    result.push(ch);
                    escape_next = true;
                }
            }
            '"' => {
                result.push(ch);
                in_string = !in_string;
            }
            '/' if !in_string && i + 1 < chars.len() => {
                let next_ch = chars[i + 1];
                if next_ch == '/' {
                    in_line_comment = true;
                    i += 2;
                    continue;
                } else if next_ch == '*' {
                    in_block_comment = true;
                    i += 2;
                    continue;
                }
            }
            '\n' if in_line_comment => {
                in_line_comment = false;
                i += 1;
                continue;
            }
            '*' if in_block_comment && i + 1 < chars.len() && chars[i + 1] == '/' => {
                in_block_comment = false;
                i += 2;
                continue;
            }
            _ => {
                if !in_line_comment && !in_block_comment {
                    result.push(ch);
                }
            }
        }
        
        i += 1;
    }
    
    result
}

// 移除尾随逗号
fn remove_trailing_commas(json_str: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = json_str.chars().collect();
    
    for (i, &ch) in chars.iter().enumerate() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => {
                result.push(ch);
                escape_next = true;
            }
            '"' => {
                result.push(ch);
                in_string = !in_string;
            }
            ',' if !in_string => {
                // 检查下一个非空白字符
                let mut found_next = false;
                for &next_ch in &chars[i+1..] {
                    if next_ch.is_whitespace() {
                        continue;
                    }
                    if next_ch == '}' || next_ch == ']' {
                        // 尾随逗号，跳过
                        found_next = true;
                        break;
                    }
                    // 不是尾随逗号，保留
                    result.push(ch);
                    found_next = true;
                    break;
                }
                if !found_next {
                    result.push(ch);
                }
            }
            _ => {
                result.push(ch);
            }
        }
    }
    
    result
}

// 手动解析JSON - 专门处理LLM返回的格式
fn manual_json_parse(json_str: &str) -> Result<serde_json::Value> {
    let mut result = serde_json::json!({
        "has_news": false,
        "news_list": [],
        "analysis_summary": "分析完成"
    });
    
    // 检查has_news字段
    if json_str.contains("\"has_news\": true") {
        result["has_news"] = serde_json::Value::Bool(true);
    }
    
    // 提取news_list数组
    if let Some(news_list_start) = json_str.find("\"news_list\"") {
        if let Some(array_start) = json_str[news_list_start..].find('[') {
            let array_start = news_list_start + array_start;
            if let Some(array_end) = find_matching_bracket(json_str, array_start, '[', ']') {
                let news_array_str = &json_str[array_start..=array_end];
                
                // 提取每个新闻项
                let mut news_items = Vec::new();
                let mut current_pos = 1; // 跳过开始的 '['
                
                while current_pos < news_array_str.len() - 1 {
                    if let Some(item_start) = news_array_str[current_pos..].find('{') {
                        let item_start = current_pos + item_start;
                        if let Some(item_end) = find_matching_bracket(news_array_str, item_start, '{', '}') {
                            let item_str = &news_array_str[item_start..=item_end];
                            
                            if let Some(news_item) = parse_news_item(item_str) {
                                news_items.push(news_item);
                            }
                            
                            current_pos = item_end + 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                result["news_list"] = serde_json::Value::Array(
                    news_items.into_iter().map(serde_json::Value::Object).collect()
                );
            }
        }
    }
    
    Ok(result)
}

// 查找匹配的括号
fn find_matching_bracket(s: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut count = 0;
    let mut in_string = false;
    let mut escape_next = false;
    
    for (i, ch) in s[start..].char_indices() {
        let actual_i = start + i;
        
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            c if c == open && !in_string => count += 1,
            c if c == close && !in_string => {
                count -= 1;
                if count == 0 {
                    return Some(actual_i);
                }
            }
            _ => {}
        }
    }
    
    None
}

// 解析单个新闻项
fn parse_news_item(item_str: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let mut item = serde_json::Map::new();
    
    // 提取title
    if let Some(title) = extract_string_field(item_str, "title") {
        item.insert("title".to_string(), serde_json::Value::String(title));
    } else {
        // 如果没有title，使用默认值
        item.insert("title".to_string(), serde_json::Value::String("未知标题".to_string()));
    }
    
    // 提取summary
    if let Some(summary) = extract_string_field(item_str, "summary") {
        item.insert("summary".to_string(), serde_json::Value::String(summary));
    } else {
        // 如果没有summary，使用默认值
        item.insert("summary".to_string(), serde_json::Value::String("暂无摘要".to_string()));
    }
    
    // 提取industry_type
    if let Some(industry_type) = extract_string_field(item_str, "industry_type") {
        item.insert("industry_type".to_string(), serde_json::Value::String(industry_type));
    } else {
        // 如果没有industry_type，使用默认值
        item.insert("industry_type".to_string(), serde_json::Value::String("其他".to_string()));
    }
    
    // 提取news_type
    if let Some(news_type) = extract_string_field(item_str, "news_type") {
        item.insert("news_type".to_string(), serde_json::Value::String(news_type));
    } else {
        // 如果没有news_type，使用默认值
        item.insert("news_type".to_string(), serde_json::Value::String("其他".to_string()));
    }
    
    // 提取confidence
    if let Some(confidence) = extract_number_field(item_str, "confidence") {
        item.insert("confidence".to_string(), serde_json::Value::Number(confidence));
    } else {
        // 如果没有confidence，使用默认值
        item.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()));
    }
    
    // 总是返回item，不再过滤
    Some(item)
}

// 提取字符串字段
fn extract_string_field(s: &str, field_name: &str) -> Option<String> {
    let pattern = format!("\"{}\":", field_name);
    if let Some(start) = s.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(quote_start) = s[value_start..].find('"') {
            let quote_start = value_start + quote_start + 1;
            if let Some(quote_end) = find_string_end(&s[quote_start..]) {
                let quote_end = quote_start + quote_end;
                let value = &s[quote_start..quote_end];
                // 处理转义字符
                let cleaned = value.replace("\\\"", "\"").replace("\\\\", "\\");
                return Some(cleaned);
            }
        }
    }
    None
}

// 提取数字字段
fn extract_number_field(s: &str, field_name: &str) -> Option<serde_json::Number> {
    let pattern = format!("\"{}\":", field_name);
    if let Some(start) = s.find(&pattern) {
        let value_start = start + pattern.len();
        let value_part = &s[value_start..];
        
        // 查找数字的结束位置
        let mut end_pos = 0;
        for (i, ch) in value_part.char_indices() {
            if ch == ',' || ch == '}' || ch.is_whitespace() {
                end_pos = i;
                break;
            }
        }
        
        if end_pos > 0 {
            let num_str = value_part[..end_pos].trim();
            if let Ok(num) = num_str.parse::<f64>() {
                return serde_json::Number::from_f64(num);
            }
        }
    }
    None
}

// 查找字符串结束位置
fn find_string_end(s: &str) -> Option<usize> {
    let mut escape_next = false;
    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => escape_next = true,
            '"' => return Some(i),
            _ => {}
        }
    }
    None
}

// 高级JSON修复
fn fix_json_advanced(json_str: &str) -> String {
    let mut fixed = json_str.to_string();
    
    // 修复转义字符问题
    fixed = fixed.replace(r#"\""#, r#"""#);
    fixed = fixed.replace(r#"\\"#, r#"\"#);
    
    // 修复字符串中的换行符
    fixed = fixed.replace(r#"\n"#, "\\n");
    fixed = fixed.replace(r#"\r"#, "\\r");
    fixed = fixed.replace(r#"\t"#, "\\t");
    
    // 移除不必要的空格
    fixed = fixed.replace(r#", "#, ",");
    fixed = fixed.replace(r#": "#, ":");
    
    // 确保JSON结构完整
    if !fixed.trim_end().ends_with('}') {
        let open_braces = fixed.matches('{').count();
        let close_braces = fixed.matches('}').count();
        let missing_braces = open_braces - close_braces;
        
        for _ in 0..missing_braces {
            fixed.push('}');
        }
    }
    
    // 修复数组结构
    if fixed.contains("\"news_list\"") {
        let open_brackets = fixed.matches('[').count();
        let close_brackets = fixed.matches(']').count();
        let missing_brackets = open_brackets - close_brackets;
        
        for _ in 0..missing_brackets {
            fixed.push(']');
        }
    }
    
    fixed
}

// 尝试从部分JSON中解析有用信息
fn try_partial_json_parse(json_str: &str) -> serde_json::Value {
    // 如果JSON被截断，尝试创建一个最小的有效结构
    let mut result = serde_json::json!({
        "has_news": false,
        "news_list": [],
        "analysis_summary": "解析失败，但已尽力提取信息"
    });
    
    // 尝试提取has_news字段
    if let Some(has_news_start) = json_str.find("\"has_news\"") {
        if let Some(colon_pos) = json_str[has_news_start..].find(':') {
            let value_start = has_news_start + colon_pos + 1;
            let value_part = &json_str[value_start..];
            if let Some(semicolon_pos) = value_part.find(',') {
                let value_str = value_part[..semicolon_pos].trim();
                if value_str == "true" {
                    result["has_news"] = serde_json::Value::Bool(true);
                }
            }
        }
    }
    
    // 尝试提取news_list中的部分信息
    if let Some(news_list_start) = json_str.find("\"news_list\"") {
        // 简单尝试提取第一个新闻项的标题
        if let Some(title_start) = json_str[news_list_start..].find("\"title\"") {
            let title_section = &json_str[news_list_start + title_start..];
            if let Some(colon_pos) = title_section.find(':') {
                let value_start = colon_pos + 1;
                if let Some(quote_end) = title_section[value_start..].find('"') {
                    let title = &title_section[value_start + 1..value_start + quote_end];
                    if !title.is_empty() {
                        result["analysis_summary"] = serde_json::Value::String(format!("部分解析成功，提取到标题: {}", title));
                    }
                }
            }
        }
    }
    
    result
}


// 分析请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub article_ids: Vec<String>,
    pub prompt_template: String,
}


// 计算文本相似度（基于编辑距离的简单算法）
fn calculate_text_similarity(text1: &str, text2: &str) -> f64 {
    if text1.is_empty() && text2.is_empty() {
        return 1.0;
    }
    if text1.is_empty() || text2.is_empty() {
        return 0.0;
    }
    
    let distance = levenshtein_distance(text1, text2);
    let max_len = text1.len().max(text2.len());
    
    if max_len == 0 {
        return 1.0;
    }
    
    1.0 - (distance as f64 / max_len as f64)
}

// 计算编辑距离
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
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

// 检查新闻是否为重复
async fn check_news_duplicate(
    db: &Arc<crate::database::Database>,
    title: &str,
    summary: &str,
    industry_type: &str,
    news_type: &str,
) -> Result<bool> {
    // 1. 检查完全重复（15天内）
    if db.check_exact_duplicate(title, summary, 15)? {
        log::info!("发现完全重复新闻: {}", title);
        return Ok(true);
    }
    
    // 2. 检查标题相似（15天内，相似度>0.8）
    let similar_news = db.check_similar_title(title, 0.8, 15)?;
    for existing in similar_news {
        // 如果标题相似且行业类型和新闻类型也相同，认为是重复
        if existing.industry_type == industry_type && existing.news_type == news_type {
            log::info!("发现相似重复新闻: {} -> {}", title, existing.title);
            return Ok(true);
        }
    }
    
    // 3. 检查是否为进展更新（保留）
    if is_progress_update(title, summary, db)? {
        log::info!("识别为进展更新，保留: {}", title);
        return Ok(false);
    }
    
    Ok(false)
}

// 检查是否为进展更新
fn is_progress_update(title: &str, summary: &str, db: &Arc<crate::database::Database>) -> Result<bool> {
    // 进展更新的关键词
    let progress_keywords = vec![
        "新一轮", "再次", "继续", "进一步", "最新", "更新", "进展",
        "第二季度", "第三季度", "第四季度", "第一季度",
        "A轮", "B轮", "C轮", "D轮", "E轮",
        "版本2.0", "版本3.0", "版本4.0", "v2.0", "v3.0", "v4.0",
        "后续", "跟进", "追加", "扩大", "升级"
    ];
    
    // 检查标题或摘要是否包含进展关键词
    let text = format!("{} {}", title, summary).to_lowercase();
    for keyword in &progress_keywords {
        if text.contains(keyword) {
            return Ok(true);
        }
    }
    
    // 检查是否为同一公司的不同事件（通过公司名识别）
    let companies = extract_company_names(title);
    if !companies.is_empty() {
        // 查找同一公司最近15天的新闻
        let recent_news = db.get_analyzed_news_since(15)?;
        for existing in recent_news {
            let existing_companies = extract_company_names(&existing.title);
            if companies.iter().any(|c| existing_companies.contains(c)) {
                // 如果是同一公司但标题差异较大，可能是不同事件，保留
                let similarity = calculate_text_similarity(title, &existing.title);
                if similarity < 0.7 {
                    return Ok(true);
                }
            }
        }
    }
    
    Ok(false)
}

// 提取公司名称
fn extract_company_names(text: &str) -> Vec<String> {
    let mut companies = Vec::new();
    
    // 常见的科技公司关键词
    let tech_companies = vec![
        "OpenAI", "Meta", "Google", "Microsoft", "Apple", "Amazon", 
        "Tesla", "NVIDIA", "AMD", "Intel", "Samsung", "SK海力士",
        "日立", "甲骨文", "Crusoe", "Yondr", "Vantage", "CoreWeave",
        "腾讯", "阿里巴巴", "百度", "字节跳动", "华为", "小米"
    ];
    
    for company in &tech_companies {
        if text.contains(company) {
            companies.push(company.to_string());
        }
    }
    
    companies
}

// 获取启用的LLM配置
fn get_enabled_llm_config(db: &Arc<crate::database::Database>) -> Result<(String, String, String, f64, i32)> {
    // 从数据库获取启用的LLM配置
    match db.get_enabled_llm_config() {
        Ok(Some(config)) => {
            let api_key = config.api_key;
            let endpoint = config.endpoint;
            let model_id = config.model_id;
            let temperature = config.temperature;
            let max_tokens = config.max_tokens;

            if api_key.is_empty() || api_key == "sk-YourOpenAIApiKeyHere" {
                return Err(anyhow::anyhow!("请在设置中配置有效的LLM API密钥"));
            }

            Ok((api_key, endpoint, model_id, temperature, max_tokens))
        }
        Ok(None) => {
            Err(anyhow::anyhow!("未找到启用的LLM配置，请在设置中配置LLM"))
        }
        Err(e) => {
            log::error!("获取LLM配置失败: {}", e);
            // 如果数据库查询失败，尝试从环境变量获取作为备选
            let api_key = std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "sk-YourOpenAIApiKeyHere".to_string());
            let endpoint = std::env::var("OPENAI_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string());
            let model_id = std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
            let temperature = std::env::var("OPENAI_TEMPERATURE")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse::<f64>()
                .unwrap_or(0.7);
            let max_tokens = std::env::var("OPENAI_MAX_TOKENS")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<i32>()
                .unwrap_or(3000);

            if api_key == "sk-YourOpenAIApiKeyHere" {
                return Err(anyhow::anyhow!("请在设置中配置有效的LLM API密钥"));
            }

            Ok((api_key, endpoint, model_id, temperature, max_tokens))
        }
    }
}

// 记录分析日志
#[tauri::command]
pub async fn add_analysis_log(
    task_id: String,
    level: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = &state.db;

    let log_entry = crate::database::models::AnalysisLog {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: task_id.clone(),
        level: level.clone(),
        message: message.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db.insert_analysis_log(&log_entry)
        .map_err(|e| format!("插入日志失败: {}", e))?;

    log::info!("记录分析日志 [{}] [{}]: {}", level, task_id, message);
    Ok("日志记录成功".to_string())
}

// 获取分析日志
#[tauri::command]
pub async fn get_analysis_logs(
    task_id: Option<String>,
    level: Option<String>,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::models::AnalysisLog>, String> {
    let db = &state.db;

    match db.get_analysis_logs(task_id.as_deref(), level.as_deref(), limit) {
        Ok(logs) => Ok(logs),
        Err(e) => Err(format!("获取日志失败: {}", e)),
    }
}

// 清空分析日志
#[tauri::command]
pub async fn clear_analysis_logs(
    task_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = &state.db;

    let count = if let Some(task_id) = task_id {
        db.clear_analysis_logs_by_task(&task_id)
            .map_err(|e| format!("清空任务日志失败: {}", e))?
    } else {
        db.clear_all_analysis_logs()
            .map_err(|e| format!("清空所有日志失败: {}", e))?
    };

    Ok(format!("已清空 {} 条日志记录", count))
}

// 获取日志统计信息
#[tauri::command]
pub async fn get_log_stats(
    task_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = &state.db;

    let (total_count, error_count, warning_count, info_count) = if let Some(task_id) = task_id {
        db.get_log_stats_by_task(&task_id)
            .map_err(|e| format!("获取任务日志统计失败: {}", e))?
    } else {
        db.get_all_log_stats()
            .map_err(|e| format!("获取日志统计失败: {}", e))?
    };

    let stats_json = serde_json::json!({
        "total": total_count,
        "error": error_count,
        "warning": warning_count,
        "info": info_count
    });

    Ok(serde_json::to_string_pretty(&stats_json).unwrap())
}
