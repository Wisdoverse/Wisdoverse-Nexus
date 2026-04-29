// Auth Types
export interface NexisConfig {
  baseUrl: string;
  timeout?: number;
}

export interface RegisterData {
  email: string;
  password: string;
  displayName?: string;
}

export interface AuthResult {
  token: string;
  refreshToken: string;
  user: User;
}

export interface User {
  id: string;
  email: string;
  displayName: string;
  avatarUrl?: string;
  createdAt: string;
}

// Room Types
export interface CreateRoomData {
  name: string;
  description?: string;
  isPrivate?: boolean;
}

export interface Room {
  id: string;
  name: string;
  description?: string;
  isPrivate: boolean;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  members?: Member[];
}

export interface Member {
  id: string;
  userId: string;
  displayName: string;
  avatarUrl?: string;
  role: MemberRole;
  joinedAt: string;
}

export type MemberRole = 'owner' | 'admin' | 'member';

// Message Types
export interface Message {
  id: string;
  roomId: string;
  senderId: string;
  senderName: string;
  content: string;
  type: MessageType;
  createdAt: string;
  updatedAt?: string;
}

export type MessageType = 'text' | 'system' | 'file';

export interface PaginationOptions {
  limit?: number;
  before?: string;
  after?: string;
}

// WebSocket Types
export type EventType = 'message' | 'member_join' | 'member_leave' | 'room_update' | 'error' | 'close';

export interface ServerMessage {
  type: string;
  data: unknown;
}

export interface ClientMessage {
  type: string;
  data?: unknown;
}

export interface EventHandler {
  (event: ServerMessage): void;
}

export interface MessageHandler {
  (message: ServerMessage): void;
}
