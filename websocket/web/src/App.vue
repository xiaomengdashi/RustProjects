<template>
  <div class="chat-container">
    <el-card class="chat-card">
      <template #header>
        <div class="card-header">
          <span>WebSocket 聊天室</span>
          <div class="header-controls">
            <el-button size="small" type="warning" @click="clearMessages" :disabled="messages.length === 0">
              清空日志
            </el-button>
            <el-tag :type="connectionStatus === 'connected' ? 'success' : 'danger'">
              {{ connectionStatus === 'connected' ? '已连接' : '未连接' }}
            </el-tag>
          </div>
        </div>
      </template>

      <div class="chat-messages" ref="messageContainer">
        <div v-for="(msg, index) in messages" :key="index" class="message-wrapper"
          :class="{ 'message-sent': msg.type === 'sent' }">
          <el-tag size="small" class="message-type">
            {{ msg.type === 'received' ? '收到' : '发送' }}
          </el-tag>
          <div class="message-bubble">
            <span class="message-content">{{ msg.content }}</span>
          </div>
        </div>
      </div>
    </el-card>
    <div class="input-area">
      <el-input v-model="inputMessage" placeholder="请输入消息" :disabled="connectionStatus !== 'connected'"
        @keyup.enter="sendMessage">
        <template #append>
          <el-button @click="sendMessage" :disabled="connectionStatus !== 'connected'">
            发送
          </el-button>
        </template>
      </el-input>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'

const ws = ref(null)
const messages = ref([])
const inputMessage = ref('')
const connectionStatus = ref('disconnected')
const messageContainer = ref(null)

const connectWebSocket = () => {
  ws.value = new WebSocket('ws://localhost:8080/ws')

  ws.value.onopen = () => {
    connectionStatus.value = 'connected'
    ElMessage.success('WebSocket 连接成功')
  }

  ws.value.onmessage = (event) => {
    messages.value.push({
      type: 'received',
      content: event.data
    })
    scrollToBottom()
  }

  ws.value.onclose = () => {
    connectionStatus.value = 'disconnected'
    ElMessage.warning('WebSocket 连接已断开')
    // 尝试重新连接
    setTimeout(connectWebSocket, 3000)
  }

  ws.value.onerror = (error) => {
    connectionStatus.value = 'disconnected'
    ElMessage.error('WebSocket 连接错误')
    console.error('WebSocket error:', error)
  }
}

const sendMessage = () => {
  if (!inputMessage.value.trim()) return

  if (ws.value && ws.value.readyState === WebSocket.OPEN) {
    ws.value.send(inputMessage.value)
    messages.value.push({
      type: 'sent',
      content: inputMessage.value
    })
    inputMessage.value = ''
    scrollToBottom()
  }
}

const scrollToBottom = async () => {
  await nextTick()
  if (messageContainer.value) {
    messageContainer.value.scrollTop = messageContainer.value.scrollHeight
  }
}

const clearMessages = () => {
  messages.value = []
  ElMessage.success('消息已清空')
}

onMounted(() => {
  connectWebSocket()
})

onUnmounted(() => {
  if (ws.value) {
    ws.value.close()
  }
})
</script>

<style scoped>
.chat-container {
  max-width: 800px;
  margin: 20px auto;
}

.chat-card {
  height: 600px;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 15px 0;
  margin-bottom: 10px;
  height: calc(100% - 140px);
  background-color: #f7f7f7;
}

.message-wrapper {
  display: flex;
  align-items: flex-start;
  margin-bottom: 15px;
  padding: 0 10px;
}

.message-sent {
  flex-direction: row-reverse;
}

.message-bubble {
  max-width: 70%;
  padding: 10px 15px;
  border-radius: 15px;
  background-color: #f0f0f0;
  word-wrap: break-word;
  margin: 0 10px;
}

.message-sent .message-bubble {
  background-color: #95ec69;
}

.message-content {
  font-size: 14px;
  line-height: 1.4;
}

.message-type {
  margin-top: 5px;
}

.input-area {
  padding: 10px 0;
}

.header-controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.chat-card {
  height: 600px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>