/**
 * Dependency analyzer for method refactoring operations
 */
export class DependencyAnalyzer {
  /**
   * Analyze method dependencies and determine if it can be moved
   */
  analyzeMethodDependencies(
    methodCode: string,
    targetClass: string,
  ): {
    dependencies: string[];
    isMovable: boolean;
    issues: string[];
  } {
    const dependencies: string[] = [];
    const issues: string[] = [];

    // Find field accesses
    const fieldMatches = methodCode.match(/self\.(\w+)/g);
    if (fieldMatches) {
      dependencies.push(...fieldMatches.map(match => match.replace('self.', '')));
    }

    // Find method calls on self
    const methodMatches = methodCode.match(/self\.(\w+)\(/g);
    if (methodMatches) {
      dependencies.push(...methodMatches.map(match => match.match(/\.(\w+)\(/)?.[1]).filter(Boolean) as string[]);
    }

    // More sophisticated dependency analysis
    const uniqueDependencies = [...new Set(dependencies)];
    const isMovable = this.analyzeDependencyMovable(uniqueDependencies, methodCode);

    if (uniqueDependencies.length > 10) {
      issues.push('Method has too many dependencies - moving may make code harder to maintain');
    }

    // Analyze coupling patterns that affect movability
    const couplingIssues = this.analyzeCouplingPatterns(methodCode, uniqueDependencies);
    issues.push(...couplingIssues);

    return { dependencies, isMovable, issues };
  }

  /**
   * Analyze whether a method can be moved based on its dependencies
   */
  private analyzeDependencyMovable(dependencies: string[], methodCode: string): boolean {
    // Basic heuristic: if method has too many dependencies or complex patterns, it's less movable
    if (dependencies.length > 10) {
      return false;
    }

    // Check for patterns that make methods harder to move
    const riskyPatterns = [
      // Side-effecting operations
      /\b(push|pop|insert|remove|clear|sort|reverse)\s*\(/,
      // I/O operations
      /\b(print|println|write|read)\s*!?/,
      // File system operations
      /\b(File::|std::fs::)/,
      // Network operations
      /\b(reqwest|hyper|tokio::net)/,
      // Unsafe operations
      /\bunsafe\s*\{/,
    ];

    // If method contains risky patterns, it's less movable
    for (const pattern of riskyPatterns) {
      if (pattern.test(methodCode)) {
        if (dependencies.length > 5) {
          return false;
        }
      }
    }

    // Default to movable if no issues found
    return true;
  }

  /**
   * Analyze coupling patterns that affect movability
   */
  private analyzeCouplingPatterns(methodCode: string, dependencies: string[]): string[] {
    const issues: string[] = [];

    // Check for tight coupling patterns
    const tightCouplingPatterns = [
      // Hard-coded class names or specific implementations
      /(Self|impl|trait)\s+\w+/,
      // Direct instantiation (new keyword equivalent in Rust would be struct initialization)
      /Some\(|None|Ok\(|Err\(/,
      // Singleton patterns or global state access
      /(static|const|global)/i,
      // Platform-specific code
      /(windows|unix|linux|macos)/i,
    ];

    for (const pattern of tightCouplingPatterns) {
      if (pattern.test(methodCode)) {
        issues.push('Method contains tightly coupled code that may prevent safe movement');
        break;
      }
    }

    // Check dependency complexity
    const complexDependencies = dependencies.filter(dep =>
      dep.includes('::') || dep.length > 20,
    );

    if (complexDependencies.length > 0) {
      issues.push(`Method depends on ${complexDependencies.length} complex dependencies`);
    }

    // Check for method chains that might introduce fragility
    const methodChainMatches = methodCode.match(/\.\w+\(.*?\)/g);
    if (methodChainMatches && methodChainMatches.length > 5) {
      issues.push('Method contains long method chains that may break during movement');
    }

    return issues;
  }

  /**
   * Extract dependencies from method code
   */
  extractDependencies(methodCode: string): string[] {
    const dependencies: string[] = [];

    // Find field accesses: self.field
    const fieldMatches = methodCode.match(/self\.(\w+)/g);
    if (fieldMatches) {
      dependencies.push(...fieldMatches.map(match => match.replace('self.', '')));
    }

    // Find method calls on self: self.method()
    const methodMatches = methodCode.match(/self\.(\w+)\(/g);
    if (methodMatches) {
      dependencies.push(...methodMatches.map(match => match.match(/\.(\w+)\(/)?.[1]).filter(Boolean) as string[]);
    }

    return [...new Set(dependencies)];
  }
}