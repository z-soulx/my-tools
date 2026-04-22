<script setup lang="ts">
import { ref, computed } from "vue";
import type { TraceNode, NodeExecResult } from "@/types";

const props = defineProps<{
  node: TraceNode;
  result: NodeExecResult;
}>();

const showDebug = ref(false);

const jcpData = computed(() => props.result.jcpResult ?? {});

function findInObj(obj: any, key: string): any {
  if (!obj || typeof obj !== "object") return undefined;
  if (!Array.isArray(obj) && key in obj) return obj[key];
  const items = Array.isArray(obj) ? obj : Object.values(obj);
  for (const v of items) {
    if (v && typeof v === "object") {
      const found = findInObj(v, key);
      if (found !== undefined) return found;
    }
  }
  return undefined;
}

// 实际路径: orderBeforeErrorVo.orderLogErrorVo.{responseCode, errorDesc, ...}
const errorVo = computed(() => {
  const parent = findInObj(jcpData.value, "orderBeforeErrorVo");
  return parent?.orderLogErrorVo ?? findInObj(jcpData.value, "orderLogErrorVo") ?? null;
});

const productLinks = computed(() => findInObj(jcpData.value, "productLogUrlVo"));

const detailUrl = computed(() => {
  const orderId = findInObj(jcpData.value, "orderId") || "";
  const traceId = findInObj(jcpData.value, "traceId") || "";
  const params = new URLSearchParams();
  if (orderId) params.set("orderId", orderId);
  if (traceId) params.set("traceId", traceId);
  const qs = params.toString();
  return `http://jcp.mis.elong.com/orderparse/getOrderLogDetail${qs ? "?" + qs : ""}`;
});

const debugJson = computed(() => {
  try {
    return JSON.stringify(jcpData.value, null, 2).slice(0, 5000);
  } catch {
    return "无法序列化响应数据";
  }
});
</script>

<template>
  <div v-if="result.status === 'running'" class="px-4 py-6 text-center">
    <span class="text-sm text-text-secondary animate-pulse">查询中...</span>
  </div>

  <div v-else-if="result.status === 'error'" class="px-4 py-3 bg-red-50/50">
    <div class="text-sm text-red-600">{{ result.error }}</div>
    <button
      v-if="result.requestParams"
      class="mt-2 text-xs text-text-secondary hover:text-text-primary"
      @click="showDebug = !showDebug"
    >{{ showDebug ? '▾' : '▸' }} 请求参数</button>
    <pre v-if="showDebug && result.requestParams" class="mt-1 text-xs text-text-secondary bg-surface p-2 rounded overflow-auto max-h-40">{{ JSON.stringify(result.requestParams, null, 2) }}</pre>
  </div>

  <div v-else-if="result.status === 'success'" class="text-sm">
    <!-- Debug 面板（默认展开，方便确认字段结构） -->
    <div class="px-4 py-2 border-b border-border/50">
      <button class="text-xs text-text-secondary hover:text-text-primary" @click="showDebug = !showDebug">
        {{ showDebug ? '▾' : '▸' }} 原始响应
      </button>
      <pre v-if="showDebug" class="mt-2 text-xs text-text-secondary bg-surface p-3 rounded-lg overflow-auto max-h-60 border border-border/50">{{ debugJson }}</pre>
    </div>

    <!-- 提取参数 -->
    <div v-if="result.extractedParams && Object.keys(result.extractedParams).length" class="px-4 py-3 border-b border-border/50">
      <div class="text-xs font-medium text-text-secondary mb-2">提取参数</div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="(val, key) in result.extractedParams"
          :key="key"
          class="inline-flex items-center text-xs px-2 py-1 rounded-lg bg-emerald-50 text-emerald-700 border border-emerald-200"
        >
          <span class="font-medium">{{ key }}</span>
          <span class="mx-1 text-emerald-400">=</span>
          <span>{{ val }}</span>
        </span>
      </div>
    </div>

    <!-- 成单前异常分析 -->
    <div v-if="errorVo" class="border-b border-border/50">
      <div class="px-4 py-2 bg-surface-alt/50 text-xs font-medium text-text-secondary text-right">成单前异常分析</div>
      <table class="w-full text-sm">
        <tbody>
          <tr class="border-b border-border/30">
            <td class="px-4 py-2.5 text-text-secondary w-32 text-right bg-surface-alt/20">错误编号及含义</td>
            <td class="px-4 py-2.5">
              成单错误码：{{ errorVo.responseCode ?? '-' }}，错误码描述：{{ errorVo.responseCodeStr ?? '-' }}
            </td>
          </tr>
          <tr class="border-b border-border/30">
            <td class="px-4 py-2.5 text-text-secondary text-right bg-surface-alt/20">异常原因</td>
            <td class="px-4 py-2.5">{{ errorVo.errorDesc ?? '-' }}</td>
          </tr>
          <tr class="border-b border-border/30">
            <td class="px-4 py-2.5 text-text-secondary text-right bg-surface-alt/20">异常跟终组</td>
            <td class="px-4 py-2.5">{{ errorVo.handlerDepartment ?? '-' }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 价格、规则、库存信息 -->
    <div v-if="productLinks" class="border-b border-border/50">
      <div class="px-4 py-2.5 flex flex-wrap items-center gap-3">
        <a v-if="productLinks.priceUrl" :href="productLinks.priceUrl" target="_blank" class="text-primary hover:underline text-sm">实时价格 ↗</a>
        <a v-if="productLinks.priceHisUrl" :href="productLinks.priceHisUrl" target="_blank" class="text-primary hover:underline text-sm">历史价格 ↗</a>
        <a v-if="productLinks.policyUrl" :href="productLinks.policyUrl" target="_blank" class="text-primary hover:underline text-sm">规则及历史 ↗</a>
        <a v-if="productLinks.inventoryUrl" :href="productLinks.inventoryUrl" target="_blank" class="text-primary hover:underline text-sm">库存及历史 ↗</a>
        <a v-if="productLinks.rateplanUrl" :href="productLinks.rateplanUrl" target="_blank" class="text-primary hover:underline text-sm">rp基础信息 ↗</a>
        <!-- 兜底：其他未知链接 -->
        <template v-for="(url, key) in productLinks" :key="key">
          <a v-if="typeof url === 'string' && url && !['priceUrl','priceHisUrl','policyUrl','inventoryUrl','rateplanUrl'].includes(String(key))"
            :href="url" target="_blank" class="text-primary hover:underline text-sm">{{ key }} ↗</a>
        </template>
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="px-4 py-3 flex items-center gap-3">
      <a :href="detailUrl" target="_blank"
        class="text-xs px-3 py-1.5 rounded-lg bg-surface-alt text-text-secondary border border-border hover:border-primary hover:text-primary transition-colors">
        成单日志分析查询详细 ↗
      </a>
      <span v-if="result.durationMs" class="text-xs text-text-secondary">{{ result.durationMs }}ms</span>
    </div>
  </div>
</template>
