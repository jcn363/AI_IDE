import { invoke } from '@tauri-apps/api/core';
import type {
  AIAnalysisError,
  CompilerDiagnostic,
  CompilerIntegrationResult,
  CompilerSpan,
  DocumentationLink,
  ErrorCodeExplanation,
  FixSuggestion,
  LearnedPattern,
} from '../types';

interface DiagnosticCache {
  [key: string]: {
    data: ErrorCodeExplanation;
    timestamp: number;
    ttl: number;
  };
}

interface DocumentationCache {
  [key: string]: {
    data: DocumentationLink[];
    timestamp: number;
    ttl: number;
  };
}

interface CompilerCheckResult {
  success: boolean;
  diagnostics: CompilerDiagnostic[];
  stderr: string;
  stdout: string;
}

interface RustcExplainResult {
  success: boolean;
  explanation: string;
  errorCode: string;
}

export class CompilerIntegrationService {
  private diagnosticCache: DiagnosticCache = {};
  private documentationCache: DocumentationCache = {};
  private readonly CACHE_TTL = 3600000; // 1 hour in milliseconds
  private readonly RUST_DOC_BASE_URL = 'https://doc.rust-lang.org';
  private readonly RUST_REFERENCE_URL = 'https://doc.rust-lang.org/reference';
  private readonly RUST_BOOK_URL = 'https://doc.rust-lang.org/book';
  private readonly ERROR_INDEX_URL = 'https://doc.rust-lang.org/error-index.html';

  /**
   * Run cargo check with JSON output and parse diagnostics
   */
  async runCargoCheck(workspacePath: string, targetFile?: string): Promise<CompilerIntegrationResult> {
    try {
      const result = await invoke<CompilerCheckResult>('run_cargo_check', {
        workspacePath,
        targetFile,
        messageFormat: 'json'
      });

      if (!result.success) {
        throw new Error(`Cargo check failed: ${result.stderr}`);
      }

      const diagnostics = this.parseDiagnostics(result.diagnostics);
      const explanations = await this.getExplanationsForDiagnostics(diagnostics);
      const suggestedFixes = this.generateFixSuggestions(diagnostics);
      const learnedPatterns = await this.getLearnedPatternsForDiagnostics(diagnostics);

      return {
        diagnostics,
        explanations,
        suggestedFixes,
        learnedPatterns
      };
    } catch (error) {
      console.error('Error running cargo check:', error);
      throw new AIAnalysisError({
        type: 'internal',
        message: 'Failed to run cargo check',
        details: error instanceof Error ? error.message : String(error),
        retryable: true,
        timestamp: Date.now(),
        context: {
          operation: 'cargo_check',
          file: targetFile
        }
      });
    }
  }

  /**
   * Parse raw cargo check JSON output into structured diagnostics
   */
  private parseDiagnostics(rawDiagnostics: any[]): CompilerDiagnostic[] {
    return rawDiagnostics
      .filter(item => item.reason === 'compiler-message')
      .map(item => this.parseCompilerMessage(item.message))
      .filter(Boolean);
  }

  /**
   * Parse a single compiler message into CompilerDiagnostic
   */
  private parseCompilerMessage(message: any): CompilerDiagnostic | null {
    if (!message) return null;

    try {
      return {
        level: message.level as 'error' | 'warning' | 'note' | 'help',
        message: message.message || '',
        code: message.code ? {
          code: message.code.code || '',
          explanation: message.code.explanation
        } : undefined,
        spans: this.parseSpans(message.spans || []),
        children: (message.children || []).map((child: any) => this.parseCompilerMessage(child)).filter(Boolean),
        rendered: message.rendered
      };
    } catch (error) {
      console.error('Error parsing compiler message:', error);
      return null;
    }
  }

  /**
   * Parse compiler spans
   */
  private parseSpans(spans: any[]): CompilerSpan[] {
    return spans.map(span => ({
      fileName: span.file_name || '',
      byteStart: span.byte_start || 0,
      byteEnd: span.byte_end || 0,
      lineStart: span.line_start || 1,
      lineEnd: span.line_end || 1,
      columnStart: span.column_start || 1,
      columnEnd: span.column_end || 1,
      isMainSpan: span.is_primary || false,
      text: span.text || [],
      label: span.label,
      suggestedReplacement: span.suggested_replacement,
      suggestionApplicability: span.suggestion_applicability
    }));
  }

