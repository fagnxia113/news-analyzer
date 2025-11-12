export interface WeChatAccount {
  id: string
  name: string
  avatar: string
  vid: string
  token: string
  status: number
  is_banned: boolean
  created_at: string
  updated_at: string
}

export interface WeChatFeed {
  id: string
  mp_name: string
  mp_intro: string
  mp_cover: string
  status: number
  sync_time: number
  update_time: number
  created_at: string
  updated_at: string
}

export interface WeChatArticle {
  id: string
  mp_id: string
  title: string
  url: string
  pic_url: string | null
  publish_time: number
  created_at: string
  updated_at: string
}

// RSS订阅源类型
export interface RSSFeed {
  id: string
  title: string
  url: string
  website_url: string
  description: string | null
  category: string | null
  status: number
  last_fetched: number
  created_at: string
  updated_at: string
}

// RSS文章类型
export interface RSSArticle {
  id: string
  feed_id: string
  title: string
  url: string
  content: string | null
  summary: string | null
  author: string | null
  publish_time: number
  created_at: string
  updated_at: string
}

// 统一的文章枚举
export type Article = 
  | { source_type: 'WeChat' } & WeChatArticle
  | { source_type: 'RSS' } & RSSArticle

// 统一的订阅源枚举
export type FeedSource = 
  | { type: 'WeChat' } & WeChatFeed
  | { type: 'RSS' } & RSSFeed

export interface LoginQRCode {
  uuid: string
  scan_url: string
}

export interface LoginResult {
  vid?: string | null
  token?: string | null
  username?: string | null
  message?: string | null
}

// 设置相关类型
export interface LlmConfig {
  id: string;
  name: string;
  api_key: string;
  endpoint: string;
  model_id: string;
  temperature: number;
  max_tokens: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface PromptTemplate {
  id: string;
  name: string;
  template: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface AllSettings {
  llm_configs: LlmConfig[];
  prompt_templates: PromptTemplate[];
}

// 智能分析相关类型
export interface AnalysisRequest {
  article_ids: string[];
  prompt_template: string;
}

export interface AnalysisTask {
  id: string;
  status: string; // "pending", "running", "completed", "failed"
  total_articles: number;
  processed_articles: number;
  success_count: number;
  failed_count: number;
  start_time: number;
  end_time?: number;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface AnalyzedNews {
  id: string;
  task_id: string;
  article_id: string;
  title: string;
  content: string;
  summary: string;
  is_soft_news: boolean;
  industry_type: string;
  news_type: string;
  confidence: number;
  keywords: string;
  original_url: string;
  analyzed_at: string;
  created_at: string;
  updated_at: string;
}

// 实时日志相关类型
export interface AnalysisLog {
  id: string;
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  task_id: string;
  context?: {
    article_title?: string;
    current_step?: string;
    progress?: number;
    total?: number;
  };
}
