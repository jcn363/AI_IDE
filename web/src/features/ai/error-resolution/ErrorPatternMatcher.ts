import {
  ErrorPattern,
  FixSuggestion,
  ChangeType,
  CodeChange,
  ErrorCategory,
  PatternMatch,
  PatternMatchResult,
  DocumentationLink
} from '../types/error-resolution';

/**
 * Pattern Matching Engine for Error Resolution
 * Handles matching error messages against known patterns and generating fixes
 */
export class ErrorPatternMatcher {
  private patterns: Map<string, ErrorPattern> = new Map();
  private patternCache: Map<string, PatternMatchResult[]> = new Map();

  constructor() {
    this.initializeBuiltInPatterns();
  }

  /**
   * Match error message against all known patterns
   */
  matchError(errorMessage: string, context?: string, language?: string): PatternMatchResult[] {
    const results: PatternMatchResult[] = [];
    const cacheKey = `${errorMessage}:${context || ''}:${language || ''}`;

    // Check cache first
    if (this.patternCache.has(cacheKey)) {
      return this.patternCache.get(cacheKey)!;
    }

    for (const [patternId, pattern] of this.patterns) {
      const confidence = this.calculatePatternConfidence(errorMessage, pattern, context, language);
      if (confidence > 0) {
        const matches = this.extractMatches(errorMessage, pattern);
        const suggestedFixes = this.generateFixesForPattern(pattern, matches, errorMessage);

        results.push({
          pattern,
          confidence,
          matches,
          suggestedFixes
        });
      }
    }

    // Cache results
    this.patternCache.set(cacheKey, results);
    return results;
  }

  /**
   * Register a new error pattern
   */
  registerPattern(pattern: ErrorPattern): void {
    this.patterns.set(pattern.id, pattern);
    // Clear cache when patterns change
    this.patternCache.clear();
  }

  /**
   * Generate fixes for a matched pattern
   */
  private generateFixesForPattern(
    pattern: ErrorPattern,
    matches: PatternMatch[],
    originalError: string
  ): FixSuggestion[] {
    const fixes: FixSuggestion[] = [];

    // Generate fix based on pattern type and category
    switch (pattern.errorType) {
      case 'unused_variable':
        fixes.push(this.generateUnusedVariableFix(matches[0], originalError));
        break;
      case 'borrow_check':
        fixes.push(...this.generateBorrowCheckFixes(matches, originalError));
        break;
      case 'type_mismatch':
        fixes.push(this.generateTypeMismatchFix(matches[0], originalError));
        break;
      default:
        fixes.push(this.generateGenericFix(pattern, originalError));
    }

    return fixes;
  }

  /**
   * Generate fix for unused variables
   */
  private generateUnusedVariableFix(match: PatternMatch, originalError: string): FixSuggestion {
    return {
      id: `fix_${Date.now()}_${Math.random()}`,
      title: 'Prefix with underscore to indicate intentional unused variable',
      description: 'Variables prefixed with underscore indicate they are intentionally unused',
      errorId: `error_${Date.now()}`,
      priority: 'medium',
      fixType: 'add-missing',
      changes: this.generateVariableChanges(match, '_'),
      confidence: 0.8,
      estimatedEffort: 'trivial',
      benefits: ['Eliminates warning from unused variable', 'Makes code intent clear'],
      risks: ['May mask actual issues if variable should be used'],
      dependencies: [],
      testSuggestions: []
    };
  }

