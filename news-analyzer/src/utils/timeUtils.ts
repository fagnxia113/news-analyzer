/**
 * 统一的时间处理工具函数
 * 用于处理UTC和北京时间之间的转换，以及统一的时间格式化
 */

// 北京时区偏移量（UTC+8，更清晰的定义方式）
const BEIJING_TIMEZONE_OFFSET_MS = 8 * 60 * 60 * 1000

// 格式化选项类型定义
type FormatType = 'datetime' | 'date' | 'time' | 'relative'

/**
 * 将UTC时间戳转换为北京时间Date对象
 * @param utcTimestamp UTC时间戳（秒）
 * @returns 北京时间的Date对象
 */
export function utcToBeijingTime(utcTimestamp: number): Date {
  // 将秒转换为毫秒
  const utcTimeMs = utcTimestamp * 1000
  // 加上北京时区偏移量
  const beijingTimeMs = utcTimeMs + BEIJING_TIMEZONE_OFFSET_MS
  return new Date(beijingTimeMs)
}

/**
 * 将北京时间Date对象转换为UTC时间戳
 * @param beijingDate 北京时间的Date对象
 * @returns UTC时间戳（秒）
 */
export function beijingToUtcTimestamp(beijingDate: Date): number {
  // 获取北京时间的毫秒时间戳
  const beijingTimeMs = beijingDate.getTime()
  // 减去北京时区偏移量得到UTC时间
  const utcTimeMs = beijingTimeMs - BEIJING_TIMEZONE_OFFSET_MS
  // 转换为秒并返回
  return Math.floor(utcTimeMs / 1000)
}

/**
 * 获取当前北京时间的Date对象
 * @returns 当前北京时间的Date对象
 */
export function getCurrentBeijingTime(): Date {
  const now = new Date()
  const utcTimeMs = now.getTime()
  const beijingTimeMs = utcTimeMs + BEIJING_TIMEZONE_OFFSET_MS
  return new Date(beijingTimeMs)
}

/**
 * 获取当前UTC时间戳（秒）
 * @returns 当前UTC时间戳（秒）
 */
export function getCurrentUtcTimestamp(): number {
  return Math.floor(Date.now() / 1000)
}

/**
 * 格式化UTC时间戳为北京时间字符串
 * @param utcTimestamp UTC时间戳（秒）
 * @param format 格式类型，默认为'datetime'
 * @returns 格式化后的北京时间字符串
 */
export function formatUtcTimestamp(utcTimestamp: number, format: FormatType = 'datetime'): string {
  const beijingDate = utcToBeijingTime(utcTimestamp)
  return formatDateToString(beijingDate, format)
}

/**
 * 格式化ISO时间字符串为北京时间字符串
 * @param isoString ISO时间字符串
 * @param format 格式类型，默认为'datetime'
 * @returns 格式化后的北京时间字符串
 */
export function formatIsoString(isoString: string, format: FormatType = 'datetime'): string {
  try {
    const date = new Date(isoString)
    return formatDateToString(date, format)
  } catch (error) {
    console.error('格式化ISO时间字符串失败:', error)
    return isoString
  }
}

/**
 * 通用格式化函数 - 消除重复代码
 * @param date Date对象
 * @param format 格式类型
 * @returns 格式化后的字符串
 */
function formatDateToString(date: Date, format: FormatType): string {
  try {
    switch (format) {
      case 'datetime':
        return date.toLocaleString('zh-CN', {
          year: 'numeric',
          month: '2-digit',
          day: '2-digit',
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
          hour12: false
        })
      case 'date':
        return date.toLocaleDateString('zh-CN', {
          year: 'numeric',
          month: '2-digit',
          day: '2-digit'
        })
      case 'time':
        return date.toLocaleTimeString('zh-CN', {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
          hour12: false
        })
      case 'relative':
        return formatRelativeTime(date)
      default:
        return date.toLocaleString('zh-CN')
    }
  } catch (error) {
    console.error('格式化Date对象失败:', error)
    return date.toString()
  }
}

/**
 * 格式化Date对象为北京时间字符串
 * @param date Date对象
 * @param format 格式类型，默认为'datetime'
 * @returns 格式化后的北京时间字符串
 */
export function formatDate(date: Date, format: FormatType = 'datetime'): string {
  return formatDateToString(date, format)
}

/**
 * 格式化相对时间（如"5分钟前"、"2小时前"等）
 * @param date Date对象
 * @returns 相对时间字符串
 */
function formatRelativeTime(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  // 计算时间差
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) {
    return `${days}天前`
  } else if (hours > 0) {
    return `${hours}小时前`
  } else if (minutes > 0) {
    return `${minutes}分钟前`
  } else {
    return '刚刚'
  }
}

/**
 * 获取北京时间的日期范围（用于筛选）
 * @param range 日期范围类型
 * @returns 包含开始和结束UTC时间戳的对象
 */
export function getBeijingDateRange(range: 'today' | 'yesterday' | 'week' | 'month' = 'today'): { start: number; end: number } {
  const beijingNow = getCurrentBeijingTime()
  
  let startDate: Date
  let endDate: Date
  
  switch (range) {
    case 'today':
      // 北京时间的今天 00:00:00 到 23:59:59
      startDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate(), 0, 0, 0)
      endDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate(), 23, 59, 59)
      break
    case 'yesterday':
      // 北京时间的昨天 00:00:00 到 23:59:59
      startDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate() - 1, 0, 0, 0)
      endDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate() - 1, 23, 59, 59)
      break
    case 'week':
      // 最近7天（从今天往前算）
      startDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate() - 6, 0, 0, 0)
      endDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate(), 23, 59, 59)
      break
    case 'month':
      // 最近30天（从今天往前算）
      startDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate() - 29, 0, 0, 0)
      endDate = new Date(beijingNow.getFullYear(), beijingNow.getMonth(), beijingNow.getDate(), 23, 59, 59)
      break
    default:
      throw new Error(`不支持的日期范围: ${range}`)
  }
  
  // 转换为UTC时间戳
  return {
    start: beijingToUtcTimestamp(startDate),
    end: beijingToUtcTimestamp(endDate)
  }
}

/**
 * 检查UTC时间戳是否在北京时间的指定日期范围内
 * @param utcTimestamp UTC时间戳（秒）
 * @param range 日期范围类型
 * @returns 是否在范围内
 */
export function isUtcTimestampInBeijingRange(utcTimestamp: number, range: 'today' | 'yesterday' | 'week' | 'month' = 'today'): boolean {
  const { start, end } = getBeijingDateRange(range)
  return utcTimestamp >= start && utcTimestamp <= end
}

/**
 * 将ISO时间字符串转换为UTC时间戳
 * @param isoString ISO时间字符串
 * @returns UTC时间戳（秒）
 */
export function isoStringToUtcTimestamp(isoString: string): number {
  try {
    const date = new Date(isoString)
    return Math.floor(date.getTime() / 1000)
  } catch (error) {
    console.error('转换ISO时间字符串为UTC时间戳失败:', error)
    return 0
  }
}

/**
 * 将UTC时间戳转换为ISO时间字符串
 * @param utcTimestamp UTC时间戳（秒）
 * @returns ISO时间字符串
 */
export function utcTimestampToIsoString(utcTimestamp: number): string {
  try {
    const date = new Date(utcTimestamp * 1000)
    return date.toISOString()
  } catch (error) {
    console.error('转换UTC时间戳为ISO时间字符串失败:', error)
    return new Date().toISOString()
  }
}