<template>
  <div class="navigation">
    <aside class="sidebar">
      <h2>导航分类</h2>
      <ul>
        <li v-for="category in categories" :key="category.name">
          {{ category.name }}
        </li>
      </ul>
    </aside>
    <main class="content">
      <h2>链接列表</h2>
      <div class="link-card" v-for="link in links" :key="link.name">
        <h3>{{ link.name }}</h3>
        <p>{{ link.description }}</p>
      </div>
    </main>
  </div>
</template>

<script setup>
const categories = [
  { name: '个人站点' },
  { name: '前端常用' },
  // 添加更多分类
]

const links = [
  { name: '个人博客', description: '不忘初心，方得始终。' },
  { name: 'HTML参考手册', description: '在本手册中，您可以查找到HTML的使用方法。' },
  // 添加更多链接
]
</script>

<style scoped>
.navigation {
  display: flex;
}

.sidebar {
  width: 200px;
  background-color: #f5f5f5;
  padding: 20px;
}

.content {
  flex: 1;
  padding: 20px;
}

.link-card {
  background-color: #fff;
  padding: 15px;
  margin-bottom: 10px;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
</style> 