/**
 * Integration Test Configuration and Test Runner
 * @jest-environment jsdom
 */

import { jest } from '@jest/globals';
import { setupTestEnvironment, teardownTestEnvironment } from './setup';

// Global test configuration
const TEST_CONFIG = {
  // Test timeouts
  INTEGRATION_TEST_TIMEOUT: 30000,
  UNIT_TEST_TIMEOUT: 5000,

  // Cache testing configuration
  CACHE_DURATION_MS: 5 * 60 * 1000, // 5 minutes

  // Mock service configurations
  BACKEND_SERVICE_DELAY: 100, // ms
  AI_SERVICE_DELAY: 250,
  LSP_SERVICE_DELAY: 150,

  // Retry configurations
  MAX_RETRY_ATTEMPTS: 3,
  RETRY_DELAY_MS: 100,

  // Performance thresholds
  MAX_BACKEND_RESPONSE_TIME: 2000, // ms
  MAX_CACHE_OPERATION_TIME: 50,   // ms
  TARGET_CACHE_HIT_RATIO: 0.8,

  // Backend capability configurations
  SUPPORTED_REFACTORINGS: [
    'rename',
    'extract-method',
    'extract-variable',
    'extract-interface',
    'move-method',
    'inline-method',
    'introduce-parameter',
    'convert-to-async',
    'pattern-conversion',
  ],

  SUPPORTED_FILE_TYPES: ['rs', 'ts', 'js', 'py'],

  // Test data configurations
  SAMPLE_RUST_CODE: `
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &num in numbers {
        sum += num;
    }
    sum
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let result = calculate_sum(&numbers);
    println!("Sum: {}", result);
}
  `,

  SAMPLE_TYPESCRIPT_CODE: `
function calculateSum(numbers: number[]): number {
    let sum = 0;
    for (const num of numbers) {
        sum += num;
    }
    return sum;
}

const numbers = [1, 2, 3, 4, 5];
const result = calculateSum(numbers);
console.log("Sum:", result);
  `,
};

/**
 * Global setup for all integration tests
 */
beforeAll(() => {
  setupTestEnvironment();

  // Configure global test settings
  jest.setTimeout(TEST_CONFIG.INTEGRATION_TEST_TIMEOUT);

  // Setup global mocks
  global.testConfig = TEST_CONFIG;

  console.log('🧪 Integration test environment configured');
});

/**
 * Global teardown for all integration tests
 */
afterAll(() => {
  teardownTestEnvironment();
  console.log('🧪 Integration test environment cleaned up');
});

/**
 * Setup for each test
 */
beforeEach(() => {
  // Reset all mocks before each test
  jest.clearAllMocks();

  // Reset global state
  global.mockBackendCapabilities = null;
  global.testMetrics = {
    totalTests: 0,
    passedTests: 0,
    failedTests: 0,
    skippedTests: 0,
    startTime: Date.now(),
  };
});

/**
 * Custom matchers for refactoring-specific assertions
 */
expect.extend({
  /**
   * Check if a refactoring result is valid
   */
  toBeValidRefactoringResult(received) {
    const pass = received
      && typeof received === 'object'
      && typeof received.id === 'string'
      && Array.isArray(received.changes)
      && typeof received.success === 'boolean';

    if (pass) {
      return {
        message: () => 'expected not to be a valid refactoring result',
        pass: true,
      };
    } else {
      return {
        message: () => 'expected to be a valid refactoring result',
        pass: false,
      };
    }
  },

  /**
   * Check if capabilities are properly structured
   */
  toHaveValidBackendCapabilities(received) {
    const pass = received
      && Array.isArray(received.supported_refactorings)
      && Array.isArray(received.supported_file_types)
      && typeof received.features === 'object'
      && received.features !== null;

    if (pass) {
      return {
        message: () => 'expected capabilities to be properly structured',
        pass: true,
      };
    } else {
      return {
        message: () => 'expected capabilities to have valid structure',
        pass: false,
      };
    }
  },

  /**
   * Check if an error response is properly structured
   */
  toBeStructuredErrorResponse(received) {
    const pass = received
      && typeof received.code === 'string'
      && typeof received.message === 'string'
      && typeof received.recoverable === 'boolean';

    if (pass) {
      return {
        message: () => 'expected error response to be properly structured',
        pass: true,
      };
    } else {
      return {
        message: () => 'expected error response to have valid structure',
        pass: false,
      };
    }
  },

  /**
   * Check if cache statistics are valid
   */
  toHaveValidCacheStatistics(received) {
    const pass = received
      && typeof received.totalEntries === 'number'
      && typeof received.freshEntries === 'number'
      && typeof received.staleEntries === 'number'
      && received.totalEntries >= 0
      && received.freshEntries >= 0
      && received.staleEntries >= 0
      && received.freshEntries + received.staleEntries <= received.totalEntries;

    if (pass) {
      return {
        message: () => 'expected cache statistics to be valid',
        pass: true,
      };
    } else {
      return {
        message: () => 'expected cache statistics to be consistent',
        pass: false,
      };
    }
  },
});

/**
 * Test metrics collector
 */
declare global {
  var testConfig: typeof TEST_CONFIG;
  var testMetrics: {
    totalTests: number;
    passedTests: number;
    failedTests: number;
    skippedTests: number;
    startTime: number;
  };
  var mockBackendCapabilities: any;
}

/**
 * Utility functions for integration tests
 */
export const testUtils = {
  /**
   * Create a mock backend response with specified latency
   */
  withLatency: async <T>(data: T, delay: number = TEST_CONFIG.BACKEND_SERVICE_DELAY): Promise<T> => {
    await new Promise(resolve => setTimeout(resolve, delay));
    return data;
  },

  /**
   * Simulate network conditions
   */
  simulateNetworkConditions: (latency: number, packetLossRatio: number = 0) => {
    // Implementation for network simulation in tests
  },

  /**
   * Wait for backend service availability
   */
  waitForService: async (maxWaitMs: number = 5000): Promise<boolean> => {
    const startTime = Date.now();
    while (Date.now() - startTime < maxWaitMs) {
      try {
        // Mock service availability check
        return true;
      } catch {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    return false;
  },

  /**
   * Validate refactoring test data
   */
  validateRefactoringTestData: (data: any): boolean => {
    return data
      && typeof data === 'object'
      && Array.isArray(data.changes)
      && typeof data.success === 'boolean';
  },
};

/**
 * Performance monitoring for integration tests
 */
export const performanceMonitor = {
  startTest(name: string): { end: () => number } {
    const startTime = Date.now();
    return {
      end: () => {
        const duration = Date.now() - startTime;
        console.log(`⚡ Test "${name}" completed in ${duration}ms`);
        return duration;
      }
    };
  },

  measureAverageDuration(testFunction: () => Promise<any>, iterations: number = 5): Promise<number> {
    return Promise.resolve(0); // Placeholder for actual implementation
  },
};

/**
 * Export configuration for use in other test files
 */
export { TEST_CONFIG };
export { TEST_CONFIG as config };

export default {
  TEST_CONFIG,
  testUtils,
  performanceMonitor,
};