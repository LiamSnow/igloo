import type { OneShotQuery, WatchQuery } from './queries';
import type { DeviceID, EntityIndex } from './ids';
import type { IglooValue } from './values';

export type ClientMsg =
  | "Unregister"
  | "UnsubAll"
  | { Eval: { query_id: number; query: OneShotQuery } }
  | { Sub: { query_id: number; query: WatchQuery } };

export type IglooResponse =
  | { Registered: { client_id: number } }
  | { EvalResult: { query_id: number; result: { Ok: any } | { Err: string } } }
  | { WatchUpdate: { query_id: number; value: WatchUpdate } };

export type WatchUpdate =
  | { Metadata: MetadataUpdate[] }
  | { ComponentAggregate: IglooValue }
  | { ComponentValue: [DeviceID, EntityIndex, IglooValue] };

export type MetadataUpdate =
  | { Device: [DeviceID, DeviceMetadata] }
  | { DeviceRemoved: DeviceID };

export interface DeviceMetadata {
  name: string;
}
