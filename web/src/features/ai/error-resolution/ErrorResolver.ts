import { invoke } from '@tauri-apps/api/core';
import { CodeAction, CodeActionKind, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit } from 'vscode-languageserver';
import { Diagnostic, DiagnosticSeverity } from 'vscode-languageserver-types';
import type {
  AIContext,
  AnalysisPreferences,
  CodeChange,
  CompilerDiagnostic,
  CompilerIntegrationResult,
  DocumentationLink,
  EnhancedErrorResolutionResult,
  ErrorCodeExplanation,
  ErrorPattern,
  ErrorResolutionResult,
  FixSuggestion,
  LearnedPattern,
  LearningSystemRequest
} from '../types';

// Enhanced error resolution result with learning capabilities
export interface EnhancedErrorResolutionResult {
  quickFixes: FixSuggestion[];
  explanations: ErrorCodeExplanation[];
  relatedDocumentation: DocumentationLink[];
  learnedPatterns: LearnedPattern[];
  compilerDiagnostics: CompilerDiagnostic[];
  confidence: number;
  estimatedSuccessRate: number;
}

// Error resolution request for backend
interface ErrorResolutionRequest {
  filePath: string;
  content: string;
  errors: string[];
  cursorPosition?: [number, number];
  useLearnedPatterns: boolean;
}

// Compiler diagnostics request
interface CompilerDiagnosticsRequest {
  workspacePath: string;
  includeExplanations: boolean;
  includeSuggestedFixes: boolean;
}


export class ErrorResolver {
  private static instance: ErrorResolver;
  private errorPatterns: Map<string, ErrorPattern> = new Map();
  private learnedPatterns: Map<string, LearnedPattern[]> = new Map();
  private explanationCache: Map<string, ErrorCodeExplanation> = new Map();
  private confidenceThreshold: number = 0.7;
  private enableLearning: boolean = true;

  private constructor() {
    this.initializePatterns();
    this.loadLearnedPatterns();
  }

  public static getInstance(): ErrorResolver {
    if (!ErrorResolver.instance) {
      ErrorResolver.instance = new ErrorResolver();
    }
    return ErrorResolver.instance;
  }

  /**
   * Configure the error resolver with preferences
   */
  public configure(preferences: {
    confidenceThreshold?: number;
    enableLearning?: boolean;
  }): void {
    if (preferences.confidenceThreshold !== undefined) {
      this.confidenceThreshold = preferences.confidenceThreshold;
    }
    if (preferences.enableLearning !== undefined) {
      this.enableLearning = preferences.enableLearning;
    }
  }

  /**
   * Enhanced error resolution with AI assistance and learning capabilities
   */
  public async resolveErrorWithAI(
    error: Diagnostic,
    documentText: string,
    filePath: string,
    projectContext?: Record<string, string>,
    cursorPosition?: [number, number]
  ): Promise<EnhancedErrorResolutionResult> {
    try {
      // Prepare error context for backend analysis
      const errorMessages = [error.message];

      // Add error code if available
      if (error.code) {
        errorMessages.push(`Error code: ${error.code}`);
      }

      const request: ErrorResolutionRequest = {
        filePath,
        content: documentText,
        errors: errorMessages,
        cursorPosition,
        useLearnedPatterns: this.enableLearning
      };

      // Get AI-powered error resolution from backend
      const backendResult = await invoke<FixSuggestion[]>('resolve_errors_with_ai', { request });

      // Get compiler diagnostics and explanations
      const compilerResult = await this.getCompilerDiagnostics(filePath);

      // Get learned patterns for this error type
      const learnedPatterns = this.enableLearning
        ? await this.getLearnedPatternsForError(error.message)
        : [];

      // Get error code explanation if available
      const explanations = await this.getErrorExplanations(error);

      // Calculate overall confidence and success rate
      const confidence = this.calculateOverallConfidence(backendResult, learnedPatterns);
      const estimatedSuccessRate = this.calculateSuccessRate(learnedPatterns);

      // Generate documentation links
      const relatedDocumentation = await this.generateDocumentationLinks(error, explanations);

      return {
        quickFixes: backendResult,
        explanations,
        relatedDocumentation,
        learnedPatterns,
        compilerDiagnostics: compilerResult?.diagnostics || [],
        confidence,
        estimatedSuccessRate
      };
    } catch (error) {
      console.error('Enhanced error resolution failed:', error);
      // Fallback to basic resolution
      const basicResult = this.resolveError(error, documentText, filePath);
      return {
        quickFixes: this.convertToFixSuggestions(basicResult.quickFixes),
        explanations: this.convertToErrorExplanations(basicResult.explanations),
        relatedDocumentation: basicResult.relatedDocumentation,
        learnedPatterns: [],
        compilerDiagnostics: [],
        confidence: 0.5,
        estimatedSuccessRate: 0.0
      };
    }
  }

