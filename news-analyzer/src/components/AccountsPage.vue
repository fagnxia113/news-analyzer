<template>
  <div class="accounts-page">
    <div class="accounts-header">
      <div class="accounts-count">共 {{ accounts.length }} 个账号</div>
      <div v-if="statusMessage" class="status-message">{{ statusMessage }}</div>
      <button class="btn btn-primary" @click="openLoginModal">
        <PlusIcon />
        添加读书账号
      </button>
    </div>
    
    <div class="accounts-table">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>用户名</th>
            <th>状态</th>
            <th>更新时间</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="account in accounts" :key="account.id">
            <td class="account-id">{{ account.vid }}</td>
            <td class="account-name">{{ account.name }}</td>
            <td>
              <span class="status-chip" :class="statusClass(account.status)">
                {{ statusText(account.status) }}
              </span>
              <span v-if="account.is_banned" class="status-chip status-banned" title="账号被封禁">
                小黑屋
              </span>
            </td>
            <td class="update-time">{{ formatTime(account.updated_at) }}</td>
            <td class="actions">
              <select 
                class="status-dropdown" 
                :value="account.status"
                @change="updateAccountStatus(account.id, $event)"
              >
                <option value="1">启用</option>
                <option value="2">禁用</option>
                <option value="0">失效</option>
              </select>
              <button class="btn btn-danger btn-small" @click="deleteAccount(account.id)">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 登录对话框 -->
    <div v-if="showLoginModal" class="modal-overlay" @click="closeLoginModal">
      <div class="modal" @click.stop>
        <div class="modal-header">
          <div class="modal-title">添加读书账号</div>
          <button class="modal-close" @click="closeLoginModal">×</button>
        </div>
        <div class="modal-body">
          <div class="login-content">
            <div v-if="loginData && loginData.uuid" class="qr-container">
              <div class="qr-wrapper">
                <!-- 错误遮罩 -->
                <div v-if="loginResult?.message" class="error-overlay">
                  <div class="error-message">{{ loginResult.message }}</div>
                </div>
                <canvas ref="qrCanvas" width="150" height="150"></canvas>
              </div>
              <div class="login-text">
                微信扫码登录
                <span v-if="!loginResult?.message && countdown > 0" class="countdown">({countdown}s)</span>
              </div>
            </div>
            <div v-else class="loading">
              <div class="spinner"></div>
              二维码加载中
              <!-- 调试信息 -->
              <div style="font-size: 10px; color: #999; margin-top: 10px;">
                调试: loginData={{ loginData ? '有数据' : '无数据' }}
                <br>
                调试: loginData.uuid={{ loginData?.uuid || 'null' }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, h, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import QRCode from 'qrcode'
import type { WeChatAccount, LoginQRCode, LoginResult } from '../types'

// PlusIcon 组件
const PlusIcon = () => h('svg', {
  width: '16',
  height: '16',
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2'
}, [
  h('line', { x1: '12', y1: '5', x2: '12', y2: '19' }),
  h('line', { x1: '5', y1: '12', x2: '19', y2: '12' })
])

const accounts = ref<WeChatAccount[]>([])
const showLoginModal = ref(false)
const loginData = ref<LoginQRCode | null>(null)
const loginResult = ref<LoginResult | null>(null)
const countdown = ref(0)
const statusMessage = ref('')
const qrCanvas = ref<HTMLCanvasElement | null>(null)

let pollingTimer: NodeJS.Timeout | null = null
let countdownTimer: NodeJS.Timeout | null = null

// 使用真正的二维码库生成二维码
const generateQRCode = async (text: string, canvas: HTMLCanvasElement) => {
  console.log('generateQRCode 被调用，text:', text, 'canvas:', canvas)
  try {
    await QRCode.toCanvas(canvas, text, {
      width: 150,
      margin: 1,
      color: {
        dark: '#000000',
        light: '#ffffff'
      }
    })
    console.log('二维码生成成功')
  } catch (error) {
    console.error('生成二维码失败:', error)
    // 如果生成失败，显示错误信息
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.fillStyle = '#ffffff'
      ctx.fillRect(0, 0, 150, 150)
      ctx.fillStyle = '#dc3545'
      ctx.font = '14px Arial'
      ctx.textAlign = 'center'
      ctx.fillText('二维码生成失败', 75, 75)
    }
  }
}

onMounted(() => {
  loadAccounts()
})

onUnmounted(() => {
  stopPolling()
})