  /**
   * Generate fixes for borrowing issues
   */
  private generateBorrowCheckFixes(matches: PatternMatch[], originalError: string): FixSuggestion[] {
    const fixes: FixSuggestion[] = [];

    // Fix for mutable borrow after immutable borrow
    if (originalError.includes('cannot borrow') && originalError.includes('as mutable')) {
      fixes.push({
        id: `fix_${Date.now()}_mut_borrow_1`,
        title: 'Use RefCell for interior mutability',
        description: 'Wrap the value in RefCell to allow interior mutability',
        errorId: `error_${Date.now()}`,
        priority: 'high',
        fixType: 'refactor',
        changes: this.generateRefCellChanges(),
        confidence: 0.7,
        estimatedEffort: 'medium',
        benefits: ['Allows mutation of borrowed data', 'Maintains memory safety'],
        risks: ['Runtime borrow checking overhead'],
        dependencies: [],
        testSuggestions: ['Verify RefCell borrowing rules are respected']
      });

      // Clone alternative
      fixes.push({
        id: `fix_${Date.now()}_mut_borrow_2`,
        title: 'Use .clone() to create owned copy',
        description: 'Clone the value to avoid borrowing conflicts',
        errorId: `error_${Date.now()}`,
        priority: 'medium',
        fixType: 'refactor',
        changes: this.generateCloneChanges(),
        confidence: 0.6,
        estimatedEffort: 'low',
        benefits: ['Simple solution for small data structures'],
        risks: ['Performance impact for large data structures'],
        dependencies: [],
        testSuggestions: []
      });
    }

    return fixes;
  }

  /**
   * Generate fix for type mismatches
   */
  private generateTypeMismatchFix(match: PatternMatch, originalError: string): FixSuggestion {
    return {
      id: `fix_${Date.now()}_type_mismatch`,
      title: 'Add type annotation to resolve mismatch',
      description: 'Explicit type annotation to help compiler infer correct types',
      errorId: `error_${Date.now()}`,
      priority: 'high',
      fixType: 'add-missing',
      changes: this.generateTypeAnnotationChanges(match),
      confidence: 0.7,
      estimatedEffort: 'trivial',
      benefits: ['Resolves type mismatch error', 'Makes code more explicit'],
      risks: ['May not be the intended type'],
      dependencies: [],
      testSuggestions: []
    };
  }

  /**
   * Generate generic fallback fix
   */
  private generateGenericFix(pattern: ErrorPattern, originalError: string): FixSuggestion {
    return {
      id: `fix_${Date.now()}_generic`,
      title: 'Generic error fix suggestion',
      description: 'Please review the error message and consider manual fixes',
      errorId: `error_${Date.now()}`,
      priority: 'low',
      fixType: 'refactor',
      changes: [],
      confidence: 0.3,
      estimatedEffort: 'high',
      benefits: [],
      risks: [],
      dependencies: [],
      testSuggestions: [],
      documentationLinks: this.generateDocumentationLinks(pattern.errorType)
    };
  }

  /**
   * Initialize built-in patterns for common Rust errors
   */
  private initializeBuiltInPatterns(): void {
    // Unused variable pattern
    this.registerPattern({
      id: 'unused_variable',
      errorType: 'unused_variable',
      pattern: 'unused variable',
      context: '',
      frequency: 0,
      lastSeen: new Date().toISOString(),
      confidence: 1.0,
      language: 'rust'
    });

    // Borrow checker patterns
    this.registerPattern({
      id: 'borrow_check_mutable',
      errorType: 'borrow_check',
      pattern: 'cannot borrow .* as mutable',
      context: '',
      frequency: 0,
      lastSeen: new Date().toISOString(),
      confidence: 0.9,
      language: 'rust'
    });

    this.registerPattern({
      id: 'borrow_check_immutable',
      errorType: 'borrow_check',
      pattern: 'cannot borrow .* immutable borrow',
      context: '',
      frequency: 0,
      lastSeen: new Date().toISOString(),
      confidence: 0.9,
      language: 'rust'
    });

    // Type mismatch patterns
    this.registerPattern({
      id: 'type_mismatch',
      errorType: 'type_mismatch',
      pattern: 'expected .* found .*',
      context: '',
      frequency: 0,
      lastSeen: new Date().toISOString(),
      confidence: 0.8,
      language: 'rust'
    });

    // Missing trait implementation
    this.registerPattern({
      id: 'trait_not_implemented',
      errorType: 'trait_error',
      pattern: '.* the trait .* is not implemented',
      context: '',
      frequency: 0,
      lastSeen: new Date().toISOString(),
      confidence: 0.8,
      language: 'rust'
    });
  }

