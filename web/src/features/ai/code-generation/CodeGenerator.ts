import { TextEdit } from 'vscode-languageserver';
import type { Range } from 'vscode-languageserver-types';
import { invoke } from '@tauri-apps/api/core';
import type { 
  CodeGenerationOptions as CodeGenOpts, 
  GeneratedCode, 
  AIContext,
  AnalysisPreferences,
  Dependency,
  CodeAnalysisResult,
  AIProvider,
  PerformanceHint,
  StyleViolation,
  ArchitectureSuggestion,
  CodeSmell
} from '../types';

export type { GeneratedCode };
export type CodeGenerationOptions = CodeGenOpts;

// Enhanced generation request for backend
interface CodeGenerationRequest {
  generation_type: 'test' | 'documentation' | 'boilerplate' | 'example' | 'stub';
  context: AIContext;
  options: {
    max_length?: number;
    temperature?: number;
    include_examples?: boolean;
    include_edge_cases?: boolean;
    style_preferences?: StylePreferences;
    incremental?: boolean;
    chunk_size?: number;
  };
  template_hints?: TemplateHints;
}

interface StylePreferences {
  naming_convention: 'snake_case' | 'camelCase' | 'PascalCase' | 'kebab-case';
  documentation_style: 'rustdoc' | 'jsdoc' | 'sphinx' | 'minimal';
  test_framework: 'built_in' | 'jest' | 'mocha' | 'pytest' | 'custom';
  code_style: 'compact' | 'verbose' | 'idiomatic';
}

interface TemplateHints {
  function_signature?: string;
  expected_return_type?: string;
  parameter_types?: Record<string, string>;
  imports_needed?: string[];
  traits_to_implement?: string[];
  error_handling_style?: 'result' | 'option' | 'panic' | 'exception';
}

interface GenerationProgress {
  id: string;
  status: 'queued' | 'generating' | 'completed' | 'failed';
  progress_percentage: number;
  current_chunk?: number;
  total_chunks?: number;
  generated_so_far?: string;
  estimated_completion?: string;
}

interface ProjectContext {
  workspace_structure: Record<string, string[]>;
  dependencies: Dependency[];
  existing_patterns: CodePattern[];
  style_guide?: StyleGuide;
  recent_analysis?: CodeAnalysisResult;
}

interface CodePattern {
  pattern_type: 'function' | 'struct' | 'enum' | 'trait' | 'module';
  name: string;
  signature: string;
  usage_frequency: number;
  examples: string[];
}

interface StyleGuide {
  max_line_length: number;
  indent_style: 'spaces' | 'tabs';
  indent_size: number;
  naming_conventions: Record<string, string>;
  documentation_requirements: string[];
}

export class CodeGenerator {
  private static instance: CodeGenerator;
  private isInitialized = false;
  private cache = new Map<string, { result: GeneratedCode; timestamp: number }>();
  private pendingGenerations = new Map<string, Promise<GeneratedCode>>();
  private templates = new Map<string, CodeTemplate>();
  private projectContext: ProjectContext | null = null;
  private defaultStylePreferences: StylePreferences;
  
  private constructor() {
    this.defaultStylePreferences = {
      naming_convention: 'snake_case',
      documentation_style: 'rustdoc',
      test_framework: 'built_in',
      code_style: 'idiomatic',
    };
    this.initializeTemplates();
  }

  public static getInstance(): CodeGenerator {
    if (!CodeGenerator.instance) {
      CodeGenerator.instance = new CodeGenerator();
    }
    return CodeGenerator.instance;
  }

