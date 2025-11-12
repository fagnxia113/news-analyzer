use crate::state::AppState;
use crate::database::models::{LlmConfig, AllSettings};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use chrono;

// 用于接收前端数据的结构（不包含 id、created_at、updated_at）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigInput {
    pub name: String,
    pub api_key: String,
    pub endpoint: String,
    pub model_id: String,
    pub temperature: f64,
    pub max_tokens: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplateInput {
    pub name: String,
    pub template: String,
    pub is_default: bool,
}


// 添加 LLM 配置
#[tauri::command]
pub async fn add_llm_config(
    config: LlmConfigInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("添加 LLM 配置: {:?}", config);
    
    // 生成 ID 和时间戳
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let full_config = LlmConfig {
        id: id.clone(),
        name: config.name,
        api_key: config.api_key,
        endpoint: config.endpoint,
        model_id: config.model_id,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        enabled: config.enabled,
        created_at: now.clone(),
        updated_at: now,
    };
    
    // 保存到数据库
    let db = &state.db;
    db.insert_llm_config(&full_config)
        .map_err(|e| format!("保存 LLM 配置失败: {}", e))?;
    
    Ok("LLM 配置已添加".to_string())
}

// 更新 LLM 配置
#[tauri::command]
pub async fn update_llm_config(
    id: String,
    config: LlmConfigInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("更新 LLM 配置: {} {:?}", id, config);
    
    let now = chrono::Utc::now().to_rfc3339();
    
    let full_config = LlmConfig {
        id: id.clone(),
        name: config.name,
        api_key: config.api_key,
        endpoint: config.endpoint,
        model_id: config.model_id,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        enabled: config.enabled,
        created_at: String::new(), // 将从数据库获取
        updated_at: now,
    };
    
    // 更新数据库中的配置
    let db = &state.db;
    db.update_llm_config(&id, &full_config)
        .map_err(|e| format!("更新 LLM 配置失败: {}", e))?;
    
    Ok("LLM 配置已更新".to_string())
}

// 删除 LLM 配置
#[tauri::command]
pub async fn delete_llm_config(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("删除 LLM 配置: {}", id);
    
    // 从数据库删除配置
    let db = &state.db;
    db.delete_llm_config(&id)
        .map_err(|e| format!("删除 LLM 配置失败: {}", e))?;
    
    Ok("LLM 配置已删除".to_string())
}

// 切换 LLM 配置启用状态
#[tauri::command]
pub async fn toggle_llm_config(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("切换 LLM 配置状态: {}", id);
    
    // 切换数据库中的启用状态
    let db = &state.db;
    db.toggle_llm_config(&id)
        .map_err(|e| format!("切换 LLM 配置状态失败: {}", e))?;
    
    Ok("LLM 配置状态已切换".to_string())
}

// 添加提示词模板
#[tauri::command]
pub async fn add_prompt_template(
    template: PromptTemplateInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("添加提示词模板: {}", template.name);
    
    // 生成 ID 和时间戳
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let prompt_template = crate::database::models::PromptTemplate {
        id: id.clone(),
        name: template.name,
        template: template.template,
        is_default: template.is_default,
        created_at: now.clone(),
        updated_at: now,
    };
    
    // 保存到数据库
    let db = &state.db;
    db.insert_prompt_template(&prompt_template)
        .map_err(|e| format!("保存提示词模板失败: {}", e))?;
    
    Ok("提示词模板已添加".to_string())
}

// 更新提示词模板
#[tauri::command]
pub async fn update_prompt_template(
    id: String,
    template: PromptTemplateInput,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("更新提示词模板: {} {}", id, template.name);
    
    let now = chrono::Utc::now().to_rfc3339();
    
    let prompt_template = crate::database::models::PromptTemplate {
        id: id.clone(),
        name: template.name,
        template: template.template,
        is_default: template.is_default,
        created_at: String::new(), // 将从数据库获取
        updated_at: now,
    };
    
    // 更新数据库
    let db = &state.db;
    db.update_prompt_template(&id, &prompt_template)
        .map_err(|e| format!("更新提示词模板失败: {}", e))?;
    
    Ok("提示词模板已更新".to_string())
}

// 删除提示词模板
#[tauri::command]
pub async fn delete_prompt_template(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("删除提示词模板: {}", id);
    
    // 从数据库删除
    let db = &state.db;
    db.delete_prompt_template(&id)
        .map_err(|e| format!("删除提示词模板失败: {}", e))?;
    
    Ok("提示词模板已删除".to_string())
}

