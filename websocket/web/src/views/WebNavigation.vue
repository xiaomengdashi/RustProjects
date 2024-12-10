<template>
  <div class="tech-nav-container">
    <div class="nav-section">
      <h2>前端常用</h2>
      <div class="nav-grid">
        <div class="nav-item" v-for="item in frontendTools" :key="item.id">
          <img :src="item.icon" :alt="item.name">
          <div class="item-content">
            <h3>{{ item.name }}</h3>
            <p>{{ item.description }}</p>
          </div>
        </div>
      </div>
    </div>
    
    <div class="nav-section">
      <h2>UI框架</h2>
      <div class="nav-grid">
        <div class="nav-item" v-for="item in uiFrameworks" :key="item.id">
          <img :src="item.icon" :alt="item.name">
          <div class="item-content">
            <h3>{{ item.name }}</h3>
            <p>{{ item.description }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'TechNav',
  data() {
    return {
      frontendTools: [
        {
          id: 1,
          name: 'HTML基础手册',
          description: '在实手册中，您可以很快掌握HTML的使用方法。',
          icon: '路径到HTML图标'
        },
        // 添加更多工具...
      ],
      uiFrameworks: [
        {
          id: 1,
          name: 'Element Plus',
          description: '基于 Vue3 的组件库和开发者的好帮手。',
          icon: '路径到Element Plus图标'
        },
        // 添加更多框架...
      ]
    }
  }
}
</script>

<style scoped>
.tech-nav-container {
  padding: 20px;
}

.nav-section {
  margin-bottom: 30px;
}

.nav-section h2 {
  margin-bottom: 20px;
  font-size: 18px;
  color: #333;
}

.nav-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 15px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.nav-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
}

.nav-item img {
  width: 40px;
  height: 40px;
  margin-right: 15px;
}

.item-content h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.item-content p {
  margin: 5px 0 0;
  font-size: 14px;
  color: #666;
}
</style>