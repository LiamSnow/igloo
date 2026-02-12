import type { Component, JSX } from 'solid-js';
import type { WatchQuery, OneShotQuery } from './queries';
import type { WatchUpdate } from './messages';

export interface PluginAPI {
  sub(query: WatchQuery, callback: (update: WatchUpdate) => void): number;
  eval(query: OneShotQuery): Promise<any>;
}

export interface PluginElementProps {
  api: PluginAPI;
  body?: JSX.Element;
  [key: string]: any;
}

export type PluginComponent = Component<PluginElementProps>;
