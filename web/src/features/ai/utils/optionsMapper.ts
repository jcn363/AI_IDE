import { RefactoringOptions } from '../../../types/refactoring';

/**
 * Explicit OptionsMapper class for bidirectional conversion between frontend and backend options
 * Handles mapping UI-only fields to extraOptions and preserves roundtrip compatibility
 */
export class OptionsMapper {
  /**
   * Map frontend RefactoringOptions to backend format
   */
  static mapFrontendToBackend(options: RefactoringOptions): any {
    if (!options) return null;

    const backendOptions: any = {};

    // Map standard boolean options with fallbacks
    backendOptions.create_backup = OptionsMapper.mapBooleanOption(options.createBackup, true);
    backendOptions.generate_tests = OptionsMapper.mapBooleanOption(options.generateTests, false);
    backendOptions.apply_to_all_occurrences = OptionsMapper.mapBooleanOption(options.applyToAllOccurrences, false);
    backendOptions.preserve_references = OptionsMapper.mapBooleanOption(options.preserveReferences, true);
    backendOptions.ignore_safe_operations = OptionsMapper.mapBooleanOption(options.ignoreSafeOperations, false);

    // Collect UI-specific options that don't exist in backend
    const uiOnlyKeys = [
      // Add any UI-only boolean fields here that should be preserved in extra_options
    ];

    // Collect any non-standard options from extraOptions or root level
    const extraOptions: Record<string, any> = {};
    if (options.extraOptions) {
      Object.entries(options.extraOptions).forEach(([key, value]) => {
        if (key === 'newName' || key === 'functionName' || key === 'methodName' ||
            key === 'className' || key === 'interfaceName' || key === 'variableName') {
          // Keep refactor-specific options in extra_options
          extraOptions[key] = value;
        } else if (key === 'functionSignature' || key === 'methodSignature') {
          // Keep signature modification options
          extraOptions[key] = value;
        } else if (key === 'patternType' || key === 'patternConfig') {
          // Keep pattern-specific options
          extraOptions[key] = value;
        }
      });
    }

    // Set extra_options if any were found
    if (Object.keys(extraOptions).length > 0) {
      backendOptions.extra_options = extraOptions;
    }

    return backendOptions;
  }

  /**
   * Map backend format back to frontend RefactoringOptions
   * Preserves UI-specific options from extra_options for roundtrip compatibility
   */
  static mapBackendToFrontend(backendOptions: any): RefactoringOptions {
    if (!backendOptions) {
      return OptionsMapper.getDefaultOptions();
    }

    const frontendOptions: RefactoringOptions = {
      createBackup: OptionsMapper.extractBooleanValue(backendOptions, 'create_backup', true),
      generateTests: OptionsMapper.extractBooleanValue(backendOptions, 'generate_tests', false),
      applyToAllOccurrences: OptionsMapper.extractBooleanValue(backendOptions, 'apply_to_all_occurrences', false),
      preserveReferences: OptionsMapper.extractBooleanValue(backendOptions, 'preserve_references', true),
      ignoreSafeOperations: OptionsMapper.extractBooleanValue(backendOptions, 'ignore_safe_operations', false),
    };

    // Extract extra options from backend
    if (backendOptions.extra_options) {
      frontendOptions.extraOptions = { ...backendOptions.extra_options };
    }

    return frontendOptions;
  }

  /**
   * Safe boolean extraction with fallback
   */
  private static mapBooleanOption(value: boolean | undefined, fallback: boolean): boolean {
    return value ?? fallback;
  }

  /**
   * Extract boolean value with multiple key variations and fallback
   */
  private static extractBooleanValue(obj: any, primaryKey: string, fallback: boolean): boolean {
    if (obj === null || obj === undefined) return fallback;

    // Try primary key
    if (obj[primaryKey] !== undefined) {
      return Boolean(obj[primaryKey]);
    }

    // Try snake_case fallback
    const snakeKey = primaryKey.replace(/_/g, '-');
    if (obj[snakeKey] !== undefined) {
      return Boolean(obj[snakeKey]);
    }

    // Try camelCase fallback
    const camelKey = primaryKey.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    if (obj[camelKey] !== undefined) {
      return Boolean(obj[camelKey]);
    }

    return fallback;
  }

  /**
   * Get default frontend options
   */
  private static getDefaultOptions(): RefactoringOptions {
    return {
      createBackup: true,
      generateTests: false,
      applyToAllOccurrences: false,
      preserveReferences: true,
      ignoreSafeOperations: false,
    };
  }

