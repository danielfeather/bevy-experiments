import { createRouter, createWebHashHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import BevyView from '../views/BevyView.vue';
import GameView from '../views/GameView.vue';

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/experiment/:experiment',
      name: 'game',
      component: GameView,
    },
    {
      path: '/bevy/:experiment',
      name: 'bevy',
      component: BevyView
    }
  ],
});

export default router;