  /**
   * Legacy error resolution method for backward compatibility
   */
  public resolveError(
    error: Diagnostic,
    documentText: string,
    filePath: string,
  ): ErrorResolutionResult {
    const result: ErrorResolutionResult = {
      quickFixes: [],
      explanations: [],
      relatedDocumentation: [],
    };

    // Try to match error against known patterns
    this.matchErrorPatterns(error, documentText, result);

    // Enhance with learned patterns if available
    this.enhanceWithLearnedPatterns(error, documentText, result);

    // Add generic error explanation if no specific pattern matched
    if (result.explanations.length === 0) {
      result.explanations.push(this.generateGenericExplanation(error));
    }

    // Add documentation links
    this.addDocumentationLinks(error, result);

    return result;
  }

  /**
   * Learn from applied fixes to improve future suggestions
   */
  public async learnFromFix(
    error: Diagnostic,
    fix: CodeAction | FixSuggestion,
    wasSuccessful: boolean,
    userFeedback?: 'positive' | 'negative' | 'neutral',
    context?: string
  ): Promise<void> {
    if (!this.enableLearning) {
      return;
    }

    try {
      // Create error pattern from diagnostic
      const errorPattern: ErrorPattern = {
        id: `pattern_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        errorType: this.categorizeError(error),
        pattern: error.message,
        context: context || '',
        frequency: 1,
        lastSeen: new Date().toISOString(),
        confidence: wasSuccessful ? 0.8 : 0.3
      };

      // Convert CodeAction to FixSuggestion if needed
      const fixSuggestion: FixSuggestion = this.isFixSuggestion(fix)
        ? fix
        : this.convertCodeActionToFixSuggestion(fix);

      // Prepare learning request for backend
      const learningRequest: LearningSystemRequest = {
        errorPattern,
        appliedFix: fixSuggestion,
        success: wasSuccessful,
        userFeedback,
        context: context || `Error at line ${error.range.start.line + 1}`
      };

      // Record the fix in the backend learning system
      await invoke('record_successful_fix', { request: learningRequest });

      // Update local learned patterns cache
      await this.updateLocalLearnedPatterns(errorPattern, fixSuggestion, wasSuccessful);

      console.log(`Recorded ${wasSuccessful ? 'successful' : 'failed'} fix for error: ${error.message}`);
    } catch (error) {
      console.error('Failed to record fix for learning:', error);
    }
  }

  /**
   * Apply a fix suggestion and optionally record it for learning
   */
  public async applyFixSuggestion(
    fixSuggestion: FixSuggestion,
    recordForLearning: boolean = true
  ): Promise<{ success: boolean; errors: string[] }> {
    try {
      const request = {
        suggestionId: fixSuggestion.id,
        changes: fixSuggestion.changes,
        createBackup: true,
        recordForLearning
      };

      const result = await invoke<string>('apply_ai_suggestion', { request });

      return {
        success: true,
        errors: []
      };
    } catch (error) {
      return {
        success: false,
        errors: [error instanceof Error ? error.message : String(error)]
      };
    }
  }

  /**
   * Get compiler diagnostics for enhanced error context
   */
  private async getCompilerDiagnostics(filePath: string): Promise<CompilerIntegrationResult | null> {
    try {
      // Extract workspace path from file path
      const workspacePath = this.extractWorkspacePath(filePath);

      const request: CompilerDiagnosticsRequest = {
        workspacePath,
        includeExplanations: true,
        includeSuggestedFixes: true
      };

      return await invoke<CompilerIntegrationResult>('get_compiler_diagnostics', { request });
    } catch (error) {
      console.warn('Failed to get compiler diagnostics:', error);
      return null;
    }
  }

  /**
   * Get learned patterns for similar errors
   */
  private async getLearnedPatternsForError(errorMessage: string): Promise<LearnedPattern[]> {
    try {
      return await invoke<LearnedPattern[]>('get_learned_patterns', {
        errorContext: errorMessage
      });
    } catch (error) {
      console.warn('Failed to get learned patterns:', error);
      return [];
    }
  }

  /**
   * Get detailed error explanations
   */
  private async getErrorExplanations(error: Diagnostic): Promise<ErrorCodeExplanation[]> {
    const explanations: ErrorCodeExplanation[] = [];

    // Try to get explanation for error code if available
    if (error.code) {
      const errorCode = String(error.code);

      // Check cache first
      if (this.explanationCache.has(errorCode)) {
        explanations.push(this.explanationCache.get(errorCode)!);
      } else {
        try {
          const explanation = await invoke<ErrorCodeExplanation>('explain_error_code', {
            errorCode
          });
          this.explanationCache.set(errorCode, explanation);
          explanations.push(explanation);
        } catch (error) {
          console.warn(`Failed to get explanation for error code ${errorCode}:`, error);
        }
      }
    }

    return explanations;
  }

  /**
   * Generate comprehensive documentation links
   */
  private async generateDocumentationLinks(
    error: Diagnostic,
    explanations: ErrorCodeExplanation[]
  ): Promise<DocumentationLink[]> {
    const links: DocumentationLink[] = [];

    // Add links from error explanations
    explanations.forEach(explanation => {
      links.push(...explanation.documentationLinks);
    });

    // Add general Rust documentation links based on error type
    const errorType = this.categorizeError(error);
    const generalLinks = this.getGeneralDocumentationLinks(errorType);
    links.push(...generalLinks);

    // Add context-specific links based on error message
    const contextLinks = this.getContextSpecificLinks(error.message);
    links.push(...contextLinks);

    // Remove duplicates
    return this.deduplicateLinks(links);
  }

  /**
   * Enhanced pattern matching with learned patterns
   */
  private matchErrorPatterns(
    error: Diagnostic,
    documentText: string,
    result: ErrorResolutionResult,
  ): void {
    // Try to match against each known pattern
    for (const [patternId, pattern] of this.errorPatterns) {
      if (pattern.matcher(error, documentText)) {
        // Add pattern-specific quick fixes
        if (pattern.quickFixes) {
          result.quickFixes.push(...pattern.quickFixes(error, documentText));
        }

        // Add pattern-specific explanations
        if (pattern.explanation) {
          result.explanations.push(pattern.explanation(error, documentText));
        }
      }
    }
  }

  /**
   * Enhance results with learned patterns
   */
  private enhanceWithLearnedPatterns(
    error: Diagnostic,
    documentText: string,
    result: ErrorResolutionResult
  ): void {
    const errorType = this.categorizeError(error);
    const patterns = this.learnedPatterns.get(errorType) || [];

    for (const pattern of patterns) {
      // Only use patterns above confidence threshold
      if (pattern.confidence >= this.confidenceThreshold) {
        // Convert learned fix to CodeAction
        const codeAction = this.convertFixSuggestionToCodeAction(
          pattern.successfulFix,
          error.source || 'unknown'
        );

        if (codeAction) {
          result.quickFixes.push(codeAction);
        }

        // Add explanation with confidence info
        const explanation = `Learned fix (${Math.round(pattern.confidence * 100)}% confidence): ${pattern.successfulFix.description}`;
        result.explanations.push(explanation);
      }
    }
  }

  private getSeverityText(severity: DiagnosticSeverity | undefined): string {
    if (severity === undefined) return 'unknown';

    switch (severity) {
      case DiagnosticSeverity.Error: return 'error';
      case DiagnosticSeverity.Warning: return 'warning';
      case DiagnosticSeverity.Information: return 'information';
      case DiagnosticSeverity.Hint: return 'hint';
      default: return 'unknown';
    }
  }

  private generateGenericExplanation(error: Diagnostic): string {
    const severityText = this.getSeverityText(error.severity);
    return `Error: ${error.message}\n\nThis appears to be a ${severityText} issue. ` +
      `The error occurred at line ${error.range.start.line + 1}.`;
  }

  /**
   * Calculate overall confidence score
   */
  private calculateOverallConfidence(
    fixes: FixSuggestion[],
    learnedPatterns: LearnedPattern[]
  ): number {
    if (fixes.length === 0 && learnedPatterns.length === 0) {
      return 0.0;
    }

    let totalConfidence = 0;
    let count = 0;

    // Include confidence from AI-generated fixes
    fixes.forEach(fix => {
      totalConfidence += fix.confidence;
      count++;
    });

    // Include confidence from learned patterns
    learnedPatterns.forEach(pattern => {
      totalConfidence += pattern.confidence;
      count++;
    });

    return count > 0 ? totalConfidence / count : 0.0;
  }

  /**
   * Calculate estimated success rate based on learned patterns
   */
  private calculateSuccessRate(learnedPatterns: LearnedPattern[]): number {
    if (learnedPatterns.length === 0) {
      return 0.0;
    }

    let totalSuccesses = 0;
    let totalAttempts = 0;

    learnedPatterns.forEach(pattern => {
      totalSuccesses += pattern.successCount;
      totalAttempts += pattern.successCount + pattern.failureCount;
    });

    return totalAttempts > 0 ? totalSuccesses / totalAttempts : 0.0;
  }

  /**
   * Load learned patterns from backend
   */
  private async loadLearnedPatterns(): Promise<void> {
    try {
      // This would typically load from a local cache or backend
      // For now, we'll initialize with empty patterns
      this.learnedPatterns.clear();
    } catch (error) {
      console.warn('Failed to load learned patterns:', error);
    }
  }

  /**
   * Update local learned patterns cache
   */
  private async updateLocalLearnedPatterns(
    errorPattern: ErrorPattern,
    fixSuggestion: FixSuggestion,
    wasSuccessful: boolean
  ): Promise<void> {
    const errorType = errorPattern.errorType;
    const patterns = this.learnedPatterns.get(errorType) || [];

    // Find existing pattern or create new one
    let existingPattern = patterns.find(p =>
      p.errorPattern.pattern === errorPattern.pattern
    );

    if (existingPattern) {
      // Update existing pattern
      if (wasSuccessful) {
        existingPattern.successCount++;
      } else {
        existingPattern.failureCount++;
      }
      existingPattern.lastUsed = new Date().toISOString();
      existingPattern.confidence = existingPattern.successCount /
        (existingPattern.successCount + existingPattern.failureCount);
    } else {
      // Create new learned pattern
      const newPattern: LearnedPattern = {
        id: `learned_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        errorPattern,
        successfulFix: fixSuggestion,
        successCount: wasSuccessful ? 1 : 0,
        failureCount: wasSuccessful ? 0 : 1,
        confidence: wasSuccessful ? 1.0 : 0.0,
        lastUsed: new Date().toISOString(),
        userFeedback: null,
        context: errorPattern.context
      };
      patterns.push(newPattern);
    }

    this.learnedPatterns.set(errorType, patterns);
  }

