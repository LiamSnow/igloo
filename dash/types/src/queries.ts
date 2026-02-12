import type { ComponentType } from './components';
import type { IglooValue } from './values';
import type { DeviceFilter, EntityFilter } from './filters';

export type WatchQuery =
  | "Metadata"
  | { Component: WatchComponentQuery };

export interface WatchComponentQuery {
  device_filter: DeviceFilter;
  entity_filter: EntityFilter;
  component: ComponentType;
  // TODO FIXME
  post_op: any | null;
}

export type OneShotQuery =
  | { Component: ComponentQuery };

export interface ComponentQuery {
  device_filter: DeviceFilter;
  entity_filter: EntityFilter;
  action: ComponentAction;
  component: ComponentType;
  post_op: any | null;
  include_parents: boolean;
  limit: number | null;
}

export type ComponentAction =
  | "GetValue"
  | { Set: IglooValue }
  | { Put: IglooValue };
