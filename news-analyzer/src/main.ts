import { createApp } from 'vue'
import App from './App.vue'

const app = createApp(App)

// 全局错误处理
app.config.errorHandler = (err, instance, info) => {
  console.error('Vue错误:', err)
  console.error('组件实例:', instance)
  console.error('错误信息:', info)
  
  // 防止应用崩溃，显示用户友好的错误信息
  if (err instanceof Error) {
    console.error('应用遇到错误:', err.message)
  } else {
    console.error('应用遇到未知错误:', err)
  }
}

// 捕获未处理的Promise错误
window.addEventListener('unhandledrejection', (event) => {
  console.error('未处理的Promise错误:', event.reason)
  event.preventDefault() // 防止默认的错误处理
})

// 捕获全局错误
window.addEventListener('error', (event) => {
  console.error('全局错误:', event.error)
  event.preventDefault()
})

app.mount('#app')
