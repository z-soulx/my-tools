<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  text: string;
  keyword: string;
  activeIndex?: number;
  startMatchIndex?: number;
}>();

const segments = computed(() => {
  if (!props.keyword) return [{ text: props.text, match: false, idx: -1 }];
  const kw = props.keyword.toLowerCase();
  const src = props.text;
  const srcLower = src.toLowerCase();
  const parts: { text: string; match: boolean; idx: number }[] = [];
  let pos = 0;
  let matchIdx = props.startMatchIndex ?? 0;
  while (pos < src.length) {
    const found = srcLower.indexOf(kw, pos);
    if (found === -1) {
      parts.push({ text: src.slice(pos), match: false, idx: -1 });
      break;
    }
    if (found > pos) {
      parts.push({ text: src.slice(pos, found), match: false, idx: -1 });
    }
    parts.push({ text: src.slice(found, found + kw.length), match: true, idx: matchIdx++ });
    pos = found + kw.length;
  }
  return parts;
});
</script>

<template>
  <span>
    <template v-for="(seg, i) in segments" :key="i">
      <mark
        v-if="seg.match"
        class="rounded px-0.5"
        :class="seg.idx === activeIndex ? 'bg-amber-400 text-black' : 'bg-amber-200/70 text-inherit'"
        :data-match-idx="seg.idx"
      >{{ seg.text }}</mark>
      <span v-else>{{ seg.text }}</span>
    </template>
  </span>
</template>