  /**
   * Get detailed explanations for error codes in diagnostics
   */
  async getExplanationsForDiagnostics(diagnostics: CompilerDiagnostic[]): Promise<Record<string, ErrorCodeExplanation>> {
    const explanations: Record<string, ErrorCodeExplanation> = {};
    const errorCodes = new Set<string>();

    // Collect all unique error codes
    this.collectErrorCodes(diagnostics, errorCodes);

    // Get explanations for each error code
    for (const errorCode of errorCodes) {
      try {
        const explanation = await this.getErrorCodeExplanation(errorCode);
        if (explanation) {
          explanations[errorCode] = explanation;
        }
      } catch (error) {
        console.warn(`Failed to get explanation for error code ${errorCode}:`, error);
      }
    }

    return explanations;
  }

  /**
   * Recursively collect error codes from diagnostics
   */
  private collectErrorCodes(diagnostics: CompilerDiagnostic[], errorCodes: Set<string>): void {
    for (const diagnostic of diagnostics) {
      if (diagnostic.code?.code) {
        errorCodes.add(diagnostic.code.code);
      }
      if (diagnostic.children.length > 0) {
        this.collectErrorCodes(diagnostic.children, errorCodes);
      }
    }
  }

  /**
   * Get detailed explanation for a specific error code
   */
  async getErrorCodeExplanation(errorCode: string): Promise<ErrorCodeExplanation | null> {
    // Check cache first
    const cached = this.diagnosticCache[errorCode];
    if (cached && Date.now() - cached.timestamp < cached.ttl) {
      return cached.data;
    }

    try {
      const result = await invoke<RustcExplainResult>('rustc_explain_error', {
        errorCode
      });

      if (!result.success) {
        console.warn(`Failed to get explanation for error code ${errorCode}`);
        return null;
      }

      const explanation = this.parseErrorExplanation(errorCode, result.explanation);
      
      // Cache the result
      this.diagnosticCache[errorCode] = {
        data: explanation,
        timestamp: Date.now(),
        ttl: this.CACHE_TTL
      };

      return explanation;
    } catch (error) {
      console.error(`Error getting explanation for ${errorCode}:`, error);
      return null;
    }
  }

  /**
   * Parse rustc --explain output into structured explanation
   */
  private parseErrorExplanation(errorCode: string, explanationText: string): ErrorCodeExplanation {
    // Extract title (first line)
    const lines = explanationText.split('\n');
    const title = lines[0]?.replace(/^error\[E\d+\]:\s*/, '') || `Error ${errorCode}`;
    
    // Extract main explanation (everything before examples)
    const exampleStartIndex = lines.findIndex(line => 
      line.toLowerCase().includes('example') || 
      line.includes('```') ||
      line.toLowerCase().includes('this code')
    );
    
    const explanationEndIndex = exampleStartIndex > 0 ? exampleStartIndex : lines.length;
    const explanation = lines.slice(1, explanationEndIndex).join('\n').trim();

    // Extract examples
    const examples = this.extractExamples(lines.slice(exampleStartIndex));

    // Generate documentation links
    const documentationLinks = this.generateDocumentationLinks(errorCode, title);

    return {
      errorCode,
      title,
      explanation,
      examples,
      documentationLinks
    };
  }

