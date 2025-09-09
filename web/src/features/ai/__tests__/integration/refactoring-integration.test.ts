import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { RefactoringService, BackendCapabilitiesResponse } from '../../services/RefactoringService';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

describe('RefactoringService - Integration Tests', () => {
  let refactoringService: RefactoringService;

  beforeEach(() => {
    // Clear singleton instance for each test
    (RefactoringService as any).instance = null;
    refactoringService = RefactoringService.getInstance();

    // Clear all mocks
    vi.clearAllMocks();

    // Reset service cache
    refactoringService['clearCache']();
  });

  it('should handle concurrent requests efficiently', async () => {
    // This test demonstrates realistic concurrent behavior
    // In practice, some calls may hit the backend due to timing
    const mockCapabilities: BackendCapabilitiesResponse = {
      supported_refactorings: ['rename'],
      supported_file_types: ['rs'],
      features: {
        batch_operations: false,
        analysis: false,
        backup_recovery: false,
        test_generation: false,
        ai_analysis: false,
        lsp_integration: false,
        cross_language_support: false,
      },
    };

    mockInvoke.mockResolvedValue(mockCapabilities);

    // Fire multiple simultaneous requests
    const promises = [
      refactoringService.getBackendCapabilities(),
      refactoringService.getBackendCapabilities(),
      refactoringService.getBackendCapabilities(),
      refactoringService.getBackendCapabilities(),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed and returned consistent results
    expect(results).toHaveLength(4);
    results.forEach(result => {
      expect(result).toEqual(mockCapabilities);
    });

    // Verify that at least one call was made to backend
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it('should cache backend capabilities within cache window', async () => {
    const mockCapabilities: BackendCapabilitiesResponse = {
      supported_refactorings: ['rename', 'extract'],
      supported_file_types: ['rs', 'ts'],
      features: {
        batch_operations: true,
        analysis: true,
        backup_recovery: true,
        test_generation: true,
        ai_analysis: true,
        lsp_integration: true,
        cross_language_support: true,
      },
    };

    mockInvoke.mockResolvedValue(mockCapabilities);

    // First call should hit backend
    const result1 = await refactoringService.getBackendCapabilities();
    expect(result1).toEqual(mockCapabilities);
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Second call should use cache (due to short cache time)
    const result2 = await refactoringService.getBackendCapabilities();
    expect(result2).toEqual(mockCapabilities);
    expect(mockInvoke).toHaveBeenCalledTimes(1); // Should still be 1
  });

  it('should handle backend errors gracefully', async () => {
    const errorMessage = 'Backend service unavailable';
    mockInvoke.mockRejectedValue(new Error(errorMessage));

    const result = await refactoringService.getBackendCapabilities();
    expect(result).toBeNull();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it('should provide feature gate methods based on capabilities', async () => {
    const mockCapabilities: BackendCapabilitiesResponse = {
      supported_refactorings: ['rename'],
      supported_file_types: ['rs'],
      features: {
        batch_operations: true,
        analysis: false,
        backup_recovery: false,
        test_generation: true,
        ai_analysis: false,
        lsp_integration: false,
        cross_language_support: false,
      },
    };

    mockInvoke.mockResolvedValue(mockCapabilities);

    expect(await refactoringService['shouldShowBatchOperations']()).toBe(true);
    expect(await refactoringService['shouldShowTestGeneration']()).toBe(true);
    expect(await refactoringService['shouldShowAdvancedFeatures']()).toBe(false);
    expect(await refactoringService['shouldShowPerformanceMetrics']()).toBe(false);
  });

  it('should handle file type validation correctly', async () => {
    const mockCapabilities: BackendCapabilitiesResponse = {
      supported_refactorings: ['rename'],
      supported_file_types: ['rs', 'ts', 'js'],
      features: {
        batch_operations: false,
        analysis: false,
        backup_recovery: false,
        test_generation: false,
        ai_analysis: false,
        lsp_integration: false,
        cross_language_support: false,
      },
    };

    mockInvoke.mockResolvedValue(mockCapabilities);

    expect(await refactoringService.isFileTypeSupported('test.rs')).toBe(true);
    expect(await refactoringService.isFileTypeSupported('test.ts')).toBe(true);
    expect(await refactoringService.isFileTypeSupported('test.js')).toBe(true);
    expect(await refactoringService.isFileTypeSupported('test.py')).toBe(false); // Explicitly excluded in service
    expect(await refactoringService.isFileTypeSupported('test.txt')).toBe(false);
  });

  it('should prevent cache stampede with concurrent calls', async () => {
    const mockCapabilities: BackendCapabilitiesResponse = {
      supported_refactorings: ['rename'],
      supported_file_types: ['rs'],
      features: {
        batch_operations: false,
        analysis: false,
        backup_recovery: false,
        test_generation: false,
        ai_analysis: false,
        lsp_integration: false,
        cross_language_support: false,
      },
    };

    let callCount = 0;
    mockInvoke.mockImplementation(async () => {
      callCount++;
      // Simulate some processing time
      await new Promise(resolve => setTimeout(resolve, 10));
      return mockCapabilities;
    });

    // Create more concurrent calls to stress test
    const promises = Array.from({ length: 10 }, () =>
      refactoringService.getBackendCapabilities()
    );

    const results = await Promise.all(promises);

    // All should return same result
    results.forEach(result => {
      expect(result).toEqual(mockCapabilities);
    });

    // But backend should only be called once due to caching
    expect(callCount).toBe(1);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });
});
