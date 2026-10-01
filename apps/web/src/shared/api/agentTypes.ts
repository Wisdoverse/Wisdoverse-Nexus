export type AgentRunStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";
export interface AgentRunEvent {
  sequence: number;
  kind: string;
  text: string | null;
  timestamp: number;
}
export interface AgentRun {
  contractVersion: string;
  id: string;
  roomId: string;
  invokedBy: string;
  agentId: string;
  agentName: string;
  traceId: string;
  status: AgentRunStatus;
  answer: string;
  sourceMessageIds: string[];
  usage: {
    inputTokens: number | null;
    outputTokens: number | null;
    outputBytes: number;
    toolCalls: number;
  };
  maxOutputTokens: number;
  maxOutputBytes: number;
  deadlineMs: number;
  createdAt: number;
  finishedAt: number | null;
  latencyMs: number | null;
  errorCode: string | null;
  events: AgentRunEvent[];
}
export interface InvokeAgentOptions {
  clientRunId: string;
  prompt: string;
  sourceMessageIds: string[];
  tool: "room_history" | null;
  maxOutputTokens: number;
  deadlineMs: number;
}
export interface AgentCapabilities {
  contractVersion: string;
  agentId: string;
  agentName: string;
  enabled: boolean;
  contextMessageLimit: number;
  maxOutputTokens: number;
  maxToolCalls: number;
  provider: string;
  disclosure: string;
}
export interface AgentEventBatch {
  events: AgentRunEvent[];
  status: AgentRunStatus;
}
