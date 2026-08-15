import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
    },
    {
      path: '/processes',
      name: 'processes',
      component: () => import('@/views/ProcessListView.vue'),
    },
    {
      path: '/alerts',
      name: 'alerts',
      component: () => import('@/views/AlertConfigView.vue'),
    },
    {
      path: '/throttle',
      name: 'throttle',
      component: () => import('@/views/ThrottleManagerView.vue'),
    },
    {
      path: '/visualizer',
      name: 'visualizer',
      component: () => import('@/views/TrafficVisualizerView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
    {
      path: '/taskbar-widget',
      name: 'taskbar-widget',
      component: () => import('@/views/TaskbarWidgetView.vue'),
    },
    {
      path: '/floating-widget',
      name: 'floating-widget',
      component: () => import('@/views/FloatingWidgetView.vue'),
    },
  ],
  scrollBehavior() {
    return { top: 0 }
  },
})

export default router
