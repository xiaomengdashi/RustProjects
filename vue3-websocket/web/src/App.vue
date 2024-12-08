<template>
  <div>
    <button @click="toggleConnect">
      {{ isConnected? 'Disconnect' : 'Connect' }}
    </button>
    <span>Status:</span>
    <span :class="statusClass">{{ statusText }}</span>
    <div id="log" class="log" ref="logRef" v-html="log"></div> <!-- 使用 v-html 绑定 log.value -->
    <form @submit.prevent="sendMessage" id="chatform">
      <input type="text" id="text" v-model="message" autocomplete="off" />
      <input type="submit" id="send" />
      <button @click="clearLog">Clear Log</button> <!-- 添加清除日志按钮 -->
    </form>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted, nextTick } from 'vue';

export default {
  setup() {
    const isConnected = ref(false);
    const statusText = ref('disconnected');
    const statusClass = ref('disconnected');
    const message = ref('');
    const log = ref('');
    const logRef = ref(null); // 添加一个 ref 来引用 log 元素

    const socket = ref(null);

    const connect = () => {
      disconnect();

      const { location } = window;
      const proto = location.protocol.startsWith('https')? 'wss' : 'ws';
      const wsUri = `${proto}://127.0.0.1:8080/ws`;

      logMethod('Sending:'+ 'Connecting...')
      socket.value = new WebSocket(wsUri);

      socket.value.onopen = () => {
        logMethod('Sending:'+ 'Connected')
        updateConnectionStatus();
      };

      socket.value.onmessage = ev => {
        logMethod('Received: ' + ev.data, 'message')
      };

      socket.value.onclose = () => {
        logMethod('Sending: ' + 'Disconnected')
        socket.value = null;
        updateConnectionStatus();
      };
    };

    const disconnect = () => {
      if (socket.value) {
        logMethod('Sending:'+ 'Disconnecting...')
        socket.value.close();
        socket.value = null;
        updateConnectionStatus();
      }
    };

    const updateConnectionStatus = () => {
      if (socket.value) {
        statusText.value = 'connected';
        statusClass.value = 'connected';
        isConnected.value = true;
      } else {
        statusText.value = 'disconnected';
        statusClass.value = 'disconnected';
        isConnected.value = false;
      }
    };

    const sendMessage = () => {
      // if (message.value.trim()!== '') {
        log.value += `<p class="msg msg--message">Sending: ${message.value}</p>`;
        socket.value.send(message.value);
        message.value = '';
        scrollToBottom();
      // }
    };

    const toggleConnect = () => {
      if (isConnected.value) {
        disconnect();
      } else {
        connect();
      }
    };

    const scrollToBottom = () => {
      if (logRef.value) {
        logRef.value.scrollTop = logRef.value.scrollHeight;
      }
    };


    const logMethod = (msg, type = 'status') => {
      log.value += `<p class="msg msg--${type}">${msg}</p>`;
      nextTick(() => {
        scrollToBottom();
      });
    };

    const clearLog = () => {
      log.value = '';
      scrollToBottom();
    };

    onMounted(() => {
      updateConnectionStatus();
    });

    onUnmounted(() => {
      disconnect();
    });

    return {
      isConnected,
      statusText,
      statusClass,
      message,
      log,
      logRef,
      connect,
      disconnect,
      sendMessage,
      toggleConnect, // 添加 toggleConnect 方法
      logMethod, // 添加 logMethod 方法
      clearLog, // 添加 clearLog 方法
    };
  },
};
</script>

<style>
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  font-size: 18px;
}

input[type='text'] {
  font-size: inherit;
}

#log {
  width: 30em;
  height: 20em;
  overflow: auto;
  margin: 0.5em 0;
  border: 1px solid black;
}

#status {
  padding: 0 0.2em;
}

#text {
  width: 17em;
  padding: 0.5em;
}

.msg {
  margin: 0;
  padding: 0.25em 0.5em;
}

.msg--status {
  /* a light yellow */
  background-color: #ffffc9;
}

.msg--message {
  /* a light blue */
  background-color: #d2f4ff;
}

.msg--error {
  background-color: pink;
}

.connected {
  background-color: transparent;
  color: green;
}

.disconnected {
  background-color: red;
  color: white;
}
</style>
