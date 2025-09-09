import { useState, useCallback } from 'react';
import SpecificationService from '../services/SpecificationService';
import type { GeneratedCode, ValidationResult, SpecificationRequest } from '../types';

interface UseSpecificationGenerationReturn {
  // State
  generatedCode: GeneratedCode | null;
  validationResult: ValidationResult | null;
  loading: boolean;
  error: string | null;

  // Actions
  generateFromSpec: (description: string, language?: string, framework?: string) => Promise<void>;
  validateSpec: (description: string) => Promise<ValidationResult>;
  enhanceCode: (enhancements: string[]) => Promise<GeneratedCode | null>;
  exportCode: (basePath: string, overwrite?: boolean) => Promise<void>;

  // Utilities
  clearResults: () => void;
  getCodePreview: (generatedCode: GeneratedCode) => string;
}

/**
 * Custom hook for specification-driven code generation
 */
export const useSpecificationGeneration = (): UseSpecificationGenerationReturn => {
 const [generatedCode, setGeneratedCode] = useState<GeneratedCode | null>(null);
 const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
 const [loading, setLoading] = useState(false);
 const [error, setError] = useState<string | null>(null);

 // Generate code from natural language specification
 const generateFromSpec = useCallback(async (
   description: string,
   language: string = 'rust',
   framework?: string
 ) => {
   try {
     setLoading(true);
     setError(null);

     console.log(`Generating code from specification: ${description.substring(0, 100)}...`);

     const result = await SpecificationService.generateFromSpecification(
       description,
       language,
       framework
     );

     setGeneratedCode(result);

     console.log(`Code generation completed:`, {
       modulesCount: result.modules.length,
       filesCount: result.supportingFiles.length,
       dependenciesCount: result.dependencies.length,
       score: result.validationReport.score,
     });

   } catch (err) {
     const errorMessage = err instanceof Error ? err.message : 'Code generation failed';
     setError(errorMessage);
     console.error('Code generation failed:', err);
     throw new Error(errorMessage);
   } finally {
     setLoading(false);
   }
 }, [setGeneratedCode, setError]);

 // Validate specification
 const validateSpec = useCallback(async (description: string): Promise<ValidationResult> => {
   try {
     const result = await SpecificationService.validateSpecification(description);
     setValidationResult(result);
     return result;
   } catch (err) {
     const errorMessage = err instanceof Error ? err.message : 'Validation failed';
     console.error('Specification validation failed:', err);
     throw new Error(errorMessage);
   }
 }, [setValidationResult]);

  // Enhance generated code with additional features
  const enhanceCode = useCallback(async (enhancements: string[]): Promise<GeneratedCode | null> => {
    if (!generatedCode) {
      throw new Error('No code to enhance. Generate code first.');
    }

    try {
      setLoading(true);
      setError(null);

      const enhanced = await SpecificationService.enhanceGeneratedCode(generatedCode, enhancements);
      setGeneratedCode(enhanced);

      console.log(`Code enhancement completed with ${enhancements.length} enhancements`);
      return enhanced;

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Code enhancement failed';
      setError(errorMessage);
      console.error('Code enhancement failed:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [generatedCode, setGeneratedCode, setError]);

  // Export generated code to filesystem
  const exportCode = useCallback(async (basePath: string, overwrite: boolean = false) => {
    if (!generatedCode) {
      throw new Error('No code to export. Generate code first.');
    }

    try {
      setLoading(true);
      setError(null);

      const result = await SpecificationService.exportToFilesystem(generatedCode, basePath, overwrite);

      console.log(`Code exported successfully:`, result);

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Export failed';
      setError(errorMessage);
      console.error('Code export failed:', err);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [generatedCode, setError]);

  // Clear all results
  const clearResults = useCallback(() => {
    setGeneratedCode(null);
    setValidationResult(null);
    setError(null);
  }, [setGeneratedCode, setValidationResult, setError]);

  // Get code preview for display
  const getCodePreview = useCallback((code: GeneratedCode): string => {
    const mainFile = code.mainFile;

    let preview = `// ${mainFile.description}\n`;
    preview += `// Dependencies: ${code.dependencies.join(', ')}\n`;
    preview += `// Build: ${code.buildInstructions[0] || 'cargo build'}\n\n`;
    preview += mainFile.content;

    // Limit to first 20 lines for preview
    const lines = preview.split('\n');
    if (lines.length > 20) {
      return lines.slice(0, 20).join('\n') + '\n// ... (truncated for preview)';
    }

    return preview;
  }, []);

  return {
    generatedCode,
    validationResult,
    loading,
    error,
    generateFromSpec,
    validateSpec,
    enhanceCode,
    exportCode,
    clearResults,
    getCodePreview,
  };
};