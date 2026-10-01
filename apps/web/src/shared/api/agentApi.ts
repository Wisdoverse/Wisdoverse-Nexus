import { httpClient } from "./httpClient";
export type {
  AgentRun,
  AgentRunEvent,
  InvokeAgentOptions,
  AgentCapabilities,
} from "./agentTypes";
import type {
  AgentRun,
  InvokeAgentOptions,
  AgentCapabilities,
} from "./agentTypes";
const base = (roomId: string) => `/rooms/${encodeURIComponent(roomId)}`;
export const agentApi = {
  async capabilities(
    roomId: string,
    signal: AbortSignal,
  ): Promise<AgentCapabilities> {
    return (
      await httpClient.get<AgentCapabilities>(
        `${base(roomId)}/agents`,
        undefined,
        { signal },
      )
    ).data;
  },
  async invoke(
    roomId: string,
    command: InvokeAgentOptions,
    signal: AbortSignal,
  ): Promise<AgentRun> {
    return (
      await httpClient.post<AgentRun>(`${base(roomId)}/agent-runs`, command, {
        signal,
      })
    ).data;
  },
  async read(
    roomId: string,
    runId: string,
    signal: AbortSignal,
  ): Promise<AgentRun> {
    return (
      await httpClient.get<AgentRun>(
        `${base(roomId)}/agent-runs/${encodeURIComponent(runId)}`,
        undefined,
        { signal },
      )
    ).data;
  },
  async cancel(
    roomId: string,
    runId: string,
    signal: AbortSignal,
  ): Promise<AgentRun> {
    return (
      await httpClient.post<AgentRun>(
        `${base(roomId)}/agent-runs/${encodeURIComponent(runId)}/cancel`,
        {},
        { signal },
      )
    ).data;
  },
};
