// ! Unified shared type definitions between frontend and backend
// !
// ! This module consolidates type definitions that are used across both
// ! the React frontend and Rust backend to ensure consistency.
//
// ! All shared types are centralized here for single source of truth.

// ===== COMMON UTILITY TYPES =====================================================================

// Standard API response wrapper
export interface ApiResponse<T> {
  status: Status;
  data?: T;
  message?: string;
  timestamp: string; // ISO string
}

// Paginated response for collections
export interface PaginatedResponse<T> {
  items: T[];
  totalCount: number;
  page: number;
  perPage: number;
  hasMore: boolean;
}

// Event and stream types
export interface StreamEvent<T = unknown> {
  id: string;
  type: string;
  data: T;
  timestamp: string;
}

// Progress tracking for operations
export interface CommandProgress {
  id: string;
  stage: string;
  progress: number;
  message: string;
}

// Common enums and types
export enum Status {
  Ok = 'ok',
  Error = 'error',
  Warning = 'warning',
  Info = 'info',
  Pending = 'pending',
  Running = 'running',
  Completed = 'completed',
  Cancelled = 'cancelled',
}

export enum Priority {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Critical = 'critical',
}

// ===== CARGO TYPES =====================================================================

// Target-specific configuration
export interface CargoTargetConfig {
  compilationTarget?: string;
  rustflags?: string[];
  linker?: string;
  [key: string]: unknown;
}

// Patch configuration for overriding dependencies
export interface CargoPatch {
  [source: string]: Record<string, CargoDependency>;
}

// Core dependency specification (matches Rust CargoDependency)
export interface CargoDependency {
  // Version specification (can be exact version, range, or constraint)
  version?: string;

  // Local path dependency
  path?: string;

  // Git repository dependency
  git?: string;
  branch?: string;
  tag?: string;
  rev?: string;

  // Advanced dependency options
  features?: string[];
  optional?: boolean;
  defaultFeatures?: boolean;
  package?: string;

  // Registry and workspace options
  registry?: string;
  workspace?: boolean;
}

// Feature usage information (matches Rust FeatureUsage)
export interface FeatureUsage {
  name: string;
  enabledByDefault: boolean;
  isUsed: boolean;
  usedBy: string[];
  isDefault?: boolean;
}

// Package metadata for Cargo.toml
export interface CargoPackage {
  name?: string;
  version?: string;
  authors?: string[];
  edition?: string;
  rustVersion?: string; // camelCase for JS compatibility
  description?: string;
  documentation?: string;
  homepage?: string;
  repository?: string;
  readme?: string;
  keywords?: string[];
  categories?: string[];
  license?: string;
  licenseFile?: string; // camelCase for JS compatibility
  include?: string[];
  exclude?: string[];
  publish?: PublishConfig;
  defaultFeatures?: string[]; // camelCase for JS compatibility
  metadata?: Record<string, unknown>;
}

// Publishing configuration
export type PublishConfig = boolean | string[];

// Workspace configuration
export interface CargoWorkspace {
  members?: string[];
  exclude?: string[];
  defaultMembers?: string[]; // camelCase for JS compatibility
  metadata?: Record<string, unknown>;
  dependencies?: Record<string, CargoDependency>;
  devDependencies?: Record<string, CargoDependency>;
  buildDependencies?: Record<string, CargoDependency>;
  package?: {
    version?: string;
    authors?: string[];
    description?: string;
    documentation?: string;
    homepage?: string;
    repository?: string;
    license?: string;
    [key: string]: unknown;
  };
  resolver?: string;
}

// Profile configuration types
export interface CargoProfile {
  optLevel?: string | number;
  debug?: boolean | number;
  rpath?: boolean;
  lto?: boolean | string;
  codegenUnits?: number;
  panic?: 'unwind' | 'abort';
  incremental?: boolean;
  overflowChecks?: boolean;
  debugAssertions?: boolean;
  splitDebuginfo?: string;
}

// Target configurations
export interface LibConfig {
  name?: string;
  path?: string;
  crateType?: string[];
  edition?: string;
}

export interface BinConfig {
  name?: string;
  path?: string;
  edition?: string;
  requiredFeatures?: string[];
}

export interface ExampleConfig {
  name?: string;
  path?: string;
  edition?: string;
  requiredFeatures?: string[];
}

