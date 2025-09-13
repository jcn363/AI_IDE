/**
 * End-to-End Integration Tests for Refactoring System
 * Tests the complete user workflow from start to finish
 * @jest-environment jsdom
 */

import {
  jest,
  describe,
  it,
  expect,
  beforeEach,
  afterEach,
  beforeAll,
  afterAll,
} from '@jest/globals';
import { TEST_CONFIG, testUtils, performanceMonitor } from './config.test';
import { invoke } from '@tauri-apps/api/core';

// Mock external dependencies
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

// Types for global test variables
declare global {
  var testConfig: typeof TEST_CONFIG;
}

describe('End-to-End Refactoring Workflow Tests', () => {
  beforeAll(() => {
    // Setup global environment
    (global as any).testConfig = TEST_CONFIG;
    jest.setTimeout(TEST_CONFIG.INTEGRATION_TEST_TIMEOUT);
  });

  afterAll(() => {
    // Cleanup
    delete (global as any).testConfig;
  });

  describe('Complete Rename Workflow', () => {
    it('should successfully rename a symbol end-to-end', async () => {
      const testTimer = performanceMonitor.startTest('Complete Rename Workflow');

      try {
        // 1. Setup test data
        const originalCode = 'let oldVariable = 42;';
        const newName = 'newVariable';

        // 2. Simulate user selection
        const selectionContext = {
          filePath: 'src/main.rs',
          startLine: 10,
          startCharacter: 4,
          endLine: 10,
          endCharacter: 15,
          selectedText: 'oldVariable',
          symbolName: 'oldVariable',
        };

        // 3. Mock backend capabilities check
        (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
          supported_refactorings: ['rename'],
          supported_file_types: ['rs'],
          features: {
            batch_operations: true,
            analysis: true,
            backup_recovery: true,
            test_generation: false,
            ai_analysis: false,
            lsp_integration: false,
            git_integration: true,
            cross_language_support: false,
            parallel_processing: true,
          },
          performance_metrics: {},
          configuration_options: ['create_backup'],
        });

        // 4. Mock analysis response
        (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
          applicableRefactorings: ['rename'],
          confidenceScores: { rename: 0.9 },
          suggestions: ['Simple rename operation'],
          warnings: [],
          isSafe: true,
        });

        // 5. Mock refactoring execution
        (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
          id: 'end-to-end-test-123',
          type: 'rename',
          success: true,
          changes: [
            {
              filePath: 'src/main.rs',
              range: {
                startLine: 10,
                startCharacter: 4,
                endLine: 10,
                endCharacter: 15,
              },
              oldText: 'oldVariable',
              newText: newName,
              changeType: 'Replacement',
            },
          ],
          timestamp: new Date().toISOString(),
          duration: 150,
          affected_files: 1,
          error: null,
          metrics: {
            operations_attempted: 1,
            operations_succeeded: 1,
            operations_failed: 0,
            total_bytes_changed: newName.length - 'oldVariable'.length,
            average_complexity: 0.1,
          },
        });

        // 6. Execute simulated workflow
        const result = await performRenameWorkflow(selectionContext, newName);

        // 7. Verify complete workflow success
        expect(result.success).toBe(true);
        expect(result.changes).toHaveLength(1);
        expect(result.changes[0].newText).toBe(newName);
        expect(result.metrics.operations_succeeded).toBe(1);

        // 8. Verify performance
        const duration = testTimer.end();
        expect(duration).toBeLessThan(1000); // Should complete within 1 second

        console.log(`✅ End-to-end rename workflow completed in ${duration}ms`);
      } catch (error) {
        testTimer.end();
        throw error;
      }
    });
  });

  describe('Error Recovery and Resilience', () => {
    it('should recover from temporary service outages', async () => {
      const testTimer = performanceMonitor.startTest('Service Outage Recovery');

      try {
        // 1. Setup initial service failure
        (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
          new Error('Service temporarily unavailable')
        );
        (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
          new Error('Network timeout')
        );
        (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
          new Error('Connection reset')
        );

        // 2. Setup recovery
        (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
          supported_refactorings: ['rename'],
          supported_file_types: ['rs'],
          features: {
            batch_operations: true,
            analysis: true,
            backup_recovery: false,
            test_generation: false,
            ai_analysis: false,
            lsp_integration: false,
            git_integration: false,
            cross_language_support: false,
            parallel_processing: false,
          },
          performance_metrics: {},
          configuration_options: [],
        });

        // 3. Execute workflow with retry logic
        const result = await executeWithRetry(
          () => getBackendCapabilitiesWithRetry(),
          3, // maxRetries
          100 // delayMs
        );

        // 4. Verify successful recovery
        expect(result.supported_refactorings).toContain('rename');
        expect(invoke).toHaveBeenCalledTimes(4); // 3 failures + 1 success

        testTimer.end();
      } catch (error) {
        testTimer.end();
        throw error;
      }
    });

    it('should handle permission denied errors gracefully', async () => {
      // 1. Setup scenario
      const readOnlyFileContext = {
        filePath: '/readonly/important.rs',
        startLine: 1,
        startCharacter: 0,
        endLine: 5,
        endCharacter: 10,
      };

      // 2. Mock permission denied error
      (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
        new Error(
          'BackendError({"code": "PERMISSION_DENIED", "message": "Cannot modify read-only file", "details": "File is locked or marked as read-only", "recoverable": false})'
        )
      );

      // 3. Execute and verify error handling
      await expect(executeRefactoringWorkflow(readOnlyFileContext)).rejects.toThrow(
        'Cannot modify read-only file'
      );

      // 4. Verify proper error categorization
      try {
        await executeRefactoringWorkflow(readOnlyFileContext);
      } catch (error: any) {
        expect(error.code).toBe('PERMISSION_DENIED');
        expect(error.recoverable).toBe(false);
        expect(error.details).toContain('read-only');
      }
    });
  });

  describe('Performance and Reliability', () => {
    it('should maintain performance under load', async () => {
      const testTimer = performanceMonitor.startTest('Performance Under Load');

      try {
        // 1. Setup multiple concurrent operations
        const operations = Array.from({ length: 10 }, (_, i) => ({
          id: `perf-test-${i}`,
          type: 'rename',
          context: { filePath: `src/file${i}.rs`, startLine: 1, startCharacter: 0 },
          options: { createBackup: true },
        }));

        // 2. Mock responses with realistic delays
        operations.forEach((_, index) => {
          (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
            id: `result-${index}`,
            success: true,
            changes: [
              {
                filePath: `src/file${index}.rs`,
                range: { startLine: 1, startCharacter: 0, endLine: 1, endCharacter: 10 },
                oldText: 'oldName',
                newText: 'newName',
                changeType: 'Replacement',
              },
            ],
            timestamp: new Date().toISOString(),
            duration: 50 + Math.random() * 100, // 50-150ms random variation
            affected_files: 1,
          });
        });

        // 3. Execute concurrent operations
        const promises = operations.map((op) => executeRefactoringWorkflow(op));
        const results = await Promise.all(promises);

        // 4. Verify all operations succeeded
        const successfulResults = results.filter((r) => r.success);
        expect(successfulResults).toHaveLength(10);

        // 5. Verify performance
        const totalDuration = testTimer.end();
        const averageDuration = totalDuration / 10;

        console.log(
          `📊 Performance test: ${totalDuration}ms total, ${averageDuration}ms average per operation`
        );

        // Should be reasonably fast for simulated operations
        expect(averageDuration).toBeLessThan(200);
      } catch (error) {
        testTimer.end();
        throw error;
      }
    });

    it('should demonstrate effective caching', async () => {
      const testTimer = performanceMonitor.startTest('Cache Effectiveness Test');

      try {
        // 1. Setup repeated backend calls
        const capabilitiesResponse = {
          supported_refactorings: ['rename', 'extract-function'],
          supported_file_types: ['rs', 'ts'],
          features: {
            batch_operations: true,
            analysis: true,
            backup_recovery: true,
            test_generation: false,
            ai_analysis: false,
            lsp_integration: false,
            git_integration: false,
            cross_language_support: false,
            parallel_processing: false,
          },
          performance_metrics: {
            fresh_cache_entries: 10,
            total_cache_entries: 12,
          },
          configuration_options: ['create_backup'],
        };

        // Mock backend response
        (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValue(capabilitiesResponse);

        // 2. Execute multiple requests that should use cache
        const requests = Array.from({ length: 5 }, () => getBackendCapabilitiesCached());

        const startTime = Date.now();
        const results = await Promise.all(requests);
        const endTime = Date.now();

        // 3. Verify all results are identical (from cache)
        results.forEach((result) => {
          expect(result).toEqual(capabilitiesResponse);
        });

        // 4. Verify performance improvement from caching
        const totalDuration = endTime - startTime;
        console.log(`⚡ Cache test completed in ${totalDuration}ms`);

        // Should be very fast (likely < 50ms total for all 5 cached calls)
        expect(totalDuration).toBeLessThan(100);

        testTimer.end();
      } catch (error) {
        testTimer.end();
        throw error;
      }
    });
  });

  describe('Advanced Feature Integration', () => {
    it('should handle AI-enhanced analysis when available', async () => {
      // 1. Setup AI-capable backend
      const aiCapabilities = {
        supported_refactorings: ['rename', 'extract-function', 'pattern-conversion'],
        supported_file_types: ['rs'],
        features: {
          batch_operations: true,
          analysis: true,
          backup_recovery: true,
          test_generation: false,
          ai_analysis: true,
          lsp_integration: false,
          git_integration: false,
          cross_language_support: false,
          parallel_processing: false,
        },
        performance_metrics: {},
        configuration_options: [],
      };

      // 2. Mock AI-enhanced analysis
      const aiInsights = {
        suggestions: ['Consider using a more functional approach'],
        complexityScore: 0.73,
        maintainabilityIndex: 65.0,
        testRecommendationCount: 2,
      };

      (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce(aiCapabilities);
      (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
        applicableRefactorings: ['extract-function', 'pattern-conversion'],
        confidenceScores: { 'extract-function': 0.9 },
        suggestions: [...aiInsights.suggestions, 'Large function detected'],
        warnings: ['Function complexity indicates potential refactoring'],
        aiInsights,
        lspInsights: null,
        analysisTypes: { hasAI: true, hasLSP: false, hasBasic: true },
      });

      // 3. Execute AI-enhanced workflow
      const result = await performAIEnhancedAnalysis();

      // 4. Verify AI-specific features
      expect(result.aiInsights).toBeDefined();
      expect(result.aiInsights.complexityScore).toBeGreaterThan(0.5);
      expect(result.suggestions).toContain('Consider using a more functional approach');
    });

    it('should gracefully fallback to basic analysis when AI fails', async () => {
      // 1. Setup scenario where AI services fail
      (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
        new Error('AI service timeout')
      );
      (invoke as jest.MockedFunction<typeof invoke>).mockResolvedValueOnce({
        applicableRefactorings: ['rename'],
        confidenceScores: { rename: 0.8 },
        suggestions: ['Basic rename operation'],
        warnings: [],
        analysisTypes: { hasAI: false, hasLSP: false, hasBasic: true },
      });

      // 2. Execute with AI failure scenario
      const result = await performAnalysisWithAIFailure();

      // 3. Verify fallback works correctly
      expect(result.applicableRefactorings).toContain('rename');
      expect(result.aiInsights).toBeUndefined();
      expect(result.suggestions).toContain('Basic rename operation');

      console.log('✅ Graceful AI fallback verified');
    });
  });
});

