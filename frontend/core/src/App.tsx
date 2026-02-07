import { createSignal, onMount, onCleanup, Show, type Component } from 'solid-js';
import { WebSocketManager } from './ws';
import { DashboardLoader } from './DashboardLoader';
import type { WatchUpdate } from '@igloo/types';

const App: Component = () => {
  const [wsManager] = createSignal(new WebSocketManager());
  const [connected, setConnected] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [connecting, setConnecting] = createSignal(true);

  onMount(async () => {
    try {
      const handleMetadata = (update: WatchUpdate) => {
        console.log('[App] Metadata update:', update);
      };

      await wsManager().connect('ws://localhost:4299/ws', handleMetadata);
      setConnected(true);
      setError(null);
    } catch (err) {
      console.error('[App] Failed to connect to WebSocket:', err);
      setError(err instanceof Error ? err.message : 'Connection failed');
    } finally {
      setConnecting(false);
    }
  });

  onCleanup(() => {
    wsManager().disconnect();
  });

  return (
    <div style={{ padding: '20px', 'font-family': 'system-ui, sans-serif' }}>
      <Show when={connecting()}>
        <div>Connecting to Igloo server...</div>
      </Show>

      <Show when={error()}>
        <div style={{
          padding: '16px',
          background: '#fee2e2',
          border: '1px solid #ef4444',
          'border-radius': '6px',
          color: '#991b1b',
        }}>
          Connection Error: {error()}
        </div>
      </Show>

      <Show when={connected()}>
        <DashboardLoader 
          dashboardId="main" 
          api={wsManager()} 
        />
      </Show>
    </div>
  );
};

export default App;