export interface TestConfig {
  name?: string;
  path?: string;
  edition?: string;
  harness?: boolean;
  requiredFeatures?: string[];
}

export interface BenchConfig {
  name?: string;
  path?: string;
  edition?: string;
  harness?: boolean;
  requiredFeatures?: string[];
}

// Root manifest structure
export interface CargoManifest {
  package?: CargoPackage;

  dependencies?: Record<string, CargoDependency>;
  // Support both camelCase and dash-based property names for compatibility
  devDependencies?: Record<string, CargoDependency>;
  'dev-dependencies'?: Record<string, CargoDependency>;
  buildDependencies?: Record<string, CargoDependency>;
  'build-dependencies'?: Record<string, CargoDependency>;
  target?: Record<string, Record<string, CargoDependency>>;

  features?: Record<string, string[]>;
  workspace?: CargoWorkspace;

  profile?: Record<string, CargoProfile>;

  // Badge configuration
  badges?: Record<string, Record<string, string>>;

  lib?: LibConfig;
  bin?: BinConfig[];
  example?: ExampleConfig[];
  test?: TestConfig[];
  bench?: BenchConfig[];
}

// Dependency management
export type DependencySection = 'dependencies' | 'dev-dependencies' | 'build-dependencies';

export interface DependencyLocation {
  section: DependencySection;
  name: string;
  dependency: CargoDependency;
}

export interface FeatureConfig {
  name: string;
  enabledByDefault: boolean;
  dependencies: string[];
}

// Dependency update information
export interface DependencyUpdate {
  name: string;
  currentVersion: string;
  latestVersion: string;
  updateType: 'major' | 'minor' | 'patch';
  usedIn: Array<{ member: string; version: string }>;
  changelogUrl?: string;
  isUpdating: boolean;
  updateError?: string;
}

// ===== METADATA TYPES =====================================================================

// Cargo metadata structures
export interface CargoMetadata {
  packages: CargoPackageMetadata[];
  workspaceRoot: string;
  targetDirectory: string;
  resolve?: ResolveNode;
  workspaceMembers: string[];
}

export interface CargoPackageMetadata {
  name: string;
  version: string;
  id: string;
  source?: string;
  dependencies: PackageDependency[];
  manifestPath: string;
  features: Record<string, string[]>;
}

export interface PackageDependency {
  name: string;
  source?: string;
  req: string;
  kind?: string;
  rename?: string;
  optional: boolean;
  usesDefaultFeatures: boolean;
  features: string[];
}

export interface ResolveNode {
  nodes: ResolveNodeItem[];
  root?: string;
}

export interface ResolveNodeItem {
  id: string;
  dependencies: string[];
}

export interface BuildInfo {
  buildTime: number;
  featuresUsed: string[];
  profile: string;
}

// ===== PERFORMANCE TYPES =====================================================================

// Crate metrics
export interface CrateMetrics {
  buildTime?: number;
  dependencies?: string[];
  featuresUsed?: string[];
}

// Performance metrics
export interface PerformanceMetrics {
  totalTime: number;
  compilationTime?: number;
  analysisTime?: number;
  memoryUsage?: number;
  cpuUsage?: number;
}

// ===== ERROR TYPES =====================================================================

// API error structure
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

// Command error structure
export interface CommandError {
  command: string;
  exitCode?: number;
  stdout: string;
  stderr: string;
}

// ===== CONFIGURATION TYPES =====================================================================

// User preferences
export interface UserPreferences {
  theme: Theme;
  keybindings: Record<string, string>;
  editor: EditorPreferences;
  cargo: CargoPreferences;
}

// UI themes
export type Theme = 'light' | 'dark' | 'system';

// Editor preferences
export interface EditorPreferences {
  fontSize: number;
  tabSize: number;
  insertSpaces: boolean;
  wordWrap: boolean;
}

// Cargo preferences
export interface CargoPreferences {
  autoUpdate: boolean;
  defaultProfile: string;
  showTimings: boolean;
}

// Project configuration
export interface ProjectConfig {
  id: string;
  name: string;
  path: string;
  cargoConfig?: CargoManifest;
  preferences: UserPreferences;
}

// ===== UTILITY IDENTIFIERS =====================================================================

