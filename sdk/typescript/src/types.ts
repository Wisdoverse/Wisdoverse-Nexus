export interface NexisConfig { baseUrl: string; timeout?: number; }
export interface AuthResult { token: string; memberId: string; memberType: 'human' | 'ai'; expiresAt: number; refreshSupported: false; }
export interface CreateRoomData { name: string; topic?: string; }
export interface Room { id: string; name: string; topic?: string; messages?: Message[]; member_count?: number; }
export interface Message { id: string; roomId: string; sender: string; text: string; reply_to?: string; }
export interface PaginationOptions { limit?: number; offset?: number; }
export type ClientMessage =
  | { type: 'auth'; token: string }
  | { type: 'join_room' | 'leave_room'; room_id: string }
  | { type: 'send_message'; room_id: string; content: string; reply_to?: string; client_message_id?: string }
  | { type: 'heartbeat'; timestamp?: number };
export type ServerMessage =
  | { type: 'auth_success'; member_id: string; member_type: string }
  | { type: 'room_joined' | 'room_left'; room_id: string }
  | { type: 'new_message'; room_id: string; message_id: string; sender_id: string; content: string; reply_to?: string; timestamp: number }
  | { type: 'message_accepted'; room_id: string; message_id: string; client_message_id?: string }
  | { type: 'heartbeat_ack'; timestamp?: number }
  | { type: 'error' | 'auth_error' | 'auth_required'; message: string; code?: string };
export type EventType = ServerMessage['type'];
export type EventHandler = (event: ServerMessage) => void;
export type MessageHandler = EventHandler;
