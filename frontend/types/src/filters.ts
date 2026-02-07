import type { DeviceID, GroupID, ExtensionID, EntityID } from './ids';

export type IDFilter<T> =
  | "Any"
  | { Is: T }
  | { OneOf: T[] };

export type DeviceGroupFilter =
  | "Any"
  | { In: GroupID }
  | { InAny: GroupID[] }
  | { InAll: GroupID[] };

export type EntityIDFilter =
  | "Any"
  | { Is: EntityID }
  | { OneOf: EntityID[] }
  | { Matches: string };  // glob pattern

export interface DeviceFilter {
  id: IDFilter<DeviceID>;
  owner: IDFilter<ExtensionID>;
  group: DeviceGroupFilter;
}

export interface EntityFilter {
  id: EntityIDFilter;
  // TODO FIXME
  type_filter: any | null;
}
