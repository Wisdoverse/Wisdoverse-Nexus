import WebSocket from 'ws';
import type { ClientMessage, MessageHandler } from './types';

export type ConnectionState = 'connecting' | 'connected' | 'disconnecting' | 'reconnecting';
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
  private listeners: ConnectionListener[] = [];
  private queue: ClientMessage[] = [];
  private readonly maxQueueSize = 1000;
  private state: ConnectionState = 'disconnecting';

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

  connect(url: string, token: string): void {
    this.url = url;
    this.token = token;
    this.shouldReconnect = true;
    this.reconnectAttempts = 0;
    this.doConnect();
  }

  private doConnect(): void {
    this.state = this.reconnectAttempts > 0 ? 'reconnecting' : 'connecting';
    this.emitState();

    const headers = this.token ? { Authorization: `Bearer ${this.token}` } : undefined;
    this.ws = new WebSocket(this.url, { headers } as WebSocket.ClientOptions);

    this.ws.on('open', () => {
      this.reconnectAttempts = 0;
      this.state = 'connected';
      this.emitState();
      this.flushQueue();
    });

    this.ws.on('message', (data: WebSocket.Data) => {
      try {
        const message = JSON.parse(data.toString());
        this.messageHandler?.(message);
      } catch {
        // ignore malformed
      }
    });

    this.ws.on('close', () => {
      this.state = 'disconnecting';
      this.emitState();
      if (this.shouldReconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
        this.scheduleReconnect();
      }
    });

    this.ws.on('error', () => {
      this.ws?.close();
    });
  }

  send(message: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
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
      if (this.ws?.readyState === WebSocket.OPEN) {
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
    this.state = 'disconnecting';
    this.emitState();
    this.queue = [];
    this.ws?.close();
    this.ws = null;
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
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

    setTimeout(() => {
      if (this.shouldReconnect) {
        this.doConnect();
      }
    }, delay);
  }
}