// 监听loginData变化，生成二维码
watch(loginData, async (newData) => {
  if (newData) {
    // 等待DOM更新
    await nextTick()
    // 等待一小段时间确保canvas元素完全准备好
    setTimeout(async () => {
      if (qrCanvas.value) {
        await generateQRCode(newData.scan_url, qrCanvas.value)
      }
    }, 100)
  }
}, { immediate: true })

const loadAccounts = async () => {
  try {
    accounts.value = await invoke<WeChatAccount[]>('get_all_accounts')
  } catch (error) {
    console.error('加载账号失败:', error)
  }
}

const openLoginModal = async () => {
  console.log('openLoginModal 被调用')
  showLoginModal.value = true
  console.log('showLoginModal 设置为:', showLoginModal.value)
  // 等待DOM更新后再开始登录
  await nextTick()
  console.log('DOM 更新完成，开始调用 startLogin')
  await startLogin()
}

const closeLoginModal = () => {
  console.log('closeLoginModal 被调用')
  showLoginModal.value = false
  stopPolling()
  loginData.value = null
  loginResult.value = null
  console.log('登录模态框已关闭，数据已清理')
}

const startLogin = async () => {
  try {
    console.log('startLogin 函数开始执行')
    console.log('调用 get_login_qrcode 命令...')
    loginData.value = await invoke<LoginQRCode>('get_login_qrcode')
    console.log('获取到登录二维码:', loginData.value)
    console.log('loginData.value.uuid:', loginData.value?.uuid)
    console.log('loginData.value.scan_url:', loginData.value?.scan_url)
    
    countdown.value = 60
    startPolling()
    startCountdown()
    
    // 等待DOM更新，多次尝试确保canvas元素准备好
    await nextTick()
    await new Promise(resolve => setTimeout(resolve, 100))
    await nextTick()
    
    // 尝试多次生成二维码
    for (let i = 0; i < 5; i++) {
      console.log(`第${i + 1}次尝试生成二维码, qrCanvas.value:`, qrCanvas.value, 'loginData.value:', loginData.value)
      if (qrCanvas.value && loginData.value) {
        console.log('开始生成二维码，URL:', loginData.value.scan_url)
        try {
          await generateQRCode(loginData.value.scan_url, qrCanvas.value)
          console.log('二维码生成完成')
          break
        } catch (error) {
          console.error(`第${i + 1}次生成二维码失败:`, error)
          if (i === 4) throw error
        }
      } else {
        console.error(`第${i + 1}次无法生成二维码 - canvas或数据为空`)
        console.error('qrCanvas.value:', qrCanvas.value)
        console.error('loginData.value:', loginData.value)
        if (i < 4) {
          await new Promise(resolve => setTimeout(resolve, 200))
        }
      }
    }
  } catch (error) {
    console.error('获取二维码失败:', error)
    statusMessage.value = '获取二维码失败'
  }
}

const startPolling = () => {
  if (!loginData.value) return
  
  pollingTimer = setInterval(async () => {
    if (!loginData.value || !pollingTimer) return
    
    try {
      const result = await invoke<LoginResult>('check_login_status', { 
        uuid: loginData.value.uuid 
      })
      
      loginResult.value = result
      
      // 登录成功
      if (result && result.vid && result.token && result.username) {
        await saveAccount(result)
      }
      // 登录失败（有错误信息）
      else if (result && result.message) {
        // 错误信息已经在UI中显示，继续轮询
      }
      // 继续等待
      else {
        // 继续轮询
      }
    } catch (error) {
      console.error('检查登录状态失败:', error)
    }
  }, 2000)
}

const saveAccount = async (result: LoginResult) => {
  if (!result.vid || !result.token || !result.username) return
  
  try {
    await invoke('save_wechat_account', { loginResult: result })
    statusMessage.value = '添加成功'
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
    
    closeLoginModal()
    await loadAccounts()
  } catch (error) {
    console.error('保存账号失败:', error)
    loginResult.value = { message: '保存账号失败' }
  }
}

const stopPolling = () => {
  if (pollingTimer) {
    clearInterval(pollingTimer)
    pollingTimer = null
  }
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

const startCountdown = () => {
  countdown.value = 60
  countdownTimer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      stopPolling()
      loginResult.value = { message: '二维码已过期，请重新获取' }
    }
  }, 1000)
}

const statusClass = (status: number) => {
  const classes = {
    1: 'status-enabled',
    2: 'status-disabled', 
    0: 'status-invalid'
  }
  return classes[status as keyof typeof classes] || 'status-invalid'
}

const statusText = (status: number) => {
  const texts = {
    1: '启用',
    2: '禁用',
    0: '失效'
  }
  return texts[status as keyof typeof texts] || '未知'
}