  /**
   * Enhanced documentation link generation
   */
  private addDocumentationLinks(
    error: Diagnostic,
    result: ErrorResolutionResult,
  ): void {
    const errorType = this.categorizeError(error);
    const links = this.getGeneralDocumentationLinks(errorType);
    result.relatedDocumentation.push(...links);
  }

  /**
   * Get general documentation links based on error type
   */
  private getGeneralDocumentationLinks(errorType: string): DocumentationLink[] {
    const baseLinks: DocumentationLink[] = [
      {
        title: "Rust Book",
        url: "https://doc.rust-lang.org/book/",
        description: "The Rust Programming Language book"
      },
      {
        title: "Rust Reference",
        url: "https://doc.rust-lang.org/reference/",
        description: "The Rust language reference"
      }
    ];

    // Add specific links based on error type
    switch (errorType) {
      case 'borrow_check':
        baseLinks.push({
          title: "Understanding Ownership",
          url: "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html",
          description: "Learn about Rust's ownership system"
        });
        break;
      case 'type_error':
        baseLinks.push({
          title: "Data Types",
          url: "https://doc.rust-lang.org/book/ch03-02-data-types.html",
          description: "Understanding Rust data types"
        });
        break;
      case 'lifetime_error':
        baseLinks.push({
          title: "Validating References with Lifetimes",
          url: "https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html",
          description: "Understanding lifetime annotations"
        });
        break;
      case 'trait_error':
        baseLinks.push({
          title: "Traits",
          url: "https://doc.rust-lang.org/book/ch10-02-traits.html",
          description: "Defining shared behavior with traits"
        });
        break;
    }

    return baseLinks;
  }

