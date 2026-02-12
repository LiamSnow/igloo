import { type Component } from 'solid-js';
import * as solidJs from 'solid-js';
import * as solidJsWeb from 'solid-js/web';
import * as arkUiSolid from '@ark-ui/solid';
import * as iglooTypes from '@igloo/types';

interface Container {
  init(shareScope: any): Promise<void>;
  get(module: string): Promise<() => { default: Component }>;
}

declare global {
  interface Window {
    __webpack_share_scopes__: { default: any };
    __webpack_init_sharing__: (scope: string) => Promise<void>;
  }
}

let isSharedScopeInitialized = false;
const containerCache = new Map<string, Container>();

async function initSharedScope() {
  if (!isSharedScopeInitialized) {
    if (!window.__webpack_share_scopes__) {
      window.__webpack_share_scopes__ = { default: {} };
    }

    const scope = window.__webpack_share_scopes__.default;

    scope['solid-js'] = {
      '1.9.10': {
        get: () => Promise.resolve(() => solidJs),
        loaded: true,
        eager: true,
      },
    };

    scope['solid-js/web'] = {
      '1.9.10': {
        get: () => Promise.resolve(() => solidJsWeb),
        loaded: true,
        eager: true,
      },
    };

    scope['@ark-ui/solid'] = {
      '5.30.0': {
        get: () => Promise.resolve(() => arkUiSolid),
        loaded: true,
        eager: true,
      },
    };

    scope['@igloo/types'] = {
      '1.0.0': {
        get: () => Promise.resolve(() => iglooTypes),
        loaded: true,
        eager: true,
      },
    };

    console.log('Shared scope initialized:', scope);
    isSharedScopeInitialized = true;
  }
}

async function loadPluginContainer(pluginName: string, remoteUrl: string): Promise<Container> {
  if (containerCache.has(pluginName)) {
    return containerCache.get(pluginName)!;
  }

  await initSharedScope();

  const scriptId = `remote-${pluginName}`;

  // load `remoteEntry.js`
  if (!document.getElementById(scriptId)) {
    await new Promise<void>((resolve, reject) => {
      const script = document.createElement('script');
      script.id = scriptId;
      script.src = remoteUrl;
      script.onload = () => {
        console.log('[Plugin] Remote entry loaded:', remoteUrl);
        resolve();
      };
      script.onerror = () => reject(new Error(`Failed to load plugin: ${pluginName}`));
      document.head.appendChild(script);
    });
  }

  const container = (window as any)[pluginName] as Container;

  if (!container) {
    throw new Error(`Plugin ${pluginName} not found on window object`);
  }

  console.log('[Plugin] Initializing container:', pluginName);
  await container.init(window.__webpack_share_scopes__.default);

  containerCache.set(pluginName, container);

  return container;
}

export async function loadPlugin(
  pluginName: string,
  remoteUrl: string,
  moduleName: string
): Promise<Component> {
  console.log('[Plugin] Loading module:', moduleName, 'from', pluginName);

  const container = await loadPluginContainer(pluginName, remoteUrl);

  const factory = await container.get(moduleName);
  const Module = factory();

  console.log('[Plugin] Module loaded:', moduleName);
  return Module.default;
}
