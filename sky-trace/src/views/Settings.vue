<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const showForm = ref(false);
const form = ref({
  appId: "",
  appUk: "",
  token: "",
  name: "",
  env: "prod",
});

function openCreateForm() {
  form.value = { appId: "", appUk: "", token: "", name: "", env: "prod" };
  showForm.value = true;
}

async function handleSave() {
  if (!form.value.appId.trim() || !form.value.token.trim()) return;
  await store.addSkyApp({
    appId: form.value.appId.trim(),
    appUk: form.value.appUk.trim(),
    token: form.value.token.trim(),
    name: form.value.name.trim() || form.value.appUk.trim(),
    env: form.value.env,
  });
  showForm.value = false;
}

async function handleDelete(id: number) {
  await store.removeSkyApp(id);
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border shrink-0 flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold">天网配置</h2>
        <p class="text-sm text-text-secondary mt-0.5">管理天网应用 AppId / Token / 服务标识</p>
      </div>
      <button
        class="px-4 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover transition-colors"
        @click="openCreateForm"
      >
        + 添加天网应用
      </button>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="store.skyApps.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <div class="text-4xl mb-4">⚙</div>
        <p class="text-lg">暂无天网应用配置</p>
        <p class="text-sm mt-1">添加天网应用的 AppId 和 Token 来开始使用</p>
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="app in store.skyApps"
          :key="app.id"
          class="bg-surface rounded-xl border border-border p-4 group"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <div>
                <h3 class="font-medium">{{ app.name || app.appUk }}</h3>
                <p class="text-xs text-text-secondary mt-0.5">{{ app.appUk }}</p>
              </div>
            </div>
            <button
              class="text-xs text-error opacity-0 group-hover:opacity-100 transition-opacity"
              @click="handleDelete(app.id)"
            >删除</button>
          </div>
          <div class="mt-3 grid grid-cols-3 gap-4 text-sm">
            <div>
              <span class="text-text-secondary text-xs">AppId</span>
              <p class="font-mono">{{ app.appId }}</p>
            </div>
            <div>
              <span class="text-text-secondary text-xs">Token</span>
              <p class="font-mono truncate" :title="app.token">{{ app.token.slice(0, 12) }}...</p>
            </div>
            <div>
              <span class="text-text-secondary text-xs">环境</span>
              <p>{{ app.env }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showForm" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="showForm = false">
      <div class="bg-surface rounded-xl shadow-xl w-[480px]">
        <div class="px-6 py-4 border-b border-border">
          <h3 class="text-lg font-semibold">添加天网应用</h3>
        </div>
        <div class="px-6 py-4 space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1">AppId *</label>
            <input v-model="form.appId" placeholder="如：106676" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary font-mono" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">AppUk (服务标识) *</label>
            <input v-model="form.appUk" placeholder="如：mvt.dc.product.mapping" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary font-mono" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">Token *</label>
            <input v-model="form.token" placeholder="api-token" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary font-mono" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">别名</label>
            <input v-model="form.name" placeholder="如：Mapping服务" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">环境</label>
            <select v-model="form.env" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary">
              <option value="prod">PROD</option>
              <option value="uat">UAT</option>
            </select>
          </div>
        </div>
        <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
          <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="showForm = false">取消</button>
          <button
            class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
            :disabled="!form.appId.trim() || !form.token.trim()"
            @click="handleSave"
          >保存</button>
        </div>
      </div>
    </div>
  </div>
</template>
