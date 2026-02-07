export interface DashboardElement {
  /// plugin name, or undefined for core element
  plugin?: string;
  module: string;
  props?: Record<string, any>;
  body?: DashboardElement[];
}

export interface DashboardConfig {
  id: string;
  name: string;
  elements: DashboardElement[];
}
