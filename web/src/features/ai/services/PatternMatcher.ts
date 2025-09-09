import type { Range } from '../types';

/**
 * Pattern matching utilities for code analysis and refactoring
 */
export class PatternMatcher {
  /**
   * Extract variable declarations using improved Rust-aware patterns
   */
  extractVariableDeclarations(code: string): Array<{ variableName: string; line: number; initializer: string }> {
    const variables: Array<{ variableName: string; line: number; initializer: string }> = [];
    const lines = code.split('\n');

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();

      // Skip empty lines and comments
      if (!line || line.startsWith('//') || line.startsWith('/*')) {
        continue;
      }

      // Pattern for basic variable declarations: let [mut] name[=|:type|=] value;
      const varPattern = /^let\s+(mut\s+)?(\w+)(\s*:\s*[^=]+)?(\s*=\s*(.+?))?;?\s*$/;
      const match = line.match(varPattern);

      if (match && match[2]) {
        variables.push({
          variableName: match[2],
          line: i + 1,
          initializer: match[5] || '',
        });
      }

      // Pattern for destructuring: let (a, b[, ...]) = value;
      const destructuringPattern = /^let\s+(mut\s+)?(\([^)]+\)|\{[^}]+\})\s*=\s*(.+?);?\s*$/;
      const destructureMatch = line.match(destructuringPattern);

      if (destructureMatch && destructureMatch[2] && destructureMatch[3]) {
        // Extract variable names from destructuring pattern
        const pattern = destructureMatch[2];
        const varMatches = pattern.match(/\b\w+\b/g);

        if (varMatches) {
          for (const varName of varMatches) {
            variables.push({
              variableName: varName,
              line: i + 1,
              initializer: destructureMatch[3],
            });
          }
        }
      }
    }

    return variables;
  }

  /**
   * Check if a symbol has side effects based on its usage context
   */
  hasSideEffects(initializer: string): boolean {
    const sideEffectPatterns = [
      // Method calls that may mutate
      /\b(push|pop|insert|remove|clear|sort|reverse|append|extend|retain)\s*\(/,
      // I/O operations
      /\b(print|println|eprint|eprintln|write|read)\s*!?/,
      // File system operations
      /\b(create|open|write_to_string)\s*\(/,
      // Network operations
      /\b(get|post|put|delete|send|recv)\s*\(/,
      // Unsafe operations and panics
      /\b(unsafe|unwrap|expect|panic!)\s*!?\(/,
      // Mutable operations
      /\.\s*(borrow_mut)\s*\(\)/,
    ];

    return sideEffectPatterns.some(pattern => pattern.test(initializer));
  }

  /**
   * Find symbol occurrences with improved pattern matching
   */
  findSymbolOccurrences(
    content: string,
    symbolName: string,
    startRange?: Range
  ): Array<{ range: Range; context: string; originalText: string }> {
    const occurrences: Array<{ range: Range; context: string; originalText: string }> = [];

    // Comprehensive pattern matching for symbol occurrences
    const patterns = [
      // Direct symbol usage (variables, functions, etc.)
      new RegExp(`\\b${symbolName}\\b`, 'g'),
      // Method/field access: instance.symbol
      new RegExp(`\\w+\\.${symbolName}(?:[^\\w\\(]|$)`, 'g'),
      // Static access: Type::symbol
      new RegExp(`\\w+::${symbolName}\\b`, 'g'),
      // Function calls: symbol(args)
      new RegExp(`${symbolName}\\s*\\(`, 'g'),
    ];

    const lines = content.split('\n');
    let lineOffset = 0;

    if (startRange) {
      lineOffset = startRange.start.line;
    }

    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      const currentLineIndex = lineIndex + lineOffset;

      // Skip lines that are likely inside block comments
      if (this.isInsideBlockComment(line, lines, lineIndex)) {
        continue;
      }

      for (const pattern of patterns) {
        let match;
        pattern.lastIndex = 0;

        while ((match = pattern.exec(line)) !== null) {
          const matchText = match[0];
          const matchStart = match.index;
          const matchEnd = matchStart + matchText.length;

          // Skip if this match is inside a string literal
          if (this.isInsideStringLiteral(line, matchStart)) {
            continue;
          }

          // Create range for this occurrence
          const range: Range = {
            start: { line: currentLineIndex, character: matchStart },
            end: { line: currentLineIndex, character: matchEnd },
          };

          // Get context (surrounding lines for better understanding)
          const contextStart = Math.max(0, lineIndex - 1);
          const contextEnd = Math.min(lines.length - 1, lineIndex + 1);
          const contextLines = lines.slice(contextStart, contextEnd + 1);
          const context = contextLines.join('\n');

          occurrences.push({
            range,
            context,
            originalText: matchText.trim(),
          });

          // For word boundaries, break to avoid duplicates
          if (pattern.global && matchText.match(/^\w+$/)) {
            break;
          }
        }
      }
    }

    return this.removeDuplicateOccurrences(occurrences);
  }

  /**
   * Check if a position is inside a string literal
   */
  private isInsideStringLiteral(line: string, position: number): boolean {
    let inSingleQuote = false;
    let inDoubleQuote = false;
    let escapeNext = false;

    for (let i = 0; i <= position; i++) {
      const char = line[i];

      if (escapeNext) {
        escapeNext = false;
        continue;
      }

      if (char === '\\') {
        escapeNext = true;
        continue;
      }

      if (char === '"' && !inSingleQuote) {
        inDoubleQuote = !inDoubleQuote;
      } else if (char === "'" && !inDoubleQuote) {
        inSingleQuote = !inSingleQuote;
      }
    }

    return inSingleQuote || inDoubleQuote;
  }

  /**
   * Check if a line is inside a block comment
   */
  private isInsideBlockComment(line: string, allLines: string[], lineIndex: number): boolean {
    let inBlockComment = false;

    for (let i = 0; i < lineIndex; i++) {
      const prevLine = allLines[i];
      const blockStart = prevLine.match(/\/\*/);
      const blockEnd = prevLine.match(/\*\//);

      if (blockStart && (!blockEnd || blockStart.index! < blockEnd.index!)) {
        inBlockComment = true;
      }

      if (blockEnd && (!blockStart || blockEnd.index! < blockStart.index!)) {
        inBlockComment = false;
      }
    }

    return inBlockComment;
  }

  /**
   * Remove duplicate occurrences
   */
  private removeDuplicateOccurrences(
    occurrences: Array<{ range: Range; context: string; originalText: string }>
  ): Array<{ range: Range; context: string; originalText: string }> {
    const result: Array<{ range: Range; context: string; originalText: string }> = [];
    const processed = new Set<string>();

    for (const occurrence of occurrences) {
      const key = `${occurrence.range.start.line}:${occurrence.range.start.character}-${occurrence.range.end.character}`;

      if (!processed.has(key)) {
        processed.add(key);
        result.push(occurrence);
      }
    }

    return result.sort((a, b) => {
      if (a.range.start.line !== b.range.start.line) {
        return a.range.start.line - b.range.start.line;
      }
      return a.range.start.character - b.range.start.character;
    });
  }

  /**
   * Extract complex expressions for variable extraction
   */
  findComplexExpressions(selectedText: string): Array<{
    expression: string;
    startOffset: number;
    endOffset: number;
    complexity: number;
  }> {
    const complexExpressions = [
      // Find arithmetic expressions
      /[\w\s]*[+\-*/%][\w\s]*[+\-*/%]*/g,
      // Find function calls
      /\w+\s*\([^)]*\)/g,
      // Find array/closure expressions
      /\[.*\]/g,
      /\{.*\}/g,
      // Find method chains
      /\w+\.\w+.*?\./g,
    ];

    const candidates: Array<{
      expression: string;
      startOffset: number;
      endOffset: number;
      complexity: number;
    }> = [];

    for (const pattern of complexExpressions) {
      let match;
      while ((match = pattern.exec(selectedText)) !== null) {
        candidates.push({
          expression: match[0],
          startOffset: match.index,
          endOffset: match.index + match[0].length,
          complexity: this.calculateExpressionComplexity(match[0]),
        });
      }
    }

    return candidates;
  }

  /**
   * Calculate expression complexity score
   */
  calculateExpressionComplexity(expression: string): number {
    let score = 0;

    // Count operators
    score += (expression.match(/[+\-*/%]/g) || []).length;

    // Count function calls
    score += (expression.match(/\w+\s*\(/g) || []).length * 2;

    // Count method calls
    score += (expression.match(/\.\w+/g) || []).length;

    // Count literals
    score += (expression.match(/\d+/g) || []).length * 0.5;

    return score;
  }

  /**
   * Suggest variable names based on expression content
   */
  suggestVariableNames(expression: string): string[] {
    const suggestions: string[] = [];

    // Extract meaningful words from expression
    const words = expression.split(/[\s()[\]{}+\-*/%.]/).filter(word => word.length > 2);

    if (words.length > 0) {
      // Create camelCase variations
      const mainWord = words[0].toLowerCase();
      suggestions.push(`${mainWord}Result`);
      suggestions.push(`${mainWord}Value`);
      suggestions.push(`extracted${words[0]}`);

      if (words.length > 1) {
        suggestions.push(`${words[0].toLowerCase()}${words[1]}`);
      }
    }

    return suggestions.length > 0 ? suggestions : ['extractedValue', 'result', 'value'];
  }
}