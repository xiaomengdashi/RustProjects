<template>
  <el-container style="height: 100vh; background-color: #f5f5f5;">
    <!-- Header -->
    <el-header style="background-color: #409eff; color: white;">
      <h1 style="text-align: center; margin: 0; font-size: 24px; font-weight: bold;">Real-time Chat</h1>
    </el-header>

    <!-- Main Content -->
    <el-main style="padding: 20px;">
      <!-- Input and Send Button -->
      <el-row gutter="20" type="flex" justify="center">
        <el-col :span="16">
          <el-input
            v-model="message"
            placeholder="Enter your message"
            clearable
            @keyup.enter="sendMessage"
            style="border-radius: 25px; padding: 10px; background-color: #fff; box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);"
          />
        </el-col>
        <el-col :span="6">
          <el-button
            type="primary"
            @click="sendMessage"
            style="width: 100%; border-radius: 25px; height: 100%; font-size: 16px;"
          >
            Send
          </el-button>
        </el-col>
      </el-row>

      <!-- Divider -->
      <el-divider></el-divider>

      <!-- Messages Display -->
      <el-row gutter="20" type="flex" justify="center" style="max-height: 60vh; overflow-y: auto;">
        <el-col :span="16">
          <div v-for="(msg, index) in messages" :key="index" class="message-item">
            <el-card
              shadow="always"
              style="margin-bottom: 10px; border-radius: 15px; background-color: #ffffff; padding: 15px; font-size: 14px;"
            >
              <div>{{ msg }}</div>
            </el-card>
          </div>
        </el-col>
      </el-row>
    </el-main>

    <!-- Footer -->
    <el-footer style="background-color: #409eff; color: white; padding: 10px 0; text-align: center;">
      <p style="margin: 0; font-size: 14px;">Chat App using WebSocket & Vue 3</p>
    </el-footer>
  </el-container>
</template>

<script>
import { ref } from "vue";
import { ElContainer, ElHeader, ElMain, ElFooter, ElInput, ElButton, ElRow, ElCol, ElDivider, ElCard } from "element-plus";

export default {
  components: {
    ElContainer,
    ElHeader,
    ElMain,
    ElFooter,
    ElInput,
    ElButton,
    ElRow,
    ElCol,
    ElDivider,
    ElCard,
  },
  setup() {
    const socket = ref(null);  // WebSocket实例
    const message = ref("");   // 用户输入的消息
    const messages = ref([]);  // 存储消息的数组

    // 连接WebSocket并处理消息
    const connectWebSocket = () => {
      socket.value = new WebSocket("ws://localhost:8080/ws");

      socket.value.onopen = () => {
        console.log("WebSocket connected!");
      };

      // 接收消息时的处理逻辑
      socket.value.onmessage = (event) => {
        // 假设服务器返回的是纯文本消息，或者是一个JSON对象
        const data = event.data;
        console.log("Received message:", data);

        // 将接收到的消息添加到messages数组
        messages.value.push(data);
      };

      socket.value.onclose = () => {
        console.log("WebSocket closed");
      };

      socket.value.onerror = (error) => {
        console.error("WebSocket Error: ", error);
      };
    };

    // 发送消息到服务器
    const sendMessage = () => {
      if (message.value.trim() !== "" && socket.value) {
        socket.value.send(message.value);
        messages.value.push(`You: ${message.value}`); // 本地显示自己的消息
        message.value = "";  // 清空输入框
      }
    };

    // 初始连接WebSocket
    connectWebSocket();

    return {
      message,
      messages,
      sendMessage,
    };
  },
};
</script>

<style scoped>
/* Global styles */
.el-input, .el-button {
  transition: all 0.3s ease;
}

.el-input:focus, .el-button:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

/* Message display styling */
.message-item {
  display: flex;
  justify-content: center;
  margin-bottom: 10px;
}

.el-card {
  border-radius: 15px;
  background-color: #ffffff;
  padding: 15px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

/* Header and Footer styles */
.el-header {
  background-color: #409eff;
  color: white;
  padding: 10px 0;
}

.el-footer {
  background-color: #409eff;
  color: white;
  text-align: center;
  padding: 10px;
}

.el-footer p {
  margin: 0;
  font-size: 14px;
}

/* Input and Button Styles */
.el-input {
  border-radius: 25px;
  padding: 10px;
  background-color: #fff;
}

.el-button {
  border-radius: 25px;
  height: 100%;
  font-size: 16px;
}
</style>
