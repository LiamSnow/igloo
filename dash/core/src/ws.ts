import type {
  ClientMsg,
  IglooResponse,
  WatchQuery,
  OneShotQuery,
  WatchUpdate,
  PluginAPI
} from '@igloo/types';

export class WebSocketManager implements PluginAPI {
  private ws: WebSocket | null = null;
  private clientId: number | null = null;
  private nextQueryId: number = 0;
  private subCallbacks: Map<number, (update: WatchUpdate) => void> = new Map();
  private pendingEvals: Map<number, { resolve: (value: any) => void; reject: (reason: any) => void }> = new Map();
  private connected: boolean = false;
  private url: string = '';
  private reconnectAttempts: number = 0;
  private maxReconnectDelay: number = 30000;
  private reconnectTimeoutId: number | null = null;
  private metadataQueryId: number | null = null;
  private metadataCallback: ((update: WatchUpdate) => void) | null = null;

  async connect(url: string, onMetadata?: (update: WatchUpdate) => void): Promise<void> {
    this.url = url;
    this.metadataCallback = onMetadata || null;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
          console.log('[WebSocket] Connected to', url);
          this.reconnectAttempts = 0;
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event);

          if (this.clientId !== null && this.metadataQueryId !== null) {
            resolve();
          }
        };

        this.ws.onerror = (error) => {
          console.error('[WebSocket] Error:', error);
          reject(error);
        };

        this.ws.onclose = (event) => {
          console.log('[WebSocket] Connection closed:', event.code, event.reason);
          this.connected = false;
          this.clientId = null;
          this.metadataQueryId = null;
          this.attemptReconnect();
        };
      } catch (error) {
        console.error('[WebSocket] Failed to create connection:', error);
        reject(error);
      }
    });
  }

  private attemptReconnect(): void {
    if (this.reconnectTimeoutId !== null) {
      return;
    }

    this.reconnectAttempts++;

    const delay = Math.min(
      1000 * Math.pow(2, this.reconnectAttempts - 1),
      this.maxReconnectDelay
    );

    console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})...`);

    this.reconnectTimeoutId = window.setTimeout(async () => {
      this.reconnectTimeoutId = null;

      try {
        await this.connect(this.url, this.metadataCallback || undefined);
        console.log('[WebSocket] Reconnected successfully');
      } catch (error) {
        console.error('[WebSocket] Reconnect failed:', error);
      }
    }, delay);
  }

  private subscribeToMetadata(): void {
    if (this.metadataQueryId !== null) {
      console.warn('[WebSocket] Already subscribed to metadata');
      return;
    }

    const queryId = this.nextQueryId++;
    this.metadataQueryId = queryId;

    this.subCallbacks.set(queryId, (update) => {
      console.log('[WebSocket] Metadata update:', update);
      if (this.metadataCallback) {
        try {
          this.metadataCallback(update);
        } catch (error) {
          console.error('[WebSocket] Metadata callback threw error:', error);
        }
      }
    });

    const msg: ClientMsg = {
      Sub: {
        query_id: queryId,
        query: "Metadata"
      }
    };

    this.send(msg);
    console.log('[WebSocket] Subscribed to Metadata with query_id:', queryId);
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const msg: IglooResponse = JSON.parse(event.data);

      if ('Registered' in msg) {
        this.handleRegistered(msg.Registered);
      } else if ('EvalResult' in msg) {
        this.handleEvalResult(msg.EvalResult);
      } else if ('WatchUpdate' in msg) {
        this.handleWatchUpdate(msg.WatchUpdate);
      }
    } catch (error) {
      console.error('[WebSocket] Failed to parse message:', error);
    }
  }

  private handleRegistered({ client_id }: { client_id: number }): void {
    this.clientId = client_id;
    this.connected = true;
    console.log('[WebSocket] Registered with client_id:', client_id);

    this.subscribeToMetadata();
  }

  private handleEvalResult({ query_id, result }: { query_id: number; result: { Ok: any } | { Err: string } }): void {
    const pending = this.pendingEvals.get(query_id);

    if (!pending) {
      console.warn('[WebSocket] Received EvalResult for unknown query_id:', query_id);
      return;
    }

    this.pendingEvals.delete(query_id);

    if ('Ok' in result) {
      pending.resolve(result.Ok);
    } else {
      console.error('[WebSocket] Eval query failed:', result.Err);
      pending.reject(new Error(result.Err));
    }
  }

  private handleWatchUpdate({ query_id, value }: { query_id: number; value: WatchUpdate }): void {
    const callback = this.subCallbacks.get(query_id);

    if (!callback) {
      console.warn('[WebSocket] Received WatchUpdate for unknown query_id:', query_id);
      return;
    }

    try {
      callback(value);
    } catch (error) {
      console.error('[WebSocket] Subscription callback threw error:', error);
    }
  }

  private send(msg: ClientMsg): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.error('[WebSocket] Cannot send message: not connected');
      throw new Error('WebSocket not connected');
    }

    this.ws.send(JSON.stringify(msg));
  }

  sub(query: WatchQuery, callback: (update: WatchUpdate) => void): number {
    const queryId = this.nextQueryId++;

    this.subCallbacks.set(queryId, callback);

    const msg: ClientMsg = {
      Sub: {
        query_id: queryId,
        query
      }
    };

    this.send(msg);
    console.log('[WebSocket] Subscribed to query:', queryId);

    return queryId;
  }

  eval(query: OneShotQuery): Promise<any> {
    const queryId = this.nextQueryId++;

    const promise = new Promise((resolve, reject) => {
      this.pendingEvals.set(queryId, { resolve, reject });
    });

    const msg: ClientMsg = {
      Eval: {
        query_id: queryId,
        query
      }
    };

    this.send(msg);
    console.log('[WebSocket] Sent eval query:', queryId);

    return promise;
  }

  unsubAll(): void {
    const msg: ClientMsg = "UnsubAll";
    this.send(msg);

    this.subCallbacks.clear();
    this.metadataQueryId = null;

    console.log('[WebSocket] Unsubscribed from all queries');

    this.subscribeToMetadata();
  }

  disconnect(): void {
    if (this.reconnectTimeoutId !== null) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.connected = false;
    this.clientId = null;
    this.metadataQueryId = null;
    this.subCallbacks.clear();
    this.pendingEvals.clear();
  }

  isConnected(): boolean {
    return this.connected && this.ws?.readyState === WebSocket.OPEN;
  }
}
