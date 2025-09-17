//! CONSOLIDATED CARGO TYPES - BACKWARD COMPATIBILITY LAYER
//!
//! IMPORTANT: This file now imports consolidated types from the shared location.
//! New code should import directly from '../shared/types/index' for better type safety.
//! This file is maintained only for existing imports to continue working.

import type {
  CargoDependency,
  FeatureUsage,
  CargoPackage,
  CargoWorkspace,
  CargoTargetConfig,
  CargoProfile,
  CargoPatch,
  CargoManifest,
  DependencySection,
  DependencyLocation,
  FeatureConfig,
  DependencyUpdate,
  LibConfig,
  BinConfig,
  ExampleConfig,
  TestConfig,
  BenchConfig,
} from '../shared/types/index';

// Import additional types from generated shared types
import type { User, Theme, UserPreferences, UserSettings } from './generated';

// Re-export types from consolidated location for backward compatibility
export type {
  CargoDependency,
  FeatureUsage,
  CargoPackage,
  CargoWorkspace,
  CargoTargetConfig,
  CargoProfile,
  CargoPatch,
  CargoManifest,
  DependencySection,
  DependencyLocation,
  FeatureConfig,
  DependencyUpdate,
  LibConfig,
  BinConfig,
  ExampleConfig,
  TestConfig,
  BenchConfig,
};

// ===== COLLABORATION-ENHANCED CARGO TYPES =====

// Collaborative Cargo Session Types
export interface CollaborativeCargoSession {
  sessionId: string;
  participants: string[];
  sessionState: 'active' | 'paused' | 'terminated';
  createdAt: Date;
  lastActivity: Date;
}

// Shared Dependency Types
export interface SharedCargoDependency extends CargoDependency {
  sharedBy: string;
  sharedAt: Date;
  locked: boolean;
  collaborators: string[];
}

// Collaborative Build Types
export interface CollaborativeBuild {
  buildId: string;
  sessionId: string;
  buildCommand: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  participants: string[];
  logs: string[];
  startedAt: Date;
  completedAt?: Date;
}

// ===== PRESERVED UNIQUE FUNCTIONS (NOT CONSOLIDATED) =====
// These functions are specific to this file and provide TOML parsing functionality

// Basic TOML parser implementation (simplified)
function parseTOMLBasic(toml: string): any {
  // This is a very basic TOML parser for demonstration
  // A real implementation would use a proper TOML library

  const lines = toml.split('\n');
  const result: any = {};
  let currentSection = result;
  let currentTable: string | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip empty lines and comments
    if (!trimmed || trimmed.startsWith('#')) {
      continue;
    }

    // Check for section header
    const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      const sectionPath = sectionMatch[1].replace(/"/g, '');
      const parts = sectionPath.split('.');

      let current = result;
      for (const part of parts) {
        if (!current[part]) {
          current[part] = {};
        }
        current = current[part];
      }
      currentSection = current;
      currentTable = null;
      continue;
    }

    // Check for array of tables
    const arrayTableMatch = trimmed.match(/^\[\[([^\]]+)\]\]$/);
    if (arrayTableMatch) {
      const sectionPath = arrayTableMatch[1].replace(/"/g, '');
      const parts = sectionPath.split('.');

      let current = result;
      for (const part of parts) {
        if (!Array.isArray(current[part])) {
          current[part] = [];
        }
        current = current[part];
      }

      if (!Array.isArray(current)) {
        current = result;
      }

      // Get or create last array element
      if (!Array.isArray(currentSection)) {
        currentSection = [];
        current[current.length - 1] = currentSection;
      }
      currentTable = sectionPath;
      continue;
    }

    // Parse key-value pairs
    const keyValueMatch = trimmed.match(/^([^=]+)=(.*)$/);
    if (keyValueMatch) {
      const [, keyRaw, valueRaw] = keyValueMatch;
      const key = keyRaw.trim().replace(/"/g, '');
      const value = parseTOMLValue(valueRaw.trim());

      currentSection[key] = value;
      continue;
    }
  }

  return result;
}

// Parse TOML value
function parseTOMLValue(value: string): any {
  // String
  if (value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1).replace(/\\"/g, '"');
  }

  // Array
  if (value.startsWith('[') && value.endsWith(']')) {
    const items = value
      .slice(1, -1)
      .split(',')
      .map((item) => parseTOMLValue(item.trim()));
    return items;
  }

  // Boolean
  if (value === 'true') return true;
  if (value === 'false') return false;

  // Number
  if (!isNaN(Number(value))) return Number(value);

  // String without quotes
  if (value && !value.includes(' ')) return value;

  return value;
}

// Basic TOML stringifier implementation (simplified)
function stringifyTOMLBasic(obj: any, indent = ''): string {
  let result = '';
  const keys = Object.keys(obj);

  for (const key of keys) {
    const value = obj[key];

    if (typeof value === 'object' && value !== null) {
      if (Array.isArray(value)) {
        // Handle array of objects
        if (value.length > 0 && typeof value[0] === 'object') {
          for (const item of value) {
            result += `[[${key}]]\n`;
            result += stringifyTOMLBasic(item, indent);
            result += '\n';
          }
        } else {
          result += `${indent}${key} = [${value.map((v: any) => JSON.stringify(v)).join(', ')}]\n`;
        }
      } else {
        // Handle nested objects
        result += `${indent}[${key}]\n`;
        result += stringifyTOMLBasic(value, indent);
        result += '\n';
      }
    } else {
      result += `${indent}${key} = ${JSON.stringify(value)}\n`;
    }
  }

  return result.trim();
}

// Utility functions for Cargo.toml parsing and stringification
export async function parseCargoToml(toml: string): Promise<CargoManifest> {
  try {
    // Try to use a proper TOML library if available
    try {
      const { parse } = await import('@iarna/toml');
      return parse(toml) as CargoManifest;
    } catch (importError) {
      console.warn('TOML library not available, using basic parser');
      // Fallback to basic TOML parser implementation
      const parsedToml = parseTOMLBasic(toml);
      return parsedToml as CargoManifest;
    }
  } catch (e) {
    console.error('Failed to parse Cargo.toml:', e);
    // Fallback to Rust backend Tauri command if available
    console.warn('Falling back to Rust backend for TOML parsing');
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<CargoManifest>('parse_cargo_toml', { content: toml });
      return result;
    } catch (tauriError) {
      console.error('Tauri fallback also failed:', tauriError);
      return {};
    }
  }
}

export async function stringifyCargoToml(manifest: CargoManifest): Promise<string> {
  try {
    // Try to use a proper TOML library if available
    try {
      const { stringify } = await import('@iarna/toml');
      return stringify(manifest);
    } catch (importError) {
      console.warn('TOML library not available, using basic stringifier');
      // Fallback to basic TOML stringifier implementation
      const toml = stringifyTOMLBasic(manifest);
      return toml;
    }
  } catch (e) {
    console.error('Failed to stringify Cargo.toml:', e);
    // Fallback to Rust backend Tauri command if available
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<string>('stringify_cargo_toml', { manifest });
      return result;
    } catch (tauriError) {
      console.error('Tauri fallback also failed:', tauriError);
      // Final fallback to JSON
      return JSON.stringify(manifest, null, 2);
    }
  }
}