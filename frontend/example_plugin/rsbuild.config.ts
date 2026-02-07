import { defineConfig } from '@rsbuild/core';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginSolid } from '@rsbuild/plugin-solid';

export default defineConfig({
  mode: 'production',  

  plugins: [
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
    pluginSolid(),
  ],
  
  server: {
    port: 3001,
  },
  
  dev: {
    assetPrefix: true,
  },
  
  moduleFederation: {
    options: {
      name: 'example_plugin',
      filename: 'remoteEntry.js',
      exposes: {
        'DimmerSlider': './src/DimmerSlider.tsx',
      },
      shared: {
        'solid-js': {
          singleton: true,
          requiredVersion: '^1.9.10',
        },
        '@ark-ui/solid': {
          singleton: true,
        },
        '@igloo/types': {
          singleton: true,
        },
      },
    },
  },
  
  output: {
    assetPrefix: '/plugins/example_plugin/',
  },
});
