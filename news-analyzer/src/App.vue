<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <div class="sidebar">
      <div class="sidebar-header">
        <div class="app-logo">
          <div class="logo-icon">📰</div>
          <div class="logo-text">
            <div class="logo-title">新闻分析器</div>
            <div class="logo-subtitle">News Analyzer</div>
          </div>
        </div>
        <div class="app-version">v1.0.0</div>
      </div>
      <nav class="nav-menu">
        <div 
          class="nav-item" 
          :class="{ active: currentPage === 'accounts' }"
          @click="switchPage('accounts')"
        >
          <div class="nav-icon">👤</div>
          <div class="nav-content">
            <span class="nav-title">账号管理</span>
            <span class="nav-desc">管理微信读书账号</span>
          </div>
        </div>
        <div 
          class="nav-item"
          :class="{ active: currentPage === 'feeds' }"
          @click="switchPage('feeds')"
        >
          <div class="nav-icon">📚</div>
          <div class="nav-content">
            <span class="nav-title">订阅源</span>
            <span class="nav-desc">管理RSS订阅源</span>
          </div>
        </div>
        <div 
          class="nav-item"
          :class="{ active: currentPage === 'analysis-results' }"
          @click="switchPage('analysis-results')"
        >
          <div class="nav-icon">📊</div>
          <div class="nav-content">
            <span class="nav-title">分析结果</span>
            <span class="nav-desc">查看新闻分析结果</span>
          </div>
        </div>
        <div 
          class="nav-item"
          :class="{ active: currentPage === 'settings' }"
          @click="switchPage('settings')"
        >
          <div class="nav-icon">⚙️</div>
          <div class="nav-content">
            <span class="nav-title">设置</span>
            <span class="nav-desc">配置系统参数</span>
          </div>
        </div>
      </nav>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 顶部栏 -->
      <div class="top-bar">
        <div class="page-info">
          <div class="page-title">{{ pageTitle }}</div>
          <div class="page-subtitle">{{ pageSubtitle }}</div>
        </div>
        <div class="top-actions">
          <button v-if="currentPage === 'accounts'" class="btn btn-primary" @click="openLoginModal">
            <span class="btn-icon">+</span>
            添加读书账号
          </button>
        </div>
      </div>

      <!-- 内容区域 -->
      <div class="content-area">
        <!-- 账号管理页面 -->
        <AccountsPage 
          v-if="currentPage === 'accounts'"
          :show-login-modal="showLoginModal"
          @close-login-modal="closeLoginModal"
        />

        <!-- 订阅源页面 -->
        <FeedsPage v-if="currentPage === 'feeds'" />

        <!-- 分析结果页面 -->
        <AnalysisResultsPage v-if="currentPage === 'analysis-results'" />

        <!-- 设置页面 -->
        <SettingsPage v-if="currentPage === 'settings'" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import AccountsPage from './components/AccountsPage.vue'
import FeedsPage from './components/FeedsPage.vue'
import SettingsPage from './components/SettingsPage.vue'
import AnalysisResultsPage from './components/AnalysisResultsPage.vue'

const currentPage = ref('accounts')
const showLoginModal = ref(false)

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    accounts: '账号管理',
    feeds: '订阅源管理',
    'analysis-results': '分析结果',
    settings: '设置'
  }
  return titles[currentPage.value] || '新闻分析器'
})

const pageSubtitle = computed(() => {
  const subtitles: Record<string, string> = {
    accounts: '管理微信读书账号，配置登录信息',
    feeds: '管理RSS订阅源，添加和编辑新闻源',
    'analysis-results': '查看AI分析的新闻结果和统计数据',
    settings: '配置LLM参数和系统设置'
  }
  return subtitles[currentPage.value] || '智能新闻分析平台'
})

const switchPage = (page: string) => {
  currentPage.value = page
}

const openLoginModal = () => {
  showLoginModal.value = true
}

const closeLoginModal = () => {
  showLoginModal.value = false
}
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.app-container {
  display: flex;
  height: 100vh;
  background: #f5f5f5;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif;
}

/* 侧边栏 */
.sidebar {
  width: 240px;
  background: white;
  color: #333;
  display: flex;
  flex-direction: column;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
  border-right: 1px solid #e9ecef;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid #e9ecef;
  text-align: center;
}

.app-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-bottom: 8px;
}

.logo-icon {
  font-size: 24px;
  color: #667eea;
}

.logo-text {
  text-align: left;
}

.logo-title {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 2px;
}

.logo-subtitle {
  font-size: 10px;
  color: #6c757d;
  font-weight: 500;
}

.app-version {
  font-size: 9px;
  color: #adb5bd;
  font-weight: 500;
  margin-top: 6px;
}

.nav-menu {
  flex: 1;
  padding: 16px 0;
  overflow-y: auto;
}

.nav-item {
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 2px 8px;
  border-radius: 8px;
}

.nav-item:hover {
  background: #f8f9fa;
}

.nav-item.active {
  background: #667eea;
  color: white;
}

.nav-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
}

.nav-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-title {
  font-size: 13px;
  font-weight: 600;
  color: inherit;
}

.nav-desc {
  font-size: 10px;
  color: inherit;
  opacity: 0.7;
}

.nav-item:hover .nav-title,
.nav-item.active .nav-title {
  color: inherit;
}

.nav-item:hover .nav-desc,
.nav-item.active .nav-desc {
  color: inherit;
  opacity: 0.8;
}

/* 主内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f8f9fa;
}

.top-bar {
  height: 60px;
  background: white;
  border-bottom: 1px solid #e9ecef;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.page-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
  color: #2c3e50;
}

.page-subtitle {
  font-size: 12px;
  color: #6c757d;
  font-weight: 500;
}

.top-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover {
  background: #5a6fd8;
}

.btn-secondary {
  background: white;
  color: #6c757d;
  border: 1px solid #e9ecef;
}

.btn-secondary:hover {
  background: #f8f9fa;
  border-color: #dee2e6;
}

.btn-icon {
  font-size: 14px;
}

/* 内容区域 */
.content-area {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  background: #f8f9fa;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(135deg, #5a6fd8, #6a4190);
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 80px 20px;
  color: #6c757d;
  position: relative;
  z-index: 1;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 24px;
  opacity: 0.3;
  filter: grayscale(100%);
}

.empty-title {
  font-size: 20px;
  margin-bottom: 8px;
  font-weight: 600;
  color: #495057;
}

.empty-description {
  font-size: 14px;
  color: #6c757d;
  max-width: 400px;
  margin: 0 auto;
  line-height: 1.6;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sidebar {
    width: 200px;
  }
  
  .top-bar {
    padding: 0 16px;
    height: 50px;
  }
  
  .page-title {
    font-size: 16px;
  }
  
  .content-area {
    padding: 16px;
  }
}

/* 动画效果 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.content-area > * {
  animation: fadeIn 0.6s ease-out;
}
</style>
