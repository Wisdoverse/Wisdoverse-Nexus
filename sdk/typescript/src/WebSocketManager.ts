import WebSocket from 'ws';
import type { ClientMessage, MessageHandler } from './types';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'reconnecting';
export type ConnectionListener = (state: ConnectionState, attempt?: number) => void;

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private messageHandler: MessageHandler | null = null;
  private reconnectAttempts = 0;
  private readonly maxReconnectAttempts = 10;
  private readonly initialDelay = 1000;
  private readonly maxDelay = 30000;
  private shouldReconnect = false;
  private url = '';
  private token = '';
  private roomId?: string;
  private authenticated = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private listeners: ConnectionListener[] = [];
  private queue: ClientMessage[] = [];
  private readonly maxQueueSize = 1000;
  private state: ConnectionState = 'disconnected';

  onConnectionChange(listener: ConnectionListener): void {
    this.listeners.push(listener);
    listener(this.state);
  }

  offConnectionChange(listener: ConnectionListener): void {
    this.listeners = this.listeners.filter((l) => l !== listener);
  }

  private emitState(): void {
    for (const l of this.listeners) {
      l(this.state, this.reconnectAttempts || undefined);
    }
  }

  connect(url: string, token: string, roomId?: string): void {
    this.close();
    this.roomId = roomId;
    this.url = url;
    this.token = token;
    this.shouldReconnect = true;
    this.reconnectAttempts = 0;
    this.doConnect();
  }

  private doConnect(): void {
    this.state = this.reconnectAttempts > 0 ? 'reconnecting' : 'connecting';
    this.emitState();

    this.authenticated = false;
    const socket = new WebSocket(this.url);
    this.ws = socket;

    socket.on('open', () => {
      if (this.ws !== socket) return;
      socket.send(JSON.stringify({ type: 'auth', token: `Bearer ${this.token}` }));
    });
    socket.on('message', (data: WebSocket.Data) => {
      if (this.ws !== socket) return;
      let message;
      try { message = JSON.parse(data.toString()); } catch { return; }
      if (message.type === 'auth_error' || message.type === 'auth_required') {
        this.shouldReconnect = false;
        this.queue = [];
        socket.close();
      } else if (message.type === 'auth_success') {
        if (this.roomId) socket.send(JSON.stringify({ type: 'join_room', room_id: this.roomId }));
        else this.markConnected();
      } else if (message.type === 'room_joined' && message.room_id === this.roomId) {
        this.markConnected();
      } else if (message.type === 'error' && !this.authenticated) {
        this.shouldReconnect = false;
        socket.close();
      }
      this.messageHandler?.(message);
    });
    socket.on('close', () => {
      if (this.ws !== socket) return;
      this.authenticated = false;
      this.state = 'disconnected';
      this.emitState();
      if (this.shouldReconnect && this.reconnectAttempts < this.maxReconnectAttempts) this.scheduleReconnect();
    });
    socket.on('error', () => socket.close());
  }

  private markConnected(): void {
    this.authenticated = true;
    this.reconnectAttempts = 0;
    this.state = 'connected';
    this.emitState();
    this.flushQueue();
  }

  send(message: ClientMessage): void {
    if (this.authenticated && this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      // Buffer message for later delivery
      if (this.queue.length < this.maxQueueSize) {
        this.queue.push(message);
      }
    }
  }

  private flushQueue(): void {
    while (this.queue.length > 0) {
      const msg = this.queue.shift()!;
      if (this.authenticated && this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify(msg));
      } else {
        this.queue.unshift(msg);
        break;
      }
    }
  }

  onMessage(handler: MessageHandler): void {
    this.messageHandler = handler;
  }

  close(): void {
    this.shouldReconnect = false;
    this.authenticated = false;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;
    this.state = 'disconnected';
    this.emitState();
    this.queue = [];
    this.ws?.close();
    this.ws = null;
  }

  isConnected(): boolean {
    return this.authenticated && this.ws?.readyState === WebSocket.OPEN;
  }

  getState(): ConnectionState {
    return this.state;
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++;
    // Exponential backoff with ±20% jitter
    const base = Math.min(this.initialDelay * Math.pow(2, this.reconnectAttempts - 1), this.maxDelay);
    const jitter = base * 0.2 * (Math.random() * 2 - 1);
    const delay = Math.round(base + jitter);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.shouldReconnect) {
        this.doConnect();
      }
    }, delay);
  }
}
