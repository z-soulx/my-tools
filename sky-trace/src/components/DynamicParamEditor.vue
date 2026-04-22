<script setup lang="ts">
import { ref, computed } from "vue";
import type { DynamicParam } from "@/types";

const props = defineProps<{ params: DynamicParam[] }>();
const emit = defineEmits<{ close: []; save: [params: DynamicParam[]] }>();

interface ParamItem extends DynamicParam {
  _uid: number;
  _optionsText: string;
  _snippetsText: string;
}

let nextUid = 1;

const list = ref<ParamItem[]>(
  props.params.map((p) => ({
    ...p,
    hint: p.hint ?? "",
    options: p.options ? [...p.options] : [],
    snippets: p.snippets ? [...p.snippets] : [],
    allowCustom: p.allowCustom ?? true,
    hidden: p.hidden ?? false,
    paramType: p.paramType ?? "text",
    _uid: nextUid++,
    _optionsText: (p.options ?? []).join("\n"),
    _snippetsText: (p.snippets ?? []).join("\n"),
  }))
);

const search = ref("");
const expandedUid = ref<number | null>(null);

const filteredItems = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return list.value;
  return list.value.filter(
    (p) =>
      p.key.toLowerCase().includes(q) ||
      p.label.toLowerCase().includes(q) ||
      (p.hint ?? "").toLowerCase().includes(q)
  );
});

function addParam() {
  const item: ParamItem = {
    key: "",
    label: "",
    required: false,
    defaultValue: "",
    hint: "",
    options: [],
    snippets: [],
    allowCustom: true,
    hidden: false,
    paramType: "text" as const,
    _uid: nextUid++,
    _optionsText: "",
    _snippetsText: "",
  };
  list.value.push(item);
  search.value = "";
  expandedUid.value = item._uid;
}

function removeParam(uid: number) {
  const idx = list.value.findIndex((p) => p._uid === uid);
  if (idx < 0) return;
  list.value.splice(idx, 1);
  if (expandedUid.value === uid) expandedUid.value = null;
}

function toggleExpand(uid: number) {
  expandedUid.value = expandedUid.value === uid ? null : uid;
}

function itemIndex(uid: number): number {
  return list.value.findIndex((p) => p._uid === uid);
}

function moveUp(uid: number) {
  const idx = itemIndex(uid);
  if (idx <= 0) return;
  const [item] = list.value.splice(idx, 1);
  list.value.splice(idx - 1, 0, item);
}

function moveDown(uid: number) {
  const idx = itemIndex(uid);
  if (idx < 0 || idx >= list.value.length - 1) return;
  const [item] = list.value.splice(idx, 1);
  list.value.splice(idx + 1, 0, item);
}

function moveTo(uid: number, targetPos: number) {
  const fromIdx = itemIndex(uid);
  if (fromIdx < 0) return;
  // targetPos is 1-based, clamp to valid range
  const toIdx = Math.max(0, Math.min(list.value.length - 1, targetPos - 1));
  if (fromIdx === toIdx) return;
  const [item] = list.value.splice(fromIdx, 1);
  list.value.splice(toIdx, 0, item);
}

