/**
 * Range conversion utilities for consistent handling between frontend, backend, and LSP
 * Handles different indexing conventions to prevent off-by-one errors
 */

export interface FrontendRange {
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
}

export interface BackendRange {
  start_line: number;
  start_character: number;
  end_line: number;
  end_character: number;
}

export interface LSPRange {
  start: {
    line: number;
    character: number;
  };
  end: {
    line: number;
    character: number;
  };
}

/**
 * Convert LSP range (0-based, exclusive end) to frontend range (0-based)
 */
export function lspRangeToFrontend(lspRange: LSPRange): FrontendRange {
  return {
    startLine: lspRange.start.line,
    startCharacter: lspRange.start.character,
    endLine: lspRange.end.line,
    endCharacter: lspRange.end.character,
  };
}

/**
 * Convert frontend range to LSP range
 */
export function frontendRangeToLSP(frontendRange: FrontendRange): LSPRange {
  return {
    start: {
      line: frontendRange.startLine,
      character: frontendRange.startCharacter,
    },
    end: {
      line: frontendRange.endLine,
      character: frontendRange.endCharacter,
    },
  };
}

/**
 * Convert backend range (0-based) to frontend range (0-based)
 */
export function backendRangeToFrontend(backendRange: BackendRange): FrontendRange {
  return {
    startLine: backendRange.start_line,
    startCharacter: backendRange.start_character,
    endLine: backendRange.end_line,
    endCharacter: backendRange.end_character,
  };
}

/**
 * Convert frontend range to backend range
 */
export function frontendRangeToBackend(frontendRange: FrontendRange): BackendRange {
  return {
    start_line: frontendRange.startLine,
    start_character: frontendRange.startCharacter,
    end_line: frontendRange.endLine,
    end_character: frontendRange.endCharacter,
  };
}

/**
 * Convert backend range to LSP range (both 0-based, but need to handle character offsets)
 */
export function backendRangeToLSP(backendRange: BackendRange): LSPRange {
  return {
    start: {
      line: backendRange.start_line,
      character: backendRange.start_character,
    },
    end: {
      line: backendRange.end_line,
      character: backendRange.end_character,
    },
  };
}

/**
 * Convert LSP range to backend range
 */
export function lspRangeToBackend(lspRange: LSPRange): BackendRange {
  return {
    start_line: lspRange.start.line,
    start_character: lspRange.start.character,
    end_line: lspRange.end.line,
    end_character: lspRange.end.character,
  };
}

/**
 * Create a minimal range for cursor position only
 */
export function createCursorRange(line: number, character: number): FrontendRange {
  return {
    startLine: line,
    startCharacter: character,
    endLine: line,
    endCharacter: character,
  };
}

/**
 * Validate range integrity
 */
export function isValidRange(range: FrontendRange | BackendRange | LSPRange): boolean {
  let startLine: number, startChar: number, endLine: number, endChar: number;

  // Extract line and character values based on range type
  if ('start_line' in range) {
    // BackendRange
    startLine = range.start_line;
    startChar = range.start_character;
    endLine = range.end_line;
    endChar = range.end_character;
  } else if ('startLine' in range) {
    // FrontendRange
    startLine = range.startLine;
    startChar = range.startCharacter;
    endLine = range.endLine;
    endChar = range.endCharacter;
  } else {
    // LSPRange
    startLine = range.start.line;
    startChar = range.start.character;
    endLine = range.end.line;
    endChar = range.end.character;
  }

  // Validate non-negative
  if (startLine < 0 || startChar < 0 || endLine < 0 || endChar < 0) {
    return false;
  }

  // Validate ordering
  if (startLine > endLine) {
    return false;
  }

  if (startLine === endLine && startChar > endChar) {
    return false;
  }

  return true;
}

/**
 * Get range length in characters
 */
export function getRangeLength(range: FrontendRange | BackendRange | LSPRange): number {
  let startLine: number, startChar: number, endLine: number, endChar: number;

  // Extract values
  if ('start_line' in range) {
    startLine = range.start_line;
    startChar = range.start_character;
    endLine = range.end_line;
    endChar = range.end_character;
  } else if ('startLine' in range) {
    startLine = range.startLine;
    startChar = range.startCharacter;
    endLine = range.endLine;
    endChar = range.endCharacter;
  } else {
    startLine = range.start.line;
    startChar = range.start.character;
    endLine = range.end.line;
    endChar = range.end.character;
  }

  if (startLine === endLine) {
    return endChar - startChar;
  }

  // Simplified calculation - doesn't account for actual line lengths
  return -1; // Represents multi-line range
}

