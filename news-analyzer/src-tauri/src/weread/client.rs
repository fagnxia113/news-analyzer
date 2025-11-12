use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Clone)]
pub struct WeReadClient {
    pub(crate) client: Client,
    pub(crate) base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginQRCode {
    pub uuid: String,
    #[serde(rename = "scanUrl")]
    pub scan_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResult {
    pub vid: Option<String>,
    pub token: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpInfo {
    pub id: String,
    pub name: String,
    pub intro: String,
    pub cover: String,
    pub update_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>, // API 返回 picUrl，我们映射为 pic_url
    #[serde(rename = "publishTime")]
    pub publish_time: u64, // API 返回 publishTime，我们映射为 publish_time
}

impl WeReadClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
        
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }
    
    /// 创建登录二维码
    pub async fn create_login_url(&self) -> Result<LoginQRCode> {
        let response = self.client
            .get(&format!("{}/api/v2/login/platform", self.base_url))
            .send()
            .await?;
        
        if response.status().is_success() {
            let response_text = response.text().await?;
            log::info!("API响应原始内容: {}", response_text);
            
            let qr_code: LoginQRCode = serde_json::from_str(&response_text)
                .map_err(|e| anyhow::anyhow!("JSON解析失败: {}, 原始数据: {}", e, response_text))?;
            
            log::info!("解析后的二维码数据: uuid={}, scan_url={}", qr_code.uuid, qr_code.scan_url);
            Ok(qr_code)
        } else {
            Err(anyhow::anyhow!("获取登录二维码失败: {}", response.status()))
        }
    }
    
    /// 检查登录结果
    pub async fn get_login_result(&self, uuid: &str) -> Result<Option<LoginResult>> {
        let response = self.client
            .get(&format!("{}/api/v2/login/platform/{}", self.base_url, uuid))
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;
        
        if response.status().is_success() {
            let response_text = response.text().await?;
            log::info!("登录检查API响应: {}", response_text);
            
            #[derive(Deserialize)]
            struct LoginResponse {
                message: Option<String>,
                vid: Option<i64>,  // 改为i64，因为API返回的是数字
                token: Option<String>,
                username: Option<String>,
            }
            
            let data: LoginResponse = serde_json::from_str(&response_text)
                .map_err(|e| anyhow::anyhow!("JSON解析失败: {}, 原始数据: {}", e, response_text))?;
            
            // 检查是否有完整的登录信息
            if data.vid.is_some() && data.token.is_some() && data.username.is_some() {
                Ok(Some(LoginResult {
                    vid: Some(data.vid.unwrap().to_string()), // 转换为字符串
                    token: Some(data.token.unwrap()),
                    username: Some(data.username.unwrap()),
                }))
            } else {
                // 如果有message字段，说明是等待状态或其他状态，继续等待
                log::info!("登录未完成，继续等待...");
                Ok(None)
            }
        } else {
            Err(anyhow::anyhow!("检查登录状态失败: {}", response.status()))
        }
    }
    
    /// 根据微信文章链接获取公众号信息
    pub async fn get_mp_info(&self, url: &str, account_vid: &str, account_token: &str) -> Result<MpInfo> {
        let response = self.client
            .post(&format!("{}/api/v2/platform/wxs2mp", self.base_url))
            .header("xid", account_vid)
            .header("Authorization", format!("Bearer {}", account_token))
            .json(&serde_json::json!({ "url": url }))
            .send()
            .await?;
        
        if response.status().is_success() {
            let response_text = response.text().await?;
            log::info!("获取公众号信息API响应: {}", response_text);
            
            #[derive(Deserialize)]
            struct MpResponse {
                id: String,
                name: String,
                intro: String,
                cover: String,
                #[serde(rename = "updateTime")]
                update_time: Option<u64>, // 改为可选，并重命名
            }
            
            let data: Vec<MpResponse> = serde_json::from_str(&response_text)
                .map_err(|e| anyhow::anyhow!("JSON解析失败: {}, 原始数据: {}", e, response_text))?;
            
            if let Some(mp) = data.first() {
                Ok(MpInfo {
                    id: mp.id.clone(),
                    name: mp.name.clone(),
                    intro: mp.intro.clone(),
                    cover: mp.cover.clone(),
                    update_time: mp.update_time.unwrap_or(0), // 提供默认值
                })
            } else {
                Err(anyhow::anyhow!("未找到公众号信息"))
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("获取公众号信息失败: {}, 错误: {}", status, error_text))
        }
    }
    
    /// 获取公众号文章列表
    pub async fn get_mp_articles(&self, mp_id: &str, account_vid: &str, account_token: &str, page: i32) -> Result<Vec<Article>> {
        let url = format!("{}/api/v2/platform/mps/{}/articles", self.base_url, mp_id);
        log::info!("请求URL: {}", url);
        
        let response = self.client
            .get(&url)
            .header("xid", account_vid)
            .header("Authorization", format!("Bearer {}", account_token))
            .query(&[("page", &page.to_string())])
            .send()
            .await?;
        
        log::info!("响应状态: {}", response.status());
        
        if response.status().is_success() {
            // 先获取原始响应文本用于调试
            let response_text = response.text().await?;
            log::info!("API响应长度: {} 字符", response_text.len());
            
            if response_text.trim().is_empty() {
                log::warn!("API返回空响应");
                return Ok(vec![]);
            }
            
            // 解析JSON
            let data: Vec<Article> = serde_json::from_str(&response_text)
                .map_err(|e| anyhow::anyhow!("JSON解析失败: {}, 原始数据: {}", e, response_text))?;
            
            log::info!("解析到{}篇文章", data.len());
            
            let articles = data.into_iter().map(|mut item| {
                // 构造微信文章链接
                item.url = format!("https://mp.weixin.qq.com/s/{}", item.id);
                item
            }).collect();
            
            Ok(articles)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("API请求失败: {}, 错误: {}", status, error_text);
            Err(anyhow::anyhow!("获取文章列表失败: {}, 错误: {}", status, error_text))
        }
    }
}
