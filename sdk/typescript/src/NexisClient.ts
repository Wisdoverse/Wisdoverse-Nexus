import { randomUUID } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import axios, { type AxiosInstance } from 'axios';
import { WebSocketManager } from './WebSocketManager';
import type { NexisConfig, AuthResult, CreateRoomData, Room, Message, PaginationOptions, EventType, EventHandler, ServerMessage } from './types';
import type { AgentRun, AgentRunEvent, AgentEventBatch, AgentCapabilities, InvokeAgentOptions } from './agentTypes';

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
  async agentCapabilities(roomId: string): Promise<AgentCapabilities> {
    this.requireAuth();
    return (await this.http.get<AgentCapabilities>(`/v1/rooms/${encodeURIComponent(roomId)}/agents`)).data;
  }
  async invokeAgent(roomId: string, prompt: string, options: Partial<Omit<InvokeAgentOptions, 'prompt'>> = {}): Promise<AgentRun> {
    this.requireAuth();
    const command: InvokeAgentOptions = { clientRunId: options.clientRunId ?? randomUUID(), prompt, sourceMessageIds: options.sourceMessageIds ?? [], tool: options.tool === undefined ? 'room_history' : options.tool, maxOutputTokens: options.maxOutputTokens ?? 1024, deadlineMs: options.deadlineMs ?? 60000 };
    return (await this.http.post<AgentRun>(`/v1/rooms/${encodeURIComponent(roomId)}/agent-runs`, command)).data;
  }
  async getAgentRun(roomId: string, runId: string): Promise<AgentRun> {
    this.requireAuth();
    return (await this.http.get<AgentRun>(`/v1/rooms/${encodeURIComponent(roomId)}/agent-runs/${encodeURIComponent(runId)}`)).data;
  }
  async getAgentEvents(roomId: string, runId: string, after = 0, signal?: AbortSignal): Promise<AgentEventBatch> {
    this.requireAuth();
    return (await this.http.get<AgentEventBatch>(`/v1/rooms/${encodeURIComponent(roomId)}/agent-runs/${encodeURIComponent(runId)}/events`, { params: { after }, signal })).data;
  }
  async cancelAgentRun(roomId: string, runId: string): Promise<AgentRun> {
    this.requireAuth();
    return (await this.http.post<AgentRun>(`/v1/rooms/${encodeURIComponent(roomId)}/agent-runs/${encodeURIComponent(runId)}/cancel`)).data;
  }
  async *observeAgentRun(roomId: string, runId: string, options: { timeoutMs?: number; pollMs?: number; signal?: AbortSignal } = {}): AsyncGenerator<AgentRunEvent> {
    const timeoutMs = options.timeoutMs ?? 65000;
    const pollMs = options.pollMs ?? 200;
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 2_147_483_647 || !Number.isFinite(pollMs) || pollMs < 10) throw new Error('Invalid observation budget');
    const timeout = AbortSignal.timeout(timeoutMs);
    const signal = options.signal ? AbortSignal.any([options.signal, timeout]) : timeout;
    let after = 0;
    while (true) {
      const batch = await this.getAgentEvents(roomId, runId, after, signal);
      for (const event of batch.events) { if (event.sequence > after) { after = event.sequence; yield event; } }
      if (['completed', 'failed', 'cancelled'].includes(batch.status)) return;
      await delay(pollMs, undefined, { signal });
    }
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
