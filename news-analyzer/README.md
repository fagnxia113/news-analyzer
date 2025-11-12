# 新闻分析器 MVP

这是一个最小可行版本（MVP），用于验证基本的 Tauri + Vue 3 + Rust 技术栈。

## 功能

- 基本的 Tauri 桌面应用框架
- Vue 3 前端界面
- Rust 后端

## 开发环境要求

- Node.js (v16+)
- Rust (最新稳定版)
- pnpm (推荐) 或 npm

## 安装依赖

```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖 (会自动处理)
```

## 运行

```bash
# 开发模式
pnpm dev
```

## 构建

```bash
# 构建生产版本
pnpm build
```

## 项目结构

```
news-analyzer-mvp/
├── src/              # 前端源码
│   ├── App.vue       # 主应用组件
│   └── main.ts       # 入口文件
├── src-tauri/        # Rust 后端源码
│   ├── src/
│   │   └── main.rs   # 主程序入口
│   ├── Cargo.toml    # Rust 依赖配置
│   └── tauri.conf.json # Tauri 配置
├── index.html        # HTML 入口
├── package.json      # Node.js 依赖配置
├── tsconfig.json     # TypeScript 配置
└── vite.config.ts    # Vite 配置
```