  /**
   * Calculate confidence score for pattern matching
   */
  private calculatePatternConfidence(
    errorMessage: string,
    pattern: ErrorPattern,
    context?: string,
    language?: string
  ): number {
    let confidence = pattern.confidence;

    // Language match bonus
    if (language && pattern.language && pattern.language === language) {
      confidence += 0.1;
    }

    // Exact pattern match
    if (typeof pattern.pattern === 'string' && errorMessage.includes(pattern.pattern)) {
      confidence += 0.1;
    }

    // Regex pattern match
    if (pattern.pattern instanceof RegExp && pattern.pattern.test(errorMessage)) {
      confidence += 0.1;
    }

    return Math.min(confidence, 1.0);
  }

  /**
   * Extract pattern matches from error message
   */
  private extractMatches(errorMessage: string, pattern: ErrorPattern): PatternMatch[] {
    const matches: PatternMatch[] = [];

    if (typeof pattern.pattern === 'string') {
      const index = errorMessage.indexOf(pattern.pattern);
      if (index !== -1) {
        matches.push({
          line: 0, // Would need line number parsing for real implementation
          column: index,
          length: pattern.pattern.length,
          context: errorMessage,
          capturedGroups: {}
        });
      }
    } else if (pattern.pattern instanceof RegExp) {
      const match = pattern.pattern.exec(errorMessage);
      if (match) {
        matches.push({
          line: 0,
          column: match.index,
          length: match[0].length,
          context: errorMessage,
          capturedGroups: match.groups || {}
        });
      }
    }

    return matches;
  }

  /**
   * Helper methods for generating specific changes
   */
  private generateVariableChanges(match: PatternMatch, prefix: string): CodeChange[] {
    return [{
      filePath: 'unknown.rs', // Would be provided by caller
      changeType: ChangeType.Replace,
      range: {
        startLine: match.line,
        startColumn: match.column,
        endLine: match.line,
        endColumn: match.column + match.length
      },
      newText: `${prefix}${match.context.substring(match.column, match.column + match.length)}`,
      description: `Prefix variable with underscore`
    }];
  }

  private generateRefCellChanges(): CodeChange[] {
    return [{
      filePath: 'unknown.rs',
      changeType: ChangeType.Insert,
      range: {
        startLine: 0,
        startColumn: 0,
        endLine: 0,
        endColumn: 0
      },
      newText: 'use std::cell::RefCell;',
      description: 'Add RefCell import'
    }];
  }

  private generateCloneChanges(): CodeChange[] {
    return [{
      filePath: 'unknown.rs',
      changeType: ChangeType.Replace,
      range: {
        startLine: 0,
        startColumn: 0,
        endLine: 0,
        endColumn: 0
      },
      newText: '.clone()',
      description: 'Clone value to resolve borrowing conflict'
    }];
  }

  private generateTypeAnnotationChanges(match: PatternMatch): CodeChange[] {
    return [{
      filePath: 'unknown.rs',
      changeType: ChangeType.Insert,
      range: {
        startLine: match.line,
        startColumn: match.column,
        endLine: match.line,
        endColumn: match.column + match.length
      },
      newText: ': TypeName',
      description: 'Add explicit type annotation'
    }];
  }

  private generateDocumentationLinks(errorType: string): DocumentationLink[] {
    const links: DocumentationLink[] = [];

    switch (errorType) {
      case 'borrow_check':
        links.push({
          title: 'Understanding Ownership',
          url: 'https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html',
          description: 'Learn about Rust ownership and borrowing',
          relevance: 'high',
          type: 'official-docs'
        });
        break;
      case 'type_mismatch':
        links.push({
          title: 'Data Types',
          url: 'https://doc.rust-lang.org/book/ch03-02-data-types.html',
          description: 'Understanding Rust data types',
          relevance: 'medium',
          type: 'official-docs'
        });
        break;
      default:
        links.push({
          title: 'The Rust Programming Language',
          url: 'https://doc.rust-lang.org/book/',
          description: 'Official Rust programming language guide',
          relevance: 'low',
          type: 'official-docs'
        });
    }

    return links;
  }
}