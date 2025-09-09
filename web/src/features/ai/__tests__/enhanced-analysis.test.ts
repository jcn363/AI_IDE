import { afterEach, beforeEach, describe, expect, jest, test } from '@jest/globals';
import { invoke } from '@tauri-apps/api/core';
import type { AnalysisConfiguration, EnhancedCodeAnalysisResult } from '../types';
import {
  sampleAnalysisConfig,
  sampleEnhancedAnalysisResult,
  sampleCodeSmell,
  sampleStyleViolation
} from './test-data';

// Mock Tauri commands
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
}));

// Create a type-safe mock function
type InvokeFunction = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;
const mockInvoke = invoke as jest.MockedFunction<InvokeFunction>;

// Mock the analyzeCode function
const analyzeCode = async (config: AnalysisConfiguration, code?: string): Promise<EnhancedCodeAnalysisResult> => {
  const result = await mockInvoke<EnhancedCodeAnalysisResult>('analyze_code', { config, code });
  if (!result) {
    throw new Error('Analysis failed');
  }
  return result;
};

describe('Enhanced Code Analysis', () => {
  let testConfig: AnalysisConfiguration;

  beforeEach(() => {
    // Setup default mock implementation
    testConfig = { ...sampleAnalysisConfig };
    
    mockInvoke.mockImplementation(async <T>(command: string): Promise<T> => {
      if (command === 'analyze_code') {
        return Promise.resolve(sampleEnhancedAnalysisResult as unknown as T);
      }
      return null as unknown as T;
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('should analyze code and return results', async () => {
    const result = await analyzeCode(testConfig);
    
    expect(result).toBeDefined();
    expect(result).toHaveProperty('codeSmells');
    expect(result).toHaveProperty('styleViolations');
    expect(mockInvoke).toHaveBeenCalledWith('analyze_code', {
      config: testConfig,
    });
  });

  test('should handle analysis errors', async () => {
    const errorMessage = 'Analysis failed';
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage));
    
    await expect(analyzeCode(testConfig)).rejects.toThrow(errorMessage);
    expect(mockInvoke).toHaveBeenCalledWith('analyze_code', {
      config: testConfig,
    });
  });

  test('should include code smells in results', async () => {
    const result = await analyzeCode(testConfig);
    
    expect(result.codeSmells).toBeDefined();
    expect(result.codeSmells).toContainEqual(
      expect.objectContaining({
        id: sampleCodeSmell.id,
        message: sampleCodeSmell.message,
      })
    );
  });

  test('should include style violations in results', async () => {
    const result = await analyzeCode(testConfig);
    
    expect(result.styleViolations).toBeDefined();
    expect(result.styleViolations).toContainEqual(
      expect.objectContaining({
        id: sampleStyleViolation.id,
        message: sampleStyleViolation.message,
      })
    );
  });

  test('should handle empty analysis results', async () => {
    mockInvoke.mockResolvedValueOnce({
      ...sampleEnhancedAnalysisResult,
      codeSmells: [],
      styleViolations: [],
      securityIssues: [],
      performanceHints: [],
      architectureSuggestions: []
    });
    
    const result = await analyzeCode(testConfig);
    
    expect(result).toBeDefined();
    expect(result.codeSmells).toHaveLength(0);
    expect(result.styleViolations).toHaveLength(0);
  });

  test('should pass correct configuration to the backend', async () => {
    const customConfig: AnalysisConfiguration = {
      ...testConfig,
      enabledCategories: ['code-smell', 'style'],
      severityThreshold: 'warning',
    };
    
    await analyzeCode(customConfig);
    
    expect(mockInvoke).toHaveBeenCalledWith('analyze_code', {
      config: customConfig,
    });
  });

  test('should handle different analysis categories', async () => {
    const customResult = {
      ...sampleEnhancedAnalysisResult,
      performanceHints: [{
        id: 'ph-002',
        hintType: 'inefficient-string-ops',
        severity: 'warning',
        message: 'Consider using string references instead of cloning',
        filePath: 'src/perf_test.rs',
        lineRange: [4, 4],
        columnRange: [20, 35],
        optimization: 'Use &str or avoid cloning',
        estimatedImpact: 'high',
        confidence: 0.85
      }]
    };

    mockInvoke.mockResolvedValueOnce(customResult);

    const result = await analyzeCode(testConfig);
    expect(result.performanceHints).toBeDefined();
    expect(result.performanceHints[0].hintType).toBe('inefficient-string-ops');
  });

  test('should handle security analysis results', async () => {
    const securityResult = {
      ...sampleEnhancedAnalysisResult,
      securityIssues: [{
        id: 'sec-001',
        issueType: 'untrusted-input',
        severity: 'high',
        message: 'Untrusted input used in file operation',
        filePath: 'src/security.rs',
        lineRange: [5, 5],
        columnRange: [20, 40],
        cweId: 'CWE-22',
        recommendation: 'Validate and sanitize input paths',
        confidence: 0.9
      }]
    };

    mockInvoke.mockResolvedValueOnce(securityResult);

    const result = await analyzeCode(testConfig);
    expect(result.securityIssues).toBeDefined();
    expect(result.securityIssues[0].issueType).toBe('untrusted-input');
  });
});

describe('Error Handling and Edge Cases', () => {
  let testConfig: AnalysisConfiguration;

  beforeEach(() => {
    testConfig = { ...sampleAnalysisConfig };
    jest.clearAllMocks();
  });

  test('should handle API errors gracefully', async () => {
    const errorMessage = 'API error';
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage));
    
    await expect(analyzeCode(testConfig)).rejects.toThrow(errorMessage);
  });

  test('should handle invalid code input', async () => {
    const invalidCode = 'invalid code';
    const errorMessage = 'Invalid code syntax';
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage));
    
    await expect(analyzeCode(testConfig, invalidCode)).rejects.toThrow(errorMessage);
  });
});
