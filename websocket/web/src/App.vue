<template>
  <div class="app-container">
    <!-- 导航菜单 -->
    <nav class="nav-menu">
      <router-link to="/" class="nav-link">Home</router-link>
      <router-link to="/about" class="nav-link">About</router-link>
      <router-link to="/websocket" class="nav-link">Websocket</router-link>
      <router-link to="/contact" class="nav-link">Contract</router-link>
      <router-link to="/nav" class="nav-link">Navigation</router-link>
    </nav>
    
    <!-- 路由出口 -->
    <router-view></router-view>
  </div>
</template>

<style scoped>
.app-container {
  padding: 20px;
}

.nav-menu {
  margin-bottom: 20px;
  padding: 10px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.nav-link {
  margin-right: 20px;
  text-decoration: none;
  color: #333;
  padding: 5px 10px;
}

.nav-link:hover {
  color: #409EFF;
}

/* 激活状态的路由链接样式 */
.router-link-active {
  color: #409EFF;
  font-weight: bold;
}
</style>