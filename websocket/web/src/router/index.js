import { createRouter, createWebHistory } from 'vue-router'
import Home from '../App.vue'
import About from '../views/About.vue'
import WebSoccket from '../views/WebSoccket.vue'
import Contact from '../views/Contact.vue'
import Navigation from '../views/Navigation.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/about',
    name: 'About',
    component: About
  },
  {
    path: '/contact',
    name: 'Contact',
    component: Contact
  },
  {
    path: '/websocket',
    name: 'WebSoccket',
    component: WebSoccket
  },
  {
    path: '/nav',
    name: 'Navigation',
    component: Navigation
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router