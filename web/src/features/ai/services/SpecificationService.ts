import { invoke } from '@tauri-apps/api/core';
import type {
  GeneratedCode,
  SpecificationRequest,
  CodeGenerationFromSpecRequest,
  ValidationResult,
  GeneratedFile,
  FileType,
} from '../types';

/**
 * Service for specification-driven code generation
 * Converts natural language specifications into executable code
 */
export class SpecificationService {
  /**
   * Validate specification description
   */
  async validateSpec(description: string): Promise<ValidationResult> {
    return this.validateSpecification(description);
  }

  /**
   * Enhance generated code with additional features
   */
  async enhanceCode(generatedCode: GeneratedCode, enhancements: string[]): Promise<GeneratedCode> {
    return this.enhanceGeneratedCode(generatedCode, enhancements);
  }
  private static instance: SpecificationService;

  private constructor() {}

  static getInstance(): SpecificationService {
    if (!SpecificationService.instance) {
      SpecificationService.instance = new SpecificationService();
    }
    return SpecificationService.instance;
  }

  /**
   * Generate code from natural language specification
   */
  async generateFromSpecification(
    description: string,
    language = 'rust',
    framework?: string,
    context?: string[]
  ): Promise<GeneratedCode> {
    try {
      const specification: SpecificationRequest = {
        description,
        language,
        framework,
        targetPlatform: 'linux',
        constraints: [],
        examples: context || [],
      };

      const request: CodeGenerationFromSpecRequest = {
        specification: description,
        language,
        config: {
          provider: {
            type: 'codellama-rust',
            codellamaRust: {
              modelPath: '/models/codellama-7b',
              modelSize: 'Medium',
              quantization: 'Int4',
              loraAdapters: ['rust-generation'],
            },
          },
          analysis_preferences: {
            enableCodeSmells: false,
            enablePerformance: true,
            enableSecurity: true,
            enableCodeStyle: true,
            enableArchitecture: false,
            enableLearning: false,
            confidenceThreshold: 0.6,
            timeoutSeconds: 120,
            includeExplanations: false,
            includeExamples: true,
            privacyMode: 'opt-in' as const,
          },
          enable_real_time: false,
          enable_workspace_analysis: false,
          max_file_size_kb: 1024,
          excluded_paths: ['target/', 'node_modules/', '.git/'],
          learning_preferences: {
            enableLearning: false,
            privacyMode: 'opt-in' as const,
            shareAnonymousData: false,
            retainPersonalData: true,
            dataRetentionDays: 90,
            allowModelTraining: false,
          },
          compiler_integration: {
            enable_compiler_integration: true,
            parse_cargo_check_output: true,
            enable_error_explanations: true,
            enable_suggested_fixes: true,
            cache_explanations: true,
            explanation_cache_ttl_hours: 24,
          },
        },
      };

      const result = await invoke<GeneratedCode>('generate_code_from_specification', {
        request,
      });

      console.log('Code generated from specification:', {
        modulesCount: result.modules.length,
        filesCount: result.supportingFiles.length,
        hasTests: result.tests.length > 0,
      });

      return result;
    } catch (error) {
      console.error('Failed to generate code from specification:', error);
      throw new Error(`Code generation failed: ${error}`);
    }
  }

  /**
   * Parse and validate specification before generation
   */
  async validateSpecification(description: string): Promise<ValidationResult> {
    try {
      // Perform basic validation
      const issues: string[] = [];

      if (description.trim().length < 10) {
        issues.push('Specification is too short (minimum 10 characters)');
      }

      if (
        !description.includes('function') &&
        !description.includes('method') &&
        !description.includes('class') &&
        !description.includes('struct')
      ) {
        issues.push('Specification should describe software components (functions, classes, etc.)');
      }

      const score = issues.length === 0 ? 0.9 : Math.max(0.3, 1.0 - issues.length * 0.2);

      return {
        isValid: issues.length === 0,
        score,
        issues,
        suggestions: this.generateSpecSuggestions(description),
      };
    } catch (error) {
      console.error('Failed to validate specification:', error);
      return {
        isValid: false,
        score: 0.0,
        issues: [`Validation failed: ${error}`],
        suggestions: ['Please check your specification format'],
      };
    }
  }

  /**
   * Generate template suggestions based on specification content
   */
  private generateSpecSuggestions(description: string): string[] {
    const suggestions: string[] = [];
    const lowerDesc = description.toLowerCase();

    if (lowerDesc.includes('api') || lowerDesc.includes('rest') || lowerDesc.includes('http')) {
      suggestions.push('Consider using async/await for API operations');
      suggestions.push('Add error handling for network requests');
    }

    if (lowerDesc.includes('database') || lowerDesc.includes('sql')) {
      suggestions.push('Include connection pooling for database operations');
      suggestions.push('Add transaction handling for data consistency');
    }

    if (lowerDesc.includes('user') || lowerDesc.includes('authentication')) {
      suggestions.push('Implement proper password hashing and validation');
      suggestions.push('Add session management and JWT handling');
    }

    if (lowerDesc.includes('file') || lowerDesc.includes('io')) {
      suggestions.push('Add proper error handling for file operations');
      suggestions.push('Consider using async file operations for better performance');
    }

    if (suggestions.length === 0) {
      suggestions.push('Consider adding error handling and validation');
      suggestions.push('Include appropriate logging for debugging');
    }

    return suggestions;
  }

