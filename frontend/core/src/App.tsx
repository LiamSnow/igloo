import { createSignal, Show, type Component } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { loadPlugin } from './loadPlugin';

const App: Component = () => {
  const [pluginComponent, setPluginComponent] = createSignal<Component | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const loadExamplePlugin = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const component = await loadPlugin(
        'example_plugin',
        '/plugins/example_plugin/remoteEntry.js',
        './Widget'
      );
      setPluginComponent(() => component);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load plugin');
      console.error('Plugin load error:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ padding: '40px', 'font-family': 'system-ui, sans-serif' }}>
      <h1>Example Dashboard</h1>
      
      <button
        onClick={loadExamplePlugin}
        disabled={loading()}
        style={{
          padding: '12px 24px',
          background: loading() ? '#9ca3af' : '#10b981',
          color: 'white',
          border: 'none',
          'border-radius': '6px',
          cursor: loading() ? 'not-allowed' : 'pointer',
          'font-size': '16px',
          margin: '20px 0'
        }}
      >
        {loading() ? 'Loading Plugin...' : 'Load Example Plugin'}
      </button>

      <Show when={error()}>
        <div style={{
          padding: '16px',
          background: '#fee2e2',
          border: '1px solid #ef4444',
          'border-radius': '6px',
          color: '#991b1b',
          margin: '20px 0'
        }}>
          Error: {error()}
        </div>
      </Show>

      <Show when={pluginComponent()}>
        <div style={{ 'margin-top': '20px' }}>
          <h2>Loaded Plugin:</h2>
          <Dynamic component={pluginComponent()!} />
        </div>
      </Show>
    </div>
  );
};

export default App;
