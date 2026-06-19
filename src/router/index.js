import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Simulator',
    component: () => import('../views/SimulatorView.vue'),
    meta: { title: '仿真监控' },
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.afterEach((to) => {
  if (to.meta?.title) {
    document.title = `${to.meta.title} - 五轴数控机床仿真监控系统`
  }
})

export default router
