import { useEffect, useRef, useState } from "react";
import {
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import {
  createAgentRunController,
  initialAgentView,
  terminalRun,
  type AgentView,
} from "../../entities/agent-run";

export function AgentPanel({ roomId }: { roomId: string }) {
  const [view, setView] = useState<AgentView>(initialAgentView);
  const [expanded, setExpanded] = useState(false);
  const [prompt, setPrompt] = useState("");
  const controller = useRef<ReturnType<typeof createAgentRunController> | null>(
    null,
  );
  useEffect(() => {
    setView(initialAgentView);
    setPrompt("");
    setExpanded(false);
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
    <View style={styles.panel}>
      <Pressable
        accessibilityRole="button"
        accessibilityState={{ expanded }}
        onPress={() => setExpanded(!expanded)}
      >
        <Text style={styles.title}>Room Assistant {expanded ? "−" : "+"}</Text>
      </Pressable>
      {expanded && (
        <ScrollView style={styles.content} keyboardShouldPersistTaps="handled">
          <Text>
            {view.capabilities?.disclosure ??
              "Ask for help using this room’s messages. Responses are read-only."}
          </Text>
          {view.capabilities && !view.capabilities.enabled && (
            <Text accessibilityLiveRegion="polite">
              The assistant is not enabled for this workspace.
            </Text>
          )}
          <TextInput
            accessibilityLabel="Your question for Room Assistant"
            placeholder="Your question"
            multiline
            maxLength={4096}
            value={prompt}
            onChangeText={setPrompt}
            editable={!view.busy}
            style={styles.input}
          />
          <Pressable
            accessibilityRole="button"
            accessibilityState={{
              disabled: !view.capabilities?.enabled || view.busy || !!running,
            }}
            disabled={
              !view.capabilities?.enabled ||
              view.busy ||
              !!running ||
              !prompt.trim()
            }
            onPress={() => void controller.current?.invoke(prompt.trim())}
            style={styles.button}
          >
            <Text style={styles.buttonText}>Ask Room Assistant</Text>
          </Pressable>
          {view.error && (
            <View>
              <Text accessibilityRole="alert">{view.error}</Text>
              <Pressable
                accessibilityRole="button"
                onPress={() => void controller.current?.retry()}
              >
                <Text>Retry</Text>
              </Pressable>
            </View>
          )}
          {view.run && (
            <View>
              <Text accessibilityLiveRegion="polite">
                {view.run.agentName}: {view.run.status}
              </Text>
              <Text selectable>{view.run.answer}</Text>
              {view.run.sourceMessageIds.length > 0 && (
                <Text selectable>
                  Source messages: {view.run.sourceMessageIds.join(", ")}
                </Text>
              )}
              {running && (
                <Pressable
                  accessibilityRole="button"
                  onPress={() => void controller.current?.cancel()}
                >
                  <Text>Cancel request</Text>
                </Pressable>
              )}
              {view.run.usage.outputTokens !== null && (
                <Text>{view.run.usage.outputTokens} output tokens</Text>
              )}
            </View>
          )}
        </ScrollView>
      )}
    </View>
  );
}
const styles = StyleSheet.create({
  panel: {
    padding: 12,
    borderTopWidth: 1,
    borderColor: "#CBD5E1",
    backgroundColor: "#FFFFFF",
  },
  title: { fontWeight: "700", fontSize: 16 },
  content: { maxHeight: 260, marginTop: 12 },
  input: {
    padding: 10,
    borderWidth: 1,
    borderColor: "#94A3B8",
    borderRadius: 8,
    marginVertical: 10,
    minHeight: 60,
  },
  button: {
    backgroundColor: "#0b5fff",
    padding: 10,
    borderRadius: 8,
    marginBottom: 10,
  },
  buttonText: { color: "#FFFFFF", fontWeight: "600" },
});