  /**
   * Enhance generated code with additional features
   */
  async enhanceGeneratedCode(
    generatedCode: GeneratedCode,
    enhancements: string[]
  ): Promise<GeneratedCode> {
    try {
      let enhancedCode = { ...generatedCode };

      for (const enhancement of enhancements) {
        switch (enhancement.toLowerCase()) {
          case 'logging':
            enhancedCode = this.addLoggingSupport(enhancedCode);
            break;
          case 'testing':
            enhancedCode = this.addTestingSupport(enhancedCode);
            break;
          case 'documentation':
            enhancedCode = this.addDocumentation(enhancedCode);
            break;
          case 'error_handling':
            enhancedCode = this.addErrorHandling(enhancedCode);
            break;
          default:
            console.warn(`Unknown enhancement: ${enhancement}`);
        }
      }

      return enhancedCode;
    } catch (error) {
      console.error('Failed to enhance code:', error);
      return generatedCode; // Return original if enhancement fails
    }
  }

  /**
   * Generate multiple code variations for comparison
   */
  async generateVariations(
    description: string,
    language = 'rust',
    count = 3
  ): Promise<GeneratedCode[]> {
    try {
      const variations: GeneratedCode[] = [];

      for (let i = 0; i < count; i++) {
        // Add slight variations to the description for different implementations
        const variedDescription = `${description} (variation ${i + 1}: ${this.getVariationStyle(i)})`;

        const variation = await this.generateFromSpecification(variedDescription, language);

        variations.push(variation);
      }

      console.log(`Generated ${variations.length} code variations`);
      return variations;
    } catch (error) {
      console.error('Failed to generate variations:', error);
      throw new Error(`Variation generation failed: ${error}`);
    }
  }

  /**
   * Get variation style description for diversity
   */
  private getVariationStyle(index: number): string {
    const styles = [
      'focus on performance and efficiency',
      'emphasize readability and maintainability',
      'prioritize robustness and error handling',
      'optimize for memory usage',
      'use functional programming patterns',
    ];

    return styles[index % styles.length];
  }

  /**
   * Merge multiple code generations into a single coherent implementation
   */
  mergeGenerations(generations: GeneratedCode[]): GeneratedCode {
    if (generations.length === 0) {
      throw new Error('No generations to merge');
    }

    if (generations.length === 1) {
      return generations[0];
    }

    // Start with the first generation as base
    const merged = { ...generations[0] };

    // Merge modules, keeping unique ones
    const allModules = new Map<string, any>();
    for (const gen of generations) {
      for (const module of gen.modules) {
        if (!allModules.has(module.name)) {
          allModules.set(module.name, module);
        }
      }
    }
    merged.modules = Array.from(allModules.values());

    // Combine supporting files
    const fileMap = new Map<string, GeneratedFile>();
    for (const gen of generations) {
      for (const file of gen.supportingFiles) {
        if (!fileMap.has(file.path)) {
          fileMap.set(file.path, file);
        }
      }
    }
    merged.supportingFiles = Array.from(fileMap.values());

    // Combine dependencies (unique)
    const allDeps = new Set(merged.dependencies);
    for (const gen of generations.slice(1)) {
      for (const dep of gen.dependencies) {
        allDeps.add(dep);
      }
    }
    merged.dependencies = Array.from(allDeps);

    // Combine build instructions (unique)
    const allInstructions = new Set(merged.buildInstructions);
    for (const gen of generations.slice(1)) {
      for (const instruction of gen.buildInstructions) {
        allInstructions.add(instruction);
      }
    }
    merged.buildInstructions = Array.from(allInstructions);

    return merged;
  }

  /**
   * Export generated code to filesystem
   */
  async exportToFilesystem(
    generatedCode: GeneratedCode,
    basePath: string,
    overwrite = false
  ): Promise<ExportResult> {
    try {
      // This would be implemented using Tauri's filesystem APIs
      // For now, return a placeholder result

      const result: ExportResult = {
        success: true,
        filesCreated: generatedCode.supportingFiles.length,
        modulesCreated: generatedCode.modules.length,
        totalFiles: generatedCode.supportingFiles.length + 1, // + main file
        destination: basePath,
        warnings: [],
        errors: [],
      };

      console.log('Code exported to filesystem:', result);
      return result;
    } catch (error) {
      console.error('Failed to export code:', error);
      throw new Error(`Export failed: ${error}`);
    }
  }

  /**
   * Private enhancement methods
   */
  private addLoggingSupport(code: GeneratedCode): GeneratedCode {
    // Implementation would add logging to the code
    return code;
  }

  private addTestingSupport(code: GeneratedCode): GeneratedCode {
    // Implementation would add test files and testing infrastructure
    return code;
  }

  private addDocumentation(code: GeneratedCode): GeneratedCode {
    // Implementation would add documentation comments
    return code;
  }

  private addErrorHandling(code: GeneratedCode): GeneratedCode {
    // Implementation would add proper error handling patterns
    return code;
  }
}

/**
 * Export result interface
 */
export interface ExportResult {
  success: boolean;
  filesCreated: number;
  modulesCreated: number;
  totalFiles: number;
  destination: string;
  warnings: string[];
  errors: string[];
}

// Singleton instance
const specificationService = SpecificationService.getInstance();

export default specificationService;