  /**
   * Get context-specific documentation links
   */
  private getContextSpecificLinks(errorMessage: string): DocumentationLink[] {
    const links: DocumentationLink[] = [];

    // Pattern matching for specific error messages
    if (errorMessage.includes('cannot borrow')) {
      links.push({
        title: "Borrowing Rules",
        url: "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html",
        description: "Understanding borrowing rules in Rust"
      });
    }

    if (errorMessage.includes('trait bound')) {
      links.push({
        title: "Trait Bounds",
        url: "https://doc.rust-lang.org/book/ch10-02-traits.html#trait-bounds",
        description: "Using trait bounds to specify functionality"
      });
    }

    if (errorMessage.includes('lifetime')) {
      links.push({
        title: "Lifetime Elision",
        url: "https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision",
        description: "Understanding lifetime elision rules"
      });
    }

    return links;
  }

  /**
   * Remove duplicate documentation links
   */
  private deduplicateLinks(links: DocumentationLink[]): DocumentationLink[] {
    const seen = new Set<string>();
    return links.filter(link => {
      const key = `${link.title}:${link.url}`;
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
  }

  /**
   * Helper methods for type checking and conversion
   */
  private isFixSuggestion(fix: CodeAction | FixSuggestion): fix is FixSuggestion {
    return 'fixType' in fix && 'changes' in fix;
  }

  private convertCodeActionToFixSuggestion(codeAction: CodeAction): FixSuggestion {
    return {
      id: `converted_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      title: codeAction.title,
      description: codeAction.title,
      fixType: 'quick-fix',
      changes: [], // Would need to extract from codeAction.edit
      confidence: 0.7,
      estimatedEffort: 'trivial',
      benefits: ['Fixes the reported issue'],
      risks: []
    };
  }

  private convertFixSuggestionToCodeAction(
    fixSuggestion: FixSuggestion,
    source: string
  ): CodeAction | null {
    if (fixSuggestion.changes.length === 0) {
      return null;
    }

    const edits: TextEdit[] = fixSuggestion.changes.map(change => ({
      range: {
        start: { line: change.range[0], character: change.range[1] },
        end: { line: change.range[2], character: change.range[3] }
      },
      newText: change.newText
    }));

    return {
      title: `${fixSuggestion.title} (${Math.round(fixSuggestion.confidence * 100)}% confidence)`,
      kind: CodeActionKind.QuickFix,
      edit: {
        documentChanges: [
          TextDocumentEdit.create(
            {
              uri: `file://${fixSuggestion.changes[0].filePath}`,
              version: null
            } as OptionalVersionedTextDocumentIdentifier,
            edits
          )
        ]
      }
    };
  }

  private convertToFixSuggestions(codeActions: CodeAction[]): FixSuggestion[] {
    return codeActions.map(action => this.convertCodeActionToFixSuggestion(action));
  }

  private convertToErrorExplanations(explanations: string[]): ErrorCodeExplanation[] {
    return explanations.map((explanation, index) => ({
      errorCode: `generic_${index}`,
      title: 'Error Explanation',
      explanation,
      examples: [],
      documentationLinks: []
    }));
  }

  /**
   * Categorize error for pattern matching
   */
  private categorizeError(error: Diagnostic): string {
    const message = error.message.toLowerCase();

    if (message.includes('borrow') || message.includes('ownership')) {
      return 'borrow_check';
    }
    if (message.includes('type') || message.includes('expected')) {
      return 'type_error';
    }
    if (message.includes('lifetime')) {
      return 'lifetime_error';
    }
    if (message.includes('trait')) {
      return 'trait_error';
    }
    if (message.includes('unused')) {
      return 'unused_code';
    }
    if (message.includes('syntax') || message.includes('parse')) {
      return 'syntax_error';
    }

    return 'general';
  }

  /**
   * Extract workspace path from file path
   */
  private extractWorkspacePath(filePath: string): string {
    // Simple heuristic: find the directory containing Cargo.toml
    const parts = filePath.split('/');
    for (let i = parts.length - 1; i >= 0; i--) {
      const potentialWorkspace = parts.slice(0, i + 1).join('/');
      // In a real implementation, you'd check if Cargo.toml exists
      // For now, assume the workspace is the parent of 'src' directory
      if (parts[i] === 'src' && i > 0) {
        return parts.slice(0, i).join('/');
      }
    }
    // Fallback to directory containing the file
    return parts.slice(0, -1).join('/');
  }

  /**
   * Initialize enhanced error patterns with confidence scoring
   */
  private initializePatterns(): void {
    // Enhanced unused variable pattern
    this.errorPatterns.set('unused_variable', {
      matcher: (error) => error.message.includes('unused variable'),
      quickFixes: (error, documentText) => {
        const { range } = error;
        const line = documentText.split('\n')[range.start.line];
        const match = line.match(/let\s+([a-zA-Z_][a-zA-Z0-9_]*)/);

        if (match) {
          const varName = match[1];
          return [{
            title: `Prefix with _ to indicate it's intentionally unused (High confidence)`,
            kind: CodeActionKind.QuickFix,
            edit: {
              documentChanges: [
                TextDocumentEdit.create(
                  {
                    uri: `file://${error.source}`,
                    version: null
                  } as OptionalVersionedTextDocumentIdentifier,
                  [
                    TextEdit.replace(
                      {
                        start: { line: range.start.line, character: range.start.character + 4 },
                        end: { line: range.start.line, character: range.start.character + 4 + varName.length },
                      },
                      `_${varName}`,
                    ),
                  ],
                ),
              ],
            },
          }];
        }
        return [];
      },
      explanation: () => "This warning occurs when you declare a variable but don't use it. " +
        "You can either use the variable or prefix it with an underscore to indicate it's intentionally unused.",
    });

    // Borrow checker error pattern
    this.errorPatterns.set('borrow_check', {
      matcher: (error) => error.message.includes('cannot borrow') || error.message.includes('already borrowed'),
      quickFixes: (error, documentText) => {
        const fixes: CodeAction[] = [];

        if (error.message.includes('mutable borrow')) {
          fixes.push({
            title: 'Consider using RefCell for interior mutability',
            kind: CodeActionKind.Refactor,
            edit: {
              documentChanges: []
            }
          });
        }

        return fixes;
      },
      explanation: () => "Rust's borrow checker ensures memory safety by preventing data races. " +
        "This error occurs when you try to borrow data in a way that violates borrowing rules.",
    });

    // Type mismatch pattern
    this.errorPatterns.set('type_mismatch', {
      matcher: (error) => error.message.includes('expected') && error.message.includes('found'),
      quickFixes: (error, documentText) => {
        const fixes: CodeAction[] = [];

        // Extract expected and found types
        const expectedMatch = error.message.match(/expected `([^`]+)`/);
        const foundMatch = error.message.match(/found `([^`]+)`/);

        if (expectedMatch && foundMatch) {
          const expected = expectedMatch[1];
          const found = foundMatch[1];

          fixes.push({
            title: `Convert ${found} to ${expected}`,
            kind: CodeActionKind.QuickFix,
            edit: {
              documentChanges: []
            }
          });
        }

        return fixes;
      },
      explanation: () => "Type mismatch errors occur when the compiler expects one type but finds another. " +
        "Check the types involved and consider type conversion or correction.",
    });

    // Add more sophisticated patterns...
  }
}

// Enhanced error pattern interface with confidence scoring
interface ErrorPattern {
  matcher: (error: Diagnostic, documentText: string) => boolean;
  quickFixes?: (error: Diagnostic, documentText: string) => CodeAction[];
  explanation?: (error: Diagnostic, documentText: string) => string;
  confidence?: number; // Confidence in this pattern's effectiveness
  category?: string; // Error category for better organization
}
