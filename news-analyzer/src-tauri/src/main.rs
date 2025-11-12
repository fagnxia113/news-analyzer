// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod weread;
mod state;
mod rss;

use state::AppState;

fn main() {
    // 设置panic处理
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap_or_else(|| std::panic::Location::caller());
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic message"
        };
        
        log::error!(
            "Panic occurred at {}:{}: {}",
            location.file(),
            location.line(),
            message
        );
        
        eprintln!("应用遇到严重错误: {} at {}:{}", message, location.file(), location.line());
    }));
    
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    log::info!("应用启动中...");
    
    // 获取应用安装目录下的数据目录
    let app_data_dir = std::env::current_exe()
        .unwrap_or_else(|e| {
            log::error!("Failed to get executable path: {}", e);
            eprintln!("Failed to get executable path: {}", e);
            std::process::exit(1);
        })
        .parent()
        .unwrap_or_else(|| {
            log::error!("Failed to get executable parent directory");
            eprintln!("Failed to get executable parent directory");
            std::process::exit(1);
        })
        .join("data");
    
    // 确保数据目录存在
    std::fs::create_dir_all(&app_data_dir).unwrap_or_else(|e| {
        log::error!("Failed to create app data directory: {}", e);
        eprintln!("Failed to create app data directory: {}", e);
    });

    // 检查并迁移旧的数据库文件
    let old_db_path = std::env::temp_dir().join("news-analyzer-mvp").join("news_analyzer.db");
    let new_db_path = app_data_dir.join("news_analyzer.db");
    let db_path = if new_db_path.exists() {
        // 新位置已有数据库，直接使用
        new_db_path.to_string_lossy().to_string()
    } else if old_db_path.exists() {
        // 旧位置有数据库，需要迁移
        log::info!("发现旧数据库文件，正在迁移到新位置...");
        match std::fs::copy(&old_db_path, &new_db_path) {
            Ok(_) => {
                log::info!("数据库迁移成功: {} -> {}", old_db_path.display(), new_db_path.display());
                // 迁移成功后可以选择删除旧数据库文件（这里保留作为备份）
                log::info!("旧数据库文件已保留作为备份: {}", old_db_path.display());
            }
            Err(e) => {
                log::error!("数据库迁移失败: {}", e);
                eprintln!("数据库迁移失败: {}", e);
                // 迁移失败，继续使用旧数据库
                old_db_path.to_string_lossy().to_string()
            }
        }
        new_db_path.to_string_lossy().to_string()
    } else {
        // 都不存在，使用新位置
        new_db_path.to_string_lossy().to_string()
    };

    log::info!("数据库路径: {}", db_path);
    
    // 初始化应用状态
    let app_state = match AppState::new(&db_path) {
        Ok(state) => {
            log::info!("应用状态初始化成功");
            state
        },
        Err(e) => {
            log::error!("Failed to initialize app state: {}", e);
            eprintln!("Failed to initialize app state: {}", e);
            std::process::exit(1);
        }
    };
    
    log::info!("开始运行Tauri应用...");
    
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // 账号管理
            commands::get_login_qrcode,
            commands::check_login_status,
            commands::save_wechat_account,
            commands::get_all_accounts,
            commands::update_account_status,
            commands::delete_account,
            
            // 订阅源管理
            commands::add_feed_from_url,
            commands::get_all_feeds,
            commands::delete_feed,
            commands::refresh_feed,
            commands::refresh_all_feeds,
            commands::interrupt_refresh_refresh,
            commands::get_database_info,
            commands::debug_database_info,
            
            // RSS订阅源管理
            commands::add_rss_feed,
            commands::get_all_rss_feeds,
            commands::delete_rss_feed,
            commands::refresh_rss_feed,
            commands::refresh_all_rss_feeds,
            commands::validate_rss_url,
            
            // 文章管理
            commands::get_feed_articles,
            commands::get_all_articles,
            commands::get_rss_articles_debug,
            commands::debug_articles,
            
            // LLM 配置管理
            commands::add_llm_config,
            commands::update_llm_config,
            commands::delete_llm_config,
            commands::toggle_llm_config,
            commands::test_llm_connection,
            
            // 提示词模板管理
            commands::add_prompt_template,
            commands::update_prompt_template,
            commands::delete_prompt_template,
            commands::get_default_prompt_template,
            commands::create_default_prompt_templates,
            
            // 智能分析
            commands::start_analysis,
            commands::get_analysis_task,
            commands::get_analysis_tasks,
            commands::get_analyzed_news,
            commands::get_all_analyzed_news,
            commands::clear_all_analyzed_news,
            commands::delete_analyzed_news,
            commands::delete_multiple_analyzed_news,
            commands::fix_analyzed_news_type_ids,
            commands::get_recent_month_stats,
            
            // 设置管理
            commands::load_all_settings,
            commands::reset_settings,
            commands::export_settings,
            commands::import_settings,
        ])
        .run(tauri::generate_context!());
    
    match result {
        Ok(_) => {
            log::info!("应用正常退出");
        }
        Err(e) => {
            log::error!("应用运行出错: {}", e);
            eprintln!("error while running tauri application: {}", e);
            std::process::exit(1);
        }
    }
}