// 获取默认提示词模板
#[tauri::command]
pub async fn get_default_prompt_template(
    state: State<'_, AppState>,
) -> Result<Option<crate::database::models::PromptTemplate>, String> {
    log::info!("获取默认提示词模板");
    
    let db = &state.db;
    db.get_default_prompt_template()
        .map_err(|e| format!("获取默认提示词模板失败: {}", e))
}

// 创建默认提示词模板
#[tauri::command]
pub async fn create_default_prompt_templates(
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("创建默认提示词模板");
    
    let db = &state.db;
    
    // 检查是否已有模板
    let existing_templates = db.get_all_prompt_templates()
        .map_err(|e| format!("获取现有模板失败: {}", e))?;
    
    if !existing_templates.is_empty() {
        return Ok("默认模板已存在".to_string());
    }
    
    // 创建默认模板
    let default_template = crate::database::models::PromptTemplate {
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
    
    db.insert_prompt_template(&default_template)
        .map_err(|e| format!("创建默认模板失败: {}", e))?;
    
    Ok("默认提示词模板已创建".to_string())
}

// 加载所有设置
#[tauri::command]
pub async fn load_all_settings(
    state: State<'_, AppState>,
) -> Result<AllSettings, String> {
    log::info!("加载所有设置");
    
    // 从数据库加载设置
    let db = &state.db;
    let settings = db.get_all_settings()
        .map_err(|e| format!("加载设置失败: {}", e))?;
    
    Ok(settings)
}

// 测试 LLM 连接
#[tauri::command]
pub async fn test_llm_connection(
    config: LlmConfigInput,
) -> Result<String, String> {
    log::info!("测试 LLM 连接: {:?}", config);
    
    // 参数验证
    if config.api_key.is_empty() {
        return Err("API 密钥不能为空".to_string());
    }
    
    if config.endpoint.is_empty() {
        return Err("API 端点不能为空".to_string());
    }
    
    if config.model_id.is_empty() {
        return Err("模型 ID 不能为空".to_string());
    }
    
    // 发送真实的测试请求
    let test_prompt = "请回复一个简单的'测试成功'来验证连接。";
    
    // 构建请求体
    let request_body = serde_json::json!({
        "model": config.model_id,
        "messages": [
            {
                "role": "user",
                "content": test_prompt
            }
        ],
        "temperature": 0.1,
        "max_tokens": 50
    });
    
    // 创建HTTP客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30)) // 测试连接用较短超时
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 发送请求
    let response = client
        .post(&config.endpoint)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败: {} - {}", status, error_text));
    }
    
    // 解析响应
    let response_text = response.text().await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    
    log::info!("LLM API 原始响应: {}", response_text);
    
    // 检查响应是否为空
    if response_text.trim().is_empty() {
        return Err("API 返回了空响应".to_string());
    }
    
    // 尝试解析JSON
    let api_response: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&response_text);
    let api_response = match api_response {
        Ok(response) => response,
        Err(e) => {
            log::error!("JSON解析失败: {}, 原始响应: {}", e, response_text);
            return Err(format!("API 响应格式错误: {}", e));
        }
    };
    
    // 检查是否有错误信息
    if let Some(error) = api_response.get("error") {
        let error_msg = error.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        return Err(format!("API 返回错误: {}", error_msg));
    }
    
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
    
    if content.is_empty() {
        log::error!("无法从响应中提取内容，完整响应: {}", api_response);
        return Err("API 返回了空内容".to_string());
    }
    
    log::info!("LLM 测试响应: {}", content);
    Ok(format!("LLM 连接测试成功: {}", content))
}

// 重置设置为默认值
#[tauri::command]
pub async fn reset_settings(
    _state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("重置所有设置为默认值");
    
    // TODO: 从数据库删除所有设置
    
    Ok("设置已重置为默认值".to_string())
}

// 导出设置
#[tauri::command]
pub async fn export_settings(
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("导出设置");
    
    // TODO: 从数据库加载设置并导出为 JSON
    
    let _settings = load_all_settings(state).await?;
    let json = serde_json::to_string_pretty(&_settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    
    Ok(json)
}

// 导入设置
#[tauri::command]
pub async fn import_settings(
    json_data: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("导入设置");
    
    // TODO: 解析 JSON 并保存到数据库
    
    let _settings: AllSettings = serde_json::from_str(&json_data)
        .map_err(|e| format!("解析设置失败: {}", e))?;
    
    // TODO: 批量保存到数据库
    
    Ok("设置导入成功".to_string())
}
