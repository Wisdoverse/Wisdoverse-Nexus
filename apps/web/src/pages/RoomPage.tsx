import { useParams } from "react-router-dom";
import { RoomDetailPage } from "../features/messages";
import { AgentPanel } from "../features/agent";
import styles from "./RoomPage.module.css";

export default function RoomPage() {
  const { roomId } = useParams<{ roomId: string }>();
  return (
    <div className={styles.layout}>
      <RoomDetailPage />
      {roomId && <AgentPanel roomId={roomId} />}
    </div>
  );
}
