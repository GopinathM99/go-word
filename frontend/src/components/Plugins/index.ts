/**
 * Plugin Components
 *
 * Export all plugin-related components for easy importing.
 */

export { PluginManager } from './PluginManager';
export type {
  Permission,
  ActivationEvent,
  PluginManifest,
  InstalledPlugin,
} from './PluginManager';

export { PluginBrowser } from './PluginBrowser';
export type { AvailablePlugin, PluginCategory } from './PluginBrowser';

export { PluginPermissions, usePermissionRequests } from './PluginPermissions';
export type { PermissionRequest } from './PluginPermissions';