function handleSave() {
  const valid = list.value
    .map((p) => ({
      key: p.key,
      label: p.label,
      required: p.required,
      defaultValue: p.defaultValue,
      hint: p.hint,
      allowCustom: p.allowCustom,
      hidden: p.hidden,
      paramType: p.paramType,
      options: p._optionsText
        ? p._optionsText.split("\n").map((s) => s.trim()).filter(Boolean)
        : [],
      snippets: p._snippetsText
        ? p._snippetsText.split("\n").map((s) => s.trim()).filter(Boolean)
        : [],
    }))
    .filter((p) => p.key.trim() && p.label.trim())
    .map((p) => ({
      ...p,
      options: p.options.length > 0 ? p.options : undefined,
      snippets: p.snippets.length > 0 ? p.snippets : undefined,
      hint: p.hint || undefined,
      hidden: p.hidden || undefined,
      paramType: p.paramType && p.paramType !== "text" ? p.paramType : undefined,
    }));
  emit("save", valid as DynamicParam[]);
}
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="emit('close')">
    <div class="bg-surface rounded-xl shadow-xl w-[620px] max-h-[85vh] flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-border shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-lg font-semibold">编辑动态参数</h3>
            <p class="text-xs text-text-secondary mt-0.5">共 {{ list.length }} 个参数 · 箭头或输入位置调整顺序</p>
          </div>
          <button
            class="px-3 py-1.5 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover transition-colors"
            @click="addParam"
          >+ 添加</button>
        </div>
        <!-- Search -->
        <div v-if="list.length > 3" class="mt-3 relative">
          <input
            v-model="search"
            placeholder="搜索参数 key / 名称 / 提示..."
            class="w-full pl-8 pr-3 py-1.5 text-sm border border-border rounded-lg outline-none focus:border-primary"
          />
          <span class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-secondary/50 text-xs">🔍</span>
        </div>
      </div>

      <!-- Param list -->
      <div class="flex-1 overflow-y-auto px-4 py-3 space-y-1">
        <div v-if="filteredItems.length === 0" class="py-8 text-center text-sm text-text-secondary">
          {{ list.length === 0 ? '暂无参数，点击右上角添加' : '无匹配结果' }}
        </div>

        <div
          v-for="item in filteredItems"
          :key="item._uid"
          class="border rounded-lg transition-all"
          :class="expandedUid === item._uid ? 'border-primary/40 bg-blue-50/20' : 'border-border'"
        >
          <!-- Compact row -->
          <div
            class="flex items-center gap-1.5 px-2 py-2 cursor-pointer select-none hover:bg-surface-alt/50 transition-colors"
            @click="toggleExpand(item._uid)"
          >
            <!-- 上下箭头 -->
            <div class="flex flex-col shrink-0" @click.stop>
              <button
                class="text-[9px] leading-none text-text-secondary/40 hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed px-0.5"
                :disabled="itemIndex(item._uid) === 0"
                title="上移"
                @click="moveUp(item._uid)"
              >▲</button>
              <button
                class="text-[9px] leading-none text-text-secondary/40 hover:text-primary disabled:opacity-20 disabled:cursor-not-allowed px-0.5"
                :disabled="itemIndex(item._uid) === list.length - 1"
                title="下移"
                @click="moveDown(item._uid)"
              >▼</button>
            </div>
            <span class="text-[10px] text-text-secondary/50 w-4 text-center shrink-0">{{ itemIndex(item._uid) + 1 }}</span>
            <span class="text-xs font-mono text-primary/80 bg-blue-50 px-1.5 py-0.5 rounded shrink-0 min-w-[60px]">{{ item.key || '(未设置)' }}</span>
            <span class="text-sm truncate flex-1">{{ item.label || '(未命名)' }}</span>
            <span v-if="item.required" class="text-[10px] px-1.5 py-0.5 bg-red-50 text-error rounded shrink-0">必填</span>
            <span v-if="item.hidden" class="text-[10px] px-1.5 py-0.5 bg-gray-100 text-text-secondary rounded shrink-0">隐藏</span>
            <span v-if="item.paramType && item.paramType !== 'text'" class="text-[10px] px-1.5 py-0.5 bg-cyan-50 text-cyan-700 rounded shrink-0">{{ item.paramType }}</span>
            <span v-if="item.defaultValue" class="text-[10px] px-1.5 py-0.5 bg-surface-alt text-text-secondary rounded truncate max-w-[80px] shrink-0" :title="item.defaultValue">= {{ item.defaultValue }}</span>
            <span v-if="item._optionsText?.trim()" class="text-[10px] px-1.5 py-0.5 bg-violet-50 text-violet-600 rounded shrink-0">{{ item._optionsText.split('\n').filter(s => s.trim()).length }} 选项</span>
            <span v-if="item._snippetsText?.trim()" class="text-[10px] px-1.5 py-0.5 bg-emerald-50 text-emerald-600 rounded shrink-0">{{ item._snippetsText.split('\n').filter(s => s.trim()).length }} 快捷</span>
            <span class="text-text-secondary/40 text-xs shrink-0">{{ expandedUid === item._uid ? '▾' : '▸' }}</span>
          </div>

          <!-- Expanded detail -->
          <div v-if="expandedUid === item._uid" class="px-3 pb-3 pt-1 border-t border-border/50 space-y-2.5">
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-[10px] text-text-secondary mb-0.5">参数标识 (key) *</label>
                <input
                  v-model="item.key"
                  placeholder="如 hotelId"
                  class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary font-mono"
                />
              </div>
              <div>
                <label class="block text-[10px] text-text-secondary mb-0.5">显示名称 *</label>
                <input
                  v-model="item.label"
                  placeholder="如 酒店ID"
                  class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-[10px] text-text-secondary mb-0.5">默认值</label>
                <input
                  v-model="item.defaultValue"
                  placeholder="可选"
                  class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary"
                />
              </div>
              <div class="flex items-end gap-3 pb-1">
                <label class="flex items-center gap-1.5 text-xs cursor-pointer">
                  <input type="checkbox" v-model="item.required" class="rounded" />
                  必填
                </label>
                <label class="flex items-center gap-1.5 text-xs cursor-pointer" title="隐藏参数不在执行面板显示，但内部仍可被节点绑定使用">
                  <input type="checkbox" v-model="item.hidden" class="rounded" />
                  隐藏
                </label>
                <div class="flex items-center gap-1">
                  <label class="text-[10px] text-text-secondary">类型</label>
                  <select
                    v-model="item.paramType"
                    class="px-1 py-0.5 text-[10px] border border-border rounded outline-none focus:border-primary"
                  >
                    <option value="text">文本</option>
                    <option value="datetime">日期时间</option>
                    <option value="date">日期</option>
                    <option value="timestamp_ms">毫秒时间戳</option>
                    <option value="timestamp_s">秒级时间戳</option>
                    <option value="day_timestamp_s">天级时间戳</option>
                  </select>
                </div>
                <div class="flex items-center gap-1 ml-auto" @click.stop>
                  <label class="text-[10px] text-text-secondary">移到第</label>
                  <input
                    type="number"
                    :value="itemIndex(item._uid) + 1"
                    min="1"
                    :max="list.length"
                    class="w-12 px-1 py-0.5 text-xs border border-border rounded outline-none focus:border-primary text-center"
                    @change="moveTo(item._uid, parseInt(($event.target as HTMLInputElement).value) || 1)"
                  />
                  <label class="text-[10px] text-text-secondary">位</label>
                </div>
              </div>
            </div>
            <div>
              <label class="block text-[10px] text-text-secondary mb-0.5">提示信息 <span class="text-text-secondary/50">(支持多行，执行时展示)</span></label>
              <textarea
                v-model="item.hint"
                rows="2"
                placeholder="执行时的提示文字 (可选)&#10;支持多行输入"
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary resize-none"
              />
            </div>
            <div>
              <label class="block text-[10px] text-text-secondary mb-0.5">预定义选项 <span class="text-text-secondary/50">(每行一个，格式: 值|显示名 或 纯文本)</span></label>
              <textarea
                v-model="item._optionsText"
                rows="2"
                placeholder="1|增量&#10;2|全量&#10;纯文本选项"
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary resize-none font-mono"
              />
            </div>
            <div v-if="item._optionsText?.trim()" class="flex items-center gap-3">
              <label class="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                <input type="checkbox" v-model="item.allowCustom" class="rounded" />
                允许自定义输入
              </label>
            </div>
            <div>
              <label class="block text-[10px] text-text-secondary mb-0.5">快捷填入 <span class="text-text-secondary/50">(每行一个，格式: 值|显示名 或 纯文本。执行时显示为可点击芯片)</span></label>
              <textarea
                v-model="item._snippetsText"
                rows="2"
                placeholder='\"innId\":\"WYN5300072\",\"roomTypeCode\":\"VDS\"|温德姆示例&#10;较长的JSON片段|含天数维度'
                class="w-full px-2 py-1.5 text-sm border border-border rounded outline-none focus:border-primary resize-none font-mono"
              />
            </div>
            <div class="flex justify-end pt-1">
              <button class="text-xs text-error hover:text-red-700 px-2 py-1 rounded hover:bg-red-50 transition-colors" @click.stop="removeParam(item._uid)">删除此参数</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-border flex justify-end gap-3 shrink-0">
        <button class="px-4 py-2 text-sm text-text-secondary hover:bg-surface-alt rounded-lg" @click="emit('close')">取消</button>
        <button class="px-4 py-2 text-sm bg-primary text-white rounded-lg hover:bg-primary-hover" @click="handleSave">保存</button>
      </div>
    </div>
  </div>
</template>
