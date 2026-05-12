import type {
  ChatMessage,
  TraceFlow,
  TraceNode,
  NodeExecResult,
} from "@/types";

const MAX_LOGS_PER_NODE = 30;
const MAX_MSG_LEN = 2000;

function smartTruncate(s: string, max = MAX_MSG_LEN): string {
  if (s.length <= max) return s;
  const keep = Math.floor((max - 30) / 2);
  return `${s.slice(0, keep)}…[省略${s.length - keep * 2}字符]…${s.slice(-keep)}`;
}

function summarizeSkynetResult(result: NodeExecResult) {
  const list = result.result?.result?.list ?? [];
  const total = result.result?.result?.count ?? list.length;
  const trimmed = list.slice(0, MAX_LOGS_PER_NODE).map((it: any) => ({
    logTime: it.logTime,
    priority: it.priority,
    filter1: it.filter1,
    filter2: it.filter2,
    msg: smartTruncate(String(it.msg ?? "")),
  }));
  return { total, sampledCount: trimmed.length, items: trimmed };
}

function summarizeJcpResult(result: NodeExecResult) {
  const r = result.jcpResult;
  if (!r) return null;
  const s = JSON.stringify(r);
  if (s.length <= 8000) return r;
  // JCP 结果过大时，返回字符串摘要而非对象（避免截断 JSON 破坏结构）
  return `[JCP结果已截断，原始大小${s.length}字符] ${s.slice(0, 7000)}…`;
}

function summarizeNodeResult(node: TraceNode, result?: NodeExecResult) {
  if (!result) return { status: "no-result" };
  const base: Record<string, any> = {
    label: node.label,
    type: node.type,
    status: result.status,
    health: result.health,
    durationMs: result.durationMs,
  };
  if (result.error) base.error = smartTruncate(result.error, 500);
  if (result.requestParams) base.requestParams = result.requestParams;
  if (result.extractedParams) base.extractedParams = result.extractedParams;
  if (node.type === "skynet_query") {
    base.logs = summarizeSkynetResult(result);
  } else if (node.type === "jcp_order") {
    base.jcp = summarizeJcpResult(result);
  }
  return base;
}

function buildSystemPrompt(
  flow: TraceFlow,
  defaultSystemPrompt: string | undefined,
  nodeSpecific?: TraceNode,
  allNodes?: TraceNode[],
): string {
  const parts: string[] = [];
  if (defaultSystemPrompt && defaultSystemPrompt.trim()) {
    parts.push(defaultSystemPrompt.trim());
  } else {
    parts.push(
      "你是 OTA 业务排查助手 SkyTrace。基于用户编排的排查流程与节点真实执行数据，输出简洁、可执行的归因和下一步建议。回答使用中文。",
    );
  }
  parts.push(`【当前流程】${flow.name}`);
  if (flow.description) parts.push(`【流程描述】${flow.description}`);
  if (flow.aiPrompt && flow.aiPrompt.trim()) {
    parts.push(`【流程级业务上下文】\n${flow.aiPrompt.trim()}`);
  }

  if (nodeSpecific) {
    if (nodeSpecific.aiPrompt && nodeSpecific.aiPrompt.trim()) {
      parts.push(`【聚焦节点】${nodeSpecific.label}（类型：${nodeSpecific.type}）\n\n# 解读规则\n${nodeSpecific.aiPrompt.trim()}`);
    } else {
      parts.push(`【聚焦节点】${nodeSpecific.label}（类型：${nodeSpecific.type}）`);
    }
  } else if (allNodes) {
    const nodesWithHints = allNodes.filter((n) => n.aiPrompt && n.aiPrompt.trim());
    if (nodesWithHints.length > 0) {
      const dict = nodesWithHints
        .map((n) => `## [${n.label}]（${n.type}）\n${n.aiPrompt!.trim()}`)
        .join("\n\n");
      parts.push(`# 本次排查链路数据字典\n解析底部 JSON 时，请严格按以下各节点规则理解字段含义：\n\n${dict}`);
    }
  }

  return parts.join("\n\n");
}

export function buildGlobalAnalysisMessages(
  flow: TraceFlow,
  execResults: Record<string, NodeExecResult>,
  dynamicValues: Record<string, string>,
  userPrompt: string,
  defaultSystemPrompt?: string,
): ChatMessage[] {
  const system = buildSystemPrompt(flow, defaultSystemPrompt, undefined, flow.nodes);

  const nodes = flow.nodes
    .filter((n) => execResults[n.id])
    .sort((a, b) => a.sortOrder - b.sortOrder)
    .map((n) => ({
      id: n.id,
      ...summarizeNodeResult(n, execResults[n.id]),
    }));

  const dataPayload = { dynamicValues, nodes };

  return [
    { role: "system", content: system },
    {
      role: "user",
      content: `【实际执行数据】\n\`\`\`json\n${JSON.stringify(dataPayload)}\n\`\`\`\n\n【分析需求】\n${userPrompt}`,
    },
  ];
}

export function buildNodeAnalysisMessages(
  flow: TraceFlow,
  node: TraceNode,
  result: NodeExecResult,
  dynamicValues: Record<string, string>,
  userPrompt: string,
  defaultSystemPrompt?: string,
): ChatMessage[] {
  const system = buildSystemPrompt(flow, defaultSystemPrompt, node);

  const dataPayload = {
    dynamicValues,
    node: summarizeNodeResult(node, result),
  };

  const ask =
    userPrompt.trim() ||
    "请基于该节点产出，判断是否存在异常、给出可能原因和下一步排查建议。";

  return [
    { role: "system", content: system },
    {
      role: "user",
      content: `【该节点执行数据】\n\`\`\`json\n${JSON.stringify(dataPayload)}\n\`\`\`\n\n【分析需求】\n${ask}`,
    },
  ];
}