const formatTime = (time: string) => {
  return new Date(time).toLocaleString('zh-CN')
}

const updateAccountStatus = async (accountId: string, event: Event) => {
  const target = event.target as HTMLSelectElement
  const newStatus = parseInt(target.value)
  const oldStatus = accounts.value.find(a => a.id === accountId)?.status
  
  try {
    await invoke('update_account_status', { accountId, status: newStatus })
    await loadAccounts()
    
    const statusTextMap = { 1: '启用', 2: '禁用', 0: '失效' }
    statusMessage.value = `账号状态已更新为${statusTextMap[newStatus as keyof typeof statusTextMap]}`
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
  } catch (error) {
    console.error('更新账号状态失败:', error)
    target.value = String(oldStatus)
    
    statusMessage.value = '状态更新失败'
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
  }
}

const deleteAccount = async (accountId: string) => {
  if (confirm('确定要删除此账号吗？')) {
    try {
      await invoke('delete_account', { accountId })
      await loadAccounts()
      
      statusMessage.value = '账号删除成功'
      setTimeout(() => {
        statusMessage.value = ''
      }, 3000)
    } catch (error) {
      console.error('删除账号失败:', error)
      statusMessage.value = '账号删除失败'
      setTimeout(() => {
        statusMessage.value = ''
      }, 3000)
    }
  }
}

</script>

<style scoped>
.accounts-page {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #f0f0f0;
  min-height: calc(100vh - 140px);
}

.accounts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  background: #f8f9fa;
  padding: 16px 20px;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.accounts-count {
  font-size: 14px;
  font-weight: 600;
  color: #2c3e50;
  display: flex;
  align-items: center;
  gap: 6px;
}

.accounts-count::before {
  content: '👥';
  font-size: 16px;
}

.status-message {
  font-size: 12px;
  color: #28a745;
  background: #d4edda;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #c3e6cb;
  font-weight: 500;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  display: inline-flex;
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

.btn-small {
  padding: 4px 8px;
  font-size: 11px;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover {
  background: #c82333;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #5a6268;
}

.accounts-table {
  background: white;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid #e9ecef;
}

.accounts-table table {
  width: 100%;
  border-collapse: collapse;
}

.accounts-table th {
  background: #f8f9fa;
  padding: 12px 16px;
  text-align: left;
  font-weight: 600;
  color: #2c3e50;
  border-bottom: 1px solid #e9ecef;
  font-size: 12px;
}

.accounts-table td {
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  vertical-align: middle;
}

.accounts-table tr {
  transition: all 0.2s ease;
}

.accounts-table tr:hover {
  background: #f8f9fa;
}

.account-id {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
  font-size: 11px;
  color: #6c757d;
  background: #f8f9fa;
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-block;
}

.account-name {
  font-weight: 600;
  color: #2c3e50;
  font-size: 13px;
}

.status-chip {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  margin-right: 4px;
}

.status-enabled {
  background: #28a745;
  color: white;
}

.status-disabled {
  background: #ffc107;
  color: #212529;
}

.status-invalid {
  background: #dc3545;
  color: white;
}

.status-banned {
  background: #6f42c1;
  color: white;
  margin-left: 4px;
}

.update-time {
  color: #6c757d;
  font-size: 11px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
}

.actions {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}

.status-dropdown {
  padding: 4px 8px;
  border: 1px solid #e9ecef;
  border-radius: 4px;
  font-size: 11px;
  background: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.status-dropdown:hover {
  border-color: #667eea;
}

.status-dropdown:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
}

/* 登录对话框 */
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
  max-width: 400px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid #e9ecef;
  background: #f8f9fa;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #6c757d;
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

.login-content {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.qr-container {
  text-align: center;
}

.qr-wrapper {
  position: relative;
  display: inline-block;
  margin-bottom: 16px;
}

.error-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 8px;
}

.error-message {
  font-size: 18px;
  color: #dc3545;
  font-weight: 600;
}

.login-text {
  font-size: 14px;
  color: #495057;
  font-weight: 500;
}

.countdown {
  color: #dc3545;
  margin-left: 4px;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: #6c757d;
  font-size: 14px;
  font-weight: 500;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid #e9ecef;
  border-top: 2px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .accounts-page {
    padding: 16px;
  }
  
  .accounts-header {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }
  
  .accounts-table {
    overflow-x: auto;
  }
  
  .actions {
    flex-direction: column;
    gap: 6px;
    align-items: stretch;
  }
  
  .modal {
    width: 95%;
    margin: 20px;
  }
}
</style>
