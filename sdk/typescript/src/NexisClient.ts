import { randomUUID } from 'node:crypto';
import axios, { type AxiosInstance } from 'axios';
import { WebSocketManager } from './WebSocketManager';
import type { NexisConfig, AuthResult, CreateRoomData, Room, Message, PaginationOptions, EventType, EventHandler, ServerMessage } from './types';

export class NexisClient {
  private readonly http: AxiosInstance;
  private token: string | null = null;
  private readonly wsManager = new WebSocketManager();
  private readonly eventHandlers = new Map<EventType, Set<EventHandler>>();

  constructor(config: NexisConfig) {
    this.http = axios.create({ baseURL: config.baseUrl.replace(/\/$/, ''), timeout: config.timeout ?? 30000 });
    this.wsManager.onMessage((message) => this.dispatchMessage(message));
  }

  /** Verify an externally issued JWT before making it the active session. */
  async authenticate(token: string): Promise<AuthResult> {
    const { data } = await this.http.get<Omit<AuthResult, 'token'>>('/v1/auth/session', { headers: { Authorization: `Bearer ${token}` } });
    this.disconnect();
    this.token = token;
    this.http.defaults.headers.common.Authorization = `Bearer ${token}`;
    return { ...data, token };
  }

  async createRoom(room: CreateRoomData): Promise<Room> {
    this.requireAuth();
    return (await this.http.post<Room>('/v1/rooms', room)).data;
  }
  async getRoom(roomId: string): Promise<Room> {
    this.requireAuth();
    return (await this.http.get<Room>(`/v1/rooms/${encodeURIComponent(roomId)}`)).data;
  }
  async listRooms(options?: PaginationOptions): Promise<Room[]> {
    this.requireAuth();
    return (await this.http.get<{ rooms: Room[]; total: number }>('/v1/rooms', { params: options })).data.rooms;
  }
  async deleteRoom(roomId: string): Promise<void> {
    this.requireAuth();
    await this.http.delete(`/v1/rooms/${encodeURIComponent(roomId)}`);
  }
  async inviteMember(roomId: string, memberId: string): Promise<void> {
    this.requireAuth();
    await this.http.post(`/v1/rooms/${encodeURIComponent(roomId)}/invite`, { memberId });
  }
  async sendMessage(roomId: string, text: string, replyTo?: string, clientMessageId = randomUUID()): Promise<Message> {
    this.requireAuth();
    return (await this.http.post<Message>('/v1/messages', { roomId, text, replyTo, clientMessageId })).data;
  }
  async getMessages(roomId: string): Promise<Message[]> {
    this.requireAuth();
    return (await this.http.get<Message[]>(`/v1/rooms/${encodeURIComponent(roomId)}/messages`)).data;
  }
  connect(roomId: string): WebSocketManager {
    this.requireAuth();
    const url = new URL(this.http.defaults.baseURL!);
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    url.pathname = '/ws';
    url.search = '';
    this.wsManager.connect(url.toString(), this.token!, roomId);
    return this.wsManager;
  }
  disconnect(): void { this.wsManager.close(); }
  logout(): void {
    this.disconnect();
    this.token = null;
    delete this.http.defaults.headers.common.Authorization;
  }
  on(event: EventType, handler: EventHandler): void {
    if (!this.eventHandlers.has(event)) this.eventHandlers.set(event, new Set());
    this.eventHandlers.get(event)!.add(handler);
  }
  off(event: EventType, handler: EventHandler): void { this.eventHandlers.get(event)?.delete(handler); }
  private requireAuth(): void {
    if (!this.token) throw new Error('Not authenticated. Call authenticate(token) first.');
  }
  private dispatchMessage(message: ServerMessage): void {
    for (const handler of this.eventHandlers.get(message.type) ?? []) handler(message);
  }
}