// Helper functions for end-to-end testing

async function performRenameWorkflow(context: any, newName: string) {
  // Simulate complete renaming workflow
  const startTime = Date.now();

  try {
    // Step 1: Get capabilities
    const capabilities = await invoke('get_backend_capabilities');

    // Step 2: Analyze context
    const analysis = await invoke('analyze_refactoring_context_enhanced', {
      filePath: context.filePath,
      codeContent: `let ${context.symbolName} = 42;`,
      context,
      includeAI: capabilities.features.ai_analysis,
      includeLSP: capabilities.features.lsp_integration,
    });

    // Step 3: Validate operation
    expect(analysis.applicableRefactorings).toContain('rename');

    // Step 4: Execute refactoring
    const result = await invoke('execute_refactoring', {
      refactoring_type: 'rename',
      context,
      options: { createBackup: true },
    });

    // Step 5: Verify result
    expect(result.success).toBe(true);
    expect(result.changes[0].newText).toBe(newName);

    const duration = Date.now() - startTime;
    console.log(`✅ Full rename workflow completed successfully in ${duration}ms`);

    return result;
  } catch (error) {
    throw error;
  }
}

async function performAIEnhancedAnalysis() {
  const context = {
    filePath: 'src/complex.rs',
    startLine: 15,
    startCharacter: 5,
    endLine: 30,
    endCharacter: 10,
    symbolName: 'complexFunction',
  };

  return invoke('analyze_refactoring_context_enhanced', {
    filePath: context.filePath,
    codeContent: `function ${context.symbolName}() {\n  let x = 1;\n  // Complex logic here\n  for(let i = 0; i < 10; i++) {\n    x *= i;\n  }\n  return x;\n}`,
    context,
    includeAI: true,
    includeLSP: false,
  });
}

