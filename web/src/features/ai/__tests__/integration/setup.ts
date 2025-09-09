/**
 * Test setup and configuration for refactoring integration tests
 */

import type { RefactoringResult } from '../../types';

// Mock Tauri environment
const mockTauriEnv = {
  invoke: jest.fn(),
  getBackendCapabilities: jest.fn(),
  analyzeRefactoringContextEnhanced: jest.fn(),
  executeRefactoring: jest.fn(),
};

// Mock implementations
export const mockBackendCapabilities = {
  supported_refactorings: ['rename', 'extract-function', 'extract-variable'],
  supported_file_types: ['rs', 'ts', 'js', 'py'],
  features: {
    batch_operations: true,
    analysis: true,
    backup_recovery: true,
    test_generation: true,
    ai_analysis: false,
    lsp_integration: true,
    git_integration: true,
    cross_language_support: true,
    parallel_processing: true,
  },
  performance_metrics: {
    fresh_cache_entries: 15,
    total_cache_entries: 25,
    operation_count: 8,
  },
  configuration_options: ['create_backup', 'generate_tests'],
};

export const mockRefactoringResult: RefactoringResult = {
  id: 'integration-test-123',
  type: 'rename',
  target: {
    type: 'variable',
    name: 'oldName',
    range: {
      start: { line: 10, character: 0 },
      end: { line: 10, character: 7 },
    },
    analysis: {
      isSafe: true,
      warnings: [],
      conflicts: [],
      dependencies: [],
      preview: {
        before: 'let oldName = 42;',
        after: 'let newName = 42;',
        changes: [],
      },
    },
  },
  original: {
    filePath: 'src/main.rs',
    startLine: 10,
    startCharacter: 0,
    endLine: 10,
    endCharacter: 7,
  },
  changes: [{
    filePath: 'src/main.rs',
    range: {
      startLine: 10,
      startCharacter: 4,
      endLine: 10,
      endCharacter: 11,
    },
    oldText: 'oldName',
    newText: 'newName',
    changeType: 'Replacement',
  }],
  analysis: {
    possibleRefactorings: ['rename'],
    confidence: 0.9,
    impact: 'low',
    affectedFiles: ['src/main.rs'],
    risks: [],
    suggestions: ['Simple rename operation'],
    isSafe: true,
    warnings: [],
    conflicts: [],
    dependencies: [],
    preview: {
      before: 'let oldName = 42;',
      after: 'let newName = 42;',
      changes: [],
    },
  },
  success: true,
  timestamp: new Date().toISOString(),
  duration: 120,
  affectedFiles: 1,
};

// Error simulation utilities
export const simulateBackendError = (errorType: string, recoverable = true) => {
  const errors = {
    PERMISSION_DENIED: {
      code: 'PERMISSION_DENIED',
      message: 'Cannot modify read-only file',
      details: 'File is marked as read-only and cannot be modified',
      recoverable: false,
    },
    INVALID_REQUEST: {
      code: 'INVALID_REQUEST',
      message: 'Invalid refactoring request',
      details: 'Required parameters are missing or invalid',
      recoverable: true,
    },
    ENGINE_NOT_READY: {
      code: 'ENGINE_NOT_READY',
      message: 'Refactoring engine not initialized',
      details: 'Please wait for the refactoring engine to finish initializing',
      recoverable: true,
    },
    DEPENDENCY_CONFLICT: {
      code: 'DEPENDENCY_CONFLICT',
      message: 'Circular dependency detected',
      details: 'The refactoring would create circular dependencies',
      recoverable: true,
    },
    NETWORK_TIMEOUT: {
      code: 'NETWORK_TIMEOUT',
      message: 'Request timed out',
      details: 'The backend service took too long to respond',
      recoverable: true,
    },
  };

  return errors[errorType as keyof typeof errors] || errors.INVALID_REQUEST;
};

export const simulateNetworkFailure = () => {
  throw new Error('BackendError({"code": "NETWORK_ERROR", "message": "Service temporarily unavailable", "recoverable": true})');
};

export const simulateServiceTimeout = () => {
  throw new Error('Request timeout after 30000ms');
};

// Test data helpers
export const createTestContext = (overrides = {}) => ({
  filePath: 'src/main.rs',
  startLine: 10,
  startCharacter: 5,
  endLine: 15,
  endCharacter: 10,
  selectedText: 'function testFunction() { }',
  symbolName: 'testFunction',
  ...overrides,
});

export const createTestOptions = (overrides = {}) => ({
  createBackup: true,
  generateTests: false,
  applyToAllOccurrences: false,
  preserveReferences: true,
  ...overrides,
});

// Performance testing utilities
export const performanceTracker = {
  startTime: 0,
  endTime: 0,

  start() {
    this.startTime = Date.now();
  },

  end() {
    this.endTime = Date.now();
    return this.endTime - this.startTime;
  },

  getDuration() {
    return this.endTime - this.startTime;
  },

  reset() {
    this.startTime = 0;
    this.endTime = 0;
  },
};

// Cache testing utilities
export const cacheTestingUtilities = {
  mockCacheStats: {
    hitRatio: 0.85,
    totalEntries: 100,
    freshEntries: 85,
    staleEntries: 10,
    evictedEntries: 5,
  },

  simulateCacheExpiration() {
    // Simulate time advance for cache expiry
    jest.advanceTimersByTime(310000); // 5min 10sec (beyond 5min cache duration)
  },

  simulateCacheRefresh() {
    // Reset cache state for testing
    this.mockCacheStats.freshEntries = 5;
    this.mockCacheStats.staleEntries = this.mockCacheStats.totalEntries - this.mockCacheStats.freshEntries;
  },
};

// Async test utilities
export const asyncTestUtils = {
  async waitForTimeout(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
  },

  async waitForNextTick() {
    return new Promise(resolve => setImmediate(resolve));
  },

  async retryAsync<T>(
    operation: () => Promise<T>,
    maxRetries = 3,
    delayMs = 100
  ): Promise<T> {
    let lastError: Error;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;

        if (attempt < maxRetries) {
          await this.waitForTimeout(delayMs * attempt); // Exponential backoff
        }
      }
    }

    throw lastError!;
  },
};

// Global test setup
export const setupTestEnvironment = () => {
  // Setup global mocks and configurations for integration tests
  jest.mock('@tauri-apps/api/core', () => ({
    invoke: jest.fn(),
  }));

  // Configure test timeouts for integration tests
  jest.setTimeout(30000); // 30 seconds for integration tests

  // Setup fake timers for cache testing
  jest.useFakeTimers();
};

export const teardownTestEnvironment = () => {
  jest.useRealTimers();
  jest.clearAllMocks();
};
// Export all utilities as a single object for easy importing
export default {
  mockBackendCapabilities,
  mockRefactoringResult,
  simulateBackendError,
  simulateNetworkFailure,
  simulateServiceTimeout,
  createTestContext,
  createTestOptions,
  performanceTracker,
  cacheTestingUtilities,
  asyncTestUtils,
  setupTestEnvironment,
  teardownTestEnvironment,
};