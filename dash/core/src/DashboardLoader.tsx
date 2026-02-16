import { type Component, createSignal, createResource, Show, For, onMount } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import type { DashboardConfig, DashboardElement, PluginAPI } from '@igloo/types';
import { loadPlugin } from './loadPlugin';

interface DashboardLoaderProps {
  dashboardId: string;
  api: PluginAPI;
}

const moduleCache = new Map<string, Component>();

async function fetchDashboard(id: string): Promise<DashboardConfig> {
  const response = await fetch(`/dashboards/${id}.json`);

  if (!response.ok) {
    throw new Error(`Failed to load dashboard: ${response.status} ${response.statusText}`);
  }

  return response.json();
}

async function loadModule(element: DashboardElement): Promise<Component> {
  const cacheKey = element.plugin
    ? `${element.plugin}::${element.module}`
    : `core::${element.module}`;

  if (moduleCache.has(cacheKey)) {
    return moduleCache.get(cacheKey)!;
  }

  let component: Component;

  if (element.plugin) {
    const pluginContainer = await loadPlugin(
      element.plugin,
      `/plugins/${element.plugin}/remoteEntry.js`,
      element.module
    );
    component = pluginContainer;
  } else {
    const module = await import(`./elements/${element.module}.tsx`);
    component = module.default;
  }

  moduleCache.set(cacheKey, component);

  return component;
}

const ElementRenderer: Component<{
  element: DashboardElement;
  api: PluginAPI;
}> = (props) => {
  const [component, setComponent] = createSignal<Component | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      const comp = await loadModule(props.element);
      setComponent(() => comp);
    } catch (err) {
      console.error('Failed to load module:', props.element, err);
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  });

  return (
    <Show when={!loading()} fallback={<div>Loading module...</div>}>
      <Show when={!error()} fallback={<div style={{ color: 'red' }}>Error: {error()}</div>}>
        <Show when={component()}>
          <Dynamic
            component={component()!}
            api={props.api}
            {...props.element.props}
            body={
              props.element.body ? (
                <For each={props.element.body}>
                  {(childElement) => (
                    <ElementRenderer
                      element={childElement}
                      api={props.api}
                    />
                  )}
                </For>
              ) : undefined
            }
          />
        </Show>
      </Show>
    </Show>
  );
};

export const DashboardLoader: Component<DashboardLoaderProps> = (props) => {
  const [dashboard] = createResource(
    () => props.dashboardId,
    fetchDashboard
  );

  return (
    <Show
      when={!dashboard.loading}
      fallback={<div>Loading dashboard...</div>}
    >
      <Show
        when={!dashboard.error}
        fallback={<div style={{ color: 'red' }}>Error loading dashboard: {dashboard.error?.message}</div>}
      >
        <Show when={dashboard()}>
          <div>
            <h1>{dashboard()!.name}</h1>
            <For each={dashboard()!.elements}>
              {(element) => (
                <ElementRenderer
                  element={element}
                  api={props.api}
                />
              )}
            </For>
          </div>
        </Show>
      </Show>
    </Show>
  );
};
