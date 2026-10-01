import { useEffect, useRef, useState } from "react";
import {
  createAgentRunController,
  initialAgentView,
  terminalRun,
  type AgentView,
} from "../../entities/agent-run";
import styles from "./AgentPanel.module.css";

export function AgentPanel({ roomId }: { roomId: string }) {
  const [view, setView] = useState<AgentView>(initialAgentView);
  const [prompt, setPrompt] = useState("");
  const controller = useRef<ReturnType<typeof createAgentRunController> | null>(
    null,
  );
  useEffect(() => {
    setView(initialAgentView);
    setPrompt("");
    const active = createAgentRunController(roomId, setView);
    controller.current = active;
    void active.initialize();
    return () => {
      active.dispose();
      controller.current = null;
    };
  }, [roomId]);
  const running = view.run && !terminalRun(view.run);
  return (
    <aside className={styles.panel} aria-label="Room assistant">
      <h2>Room Assistant</h2>
      <p>
        {view.capabilities?.disclosure ??
          "Ask for help using this room’s messages. Responses are read-only."}
      </p>
      {view.capabilities && !view.capabilities.enabled && (
        <p role="status">The assistant is not enabled for this workspace.</p>
      )}
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (
            view.capabilities?.enabled &&
            !view.busy &&
            !running &&
            prompt.trim()
          )
            void controller.current?.invoke(prompt.trim());
        }}
      >
        <label htmlFor="agent-question">Your question</label>
        <textarea
          id="agent-question"
          value={prompt}
          maxLength={4096}
          onChange={(event) => setPrompt(event.target.value)}
          disabled={view.busy}
        />
        <button
          disabled={
            !view.capabilities?.enabled ||
            view.busy ||
            !!running ||
            !prompt.trim()
          }
        >
          Ask Room Assistant
        </button>
      </form>
      {view.error && (
        <div role="alert">
          <p>{view.error}</p>
          <button
            type="button"
            onClick={() => void controller.current?.retry()}
          >
            Retry
          </button>
        </div>
      )}
      {view.run && (
        <section aria-label="Assistant response" aria-busy={!!running}>
          <p role="status">
            {view.run.agentName}: {view.run.status}
          </p>
          <div className={styles.answer} aria-live="polite">
            {view.run.answer}
          </div>
          {view.run.sourceMessageIds.length > 0 && (
            <nav aria-label="Source messages">
              {view.run.sourceMessageIds.map((id, index) => (
                <a key={id} href={`#message-${id}`}>
                  Source {index + 1}
                </a>
              ))}
            </nav>
          )}
          {running && (
            <button
              type="button"
              onClick={() => void controller.current?.cancel()}
            >
              Cancel request
            </button>
          )}
          {view.run.usage.outputTokens !== null && (
            <p>
              {view.run.usage.outputTokens} output tokens ·{" "}
              {view.run.usage.toolCalls} read-only tool calls
            </p>
          )}
        </section>
      )}
    </aside>
  );
}
