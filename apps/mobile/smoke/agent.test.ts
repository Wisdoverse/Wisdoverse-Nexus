import { expect, it } from "vitest";
import { agentApi } from "../src/shared/api/agentApi";
import {
  createAgentRunController,
  type AgentView,
} from "../src/entities/agent-run";
import { configureSessionAccess } from "../src/shared/session/sessionAccess";
import { roomsApi } from "../src/shared/api/endpoints/rooms";

it.skipIf(!process.env.NEXIS_AGENT_SMOKE)(
  "explicitly invokes and observes a scoped run through the actual frontend API/controller",
  async () => {
    const token = process.env.NEXIS_SMOKE_TOKEN!;
    configureSessionAccess({
      getSnapshot: () => ({
        token,
        tenantId: null,
        isAuthenticated: true,
        needsRefresh: () => false,
      }),
      updateSession: () => {},
      logout: () => {},
    });
    const shared = await agentApi.read(
      process.env.NEXIS_AGENT_SHARED_ROOM!,
      process.env.NEXIS_AGENT_SHARED_RUN!,
      new AbortController().signal,
    );
    expect(shared.answer).toBe("Synthetic response 🦀");
    const { data: room } = await roomsApi.create(
      "frontend-agent-smoke",
      "synthetic",
    );
    const views: AgentView[] = [];
    const controller = createAgentRunController(room.id, (view) =>
      views.push(view),
    );
    try {
      await controller.initialize();
      expect(views.at(-1)?.capabilities?.enabled).toBe(true);
      await controller.invoke("Summarize this room");
      expect(views.at(-1)?.run?.status).toBe("completed");
      expect(views.at(-1)?.run?.answer).toBe(shared.answer);
      expect(views.at(-1)?.run?.usage.outputTokens).toBe(8);
      expect(views.at(-1)?.run?.traceId).toBeTruthy();
      expect(views.at(-1)?.busy).toBe(false);
      await controller.cancel();
      expect(views.at(-1)?.run?.status).toBe("completed");
    } finally {
      controller.dispose();
    }
  },
);