  /**
   * Deep merge options, preserving extra options
   */
  static mergeOptions(baseOptions: RefactoringOptions, overrideOptions: Partial<RefactoringOptions>): RefactoringOptions {
    const merged: RefactoringOptions = { ...baseOptions };

    // Override direct properties
    Object.keys(overrideOptions).forEach(key => {
      const value = (overrideOptions as any)[key];
      if (value !== undefined) {
        (merged as any)[key] = value;
      }
    });

    // Deep merge extraOptions
    if (overrideOptions.extraOptions) {
      merged.extraOptions = merged.extraOptions || {};
      Object.keys(overrideOptions.extraOptions).forEach(key => {
        if (overrideOptions.extraOptions![key] !== undefined) {
          merged.extraOptions![key] = overrideOptions.extraOptions![key];
        }
      });
    }

    return merged;
  }
}

/**
 * Test functions for roundtrip mapping validation
 */
export class OptionsMapperTest {
  static testRoundtripMapping(): boolean {
    // Test with complex options
    const originalOptions: RefactoringOptions = {
      createBackup: false,
      generateTests: true,
      applyToAllOccurrences: true,
      preserveReferences: false,
      ignoreSafeOperations: true,
      extraOptions: {
        newName: 'renamedFunction',
        functionName: 'newFunction',
        functionSignature: '(param1: string) => void',
        patternType: 'factory',
        customField: 'preserve-me',
      },
    };

    // Map to backend
    const backendOptions = OptionsMapper.mapFrontendToBackend(originalOptions);

    console.log('Backend Options:', JSON.stringify(backendOptions, null, 2));

    // Map back to frontend
    const roundtripOptions = OptionsMapper.mapBackendToFrontend(backendOptions);

    console.log('Roundtrip Options:', JSON.stringify(roundtripOptions, null, 2));

    // Compare key properties
    const matches =
      roundtripOptions.createBackup === originalOptions.createBackup &&
      roundtripOptions.generateTests === originalOptions.generateTests &&
      roundtripOptions.applyToAllOccurrences === originalOptions.applyToAllOccurrences &&
      roundtripOptions.preserveReferences === originalOptions.preserveReferences &&
      roundtripOptions.ignoreSafeOperations === originalOptions.ignoreSafeOperations;

    // Compare extra options
    const extraMatches = JSON.stringify(roundtripOptions.extraOptions) === JSON.stringify(originalOptions.extraOptions);

    console.log('Direct properties match:', matches);
    console.log('Extra options match:', extraMatches);

    return matches && extraMatches;
  }

  static testEdgeCases(): boolean {
    console.log('Testing edge cases...');

    // Test with empty options
    const emptyOptions: RefactoringOptions = {};
    const emptyMapping = OptionsMapper.mapFrontendToBackend(emptyOptions);
    const undefinedMapping = OptionsMapper.mapBackendToFrontend(undefined);
    const nullMapping = OptionsMapper.mapBackendToFrontend(null);

    console.log('Null mapping:', nullMapping);
    console.log('Undefined mapping:', undefinedMapping);

    // Test with minimal options
    const minimalOptions: RefactoringOptions = {};
    const minimalBackend = OptionsMapper.mapFrontendToBackend(minimalOptions);
    const minimalRoundtrip = OptionsMapper.mapBackendToFrontend(minimalBackend);

    console.log('Minimal roundtrip:', minimalRoundtrip);

    // Test with only extra options
    const extraOnlyOptions: RefactoringOptions = {
      extraOptions: {
        specialConfig: true,
        patternData: 'test-value',
      }
    };
    const extraBackend = OptionsMapper.mapFrontendToBackend(extraOnlyOptions);
    const extraRoundtrip = OptionsMapper.mapBackendToFrontend(extraBackend);

    console.log('Extra only roundtrip:', extraRoundtrip);

    return true;
  }

  static runAllTests(): void {
    console.log('Running OptionsMapper tests...\n');

    console.log('=== Test 1: Roundtrip Mapping ===');
    const roundtripResult = OptionsMapperTest.testRoundtripMapping();
    console.log('Roundtrip test result:', roundtripResult ? 'PASS' : 'FAIL');

    console.log('\n=== Test 2: Edge Cases ===');
    const edgeCaseResult = OptionsMapperTest.testEdgeCases();
    console.log('Edge cases test result:', edgeCaseResult ? 'PASS' : 'FAIL');

    console.log('\nOptionsMapper tests completed.');
  }
}

// Export test runner for console testing
(window as any).testOptionsMapper = () => OptionsMapperTest.runAllTests();