  /**
   * Extract code examples from explanation text
   */
  private extractExamples(lines: string[]): Array<{
    description: string;
    code: string;
    explanation: string;
  }> {
    const examples: Array<{
      description: string;
      code: string;
      explanation: string;
    }> = [];

    let currentExample: {
      description: string;
      code: string;
      explanation: string;
    } | null = null;
    let inCodeBlock = false;
    let codeLines: string[] = [];

    let capturingExplanation = false;
    let explanationLines: string[] = [];

    const finalizeExample = () => {
      if (currentExample) {
        currentExample.explanation = explanationLines.join(' ').trim();
        examples.push(currentExample);
        currentExample = null;
        explanationLines = [];
        capturingExplanation = false;
      }
    };

    for (const line of lines) {
      if (inCodeBlock) {
        if (line.includes('```')) {
          // End of code block
          inCodeBlock = false;
          if (currentExample) {
            currentExample.code = codeLines.join('\n');
          }
          codeLines = [];
          capturingExplanation = true;
        } else {
          codeLines.push(line);
        }
        continue;
      }

      if (capturingExplanation) {
        if (line.trim() === '') {
          finalizeExample();
          continue;
        }
        if (line.includes('```')) {
          finalizeExample();
          // Start of a new code block after explanation
          inCodeBlock = true;
          if (!currentExample) {
            currentExample = {
              description: 'Example',
              code: '',
              explanation: ''
            };
          }
          continue;
        }
        explanationLines.push(line.trim());
        continue;
      }

      if (line.includes('```')) {
        // Start of code block
        inCodeBlock = true;
        if (!currentExample) {
          currentExample = {
            description: 'Example',
            code: '',
            explanation: ''
          };
        }
      } else if (line.trim() && !currentExample) {
        // Description line before code block
        currentExample = {
          description: line.trim(),
          code: '',
          explanation: ''
        };
      } else if (line.trim() && currentExample && !capturingExplanation) {
        // New description implies previous example had no explanation
        finalizeExample();
        currentExample = {
          description: line.trim(),
          code: '',
          explanation: ''
        };
      }
    }

    if (inCodeBlock && currentExample) {
      currentExample.code = codeLines.join('\n');
    }

    if (capturingExplanation) {
      finalizeExample();
    } else if (currentExample) {
      examples.push(currentExample);
    }

    return examples;
  }
  /**
   * Generate relevant documentation links for an error code
   */
  private generateDocumentationLinks(errorCode: string, title: string): DocumentationLink[] {
    const links: DocumentationLink[] = [];

    // Error index link
    links.push({
      title: `Error ${errorCode} in Error Index`,
      url: `${this.ERROR_INDEX_URL}#${errorCode.toLowerCase()}`,
      description: 'Official Rust error index entry'
    });

    // Generate contextual links based on error type
    const errorType = this.categorizeError(errorCode, title);
    links.push(...this.getContextualLinks(errorType));

    return links;
  }

  /**
   * Categorize error based on code and title
   */
  private categorizeError(errorCode: string, title: string): string {
    const titleLower = title.toLowerCase();
    
    if (titleLower.includes('borrow') || titleLower.includes('lifetime')) {
      return 'borrowing';
    } else if (titleLower.includes('trait') || titleLower.includes('impl')) {
      return 'traits';
    } else if (titleLower.includes('type') || titleLower.includes('generic')) {
      return 'types';
    } else if (titleLower.includes('macro')) {
      return 'macros';
    } else if (titleLower.includes('module') || titleLower.includes('import')) {
      return 'modules';
    } else if (titleLower.includes('pattern') || titleLower.includes('match')) {
      return 'patterns';
    } else {
      return 'general';
    }
  }

  /**
   * Get contextual documentation links based on error category
   */
  private getContextualLinks(category: string): DocumentationLink[] {
    const linkMap: Record<string, DocumentationLink[]> = {
      borrowing: [
        {
          title: 'Understanding Ownership',
          url: `${this.RUST_BOOK_URL}/ch04-00-understanding-ownership.html`,
          description: 'Rust Book chapter on ownership and borrowing'
        },
        {
          title: 'References and Borrowing',
          url: `${this.RUST_BOOK_URL}/ch04-02-references-and-borrowing.html`,
          description: 'Detailed guide on references and borrowing'
        }
      ],
      traits: [
        {
          title: 'Traits: Defining Shared Behavior',
          url: `${this.RUST_BOOK_URL}/ch10-02-traits.html`,
          description: 'Rust Book chapter on traits'
        },
        {
          title: 'Trait Objects',
          url: `${this.RUST_BOOK_URL}/ch17-02-trait-objects.html`,
          description: 'Advanced trait usage with trait objects'
        }
      ],
      types: [
        {
          title: 'Generic Types, Traits, and Lifetimes',
          url: `${this.RUST_BOOK_URL}/ch10-00-generics.html`,
          description: 'Comprehensive guide to Rust\'s type system'
        }
      ],
      macros: [
        {
          title: 'Macros',
          url: `${this.RUST_BOOK_URL}/ch19-06-macros.html`,
          description: 'Rust Book chapter on macros'
        }
      ],
      modules: [
        {
          title: 'Managing Growing Projects with Packages, Crates, and Modules',
          url: `${this.RUST_BOOK_URL}/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html`,
          description: 'Module system and project organization'
        }
      ],
      patterns: [
        {
          title: 'Patterns and Matching',
          url: `${this.RUST_BOOK_URL}/ch18-00-patterns.html`,
          description: 'Pattern matching in Rust'
        }
      ],
      general: [
        {
          title: 'The Rust Programming Language',
          url: this.RUST_BOOK_URL,
          description: 'The official Rust Book'
        },
        {
          title: 'Rust Reference',
          url: this.RUST_REFERENCE_URL,
          description: 'Comprehensive language reference'
        }
      ]
    };

    return linkMap[category] || linkMap.general;
  }

