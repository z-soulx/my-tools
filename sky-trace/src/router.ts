import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      redirect: "/flows",
    },
    {
      path: "/flows",
      name: "flows",
      component: () => import("@/views/FlowList.vue"),
    },
    {
      path: "/flows/:id",
      name: "flow-detail",
      component: () => import("@/views/FlowDetail.vue"),
    },
    {
      path: "/suppliers",
      name: "suppliers",
      component: () => import("@/views/SupplierManager.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/Settings.vue"),
    },
    {
      path: "/quick-query",
      name: "quick-query",
      component: () => import("@/views/QuickQuery.vue"),
    },
    {
      path: "/checklists",
      name: "checklists",
      component: () => import("@/views/ChecklistManager.vue"),
    },
    {
      path: "/recovery",
      name: "recovery",
      component: () => import("@/views/RecoveryManager.vue"),
    },
    {
      path: "/trash",
      name: "trash",
      component: () => import("@/views/TrashBin.vue"),
    },
  ],
});

export default router;
