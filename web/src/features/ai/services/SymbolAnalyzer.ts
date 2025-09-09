import type { Range } from '../types';

/**
 * Symbol analyzer for finding and analyzing symbol occurrences in code
 */
export class SymbolAnalyzer {
  /**
   * Find all occurrences of a symbol in the given content
   */
  findSymbolOccurrences(
    content: string,
    symbolName: string,
    startRange?: Range
  ): Array<{ range: Range; context: string; originalText: string }> {
    const occurrences: Array<{ range: Range; context: string; originalText: string }> = [];

    // Basic pattern matching for symbol occurrences
    const patterns = [
      // Variable/function declarations and usage
      new RegExp(`\\b${symbolName}\\b`, 'g'),
      // Method/field access: instance.symbol
      new RegExp(`\\w+\\.${symbolName}(?:[^\\w\\(]|$)`, 'g'),
      // Static method calls: Type::method(args)
      new RegExp(`\\w+::${symbolName}\\s*\\(`, 'g'),
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
        pattern.lastIndex = 0; // Reset regex state

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
            originalText: matchText,
          });

          // Prevent infinite loop for global patterns
          if (pattern.global) {
            break;
          }
        }
      }
    }

    // Remove duplicates and sort by position
    const uniqueOccurrences = this.removeDuplicateOccurrences(occurrences);
    return uniqueOccurrences.sort((a, b) => {
      if (a.range.start.line !== b.range.start.line) {
        return a.range.start.line - b.range.start.line;
      }
      return a.range.start.character - b.range.start.character;
    });
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
   * Remove duplicate occurrences that overlap
   */
  private removeDuplicateOccurrences(
    occurrences: Array<{ range: Range; context: string; originalText: string }>
  ): Array<{ range: Range; context: string; originalText: string }> {
    const result: Array<{ range: Range; context: string; originalText: string }> = [];
    const processed = new Set<string>();

    for (const occurrence of occurrences) {
      const key = `${occurrence.range.start.line}:${occurrence.range.start.character}-${occurrence.range.end.line}:${occurrence.range.end.character}`;

      if (!processed.has(key)) {
        processed.add(key);
        result.push(occurrence);
      }
    }

    return result;
  }
}