  /**
   * Generate fix suggestions based on diagnostics
   */
  private generateFixSuggestions(diagnostics: CompilerDiagnostic[]): FixSuggestion[] {
    const suggestions: FixSuggestion[] = [];

    for (const diagnostic of diagnostics) {
      const fixes = this.generateFixesForDiagnostic(diagnostic);
      suggestions.push(...fixes);
    }

    return suggestions;
  }

  /**
   * Generate fix suggestions for a single diagnostic
   */
  private generateFixesForDiagnostic(diagnostic: CompilerDiagnostic): FixSuggestion[] {
    const fixes: FixSuggestion[] = [];

    // Process spans with suggested replacements
    for (const span of diagnostic.spans) {
      if (span.suggestedReplacement && span.suggestionApplicability) {
        fixes.push({
          id: `fix-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          title: this.generateFixTitle(diagnostic, span),
          description: diagnostic.message,
          fixType: this.determineFixType(span.suggestionApplicability),
          changes: [{
            filePath: span.fileName,
            range: [span.lineStart - 1, span.columnStart - 1, span.lineEnd - 1, span.columnEnd - 1],
            oldText: span.text.map(t => t.text).join(''),
            newText: span.suggestedReplacement,
            changeType: 'replace'
          }],
          confidence: this.calculateFixConfidence(span.suggestionApplicability),
          estimatedEffort: this.estimateFixEffort(span.suggestionApplicability),
          benefits: [this.generateBenefitDescription(diagnostic)],
          risks: this.generateRiskAssessment(span.suggestionApplicability)
        });
      }
    }

    // Process child diagnostics
    for (const child of diagnostic.children) {
      fixes.push(...this.generateFixesForDiagnostic(child));
    }

    return fixes;
  }

  /**
   * Generate a descriptive title for a fix
   */
  private generateFixTitle(diagnostic: CompilerDiagnostic, span: CompilerSpan): string {
    if (span.label) {
      return `Fix: ${span.label}`;
    }
    
    const level = diagnostic.level;
    const message = diagnostic.message.split('.')[0]; // First sentence
    
    return `${level === 'error' ? 'Fix error' : 'Apply suggestion'}: ${message}`;
  }

  /**
   * Determine fix type based on suggestion applicability
   */
  private determineFixType(applicability: string): 'quick-fix' | 'refactoring' | 'code-generation' | 'documentation' {
    switch (applicability) {
      case 'machine-applicable':
        return 'quick-fix';
      case 'has-placeholders':
      case 'maybe-incorrect':
        return 'refactoring';
      default:
        return 'quick-fix';
    }
  }

  /**
   * Calculate confidence score based on suggestion applicability
   */
  private calculateFixConfidence(applicability: string): number {
    switch (applicability) {
      case 'machine-applicable':
        return 0.95;
      case 'has-placeholders':
        return 0.75;
      case 'maybe-incorrect':
        return 0.60;
      case 'unspecified':
      default:
        return 0.50;
    }
  }

  /**
   * Estimate effort required for fix
   */
  private estimateFixEffort(applicability: string): 'trivial' | 'low' | 'medium' | 'high' | 'very-high' {
    switch (applicability) {
      case 'machine-applicable':
        return 'trivial';
      case 'has-placeholders':
        return 'low';
      case 'maybe-incorrect':
        return 'medium';
      default:
        return 'low';
    }
  }

  /**
   * Generate benefit description for a fix
   */
  private generateBenefitDescription(diagnostic: CompilerDiagnostic): string {
    switch (diagnostic.level) {
      case 'error':
        return 'Resolves compilation error';
      case 'warning':
        return 'Improves code quality and follows best practices';
      case 'note':
        return 'Provides additional context and clarity';
      case 'help':
        return 'Implements suggested improvement';
      default:
        return 'Addresses compiler feedback';
    }
  }

  /**
   * Generate risk assessment for a fix
   */
  private generateRiskAssessment(applicability: string): string[] {
    switch (applicability) {
      case 'machine-applicable':
        return ['Low risk - automatically generated fix'];
      case 'has-placeholders':
        return ['Medium risk - requires manual review of placeholders'];
      case 'maybe-incorrect':
        return ['Higher risk - suggestion may not be appropriate for all cases'];
      default:
        return ['Unknown risk - manual review recommended'];
    }
  }

  /**
   * Get learned patterns for diagnostics from the learning system
   */
  async getLearnedPatternsForDiagnostics(diagnostics: CompilerDiagnostic[]): Promise<LearnedPattern[]> {
    try {
      const patterns = await invoke<LearnedPattern[]>('get_learned_patterns_for_diagnostics', {
        diagnostics: diagnostics.map(d => ({
          errorType: d.code?.code || 'unknown',
          message: d.message,
          level: d.level
        }))
      });

      return patterns || [];
    } catch (error) {
      console.warn('Failed to get learned patterns:', error);
      return [];
    }
  }

  /**
   * Enhanced error message with project context
   */
  async enhanceErrorWithContext(
    diagnostic: CompilerDiagnostic,
    projectPath: string
  ): Promise<CompilerDiagnostic> {
    try {
      const enhancedDiagnostic = { ...diagnostic };
      
      // Add project-specific context
      const context = await this.getProjectContext(projectPath, diagnostic);
      if (context) {
        enhancedDiagnostic.message = `${diagnostic.message}\n\nProject Context: ${context}`;
      }

      return enhancedDiagnostic;
    } catch (error) {
      console.warn('Failed to enhance error with context:', error);
      return diagnostic;
    }
  }

  /**
   * Get project-specific context for an error
   */
  private async getProjectContext(projectPath: string, diagnostic: CompilerDiagnostic): Promise<string | null> {
    try {
      // This could be enhanced to analyze project structure, dependencies, etc.
      const context = await invoke<string>('get_project_context_for_error', {
        projectPath,
        errorCode: diagnostic.code?.code,
        filePath: diagnostic.spans[0]?.fileName
      });

      return context;
    } catch (error) {
      return null;
    }
  }

  /**
   * Real-time diagnostic updates using file watching
   */
  async startRealTimeDiagnostics(
    workspacePath: string,
    onUpdate: (result: CompilerIntegrationResult) => void,
    onError: (error: AIAnalysisError) => void
  ): Promise<() => void> {
    try {
      const unlisten = await invoke<() => void>('start_real_time_diagnostics', {
        workspacePath,
        callback: async (diagnostics: CompilerDiagnostic[]) => {
          try {
            const explanations = await this.getExplanationsForDiagnostics(diagnostics);
            const suggestedFixes = this.generateFixSuggestions(diagnostics);
            const learnedPatterns = await this.getLearnedPatternsForDiagnostics(diagnostics);

            onUpdate({
              diagnostics,
              explanations,
              suggestedFixes,
              learnedPatterns
            });
          } catch (error) {
            onError({
              type: 'internal',
              message: 'Failed to process real-time diagnostics',
              details: error instanceof Error ? error.message : String(error),
              retryable: true,
              timestamp: Date.now(),
              context: {
                operation: 'real_time_diagnostics'
              }
            });
          }
        }
      });

      return unlisten;
    } catch (error) {
      throw new AIAnalysisError({
        type: 'internal',
        message: 'Failed to start real-time diagnostics',
        details: error instanceof Error ? error.message : String(error),
        retryable: true,
        timestamp: Date.now(),
        context: {
          operation: 'start_real_time_diagnostics'
        }
      });
    }
  }

  /**
   * Clear diagnostic and documentation caches
   */
  clearCache(): void {
    this.diagnosticCache = {};
    this.documentationCache = {};
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    diagnosticCacheSize: number;
    documentationCacheSize: number;
    oldestEntry: number | null;
    newestEntry: number | null;
  } {
    const diagnosticEntries = Object.values(this.diagnosticCache);
    const docEntries = Object.values(this.documentationCache);
    const allEntries = [...diagnosticEntries, ...docEntries];

    return {
      diagnosticCacheSize: diagnosticEntries.length,
      documentationCacheSize: docEntries.length,
      oldestEntry: allEntries.length > 0 ? Math.min(...allEntries.map(e => e.timestamp)) : null,
      newestEntry: allEntries.length > 0 ? Math.max(...allEntries.map(e => e.timestamp)) : null
    };
  }
}

// Export singleton instance
export const compilerIntegration = new CompilerIntegrationService();