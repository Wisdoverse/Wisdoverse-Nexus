import { View } from "react-native";
import { MessageStreamScreen } from "../../features/messages";
import { AgentPanel } from "../../features/agent";
import type { MessageStreamRouteProp } from "../../shared/navigation";

export function RoomScreen({ route }: { route: MessageStreamRouteProp }) {
  return (
    <View style={{ flex: 1 }}>
      <MessageStreamScreen route={route} />
      <AgentPanel roomId={route.params.roomId} />
    </View>
  );
}
