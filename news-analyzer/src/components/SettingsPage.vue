<template>
  <div class="settings-page">
    <div class="page-header">
      <h1>系统设置</h1>
      <p>配置新闻分析系统的各项参数</p>
    </div>

      <!-- 标签页导航 -->
      <div class="tabs-container">
        <div class="tabs">
          <button 
            class="tab-button"
            :class="{ active: activeTab === 'llm' }"
            @click="activeTab = 'llm'"
          >
            🤖 LLM设置
          </button>
          <button
            class="tab-button"
            :class="{ active: activeTab === 'prompt' }"
            @click="activeTab = 'prompt'"
          >
            📝 提示词模板
          </button>
          <button
            class="tab-button"
            :class="{ active: activeTab === 'system' }"
            @click="activeTab = 'system'"
          >
            🗄️ 系统信息
          </button>
        </div>
      </div>

    <!-- 标签页内容 -->
    <div class="tab-content">
      <!-- LLM 设置标签页 -->
      <div v-if="activeTab === 'llm'" class="tab-panel">
        <div class="panel-header">
          <h2>LLM 配置管理</h2>
          <button class="btn btn-primary" @click="showAddLlmConfig">
            ➕ 添加配置
          </button>
        </div>

        <!-- LLM 配置列表 -->
        <div class="config-list">
          <div 
            v-for="config in llmConfigs" 
            :key="config.id"
            class="config-item"
            :class="{ disabled: !config.enabled }"
          >
            <div class="config-info">
              <div class="config-name">
                {{ config.name }}
                <span v-if="config.enabled" class="status-badge enabled">启用</span>
                <span v-else class="status-badge disabled">禁用</span>
              </div>
              <div class="config-details">
                {{ config.endpoint }} - {{ config.model_id }}
              </div>
            </div>
            <div class="config-actions">
              <button class="btn btn-sm btn-secondary" @click="testLlmConfig(config)">
                🧪 测试
              </button>
              <button class="btn btn-sm btn-secondary" @click="editLlmConfig(config)">
                ✏️ 编辑
              </button>
              <button 
                class="btn btn-sm" 
                :class="config.enabled ? 'btn-warning' : 'btn-success'"
                @click="toggleLlmConfig(config)"
              >
                {{ config.enabled ? '🔒 禁用' : '🔓 启用' }}
              </button>
              <button class="btn btn-sm btn-danger" @click="deleteLlmConfig(config)">
                🗑️ 删除
              </button>
            </div>
          </div>
        </div>

        <!-- 添加/编辑 LLM 配置对话框 -->
        <div v-if="isAddingLlmConfig || editingLlmConfig" class="modal-overlay">
          <div class="modal">
            <div class="modal-header">
              <h3>{{ isAddingLlmConfig ? '添加 LLM 配置' : '编辑 LLM 配置' }}</h3>
              <button class="modal-close" @click="cancelLlmEdit">×</button>
            </div>
            <div class="modal-body">
              <div class="form-group">
                <label>配置名称</label>
                <input v-model="llmForm.name" type="text" placeholder="输入配置名称" />
              </div>
              <div class="form-group">
                <label>模型 ID</label>
                <input v-model="llmForm.modelId" type="text" placeholder="输入模型 ID，如：gpt-4" />
              </div>
              <div class="form-group">
                <label>API 密钥</label>
                <input v-model="llmForm.apiKey" type="password" placeholder="输入 API 密钥" />
              </div>
              <div class="form-group">
                <label>API 端点</label>
                <input v-model="llmForm.endpoint" type="url" placeholder="https://api.example.com/v1" />
              </div>
              <div class="form-group">
                <label>Temperature ({{ llmForm.temperature }})</label>
                <input v-model.number="llmForm.temperature" type="range" min="0" max="1" step="0.1" />
              </div>
              <div class="form-group">
                <label>最大 Tokens</label>
                <input v-model.number="llmForm.maxTokens" type="number" min="100" max="8000" />
              </div>
              <div class="form-group">
                <label>
                  <input v-model="llmForm.enabled" type="checkbox" />
                  启用此配置
                </label>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn btn-secondary" @click="cancelLlmEdit">取消</button>
              <button class="btn btn-primary" @click="saveLlmConfig">保存</button>
            </div>
          </div>
        </div>
      </div>

      <!-- 提示词模板标签页 -->
      <div v-if="activeTab === 'prompt'" class="tab-panel">
        <div class="panel-header">
          <h2>提示词模板管理</h2>
          <div class="header-actions">
            <button class="btn btn-secondary" @click="createDefaultTemplates">
              📋 创建默认模板
            </button>
            <button class="btn btn-primary" @click="showAddPromptTemplate">
              ➕ 添加模板
            </button>
          </div>
        </div>

        <!-- 提示词模板列表 -->
        <div class="config-list">
          <div 
            v-for="template in promptTemplates" 
            :key="template.id"
            class="config-item prompt-item"
            :class="{ default: template.is_default }"
          >
            <div class="config-info">
              <div class="config-name">
                {{ template.name }}
                <span v-if="template.is_default" class="status-badge default">默认</span>
              </div>
              <div class="config-details">
                {{ template.template.substring(0, 100) }}{{ template.template.length > 100 ? '...' : '' }}
              </div>
            </div>
            <div class="config-actions">
              <button class="btn btn-sm btn-secondary" @click="previewPromptTemplate(template)">
                👁️ 预览
              </button>
              <button class="btn btn-sm btn-secondary" @click="editPromptTemplate(template)">
                ✏️ 编辑
              </button>
              <button 
                v-if="!template.is_default"
                class="btn btn-sm btn-success" 
                @click="setDefaultTemplate(template)"
              >
                ⭐ 设为默认
              </button>
              <button 
                v-if="!template.is_default"
                class="btn btn-sm btn-danger" 
                @click="deletePromptTemplate(template)"
              >
                🗑️ 删除
              </button>
            </div>
          </div>
        </div>

        <!-- 添加/编辑提示词模板对话框 -->
        <div v-if="isAddingPromptTemplate || editingPromptTemplate" class="modal-overlay">
          <div class="modal prompt-modal">
            <div class="modal-header">
              <h3>{{ isAddingPromptTemplate ? '添加提示词模板' : '编辑提示词模板' }}</h3>
              <button class="modal-close" @click="cancelPromptEdit">×</button>
            </div>
            <div class="modal-body">
              <div class="form-group">
                <label>模板名称 *</label>
                <input v-model="promptForm.name" type="text" placeholder="输入模板名称" required />
              </div>
              <div class="form-group">
                <label>模板内容 *</label>
                <textarea 
                  v-model="promptForm.template" 
                  placeholder="输入提示词模板内容，使用 {content} 作为文章内容的占位符"
                  rows="15"
                  required
                ></textarea>
                <div class="template-help">
                  <p>💡 提示：使用 <code>{content}</code> 作为文章内容的占位符</p>
                  <p>📋 可以在模板中直接定义行业类型和新闻类型的分类标准</p>
                  <button type="button" class="btn btn-sm btn-secondary" @click="useReferenceTemplate">
                    📋 使用优化后的参考模板
                  </button>
                </div>
              </div>
              <div class="form-group">
                <label>
                  <input v-model="promptForm.isDefault" type="checkbox" />
                  设为默认模板
                </label>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn btn-secondary" @click="cancelPromptEdit">取消</button>
              <button class="btn btn-primary" @click="savePromptTemplate" :disabled="!promptForm.name.trim() || !promptForm.template.trim()">保存</button>
            </div>
          </div>
        </div>

        <!-- 预览模板对话框 -->
        <div v-if="previewingTemplate" class="modal-overlay">
          <div class="modal preview-modal">
            <div class="modal-header">
              <h3>预览模板: {{ previewingTemplate.name }}</h3>
              <button class="modal-close" @click="previewingTemplate = null">×</button>
            </div>
            <div class="modal-body">
              <div class="preview-content">
                <pre>{{ previewingTemplate.template }}</pre>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn btn-secondary" @click="previewingTemplate = null">关闭</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 系统信息标签页 -->
    <div v-if="activeTab === 'system'" class="tab-panel">
      <div class="panel-header">
        <h2>系统信息</h2>
        <div class="header-actions">
          <button class="btn btn-secondary" @click="loadDatabaseInfo">
            🔄 刷新信息
          </button>
          <button class="btn btn-primary" @click="showDatabaseInfo">
            📊 查看数据库
          </button>
        </div>
      </div>

      <!-- 数据库信息 -->
      <div class="info-section">
        <h3 class="section-title">🗄️ 数据库信息</h3>
        <div class="info-content">
          <div v-if="databaseInfo" class="info-display">
            <div class="info-item">
              <span class="info-label">当前数据库路径:</span>
              <span class="info-value">{{ databaseInfo.currentPath }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">数据库状态:</span>
              <span class="info-value" :class="{ 'status-ok': databaseInfo.currentExists, 'status-error': !databaseInfo.currentExists }">
                {{ databaseInfo.currentExists ? '✅ 存在' : '❌ 不存在' }}
              </span>
            </div>
            <div v-if="databaseInfo.oldExists" class="info-item warning">
              <span class="info-label">旧数据库状态:</span>
              <span class="info-value status-warning">⚠️ 旧数据库文件存在</span>
            </div>
            <div class="info-item">
              <span class="info-label">应用安装路径:</span>
              <span class="info-value">{{ databaseInfo.exePath }}</span>
            </div>
          </div>
          <div v-else class="info-empty">
            请点击"查看数据库"按钮获取详细信息
          </div>
        </div>
      </div>

      <!-- 数据库说明 -->
      <div class="info-section">
        <h3 class="section-title">📁 数据存储说明</h3>
        <div class="info-content">
          <div class="info-description">
            <h4>新版本改进:</h4>
            <ul>
              <li>数据库现在存储在应用安装目录的 <code>data</code> 文件夹中</li>
              <li>不再使用系统临时目录，便于数据管理和备份</li>
              <li>支持自动迁移旧数据库文件</li>
              <li>旧数据库文件会保留作为备份</li>
            </ul>

            <h4>位置信息:</h4>
            <ul>
              <li><strong>新位置:</strong> <code>应用安装目录/data/news_analyzer.db</code></li>
              <li><strong>旧位置:</strong> <code>系统临时目录/news-analyzer-mvp/news_analyzer.db</code></li>
            </ul>

            <h4>注意事项:</h4>
            <ul>
              <li>确保应用安装目录有写入权限</li>
              <li>备份时请包含整个 <code>data</code> 文件夹</li>
              <li>如需迁移数据库，可直接复制 <code>news_analyzer.db</code> 文件</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { LlmConfig, PromptTemplate, AllSettings } from '../types'

// 当前选中的标签页
const activeTab = ref('llm')

// LLM 配置列表
const llmConfigs = ref<LlmConfig[]>([])
const editingLlmConfig = ref<LlmConfig | null>(null)
const isAddingLlmConfig = ref(false)

// 提示词模板列表
const promptTemplates = ref<PromptTemplate[]>([])
const editingPromptTemplate = ref<PromptTemplate | null>(null)
const isAddingPromptTemplate = ref(false)
const previewingTemplate = ref<PromptTemplate | null>(null)

// 系统信息相关
const databaseInfo = ref<{
  currentPath: string
  currentExists: boolean
  oldPath: string
  oldExists: boolean
  exePath: string
} | null>(null)

// 新增/编辑表单数据
const llmForm = ref({
  name: '',
  apiKey: '',
  endpoint: '',
  modelId: '',
  temperature: 0.7,
  maxTokens: 2000,
  enabled: true
})

const promptForm = ref({
  name: '',
  template: '',
  isDefault: false
})


// LLM 配置管理方法
const showAddLlmConfig = () => {
  isAddingLlmConfig.value = true
  editingLlmConfig.value = null
  resetLlmForm()
}

const editLlmConfig = (config: LlmConfig) => {
  editingLlmConfig.value = config
  isAddingLlmConfig.value = false
  llmForm.value = {
    name: config.name,
    apiKey: config.api_key,
    endpoint: config.endpoint,
    modelId: config.model_id,
    temperature: config.temperature,
    maxTokens: config.max_tokens,
    enabled: config.enabled
  }
}

const resetLlmForm = () => {
  llmForm.value = {
    name: '',
    apiKey: '',
    endpoint: '',
    modelId: '',
    temperature: 0.7,
    maxTokens: 2000,
    enabled: true
  }
}

const cancelLlmEdit = () => {
  isAddingLlmConfig.value = false
  editingLlmConfig.value = null
  resetLlmForm()
}

const saveLlmConfig = async () => {
  try {
    if (isAddingLlmConfig.value) {
      await invoke('add_llm_config', {
        config: {
          name: llmForm.value.name,
          api_key: llmForm.value.apiKey,
          endpoint: llmForm.value.endpoint,
          model_id: llmForm.value.modelId,
          temperature: llmForm.value.temperature,
          max_tokens: llmForm.value.maxTokens,
          enabled: llmForm.value.enabled
        }
      })
    } else if (editingLlmConfig.value) {
      await invoke('update_llm_config', {
        id: editingLlmConfig.value.id,
        config: {
          name: llmForm.value.name,
          api_key: llmForm.value.apiKey,
          endpoint: llmForm.value.endpoint,
          model_id: llmForm.value.modelId,
          temperature: llmForm.value.temperature,
          max_tokens: llmForm.value.maxTokens,
          enabled: llmForm.value.enabled
        }
      })
    }
    await loadAllSettings()
    cancelLlmEdit()
  } catch (error) {
    console.error('保存 LLM 配置失败:', error)
    alert('保存失败: ' + error)
  }
}

const deleteLlmConfig = async (config: LlmConfig) => {
  if (confirm(`确定要删除配置 "${config.name}" 吗？`)) {
    try {
      await invoke('delete_llm_config', { id: config.id })
      await loadAllSettings()
    } catch (error) {
      console.error('删除 LLM 配置失败:', error)
      alert('删除失败: ' + error)
    }
  }
}

const toggleLlmConfig = async (config: LlmConfig) => {
  try {
    await invoke('toggle_llm_config', { id: config.id })
    await loadAllSettings()
  } catch (error) {
    console.error('切换 LLM 配置状态失败:', error)
    alert('操作失败: ' + error)
  }
}

const testLlmConfig = async (config: LlmConfig) => {
  try {
    const result = await invoke('test_llm_connection', {
      config: {
        name: config.name,
        api_key: config.api_key,
        endpoint: config.endpoint || null,
        model_id: config.model_id,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        enabled: config.enabled
      }
    })
    alert('连接测试成功: ' + result)
  } catch (error) {
    console.error('LLM 连接测试失败:', error)
    alert('连接测试失败: ' + error)
  }
}

// 提示词模板管理方法
const createDefaultTemplates = async () => {
  try {
    await invoke('create_default_prompt_templates')
    await loadAllSettings()
    alert('默认模板创建成功')
  } catch (error) {
    console.error('创建默认模板失败:', error)
    alert('创建失败: ' + error)
  }
}

const showAddPromptTemplate = () => {
  isAddingPromptTemplate.value = true
  editingPromptTemplate.value = null
  resetPromptForm()
}

const editPromptTemplate = (template: PromptTemplate) => {
  editingPromptTemplate.value = template
  isAddingPromptTemplate.value = false
  promptForm.value = {
    name: template.name,
    template: template.template,
    isDefault: template.is_default
  }
}

const resetPromptForm = () => {
  promptForm.value = {
    name: '',
    template: '',
    isDefault: false
  }
}

// 使用参考模板
const useReferenceTemplate = () => {
  promptForm.value.template = `分析以下文章内容，提取新闻信息：

{content}

行业类型：数据中心、算力、云计算、人工智能、大数据、跨境数据

新闻类型：融资投资、政策法规、市场动态、技术创新、财务报告、战略合作、会展信息、项目动态

重要提醒：
- 仅识别真正的新闻类内容
- 剔除技术分享、宣传推广类软文
- 剔除产品介绍、教程类内容
- 剔除广告营销、品牌宣传内容
- 只提取具有新闻价值的事件信息

返回JSON格式：
{
  "has_news": true,
  "news_list": [
    {
      "title": "新闻标题",
      "summary": "详细摘要",
      "industry_type": "行业类型",
      "news_type": "新闻类型",
      "confidence": 0.8
    }
  ],
  "analysis_summary": "分析完成"
}`
}

const cancelPromptEdit = () => {
  isAddingPromptTemplate.value = false
  editingPromptTemplate.value = null
  resetPromptForm()
}

const savePromptTemplate = async () => {
  console.log('开始保存提示词模板...', {
    isAdding: isAddingPromptTemplate.value,
    isEditing: !!editingPromptTemplate.value,
    formData: promptForm.value
  })
  
  try {
    if (isAddingPromptTemplate.value) {
      console.log('调用 add_prompt_template...')
      await invoke('add_prompt_template', {
        template: {
          name: promptForm.value.name,
          template: promptForm.value.template,
          is_default: promptForm.value.isDefault
        }
      })
      console.log('add_prompt_template 调用成功')
    } else if (editingPromptTemplate.value) {
      console.log('调用 update_prompt_template...')
      await invoke('update_prompt_template', {
        id: editingPromptTemplate.value.id,
        template: {
          name: promptForm.value.name,
          template: promptForm.value.template,
          is_default: promptForm.value.isDefault
        }
      })
      console.log('update_prompt_template 调用成功')
    }
    
    console.log('重新加载设置...')
    await loadAllSettings()
    console.log('设置重新加载完成')
    
    cancelPromptEdit()
    console.log('保存流程完成')
  } catch (error) {
    console.error('保存提示词模板失败:', error)
    alert('保存失败: ' + error)
  }
}

const deletePromptTemplate = async (template: PromptTemplate) => {
  if (confirm(`确定要删除模板 "${template.name}" 吗？`)) {
    try {
      await invoke('delete_prompt_template', { id: template.id })
      await loadAllSettings()
    } catch (error) {
      console.error('删除提示词模板失败:', error)
      alert('删除失败: ' + error)
    }
  }
}

const setDefaultTemplate = async (template: PromptTemplate) => {
  try {
    // 先将所有模板设为非默认
    for (const t of promptTemplates.value) {
      if (t.is_default) {
        await invoke('update_prompt_template', {
          id: t.id,
          template: {
            name: t.name,
            template: t.template,
            is_default: false
          }
        })
      }
    }
    
    // 设置新的默认模板
    await invoke('update_prompt_template', {
      id: template.id,
      template: {
        name: template.name,
        template: template.template,
        is_default: true
      }
    })
    
    await loadAllSettings()
    alert('已设为默认模板')
  } catch (error) {
    console.error('设置默认模板失败:', error)
    alert('设置失败: ' + error)
  }
}

const previewPromptTemplate = (template: PromptTemplate) => {
  previewingTemplate.value = template
}

// 加载所有设置
const loadAllSettings = async () => {
  try {
    const settings = await invoke<AllSettings>('load_all_settings')
    llmConfigs.value = settings.llm_configs
    promptTemplates.value = settings.prompt_templates || []
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

// 加载数据库信息
const loadDatabaseInfo = async () => {
  try {
    const info = await invoke<string>('get_database_info')
    // 解析后端返回的信息
    const lines = info.split('\n').filter(line => line.trim())
    const parsedInfo: any = {}

    lines.forEach(line => {
      const parts = line.split(':')
      if (parts.length >= 2) {
        const key = parts[0].trim()
        const value = parts.slice(1).join(':').trim()

        if (key.includes('当前数据库路径')) {
          parsedInfo.currentPath = value
        } else if (key.includes('数据库存在')) {
          parsedInfo.currentExists = value.toLowerCase().includes('true')
        } else if (key.includes('旧数据库路径')) {
          parsedInfo.oldPath = value
        } else if (key.includes('旧数据库存在')) {
          parsedInfo.oldExists = value.toLowerCase().includes('true')
        } else if (key.includes('应用可执行文件路径')) {
          parsedInfo.exePath = value
        }
      }
    })

    databaseInfo.value = parsedInfo
  } catch (error) {
    console.error('加载数据库信息失败:', error)
    alert('加载数据库信息失败: ' + error)
  }
}

// 显示数据库信息
const showDatabaseInfo = async () => {
  await loadDatabaseInfo()
}

onMounted(() => {
  loadAllSettings()
})
</script>

<style scoped>
.settings-page {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #f0f0f0;
  min-height: calc(100vh - 140px);
}

.page-header {
  margin-bottom: 24px;
  text-align: center;
  background: #f8f9fa;
  padding: 24px;
  border-radius: 12px;
  border: 1px solid #e9ecef;
}

.page-header h1 {
  font-size: 20px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 8px;
}

.page-header p {
  color: #6c757d;
  font-size: 14px;
  font-weight: 500;
}

/* 标签页样式 */
.tabs-container {
  margin-bottom: 0;
  background: white;
  border-radius: 12px 12px 0 0;
  overflow: hidden;
  border: 1px solid #e9ecef;
  border-bottom: none;
}

.tabs {
  display: flex;
  background: white;
  overflow: hidden;
}

.tab-button {
  flex: 1;
  padding: 12px 16px;
  border: none;
  background: #f8f9fa;
  color: #6c757d;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-button:hover {
  background: #e9ecef;
  color: #495057;
}

.tab-button.active {
  background: #667eea;
  color: white;
}

/* 标签页内容 */
.tab-content {
  background: white;
  border-radius: 0 0 12px 12px;
  border: 1px solid #e9ecef;
  border-top: none;
}

.tab-panel {
  padding: 24px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e9ecef;
}

.panel-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
}

/* 配置列表样式 */
.config-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: white;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.config-item:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.config-item.disabled {
  opacity: 0.6;
  background: #f8f9fa;
}

.config-info {
  flex: 1;
}

.config-name {
  font-size: 14px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.config-details {
  font-size: 12px;
  color: #6c757d;
  line-height: 1.4;
}

.status-badge {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

.status-badge.enabled {
  background: #28a745;
  color: white;
}

.status-badge.disabled {
  background: #dc3545;
  color: white;
}

.status-badge.default {
  background: #667eea;
  color: white;
}

.config-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.header-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.prompt-item.default {
  border-color: #667eea;
  background: #f8f9ff;
}

.prompt-modal {
  max-width: 800px;
}

.preview-modal {
  max-width: 900px;
}

.preview-content {
  max-height: 60vh;
  overflow-y: auto;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  padding: 16px;
}

.preview-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: #2c3e50;
}

.template-help {
  margin-top: 8px;
  padding: 12px;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
}

.template-help p {
  margin: 4px 0;
  font-size: 12px;
  color: #6c757d;
}

.template-help code {
  background: #e9ecef;
  padding: 2px 4px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: #495057;
}

/* 按钮样式 */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 11px;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover {
  background: #5a6fd8;
}

.btn-secondary {
  background: #f8f9fa;
  color: #495057;
  border: 1px solid #dee2e6;
}

.btn-secondary:hover {
  background: #e9ecef;
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-success:hover {
  background: #218838;
}

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-warning:hover {
  background: #e0a800;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover {
  background: #c82333;
}

/* 模态框样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e9ecef;
  background: #f8f9fa;
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  color: #6c757d;
  cursor: pointer;
  padding: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: #f8f9fa;
  color: #dc3545;
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 20px 24px;
  border-top: 1px solid #e9ecef;
  background: #f8f9fa;
}

/* 表单样式 */
.form-group {
  margin-bottom: 20px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 6px;
}

.form-group input,
.form-group select,
.form-group textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 13px;
  transition: all 0.2s ease;
  background: white;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

.form-group textarea {
  resize: vertical;
  min-height: 80px;
  font-family: inherit;
}

.form-group input[type="checkbox"] {
  width: auto;
  margin-right: 6px;
  accent-color: #667eea;
}

.form-group input[type="range"] {
  padding: 0;
  height: 4px;
  background: #e9ecef;
  border-radius: 2px;
  outline: none;
}

.form-group input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  background: #667eea;
  border: 2px solid white;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .settings-page {
    padding: 20px;
    border-radius: 12px;
  }
  
  .page-header {
    padding: 24px;
    margin-bottom: 24px;
  }
  
  .page-header h1 {
    font-size: 24px;
  }
  
  .tab-button {
    padding: 16px 12px;
    font-size: 14px;
  }
  
  .tab-panel {
    padding: 20px;
  }
  
  .panel-header {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }
  
  .config-item {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }
  
  .config-actions {
    justify-content: center;
  }
  
  .modal {
    width: 95%;
    margin: 20px;
  }
  
  .modal-header,
  .modal-body,
  .modal-footer {
    padding: 20px;
  }
}

/* 系统信息页面样式 */
.info-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e9ecef;
}

.info-content {
  background: #fff;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 20px;
}

.info-display {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-item.warning {
  background: #fff3cd;
  padding: 12px;
  border-radius: 6px;
  border-left: 4px solid #ffc107;
}

.info-label {
  font-size: 14px;
  font-weight: 500;
  color: #6c757d;
  min-width: 120px;
}

.info-value {
  font-size: 14px;
  color: #2c3e50;
  font-family: 'Courier New', monospace;
  word-break: break-all;
  max-width: 70%;
  text-align: right;
}

.status-ok {
  color: #28a745;
  font-weight: 600;
}

.status-error {
  color: #dc3545;
  font-weight: 600;
}

.status-warning {
  color: #ffc107;
  font-weight: 600;
}

.info-empty {
  text-align: center;
  color: #6c757d;
  padding: 40px 20px;
  font-style: italic;
}

.info-description {
  color: #495057;
  line-height: 1.6;
}

.info-description h4 {
  font-size: 15px;
  font-weight: 600;
  color: #2c3e50;
  margin: 20px 0 10px 0;
}

.info-description h4:first-child {
  margin-top: 0;
}

.info-description ul {
  margin: 0 0 16px 20px;
  padding: 0;
}

.info-description li {
  margin-bottom: 8px;
  line-height: 1.5;
}

.info-description code {
  background: #f1f3f4;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #e91e63;
}

/* 系统信息响应式设计 */
@media (max-width: 768px) {
  .info-display {
    gap: 8px;
  }

  .info-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 12px 0;
  }

  .info-label {
    min-width: auto;
    font-weight: 600;
  }

  .info-value {
    max-width: 100%;
    text-align: left;
    padding-left: 12px;
    border-left: 3px solid #667eea;
  }

  .info-description {
    font-size: 14px;
  }

  .info-description h4 {
    font-size: 14px;
  }
}
</style>
