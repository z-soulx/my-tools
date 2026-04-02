<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const showForm = ref(false);
const form = ref({
  name: "",
  code: "",
  description: "",
  serviceIds: [] as number[],
});
const editing = ref(false);

function openCreateForm() {
  form.value = { name: "", code: "", description: "", serviceIds: [] };
  editing.value = false;
  showForm.value = true;
}

async function handleSave() {
  if (!form.value.name.trim() || !form.value.code.trim()) return;
  await store.addSupplier({
    name: form.value.name.trim(),
    code: form.value.code.trim(),
    description: form.value.description.trim(),
    serviceIds: form.value.serviceIds,
  });
  showForm.value = false;
}

async function handleDelete(id: number) {
  await store.removeSupplier(id);
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 bg-surface border-b border-border shrink-0 flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold">供应商管理</h2>
        <p class="text-sm text-text-secondary mt-0.5">管理酒店供应商及其关联服务</p>
      </div>
      <button
        class="px-4 py-2 bg-primary text-white text-sm rounded-lg hover:bg-primary-hover transition-colors"
        @click="openCreateForm"
      >
        + 添加供应商
      </button>
    </header>

    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="store.suppliers.length === 0" class="flex flex-col items-center justify-center h-full text-text-secondary">
        <div class="text-4xl mb-4">🏢</div>
        <p class="text-lg">暂无供应商</p>
        <p class="text-sm mt-1">添加供应商来组织排查链路</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="supplier in store.suppliers"
          :key="supplier.id"
          class="bg-surface rounded-xl border border-border p-4 group"
        >
          <div class="flex items-start justify-between">
            <div>
              <h3 class="font-medium">{{ supplier.name }}</h3>
              <p class="text-xs text-text-secondary mt-0.5">{{ supplier.code }}</p>
            </div>
            <button
              class="text-xs text-error opacity-0 group-hover:opacity-100 transition-opacity"
              @click="handleDelete(supplier.id)"
            >删除</button>
          </div>
          <p v-if="supplier.description" class="text-sm text-text-secondary mt-2">
            {{ supplier.description }}
          </p>
          <div class="mt-3 text-xs text-text-secondary">
            {{ store.flows.filter(f => f.supplierId === supplier.id).length }} 条关联链路
          </div>
        </div>
      </div>
    </div>

    <div v-if="showForm" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="showForm = false">
      <div class="bg-surface rounded-xl shadow-xl w-[420px]">
        <div class="px-6 py-4 border-b border-border">
          <h3 class="text-lg font-semibold">{{ editing ? '编辑' : '添加' }}供应商</h3>
        </div>
        <div class="px-6 py-4 space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1">供应商名称 *</label>
            <input v-model="form.name" placeholder="如：铂涛" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">编码 *</label>
            <input v-model="form.code" placeholder="如：botao" class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">描述</label>
            <textarea v-model="form.description" rows="2" placeholder="供应商说明..." class="w-full px-3 py-2 border border-border rounded-lg text-sm outline-none focus:border-primary resize-none" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1">关联天网服务</label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="app in store.skyApps"
                :key="app.id"
                class="flex items-center gap-1.5 text-sm cursor-pointer"
              >
                <input
                  type="checkbox"
                  :checked="form.serviceIds.includes(app.id)"
                  @change="
                    form.serviceIds.includes(app.id)
                      ? (form.serviceIds = form.serviceIds.filter(id => id !== app.id))
                      : form.serviceIds.push(app.id)
                  "
                  class="rounded"
                />
                {{ app.name || app.appUk }}
              </label>
            </div>
          </div>
        </div>
        <div class="px-6 py-4 border-t border-border flex justify-end gap-3">
          <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="showForm = false">取消</button>
          <button
            class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover disabled:opacity-50"
            :disabled="!form.name.trim() || !form.code.trim()"
            @click="handleSave"
          >保存</button>
        </div>
      </div>
    </div>
  </div>
</template>
