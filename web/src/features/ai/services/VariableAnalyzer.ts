import { PatternMatcher } from './PatternMatcher';

/**
 * Variable analyzer for parameterization and mutation analysis
 */
export class VariableAnalyzer {
  private readonly patternMatcher: PatternMatcher;

  constructor() {
    this.patternMatcher = new PatternMatcher();
  }

  /**
   * Analyze if a variable can be safely parameterized
   */
  canParameterizeVariable(
    variableName: string,
    initializer: string,
    subsequentLines: string[]
  ): boolean {
    // Check for side effects in initializer
    if (this.patternMatcher.hasSideEffects(initializer)) {
      return false;
    }

    // Check if variable is mutated or used in mutation contexts
    if (this.isVariableMutated(variableName, subsequentLines)) {
      return false;
    }

    return true;
  }

  /**
   * Check if a variable is mutated in the given code lines
   */
  private isVariableMutated(variableName: string, lines: string[]): boolean {
    const escapedName = variableName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

    const mutationPatterns = [
      // Direct assignment: variable = ...
      new RegExp(`^\\s*${escapedName}\\s*=\\s*`),
      // Mutable reference: &mut variable
      new RegExp(`&mut\\s+${escapedName}\\b`),
      // Method calls that might mutate
      new RegExp(`${escapedName}\\s*\\.\\s*(push|insert|remove|clear|sort|reverse)\\s*\\(`),
      // Mutable borrow
      new RegExp(`${escapedName}\\s*\\.\\s*borrow_mut\\s*\\(\\)`),
    ];

    for (const line of lines) {
      const trimmed = line.trim();

      // Skip comments
      if (trimmed.startsWith('//') || trimmed.startsWith('/*')) {
        continue;
      }

      if (mutationPatterns.some(pattern => pattern.test(trimmed))) {
        return true;
      }

      // Check for mutation in closures or complex expressions
      if (this.isInMutatingContext(variableName, trimmed)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Check if variable is used in a mutating context
   */
  private isInMutatingContext(variableName: string, line: string): boolean {
    const escapedName = variableName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

    const contextPatterns = [
      // Move closures that capture the variable
      new RegExp(`move\\s*\\|\\|.*\\b${escapedName}\\b.*\\}`),
      // FnMut closures
      new RegExp(`FnMut.*\\b${escapedName}\\b`),
      // Explicit mutable variable declarations
      new RegExp(`let\\s+mut\\s+\\b${escapedName}\\b`),
    ];

    return contextPatterns.some(pattern => pattern.test(line));
  }

  /**
   * Extract variable declarations from a single line using Rust-aware patterns
   */
  extractVariableDeclarationFromLine(line: string): { variableName: string; initializer: string } | null {
    // Basic pattern for variable declarations with optional mut and optional type annotation
    const basicPattern = /^let\s+(mut\s+)?(\w+)(\s*:\s*[^=]+)?\s*=\s*(.+);?$/;
    let match = basicPattern.exec(line.trim());
    if (match && match[2] && match[4]) {
      return {
        variableName: match[2],
        initializer: match[4].trim()
      };
    }

    // Pattern for destructuring assignments
    const destructuringPattern = /^let\s+(mut\s+)?([({].*[})])\s*=\s*(.+);?$/;
    match = destructuringPattern.exec(line.trim());
    if (match && match[2] && match[3]) {
      // Extract first identifier from destructuring pattern
      const destructuring = match[2];
      const identifierMatch = destructuring.match(/\b(\w+)\b/);
      if (identifierMatch) {
        return {
          variableName: identifierMatch[1],
          initializer: match[3].trim()
        };
      }
    }

    return null;
  }

  /**
   * Find variable declarations in code using Rust-aware patterns
   */
  findVariableDeclarations(code: string): string[] {
    const variables: string[] = [];
    const lines = code.split('\n');

    for (const line of lines) {
      const trimmedLine = line.trim();

      // Skip empty lines and comments
      if (!trimmedLine || trimmedLine.startsWith('//') || trimmedLine.startsWith('/*')) {
        continue;
      }

      const match = this.extractVariableDeclarationFromLine(trimmedLine);
      if (match) {
        variables.push(match.variableName);
      }
    }

    // Remove duplicates and return
    return [...new Set(variables)];
  }
}