  public async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    try {
      // Initialize AI service if not already done
      await this.ensureAIServiceInitialized();
      
      // Load project context
      await this.loadProjectContext();
      
      // Initialize code templates
      await this.loadCodeTemplates();
      
      this.isInitialized = true;
      console.log('CodeGenerator initialized successfully');
    } catch (error) {
      console.error('Failed to initialize CodeGenerator:', error);
      throw new Error(`CodeGenerator initialization failed: ${error}`);
    }
  }

  private async ensureAIServiceInitialized(): Promise<void> {
    try {
      // Check if AI service is already initialized
      await invoke('get_ai_config');
    } catch (error) {
      // Initialize with default config if not initialized
      const defaultConfig = {
        provider: { OpenAI: { api_key: '', model: 'gpt-4' } },
        analysis_preferences: {
          enable_code_smells: true,
          enable_performance: true,
          enable_security: true,
          enable_style: true,
          enable_architecture: true,
          timeout_seconds: 30,
          max_suggestions_per_category: 10,
        },
        enable_real_time: true,
        enable_workspace_analysis: true,
        max_file_size_kb: 1024,
        excluded_paths: ['target/', 'node_modules/', '.git/'],
        learning_preferences: {
          enable_learning: true,
          privacy_mode: 'opt_in',
          share_anonymous_data: false,
          retain_personal_data: true,
          data_retention_days: 90,
          allow_model_training: false,
          confidence_threshold_for_learning: 0.8,
        },
        compiler_integration: {
          enable_compiler_integration: true,
          parse_cargo_check_output: true,
          enable_error_explanations: true,
          enable_suggested_fixes: true,
          cache_explanations: true,
          explanation_cache_ttl_hours: 24,
        },
      };
      
      await invoke('initialize_ai_service', { config: defaultConfig });
    }
  }

  private initializeTemplates(): void {
    // Initialize built-in code templates
    this.templates.set('rust_test', {
      name: 'Rust Unit Test',
      pattern: `#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{{function_name}}() {
        // Arrange
        {{test_setup}}
        
        // Act
        let result = {{function_call}};
        
        // Assert
        {{assertions}}
    }
    
    {{additional_tests}}
}`,
      variables: ['function_name', 'test_setup', 'function_call', 'assertions', 'additional_tests'],
      language: 'rust',
    });

    this.templates.set('rust_documentation', {
      name: 'Rust Documentation',
      pattern: `/// {{brief_description}}
///
/// {{detailed_description}}
///
/// # Arguments
///
{{parameter_docs}}
///
/// # Returns
///
/// {{return_description}}
///
/// # Examples
///
/// \`\`\`
/// {{example_code}}
/// \`\`\`
///
/// # Errors
///
/// {{error_conditions}}`,
      variables: ['brief_description', 'detailed_description', 'parameter_docs', 'return_description', 'example_code', 'error_conditions'],
      language: 'rust',
    });

    this.templates.set('rust_struct', {
      name: 'Rust Struct with Implementation',
      pattern: `/// {{struct_description}}
#[derive(Debug, Clone{{additional_derives}})]
pub struct {{struct_name}} {
    {{fields}}
}

impl {{struct_name}} {
    /// Creates a new instance of {{struct_name}}
    pub fn new({{constructor_params}}) -> Self {
        Self {
            {{field_assignments}}
        }
    }
    
    {{additional_methods}}
}

{{trait_implementations}}`,
      variables: ['struct_description', 'struct_name', 'additional_derives', 'fields', 'constructor_params', 'field_assignments', 'additional_methods', 'trait_implementations'],
      language: 'rust',
    });
  }

  public async generateTestCases(options: CodeGenerationOptions): Promise<GeneratedCode> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const cacheKey = this.generateCacheKey('test', options);
    const cached = this.getCachedResult(cacheKey);
    if (cached) {
      return cached;
    }

    const context = await this.buildAIContext(options);
    const request: CodeGenerationRequest = {
      generation_type: 'test',
      context,
      options: {
        max_length: options.maxLength || 2000,
        temperature: options.temperature || 0.3,
        include_examples: true,
        include_edge_cases: true,
        style_preferences: this.getStylePreferences(options),
        incremental: false,
      },
      template_hints: await this.extractTemplateHints(options, 'test'),
    };

    try {
      const result = await this.generateWithBackend(request);
      this.cacheResult(cacheKey, result);
      return result;
    } catch (error) {
      console.error('Test generation failed, falling back to template:', error);
      return this.generateWithTemplate('test', options);
    }
  }

  public async generateDocumentation(options: CodeGenerationOptions): Promise<GeneratedCode> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const cacheKey = this.generateCacheKey('documentation', options);
    const cached = this.getCachedResult(cacheKey);
    if (cached) {
      return cached;
    }

    const context = await this.buildAIContext(options);
    const request: CodeGenerationRequest = {
      generation_type: 'documentation',
      context,
      options: {
        max_length: options.maxLength || 1500,
        temperature: options.temperature || 0.2,
        include_examples: true,
        style_preferences: this.getStylePreferences(options),
        incremental: false,
      },
      template_hints: await this.extractTemplateHints(options, 'documentation'),
    };

    try {
      const result = await this.generateWithBackend(request);
      this.cacheResult(cacheKey, result);
      return result;
    } catch (error) {
      console.error('Documentation generation failed, falling back to template:', error);
      return this.generateWithTemplate('documentation', options);
    }
  }

  public async generateBoilerplate(options: CodeGenerationOptions): Promise<GeneratedCode> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const cacheKey = this.generateCacheKey('boilerplate', options);
    const cached = this.getCachedResult(cacheKey);
    if (cached) {
      return cached;
    }

    const context = await this.buildAIContext(options);
    const request: CodeGenerationRequest = {
      generation_type: 'boilerplate',
      context,
      options: {
        max_length: options.maxLength || 3000,
        temperature: options.temperature || 0.4,
        style_preferences: this.getStylePreferences(options),
        incremental: false,
      },
      template_hints: await this.extractTemplateHints(options, 'boilerplate'),
    };

    try {
      const result = await this.generateWithBackend(request);
      this.cacheResult(cacheKey, result);
      return result;
    } catch (error) {
      console.error('Boilerplate generation failed, falling back to template:', error);
      return this.generateWithTemplate('boilerplate', options);
    }
  }

  public async generateExample(options: CodeGenerationOptions): Promise<GeneratedCode> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const cacheKey = this.generateCacheKey('example', options);
    const cached = this.getCachedResult(cacheKey);
    if (cached) {
      return cached;
    }

    const context = await this.buildAIContext(options);
    const request: CodeGenerationRequest = {
      generation_type: 'example',
      context,
      options: {
        max_length: options.maxLength || 1000,
        temperature: options.temperature || 0.5,
        include_examples: true,
        style_preferences: this.getStylePreferences(options),
        incremental: false,
      },
      template_hints: await this.extractTemplateHints(options, 'example'),
    };

    try {
      const result = await this.generateWithBackend(request);
      this.cacheResult(cacheKey, result);
      return result;
    } catch (error) {
      console.error('Example generation failed, falling back to template:', error);
      return this.generateWithTemplate('example', options);
    }
  }

  public async generateStub(interfaceText: string, options: CodeGenerationOptions): Promise<GeneratedCode> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const enhancedOptions = { ...options, context: `${options.context}\n\nInterface to implement:\n${interfaceText}` };
    const cacheKey = this.generateCacheKey('stub', enhancedOptions);
    const cached = this.getCachedResult(cacheKey);
    if (cached) {
      return cached;
    }

    const context = await this.buildAIContext(enhancedOptions);
    const request: CodeGenerationRequest = {
      generation_type: 'stub',
      context,
      options: {
        max_length: options.maxLength || 2500,
        temperature: options.temperature || 0.3,
        style_preferences: this.getStylePreferences(options),
        incremental: false,
      },
      template_hints: await this.extractTemplateHints(enhancedOptions, 'stub'),
    };

    try {
      const result = await this.generateWithBackend(request);
      this.cacheResult(cacheKey, result);
      return result;
    } catch (error) {
      console.error('Stub generation failed, falling back to template:', error);
      return this.generateWithTemplate('stub', enhancedOptions);
    }
  }

  public async generateIncremental(
    options: CodeGenerationOptions,
    chunkSize: number = 500
  ): Promise<AsyncGenerator<GeneratedCode, void, unknown>> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const context = await this.buildAIContext(options);
    const request: CodeGenerationRequest = {
      generation_type: options.type || 'boilerplate',
      context,
      options: {
        max_length: options.maxLength || 5000,
        temperature: options.temperature || 0.4,
        style_preferences: this.getStylePreferences(options),
        incremental: true,
        chunk_size: chunkSize,
      },
      template_hints: await this.extractTemplateHints(options, options.type || 'boilerplate'),
    };

    return this.generateIncrementalWithBackend(request);
  }

  private async generateWithBackend(request: CodeGenerationRequest): Promise<GeneratedCode> {
    try {
      // Call the backend code generation command (to be implemented in backend)
      const result = await invoke('generate_code', request);
      
      return {
        content: result.content,
        range: this.calculateInsertionRange(request),
        edits: this.createEditsFromContent(request, result.content),
        confidence: result.confidence || 0.8,
        type: request.generation_type,
      };
    } catch (error) {
      console.error('Backend code generation failed:', error);
      throw error;
    }
  }

  private async *generateIncrementalWithBackend(
    request: CodeGenerationRequest
  ): AsyncGenerator<GeneratedCode, void, unknown> {
    try {
      // Start incremental generation (to be implemented in backend)
      const generationId = await invoke('start_incremental_generation', request);
      
      let isComplete = false;
      let chunkIndex = 0;
      
      while (!isComplete) {
        // Poll for next chunk
        const progress: GenerationProgress = await invoke('get_generation_progress', { 
          generation_id: generationId 
        });
        
        if (progress.status === 'failed') {
          throw new Error('Incremental generation failed');
        }
        
        if (progress.generated_so_far && progress.current_chunk !== chunkIndex) {
          chunkIndex = progress.current_chunk || 0;
          
          yield {
            content: progress.generated_so_far,
            range: this.calculateInsertionRange(request),
            edits: this.createEditsFromContent(request, progress.generated_so_far),
            confidence: 0.8 * (progress.progress_percentage / 100),
            type: request.generation_type,
          };
        }
        
        if (progress.status === 'completed') {
          isComplete = true;
        } else {
          // Wait before polling again
          await new Promise(resolve => setTimeout(resolve, 500));
        }
      }
    } catch (error) {
      console.error('Incremental generation failed:', error);
      throw error;
    }
  }

  private async generateWithTemplate(
    type: string, 
    options: CodeGenerationOptions
  ): Promise<GeneratedCode> {
    const templateKey = `${options.language || 'rust'}_${type}`;
    const template = this.templates.get(templateKey);
    
    if (!template) {
      // Fallback to basic generation
      return this.generateBasicFallback(type, options);
    }

    const variables = await this.extractTemplateVariables(template, options);
    const content = this.fillTemplate(template.pattern, variables);
    
    return {
      content,
      range: this.calculateInsertionRange({ generation_type: type as any, context: await this.buildAIContext(options), options: {} }),
      edits: this.createEditsFromContent({ generation_type: type as any, context: await this.buildAIContext(options), options: {} }, content),
      confidence: 0.7, // Lower confidence for template-based generation
      type: type as any,
    };
  }

  private async buildAIContext(options: CodeGenerationOptions): Promise<AIContext> {
    const workspaceStructure = this.projectContext?.workspace_structure || {};
    const dependencies = this.projectContext?.dependencies || [];
    
    // Analyze current code for better context
    let analysisPreferences: AnalysisPreferences = {
      enable_code_smells: true,
      enable_performance: true,
      enable_security: true,
      enable_style: true,
      enable_architecture: true,
      timeout_seconds: 10,
      max_suggestions_per_category: 5,
    };

    // Extract cursor position if available
    const cursorPosition: [number, number] | undefined = options.cursorPosition 
      ? [options.cursorPosition.line, options.cursorPosition.character]
      : undefined;

    // Build project context from recent analysis
    const projectContext: Record<string, string> = {};
    if (this.projectContext?.recent_analysis) {
      projectContext.code_quality_score = this.projectContext.recent_analysis.metrics?.maintainability?.toString() || '0';
      projectContext.complexity_score = this.projectContext.recent_analysis.metrics?.complexity?.toString() || '0';
      
      // Add insights from recent analysis
      if (this.projectContext.recent_analysis.suggestions) {
        const commonIssues = this.projectContext.recent_analysis.suggestions
          .slice(0, 3)
          .map(s => s.message)
          .join('; ');
        projectContext.common_issues = commonIssues;
      }
    }

    // Add existing code patterns
    if (this.projectContext?.existing_patterns) {
      const patterns = this.projectContext.existing_patterns
        .map(p => `${p.pattern_type}: ${p.name}`)
        .join(', ');
      projectContext.existing_patterns = patterns;
    }

    return {
      current_code: options.fileContent,
      file_name: options.filePath,
      cursor_position: cursorPosition,
      selection: undefined,
      project_context: projectContext,
      dependencies,
      workspace_structure: workspaceStructure,
      analysis_preferences: analysisPreferences,
    };
  }

  private async extractTemplateHints(
    options: CodeGenerationOptions, 
    type: string
  ): Promise<TemplateHints> {
    const hints: TemplateHints = {};
    
    // Extract function signature from context
    if (options.context) {
      const functionMatch = options.context.match(/fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*([^{]+))?/);
      if (functionMatch) {
        hints.function_signature = functionMatch[0];
        hints.expected_return_type = functionMatch[2]?.trim();
      }
      
      // Extract parameter types
      const paramMatches = options.context.matchAll(/(\w+):\s*([^,)]+)/g);
      hints.parameter_types = {};
      for (const match of paramMatches) {
        hints.parameter_types[match[1]] = match[2].trim();
      }
      
      // Extract imports
      const importMatches = options.context.matchAll(/use\s+([^;]+);/g);
      hints.imports_needed = Array.from(importMatches, m => m[1].trim());
      
      // Determine error handling style
      if (options.context.includes('Result<')) {
        hints.error_handling_style = 'result';
      } else if (options.context.includes('Option<')) {
        hints.error_handling_style = 'option';
      } else if (options.context.includes('panic!')) {
        hints.error_handling_style = 'panic';
      }
    }
    
    return hints;
  }

  private getStylePreferences(options: CodeGenerationOptions): StylePreferences {
    // Use project-specific style guide if available
    if (this.projectContext?.style_guide) {
      return {
        naming_convention: this.inferNamingConvention(this.projectContext.style_guide),
        documentation_style: options.language === 'rust' ? 'rustdoc' : 'jsdoc',
        test_framework: this.inferTestFramework(options),
        code_style: 'idiomatic',
      };
    }
    
    return this.defaultStylePreferences;
  }

  private inferNamingConvention(styleGuide: StyleGuide): 'snake_case' | 'camelCase' | 'PascalCase' | 'kebab-case' {
    const conventions = styleGuide.naming_conventions;
    if (conventions.function?.includes('snake_case') || conventions.variable?.includes('snake_case')) {
      return 'snake_case';
    }
    if (conventions.function?.includes('camelCase') || conventions.variable?.includes('camelCase')) {
      return 'camelCase';
    }
    if (conventions.type?.includes('PascalCase') || conventions.struct?.includes('PascalCase')) {
      return 'PascalCase';
    }
    return 'snake_case'; // Default for Rust
  }

  private inferTestFramework(options: CodeGenerationOptions): 'built_in' | 'jest' | 'mocha' | 'pytest' | 'custom' {
    if (options.language === 'rust') return 'built_in';
    if (options.language === 'javascript' || options.language === 'typescript') {
      // Check if jest is in dependencies
      const hasJest = this.projectContext?.dependencies.some(d => d.name === 'jest');
      return hasJest ? 'jest' : 'mocha';
    }
    if (options.language === 'python') return 'pytest';
    return 'custom';
  }

  private async loadProjectContext(): Promise<void> {
    try {
      // Load workspace structure
      const workspaceStructure = await this.getWorkspaceStructure();
      
      // Load dependencies
      const dependencies = await this.getDependencies();
      
      // Analyze existing code patterns
      const existingPatterns = await this.analyzeExistingPatterns();
      
      // Load style guide if available
      const styleGuide = await this.loadStyleGuide();
      
      this.projectContext = {
        workspace_structure: workspaceStructure,
        dependencies,
        existing_patterns: existingPatterns,
        style_guide: styleGuide,
      };
    } catch (error) {
      console.warn('Failed to load full project context:', error);
      this.projectContext = {
        workspace_structure: {},
        dependencies: [],
        existing_patterns: [],
      };
    }
  }

  private async getWorkspaceStructure(): Promise<Record<string, string[]>> {
    try {
      // This would call a backend command to get workspace structure
      return await invoke('get_workspace_structure');
    } catch (error) {
      console.warn('Failed to get workspace structure:', error);
      return {};
    }
  }

  private async getDependencies(): Promise<Dependency[]> {
    try {
      // This would call a backend command to parse Cargo.toml
      return await invoke('get_project_dependencies');
    } catch (error) {
      console.warn('Failed to get dependencies:', error);
      return [];
    }
  }

  private async analyzeExistingPatterns(): Promise<CodePattern[]> {
    try {
      // This would analyze existing code to find common patterns
      return await invoke('analyze_code_patterns');
    } catch (error) {
      console.warn('Failed to analyze code patterns:', error);
      return [];
    }
  }

  private async loadStyleGuide(): Promise<StyleGuide | undefined> {
    try {
      // Look for common style guide files
      return await invoke('load_style_guide');
    } catch (error) {
      console.warn('No style guide found:', error);
      return undefined;
    }
  }

  private async loadCodeTemplates(): Promise<void> {
    try {
      // Load additional templates from project or user preferences
      const customTemplates = await invoke('get_code_templates');
      
      for (const template of customTemplates) {
        this.templates.set(template.name, template);
      }
    } catch (error) {
      console.warn('Failed to load custom templates:', error);
    }
  }

  private generateCacheKey(type: string, options: CodeGenerationOptions): string {
    const contextHash = this.hashString(options.context + (options.fileContent || ''));
    return `${type}_${options.language}_${contextHash}`;
  }

  private getCachedResult(cacheKey: string): GeneratedCode | null {
    const cached = this.cache.get(cacheKey);
    if (cached) {
      const age = Date.now() - cached.timestamp;
      const maxAge = 10 * 60 * 1000; // 10 minutes
      
      if (age < maxAge) {
        return cached.result;
      } else {
        this.cache.delete(cacheKey);
      }
    }
    return null;
  }

  private cacheResult(cacheKey: string, result: GeneratedCode): void {
    this.cache.set(cacheKey, {
      result,
      timestamp: Date.now(),
    });
    
    // Clean up old cache entries
    if (this.cache.size > 50) {
      const entries = Array.from(this.cache.entries());
      entries.sort((a, b) => a[1].timestamp - b[1].timestamp);
      const toDelete = entries.slice(0, entries.length - 50);
      toDelete.forEach(([key]) => this.cache.delete(key));
    }
  }

  private calculateInsertionRange(request: CodeGenerationRequest): Range {
    const options = request.context;
    const type = request.generation_type;
    
    if (!options.file_name || !options.current_code) {
      return { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } };
    }

    const lines = options.current_code.split('\n');
    const cursorLine = options.cursor_position?.[0] || 0;
    
    switch (type) {
      case 'test':
        // Insert at end of file for tests
        return {
          start: { line: lines.length, character: 0 },
          end: { line: lines.length, character: 0 },
        };
        
      case 'documentation':
        // Insert before the current function/struct
        const docLine = this.findDocumentationInsertionPoint(options.current_code, cursorLine);
        return {
          start: { line: docLine, character: 0 },
          end: { line: docLine, character: 0 },
        };
        
      case 'boilerplate':
      case 'stub':
        // Insert at cursor position
        return {
          start: { line: cursorLine, character: options.cursor_position?.[1] || 0 },
          end: { line: cursorLine, character: options.cursor_position?.[1] || 0 },
        };
        
      case 'example':
        // Insert at end of file
        return {
          start: { line: lines.length, character: 0 },
          end: { line: lines.length, character: 0 },
        };
        
      default:
        return {
          start: { line: cursorLine, character: 0 },
          end: { line: cursorLine, character: 0 },
        };
    }
  }

  private createEditsFromContent(request: CodeGenerationRequest, content: string): TextEdit[] {
    const range = this.calculateInsertionRange(request);
    const type = request.generation_type;
    
    switch (type) {
      case 'test':
        return [TextEdit.insert(range.start, `\n${content}\n`)];
        
      case 'documentation':
        return [TextEdit.insert(range.start, `${content}\n`)];
        
      case 'boilerplate':
      case 'stub':
        return [TextEdit.replace(range, content)];
        
      case 'example':
        return [TextEdit.insert(range.start, `\n// Example usage:\n${content}\n`)];
        
      default:
        return [TextEdit.insert(range.start, content)];
    }
  }

  private findDocumentationInsertionPoint(code: string, cursorLine: number): number {
    const lines = code.split('\n');
    
    // Look backwards from cursor to find function/struct definition
    for (let i = cursorLine; i >= 0; i--) {
      const line = lines[i].trim();
      if (line.startsWith('fn ') || line.startsWith('pub fn ') || 
          line.startsWith('struct ') || line.startsWith('pub struct ') ||
          line.startsWith('enum ') || line.startsWith('pub enum ') ||
          line.startsWith('trait ') || line.startsWith('pub trait ')) {
        return i;
      }
    }
    
    return cursorLine;
  }

  private async extractTemplateVariables(
    template: CodeTemplate, 
    options: CodeGenerationOptions
  ): Promise<Record<string, string>> {
    const variables: Record<string, string> = {};
    
    // Extract function name from context
    const functionMatch = options.context.match(/fn\s+(\w+)/);
    if (functionMatch) {
      variables.function_name = functionMatch[1];
    }
    
    // Extract struct name
    const structMatch = options.context.match(/struct\s+(\w+)/);
    if (structMatch) {
      variables.struct_name = structMatch[1];
    }
    
    // Generate descriptions based on context
    if (template.name.includes('documentation')) {
      variables.brief_description = this.generateBriefDescription(options.context);
      variables.detailed_description = this.generateDetailedDescription(options.context);
      variables.parameter_docs = this.generateParameterDocs(options.context);
      variables.return_description = this.generateReturnDescription(options.context);
      variables.example_code = this.generateExampleCode(options.context);
      variables.error_conditions = this.generateErrorConditions(options.context);
    }
    
    if (template.name.includes('test')) {
      variables.test_setup = this.generateTestSetup(options.context);
      variables.function_call = this.generateFunctionCall(options.context);
      variables.assertions = this.generateAssertions(options.context);
      variables.additional_tests = this.generateAdditionalTests(options.context);
    }
    
    return variables;
  }

  private fillTemplate(pattern: string, variables: Record<string, string>): string {
    let result = pattern;
    
    for (const [key, value] of Object.entries(variables)) {
      const placeholder = `{{${key}}}`;
      result = result.replace(new RegExp(placeholder, 'g'), value);
    }
    
    // Clean up any remaining placeholders
    result = result.replace(/\{\{[^}]+\}\}/g, '// TODO: Fill in this section');
    
    return result;
  }

  private generateBasicFallback(type: string, options: CodeGenerationOptions): GeneratedCode {
    let content = '';
    
    switch (type) {
      case 'test':
        content = `#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // TODO: Add test implementation
        assert!(true);
    }
}`;
        break;
        
      case 'documentation':
        content = `/// TODO: Add description
///
/// # Arguments
///
/// * \`param\` - TODO: Describe parameter
///
/// # Returns
///
/// TODO: Describe return value
///
/// # Examples
///
/// \`\`\`
/// // TODO: Add example
/// \`\`\``;
        break;
        
      case 'boilerplate':
        content = `// TODO: Add implementation
pub struct NewStruct {
    // TODO: Add fields
}

impl NewStruct {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize fields
        }
    }
}`;
        break;
        
      default:
        content = `// TODO: Generated ${type} code`;
    }
    
    return {
      content,
      range: this.calculateInsertionRange({ generation_type: type as any, context: { current_code: options.fileContent || '', file_name: options.filePath }, options: {} }),
      edits: this.createEditsFromContent({ generation_type: type as any, context: { current_code: options.fileContent || '', file_name: options.filePath }, options: {} }, content),
      confidence: 0.5,
      type: type as any,
    };
  }

  // Template variable generation helpers
  private generateBriefDescription(context: string): string {
    const functionMatch = context.match(/fn\s+(\w+)/);
    if (functionMatch) {
      return `${functionMatch[1]} function`;
    }
    return 'TODO: Add brief description';
  }

  private generateDetailedDescription(context: string): string {
    return 'TODO: Add detailed description of functionality, behavior, and usage';
  }

  private generateParameterDocs(context: string): string {
    const paramMatches = context.matchAll(/(\w+):\s*([^,)]+)/g);
    const docs = [];
    
    for (const match of paramMatches) {
      docs.push(`/// * \`${match[1]}\` - TODO: Describe ${match[1]} parameter`);
    }
    
    return docs.length > 0 ? docs.join('\n') : '/// No parameters';
  }

  private generateReturnDescription(context: string): string {
    const returnMatch = context.match(/->\s*([^{]+)/);
    if (returnMatch) {
      return `Returns ${returnMatch[1].trim()}`;
    }
    return 'Returns nothing (unit type)';
  }

  private generateExampleCode(context: string): string {
    const functionMatch = context.match(/fn\s+(\w+)/);
    if (functionMatch) {
      return `let result = ${functionMatch[1]}();\n// Use result...`;
    }
    return '// TODO: Add example usage';
  }

  private generateErrorConditions(context: string): string {
    if (context.includes('Result<')) {
      return 'This function returns an error if the operation fails.';
    }
    if (context.includes('panic!')) {
      return 'This function may panic if invalid input is provided.';
    }
    return 'This function does not return errors.';
  }

  private generateTestSetup(context: string): string {
    return '// TODO: Set up test data and conditions';
  }

  private generateFunctionCall(context: string): string {
    const functionMatch = context.match(/fn\s+(\w+)/);
    if (functionMatch) {
      return `${functionMatch[1]}(/* TODO: Add parameters */)`;
    }
    return 'function_under_test()';
  }

  private generateAssertions(context: string): string {
    if (context.includes('Result<')) {
      return 'assert!(result.is_ok());\n        // assert_eq!(result.unwrap(), expected_value);';
    }
    return 'assert_eq!(result, expected_value);';
  }

  private generateAdditionalTests(context: string): string {
    return `
    #[test]
    fn test_edge_cases() {
        // TODO: Test edge cases
    }

    #[test]
    fn test_error_conditions() {
        // TODO: Test error conditions
    }`;
  }

  private hashString(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(36);
  }

  // Public utility methods
  public async updateProjectContext(analysisResult?: CodeAnalysisResult): Promise<void> {
    if (this.projectContext && analysisResult) {
      this.projectContext.recent_analysis = analysisResult;
    }
  }

  public clearCache(): void {
    this.cache.clear();
    console.log('Code generation cache cleared');
  }

  public getCacheStats(): { size: number; oldestEntry: number; newestEntry: number } {
    const entries = Array.from(this.cache.values());
    const timestamps = entries.map(entry => entry.timestamp);
    
    return {
      size: this.cache.size,
      oldestEntry: timestamps.length > 0 ? Math.min(...timestamps) : 0,
      newestEntry: timestamps.length > 0 ? Math.max(...timestamps) : 0,
    };
  }

  public async cancelGeneration(generationId: string): Promise<boolean> {
    try {
      await invoke('cancel_generation', { generation_id: generationId });
      return true;
    } catch (error) {
      console.error('Failed to cancel generation:', error);
      return false;
    }
  }

  public getAvailableTemplates(): string[] {
    return Array.from(this.templates.keys());
  }

  public async addCustomTemplate(template: CodeTemplate): Promise<void> {
    this.templates.set(template.name, template);
    
    try {
      await invoke('save_custom_template', { template });
    } catch (error) {
      console.warn('Failed to save custom template:', error);
    }
  }
}

// Template interface
interface CodeTemplate {
  name: string;
  pattern: string;
  variables: string[];
  language: string;
  description?: string;
  category?: string;
}
