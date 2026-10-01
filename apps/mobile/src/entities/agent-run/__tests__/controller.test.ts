import { describe, expect, it, vi } from "vitest";
import {
  createAgentRunController,
  type AgentRun,
  type AgentView,
} from "../controller";
import type { AgentCapabilities } from "../../../shared/api/agentApi";

const capabilities: AgentCapabilities = {
  contractVersion: "1.0",
  agentId: "assistant",
  agentName: "Room Assistant",
  enabled: true,
  contextMessageLimit: 20,
  maxOutputTokens: 4096,
  maxToolCalls: 1,
  provider: "openai-compatible",
  disclosure: "Synthetic test",
};
const completed: AgentRun = {
  contractVersion: "1.0",
  id: "run-1",
  roomId: "room-1",
  invokedBy: "owner",
  agentId: "assistant",
  agentName: "Room Assistant",
  traceId: "trace-1",
  status: "completed",
  answer: "private answer",
  sourceMessageIds: ["message-1"],
  usage: { inputTokens: 10, outputTokens: 3, outputBytes: 14, toolCalls: 1 },
  maxOutputTokens: 1024,
  maxOutputBytes: 1024,
  deadlineMs: 60000,
  createdAt: 1,
  finishedAt: 2,
  latencyMs: 1,
  errorCode: null,
  events: [],
};
const port = () => ({
  capabilities: vi.fn().mockResolvedValue(capabilities),
  invoke: vi.fn().mockResolvedValue(completed),
  read: vi.fn().mockResolvedValue(completed),
  cancel: vi.fn().mockResolvedValue(completed),
});

describe("room assistant observation and recovery", () => {
  it("reuses the request identity after losing the invocation acknowledgement", async () => {
    const api = port();
    api.invoke.mockRejectedValueOnce(new Error("acknowledgement lost"));
    const views: AgentView[] = [];
    const controller = createAgentRunController(
      "room-1",
      (view) => views.push(view),
      api,
    );
    await controller.initialize();
    await controller.invoke("Summarize this room");
    expect(views.at(-1)?.error).toContain("Retry");
    await controller.retry();
    expect(api.invoke).toHaveBeenCalledTimes(2);
    expect(api.invoke.mock.calls[0][1]).toEqual(api.invoke.mock.calls[1][1]);
    expect(views.at(-1)?.run?.answer).toBe("private answer");
    controller.dispose();
  });

  it("clears cached private output and capabilities when room permission is revoked", async () => {
    const api = port();
    const views: AgentView[] = [];
    const controller = createAgentRunController(
      "room-1",
      (view) => views.push(view),
      api,
    );
    await controller.initialize();
    await controller.invoke("Summarize");
    api.read.mockRejectedValueOnce({
      response: { status: 403, data: { code: "AGENT_ACCESS_DENIED" } },
    });
    await controller.retry();
    expect(views.at(-1)?.run).toBeNull();
    expect(views.at(-1)?.capabilities).toBeNull();
    expect(views.at(-1)?.error).toContain("access");
    await controller.retry();
    expect(api.invoke).toHaveBeenCalledTimes(1);
    expect(api.capabilities).toHaveBeenCalledTimes(2);
    controller.dispose();
  });

  it("recovers a failed capability fetch, including a null transport error", async () => {
    const api = port();
    api.capabilities.mockRejectedValueOnce(null);
    const views: AgentView[] = [];
    const controller = createAgentRunController(
      "room-1",
      (view) => views.push(view),
      api,
    );
    await controller.initialize();
    await controller.retry();
    expect(views.at(-1)?.capabilities?.enabled).toBe(true);
    expect(views.at(-1)?.error).toBeNull();
    controller.dispose();
  });

  it("ignores late results from a disposed room and releases the observation signal", async () => {
    const api = port();
    let resolve!: (run: AgentRun) => void;
    api.invoke.mockImplementation(
      () =>
        new Promise<AgentRun>((done) => {
          resolve = done;
        }),
    );
    const emit = vi.fn();
    const controller = createAgentRunController("room-1", emit, api);
    const pending = controller.invoke("Summarize");
    const count = emit.mock.calls.length;
    controller.dispose();
    expect(api.invoke.mock.calls[0][2].aborted).toBe(true);
    resolve(completed);
    await pending;
    expect(emit).toHaveBeenCalledTimes(count);
    expect(api.read).not.toHaveBeenCalled();
  });
});
