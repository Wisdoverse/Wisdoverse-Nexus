import axios, { AxiosInstance } from 'axios';
import { WebSocketManager } from './WebSocketManager';
import type {
  NexisConfig,
  AuthResult,
  RegisterData,
  CreateRoomData,
  Room,
  Message,
  PaginationOptions,
  EventType,
  EventHandler,
  ServerMessage,
} from './types';

export class NexisClient {
  private http: AxiosInstance;
  private token: string | null = null;
  private wsManager: WebSocketManager;
  private eventHandlers = new Map<EventType, Set<EventHandler>>();

  constructor(config: NexisConfig) {
    this.http = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout ?? 30000,
      headers: { 'Content-Type': 'application/json' },
    });

    this.wsManager = new WebSocketManager();
    this.wsManager.onMessage((msg: ServerMessage) => this.dispatchMessage(msg));
  }

  // Auth
  async login(email: string, password: string): Promise<AuthResult> {
    const { data } = await this.http.post<AuthResult>('/v1/auth/login', { email, password });
    this.setAuth(data);
    return data;
  }

  async register(registerData: RegisterData): Promise<AuthResult> {
    const { data } = await this.http.post<AuthResult>('/v1/auth/register', registerData);
    this.setAuth(data);
    return data;
  }

  // Rooms
  async createRoom(roomData: CreateRoomData): Promise<Room> {
    this.requireAuth();
    const { data } = await this.http.post<Room>('/v1/rooms', roomData);
    return data;
  }

  async getRoom(roomId: string): Promise<Room> {
    this.requireAuth();
    const { data } = await this.http.get<Room>(`/v1/rooms/${roomId}`);
    return data;
  }

  async listRooms(): Promise<Room[]> {
    this.requireAuth();
    const { data } = await this.http.get<Room[]>('/v1/rooms');
    return data;
  }

  async joinRoom(roomId: string): Promise<void> {
    this.requireAuth();
    await this.http.post(`/v1/rooms/${roomId}/join`);
  }

  async leaveRoom(roomId: string): Promise<void> {
    this.requireAuth();
    await this.http.post(`/v1/rooms/${roomId}/leave`);
  }

  // Messages
  async sendMessage(roomId: string, content: string): Promise<Message> {
    this.requireAuth();
    const { data } = await this.http.post<Message>(`/v1/rooms/${roomId}/messages`, { content });
    return data;
  }

  async getMessages(roomId: string, options?: PaginationOptions): Promise<Message[]> {
    this.requireAuth();
    const params: Record<string, string | number> = {};
    if (options?.limit) params.limit = options.limit;
    if (options?.before) params.before = options.before;
    if (options?.after) params.after = options.after;

    const { data } = await this.http.get<Message[]>(`/v1/rooms/${roomId}/messages`, { params });
    return data;
  }

  // WebSocket
  connect(roomId: string): WebSocketManager {
    this.requireAuth();
    const wsUrl = this.http.defaults.baseURL!.replace(/^http/, 'ws') + `/ws?room_id=${roomId}`;
    this.wsManager.connect(wsUrl, this.token!);
    return this.wsManager;
  }

  disconnect(): void {
    this.wsManager.close();
  }

  on(event: EventType, handler: EventHandler): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);
  }

  off(event: EventType, handler: EventHandler): void {
    this.eventHandlers.get(event)?.delete(handler);
  }

  // Internal
  private setAuth(result: AuthResult): void {
    this.token = result.token;
    this.http.defaults.headers.common['Authorization'] = `Bearer ${result.token}`;
  }

  private requireAuth(): void {
    if (!this.token) {
      throw new Error('Not authenticated. Call login() or register() first.');
    }
  }

  private dispatchMessage(msg: ServerMessage): void {
    const handlers = this.eventHandlers.get(msg.type as EventType);
    if (handlers) {
      for (const handler of handlers) {
        handler(msg);
      }
    }
  }
}
