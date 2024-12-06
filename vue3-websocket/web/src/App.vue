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
    const socket = ref(null);
    const message = ref("");
    const messages = ref([]);

    const connectWebSocket = () => {
      socket.value = new WebSocket("ws://localhost:8080/ws");
      
      socket.value.onopen = () => {
        console.log("WebSocket connected!");
      };
      
      socket.value.onmessage = (event) => {
        messages.value.push(event.data);
      };
      
      socket.value.onclose = () => {
        console.log("WebSocket closed");
      };
      
      socket.value.onerror = (error) => {
        console.error("WebSocket Error: ", error);
      };
    };

    const sendMessage = () => {
      if (message.value.trim() !== "" && socket.value) {
        socket.value.send(message.value);
        message.value = ""; // Clear input after sending
      }
    };

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
