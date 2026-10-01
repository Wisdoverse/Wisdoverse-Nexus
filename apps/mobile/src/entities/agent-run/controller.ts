import {
  agentApi,
  type AgentRun,
  type AgentCapabilities,
  type InvokeAgentOptions,
} from "../../shared/api/agentApi";
export type { AgentRun } from "../../shared/api/agentApi";
export interface AgentView {
  capabilities: AgentCapabilities | null;
  run: AgentRun | null;
  busy: boolean;
  error: string | null;
}
export const initialAgentView: AgentView = {
  capabilities: null,
  run: null,
  busy: false,
  error: null,
};
export const terminalRun = (run: AgentRun) =>
  ["completed", "failed", "cancelled"].includes(run.status);
const messages: Record<string, string> = {
  AGENT_DISABLED: "The assistant is not enabled for this workspace.",
  AGENT_ACCESS_DENIED: "You no longer have access to this room.",
  AGENT_CAPACITY_EXCEEDED: "The assistant is busy. Try again shortly.",
  AGENT_BUDGET_EXCEEDED: "This run reached its output or context limit.",
  AGENT_PROVIDER_FAILED: "The model could not complete this request.",
  AGENT_TIMEOUT: "The assistant reached its time limit.",
  INVALID_AGENT_REQUEST: "Check the question and request limits.",
  AGENT_UNSUPPORTED_TENANCY:
    "The assistant is unavailable in this deployment mode.",
  AGENT_RETRY_CONFLICT: "This retry differs from the original request.",
  AGENT_TOOL_DENIED: "This tool is not permitted.",
};
function sleep(signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const abort = () => {
      clearTimeout(timer);
      signal.removeEventListener("abort", abort);
      reject(new Error("Observation stopped"));
    };
    const timer = setTimeout(() => {
      signal.removeEventListener("abort", abort);
      resolve();
    }, 200);
    signal.addEventListener("abort", abort, { once: true });
    if (signal.aborted) abort();
  });
}
export function createAgentRunController(
  roomId: string,
  emit: (view: AgentView) => void,
  api = agentApi,
) {
  let view: AgentView = { ...initialAgentView };
  let revision = 0;
  let disposed = false;
  let active: AbortController | null = null;
  let lastRequest: InvokeAgentOptions | null = null;
  const publish = (patch: Partial<AgentView>) => {
    if (!disposed) {
      view = { ...view, ...patch };
      emit(view);
    }
  };
  const begin = () => {
    active?.abort();
    active = new AbortController();
    return { stamp: ++revision, signal: active.signal };
  };
  const current = (stamp: number) => !disposed && revision === stamp;
  function fail(error: unknown, stamp: number) {
    if (!current(stamp)) return;
    const response = (
      error as {
        response?: { status?: number; data?: { code?: string } };
      } | null
    )?.response;
    const denied = response?.status === 401 || response?.status === 403;
    if (denied) lastRequest = null;
    publish({
      busy: false,
      error:
        messages[response?.data?.code ?? ""] ??
        (denied
          ? "Your session or room access has changed."
          : "Connection interrupted. Retry to recover the same request."),
      ...(denied ? { run: null, capabilities: null } : {}),
    });
  }
  async function watch(runId: string, stamp: number, signal: AbortSignal) {
    while (current(stamp)) {
      const run = await api.read(roomId, runId, signal);
      if (!current(stamp)) return;
      publish({
        run,
        busy: !terminalRun(run),
        error: run.errorCode
          ? (messages[run.errorCode] ??
            "The assistant could not complete this request.")
          : null,
      });
      if (terminalRun(run)) return;
      await sleep(signal);
    }
  }
  async function start(request: InvokeAgentOptions) {
    const { stamp, signal } = begin();
    lastRequest = request;
    publish({ busy: true, error: null, run: null });
    try {
      const run = await api.invoke(roomId, request, signal);
      if (!current(stamp)) return;
      publish({ run });
      await watch(run.id, stamp, signal);
    } catch (error) {
      fail(error, stamp);
    }
  }
  async function initialize() {
    const { stamp, signal } = begin();
    publish({ busy: true, error: null });
    try {
      const capabilities = await api.capabilities(roomId, signal);
      if (current(stamp)) publish({ capabilities, busy: false });
    } catch (error) {
      fail(error, stamp);
    }
  }
  return {
    initialize,
    async invoke(prompt: string) {
      await start({
        clientRunId: `client_${Date.now()}_${Math.random().toString(36).slice(2)}_${Math.random().toString(36).slice(2)}`,
        prompt,
        sourceMessageIds: [],
        tool: "room_history",
        maxOutputTokens: 1024,
        deadlineMs: 60000,
      });
    },
    async retry() {
      if (!view.run && lastRequest) {
        await start(lastRequest);
        return;
      }
      if (view.run) {
        const id = view.run.id;
        const { stamp, signal } = begin();
        publish({ busy: true, error: null });
        try {
          await watch(id, stamp, signal);
        } catch (error) {
          fail(error, stamp);
        }
      } else {
        await initialize();
      }
    },
    async cancel() {
      if (!view.run) return;
      const id = view.run.id;
      const { stamp, signal } = begin();
      try {
        const run = await api.cancel(roomId, id, signal);
        if (current(stamp)) publish({ run, busy: false, error: null });
      } catch (error) {
        fail(error, stamp);
      }
    },
    dispose() {
      disposed = true;
      revision++;
      active?.abort();
    },
  };
}