// Module and command identifiers
export type ModuleId = string;
export type CommandId = string;
export type StreamId = string;
export type SessionId = string;

// Path types
export type ProjectPath = string;
export type FilePath = string;

// ===== UTILITY FUNCTIONS =====================================================================

// Response helper functions
export const responses = {
  ok: <T>(data: T): ApiResponse<T> => ({
    status: Status.Ok,
    data,
    timestamp: new Date().toISOString(),
  }),

  error: <T>(message: string): ApiResponse<T> => ({
    status: Status.Error,
    message,
    timestamp: new Date().toISOString(),
  }),

  warning: <T>(data: T | undefined, message: string): ApiResponse<T> => ({
    status: Status.Warning,
    data,
    message,
    timestamp: new Date().toISOString(),
  }),
};

// Type guards
export const isCargoManifest = (obj: unknown): obj is CargoManifest => {
  return obj !== null && typeof obj === 'object' && ('package' in obj || 'dependencies' in obj);
};

export const isApiResponse = (obj: unknown): obj is ApiResponse<unknown> => {
  return obj !== null && typeof obj === 'object' && 'status' in obj && 'timestamp' in obj;
};

export const isErrorResponse = (response: ApiResponse<unknown>): response is ApiResponse<never> => {
  return response.status === Status.Error || response.status === Status.Warning;
};

// Helper functions for type conversions
export const toCamelCase = (str: string): string => {
  return str.replace(/([-_][a-z])/gi, (match) => match[1].toUpperCase());
};

export const toSnakeCase = (str: string): string => {
  return str.replace(/[A-Z]/g, (match) => `_${match.toLowerCase()}`);
};

// Version utilities
export interface VersionInfo {
  major: number;
  minor: number;
  patch: number;
  preRelease?: string;
  buildMetadata?: string;
}

export const parseVersion = (version: string): VersionInfo | null => {
  // Simple semver parser - in practice, use a library like semver
  const match = version.match(
    /^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9\-.]+))?(?:\+([a-zA-Z0-9\-.]+))?$/
  );
  if (!match) return null;

  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    preRelease: match[4],
    buildMetadata: match[5],
  };
};

export const compareVersions = (v1: string, v2: string): number => {
  const parsed1 = parseVersion(v1);
  const parsed2 = parseVersion(v2);

  if (!parsed1 || !parsed2) return 0;

  if (parsed1.major !== parsed2.major) return parsed1.major - parsed2.major;
  if (parsed1.minor !== parsed2.minor) return parsed1.minor - parsed2.minor;
  return parsed1.patch - parsed2.patch;
};

// ===== REFACTORING TYPES (SHARED) =====================================================================

// Refactoring operation types
export type RefactoringType =
  | 'rename'
  | 'extractFunction'
  | 'extractVariable'
  | 'extractInterface'
  | 'convertToAsync'
  | 'move'
  | 'inline'
  | 'changeSignature'
  | 'replaceWithMethodCall'
  | 'other';

// Context information for refactoring operations
export interface RefactoringContext {
  filePath: string;
  symbolName?: string;
  symbolLineStart: number;
  symbolLineEnd: number;
  symbolType?: string;
  language: ProgrammingLanguage;
}

// Result of a refactoring operation
export interface RefactoringResult {
  success: boolean;
  changesMade: CodeChange[];
  newSymbolName?: string;
  extractedFunctionName?: string;
  extractedInterfaceName?: string;
}

// Record of a code change made during refactoring
export interface CodeChange {
  filePath: string;
  lineStart: number;
  lineEnd: number;
  originalCode: string;
  newCode: string;
}

// ===== LANGUAGE AND POSITIONING TYPES =====================================================================

export enum ProgrammingLanguage {
  Rust = 'Rust',
  TypeScript = 'TypeScript',
  JavaScript = 'JavaScript',
  Python = 'Python',
  Java = 'Java',
  CSharp = 'CSharp',
  Go = 'Go',
  Cpp = 'Cpp',
  C = 'C',
  Unknown = 'Unknown',
}

// Position in a text document (zero-based line and character)
export interface Position {
  line: number;
  character: number;
}

// Range in a text document
export interface Range {
  start: Position;
  end: Position;
}

// Location in a text document
export interface Location {
  uri: string;
  range: Range;
}
