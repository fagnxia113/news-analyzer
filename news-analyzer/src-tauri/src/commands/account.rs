use crate::database::models::{WeChatAccount, LoginQRCode};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

#[tauri::command]
pub async fn get_login_qrcode(state: State<'_, AppState>) -> Result<LoginQRCode, String> {
    log::info!("开始获取登录二维码");
    let client = state.weread_client.lock().await;
    match client.create_login_url().await {
        Ok(qr) => {
            log::info!("成功获取登录二维码: uuid={}, scan_url={}", qr.uuid, qr.scan_url);
            Ok(LoginQRCode {
                uuid: qr.uuid,
                scan_url: qr.scan_url,
            })
        }
        Err(e) => {
            log::error!("获取登录二维码失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn check_login_status(
    uuid: String,
    state: State<'_, AppState>,
) -> Result<crate::database::models::LoginResult, String> {
    log::info!("检查登录状态: uuid={}", uuid);
    let client = state.weread_client.lock().await;
    match client.get_login_result(&uuid).await {
        Ok(Some(result)) => {
            if let (Some(vid), Some(token), Some(username)) = (result.vid, result.token, result.username) {
                log::info!("登录成功: vid={}, username={}", vid, username);
                let login_result = crate::database::models::LoginResult {
                    vid: Some(vid),
                    token: Some(token),
                    username: Some(username),
                    message: None,
                };
                Ok(login_result)
            } else {
                // 这种情况不应该发生，但为了安全起见
                log::warn!("登录结果格式异常");
                Ok(crate::database::models::LoginResult {
                    vid: None,
                    token: None,
                    username: None,
                    message: Some("登录结果格式异常".to_string()),
                })
            }
        }
        Ok(None) => {
            log::info!("登录未完成，继续等待...");
            // 返回一个空的LoginResult，表示继续等待
            Ok(crate::database::models::LoginResult {
                vid: None,
                token: None,
                username: None,
                message: None,
            })
        }
        Err(e) => {
            log::error!("检查登录状态失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn save_wechat_account(
    login_result: crate::database::models::LoginResult,
    state: State<'_, AppState>,
) -> Result<WeChatAccount, String> {
    // 检查是否有错误信息
    if let Some(message) = &login_result.message {
        return Err(format!("登录失败: {}", message));
    }
    
    // 检查必要字段是否存在
    let vid = login_result.vid.ok_or("缺少用户ID")?;
    let token = login_result.token.ok_or("缺少访问令牌")?;
    let username = login_result.username.ok_or("缺少用户名")?;
    
    log::info!("保存微信账号: vid={}, username={}", vid, username);
    
    let account = WeChatAccount {
        id: Uuid::new_v4().to_string(),
        vid,
        token,
        name: username,
        avatar: String::new(), // 暂时为空，后续可以获取
        status: 1,
        is_banned: false,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    
    match state.db.insert_account(&account) {
        Ok(_) => {
            log::info!("微信账号保存成功: vid={}", account.vid);
            Ok(account)
        }
        Err(e) => {
            log::error!("微信账号保存失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_all_accounts(state: State<'_, AppState>) -> Result<Vec<WeChatAccount>, String> {
    state.db.get_all_accounts()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account_status(
    account_id: String,
    status: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("更新账号状态: account_id={}, status={}", account_id, status);
    
    match state.db.update_account_status(&account_id, status) {
        Ok(_) => {
            log::info!("账号状态更新成功: account_id={}, new_status={}", account_id, status);
            Ok(())
        }
        Err(e) => {
            log::error!("账号状态更新失败: account_id={}, error={}", account_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn delete_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("删除账号: account_id={}", account_id);
    
    match state.db.delete_account(&account_id) {
        Ok(_) => {
            log::info!("账号删除成功: account_id={}", account_id);
            Ok(())
        }
        Err(e) => {
            log::error!("账号删除失败: account_id={}, error={}", account_id, e);
            Err(e.to_string())
        }
    }
}
