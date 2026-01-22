import { type Component } from 'solid-js';
import * as solidJs from 'solid-js';
import * as solidJsWeb from 'solid-js/web';
import * as arkUiSolid from '@ark-ui/solid';

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

// hacky but it works 🤷 
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
    
    console.log('Shared scope initialized:', scope);
    isSharedScopeInitialized = true;
  }
}

export async function loadPlugin(
  pluginName: string,
  remoteUrl: string,
  moduleName: string
): Promise<Component> {
  console.log('Loading plugin:', pluginName, 'from', remoteUrl);
  
  await initSharedScope();

  const scriptId = `remote-${pluginName}`;
  
  if (!document.getElementById(scriptId)) {
    await new Promise<void>((resolve, reject) => {
      const script = document.createElement('script');
      script.id = scriptId;
      script.src = remoteUrl;
      script.onload = () => {
        console.log('Remote entry loaded:', remoteUrl);
        resolve();
      };
      script.onerror = () => reject(new Error(`Failed to load plugin: ${pluginName}`));
      document.head.appendChild(script);
    });
  }

  const container = (window as any)[pluginName] as Container;
  
  if (!container) {
    console.error('Available window properties:', Object.keys(window));
    throw new Error(`Plugin ${pluginName} not found on window object`);
  }

  console.log('Container found, initializing with shared scope');
  await container.init(window.__webpack_share_scopes__.default);

  console.log('Getting module:', moduleName);
  const factory = await container.get(moduleName);
  const Module = factory();
  
  console.log('Module loaded:', Module);
  return Module.default;
}
