import { defineConfig } from '@rsbuild/core';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginSolid } from '@rsbuild/plugin-solid';

export default defineConfig({
  plugins: [
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
    pluginSolid(),
  ],
  
  moduleFederation: {
    options: {
      name: 'dashboard_core',
      shared: {
        'solid-js': {
          singleton: true,
          requiredVersion: '^1.9.10',
          eager: true
        },
        '@ark-ui/solid': {
          singleton: true,
          eager: true
        },
      },
    },
  },
});