/**
 * Expand range by a certain number of characters on each side
 */
export function expandRange(range: FrontendRange, before: number, after: number): FrontendRange {
  return {
    startLine: range.startLine,
    startCharacter: Math.max(0, range.startCharacter - before),
    endLine: range.endLine,
    endCharacter: range.endCharacter + after,
  };
}

/**
 * Check if a position is within a range
 */
export function positionInRange(line: number, character: number, range: FrontendRange): boolean {
  if (line < range.startLine || line > range.endLine) {
    return false;
  }

  if (line === range.startLine && character < range.startCharacter) {
    return false;
  }

  if (line === range.endLine && character >= range.endCharacter) {
    return false;
  }

  return true;
}

/**
 * Convert line-column array to frontend range (useful for backend responses)
 */
export function arrayToFrontendRange(array: [number, number, number, number]): FrontendRange {
  return {
    startLine: array[0],
    startCharacter: array[1],
    endLine: array[2],
    endCharacter: array[3],
  };
}

/**
 * Range conversion test suite
 */
export class RangeConverterTest {
  static testAllConversions() {
    const testRange: FrontendRange = {
      startLine: 5,
      startCharacter: 10,
      endLine: 8,
      endCharacter: 20,
    };

    // Test roundtrip conversions
    const backendRange = frontendRangeToBackend(testRange);
    const backToFrontend = backendRangeToFrontend(backendRange);

    console.log('Original range:', testRange);
    console.log('Backend range:', backendRange);
    console.log('Roundtrip:', backToFrontend);

    const backendRoundtrip = JSON.stringify(testRange) === JSON.stringify(backToFrontend);
    console.log('Frontend -> Backend -> Frontend roundtrip:', backendRoundtrip);

    const lspRange = frontendRangeToLSP(testRange);
    const lspToFrontend = lspRangeToFrontend(lspRange);

    console.log('LSP range:', lspRange);
    console.log('LSP back to frontend:', lspToFrontend);

    const lspRoundtrip = JSON.stringify(testRange) === JSON.stringify(lspToFrontend);
    console.log('Frontend -> LSP -> Frontend roundtrip:', lspRoundtrip);

    // Test edge cases
    this.testEdgeCases();

    return backendRoundtrip && lspRoundtrip;
  }

  static testEdgeCases() {
    console.log('\n=== Testing Edge Cases ===');

    // Empty range (cursor position)
    const emptyRange: FrontendRange = { startLine: 3, startCharacter: 5, endLine: 3, endCharacter: 5 };
    console.log('Empty range valid:', isValidRange(emptyRange));
    console.log('Empty range length:', getRangeLength(emptyRange));

    // Multi-line range
    const multilineRange: FrontendRange = { startLine: 1, startCharacter: 0, endLine: 5, endCharacter: 10 };
    console.log('Multi-line valid:', isValidRange(multilineRange));
    console.log('Multi-line length:', getRangeLength(multilineRange));

    // Invalid range
    const invalidRange: FrontendRange = { startLine: 5, startCharacter: 10, endLine: 3, endCharacter: 5 };
    console.log('Invalid range (end before start):', isValidRange(invalidRange));

    // Position containment
    const testRange: FrontendRange = {
      startLine: 5,
      startCharacter: 10,
      endLine: 8,
      endCharacter: 20,
    };
    console.log('Position in range:', positionInRange(5, 12, testRange));
    console.log('Position not in range:', positionInRange(10, 5, testRange));

    const expanded = expandRange(emptyRange, 2, 3);
    console.log('Expanded empty range:', expanded);
  }

  static runTests() {
    console.log('=== RangeConverter Test Suite ===');
    const success = RangeConverterTest.testAllConversions();
    console.log('\n=== Test Results ===');
    console.log('All tests passed:', success);
    return success;
  }
}

// Export test runner for console
(window as any).testRangeConverter = () => RangeConverterTest.runTests();