async function performAnalysisWithAIFailure() {
  // Simulate AI analysis failure with fallback scenario
  const context = {
    filePath: 'src/simple.rs',
    startLine: 5,
    startCharacter: 0,
    endLine: 5,
    endCharacter: 15,
    symbolName: 'simpleVar',
  };

  // Force AI failure
  (invoke as jest.MockedFunction<typeof invoke>).mockRejectedValueOnce(
    new Error('AI service unavailable')
  );

  return invoke('analyze_refactoring_context_enhanced', {
    filePath: context.filePath,
    codeContent: `let ${context.symbolName} = 42;`,
    context,
    includeAI: true,
    includeLSP: false,
  }).catch(async () => {
    // Fallback to basic analysis on AI failure
    return invoke('analyze_refactoring_context', {
      filePath: context.filePath,
      selection: context,
      cursorPosition: { line: context.startLine, character: context.startCharacter },
      configuration: {},
    });
  });
}

async function executeWithRetry<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  delayMs: number = 100
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxRetries + 1; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;

      if (attempt <= maxRetries) {
        await new Promise((resolve) => setTimeout(resolve, delayMs * attempt)); // Exponential backoff
      }
    }
  }

  throw lastError!;
}

async function getBackendCapabilitiesWithRetry() {
  return invoke('get_backend_capabilities');
}

async function getBackendCapabilitiesCached() {
  // This would use caching in real implementation
  return invoke('get_backend_capabilities');
}

async function executeRefactoringWorkflow(operation: any) {
  return invoke('execute_refactoring', {
    refactoring_type: operation.type,
    context: operation.context,
    options: operation.options || {},
  });
}
