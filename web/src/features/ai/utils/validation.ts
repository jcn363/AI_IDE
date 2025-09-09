import type { RefactoringContext, RefactoringType } from '../types';

/**
 * Comprehensive validation utilities for refactoring operations
 */
export class RefactoringValidationUtils {
  /**
   * Validate refactoring request with comprehensive checks
   */
  static async validateRefactoringRequest(
    type: RefactoringType,
    context: RefactoringContext,
    request: any,
  ): Promise<void> {
    // Base validation
    if (!context.filePath) {
      throw new Error('File path is required for all refactoring operations');
    }

    // Validate file path format and accessibility
    if (!context.filePath || (typeof context.filePath !== 'string')) {
      throw new Error('Valid file path string is required');
    }

    // Check if file has a valid extension for the operation
    const fileExtension = this.getFileExtension(context.filePath);
    if (fileExtension && !this.isSupportedLanguage(fileExtension, type)) {
      throw new Error(`${type} refactoring is not supported for file type: ${fileExtension}`);
    }

    // Validate selection range when required
    this.validateSelection(context, type);

    // Type-specific validation
    this.validateByType(type, context, request);
  }

  /**
   * Validate selection range when operation requires it
   */
  private static validateSelection(context: RefactoringContext, operationType: RefactoringType): void {
    const requiresSelection = this.operationRequiresSelection(operationType);

    if (requiresSelection && !context.selection) {
      throw new Error(`Selection is required for ${operationType} operation`);
    }

    if (context.selection) {
      const { start, end } = context.selection;
      if (!this.isValidRange(start, end)) {
        throw new Error('Invalid selection range provided');
      }
    }
  }

  private static operationRequiresSelection(operationType: RefactoringType): boolean {
    const selectionRequiredTypes: RefactoringType[] = [
      'extract-method',
      'extract-function',
      'extract-variable',
      'extract-interface',
      'interface-extraction',
      'pattern-conversion'
    ];
    return selectionRequiredTypes.includes(operationType);
  }

  private static isValidRange(start: any, end: any): boolean {
    return start && end &&
      typeof start.line === 'number' && typeof start.character === 'number' &&
      typeof end.line === 'number' && typeof end.character === 'number' &&
      (start.line < end.line || (start.line === end.line && start.character <= end.character));
  }

  /**
   * Type-specific validation methods
   */
  private static validateByType(
    type: RefactoringType,
    context: RefactoringContext,
    request: any
  ): void {
    switch (type) {
      case 'rename':
        this.validateRenameRequest(request);
        break;

      case 'extract-method':
      case 'extract-function':
        this.validateExtractMethodRequest(context, request);
        break;

      case 'extract-variable':
        this.validateExtractVariableRequest(context, request);
        break;

      case 'extract-interface':
        this.validateExtractInterfaceRequest(request);
        break;

      case 'move-method':
      case 'move-class':
      case 'move-file':
        this.validateMoveRequest(context, request, type);
        break;

      case 'inline-method':
      case 'inline-variable':
      case 'inline-function':
        this.validateInlineRequest(context, type);
        break;

      case 'introduce-parameter':
      case 'remove-parameter':
      case 'change-signature':
        this.validateParameterRequest(context, request, type);
        break;

      case 'replace-constructor':
      case 'replace-conditionals':
      case 'convert-method-to-function':
        this.validateReplaceRequest(context, type);
        break;

      case 'split-class':
      case 'merge-classes':
        this.validateClassOperationRequest(context, request, type);
        break;

      case 'add-delegation':
      case 'remove-delegation':
      case 'encapsulate-field':
      case 'localize-variable':
        this.validateEncapsulationRequest(context, type);
        break;

      case 'add-missing-imports':
      case 'sort-imports':
      case 'convert-to-async':
      case 'generate-getters-setters':
        this.validateUtilityRequest(type);
        break;

      case 'interface-extraction':
      case 'pattern-conversion':
      case 'batch-interface-extraction':
      case 'async-await-conversion':
      case 'batch-pattern-conversion':
      case 'async-await-pattern-conversion':
        this.validateAdvancedRequest(context, request, type);
        break;

      default:
        // Log warning for unhandled refactoring types
        console.warn(`No specific validation implemented for refactoring type: ${type}. Using basic validation only.`);
    }
  }

  /**
   * Validate rename operation request
   */
  private static validateRenameRequest(request: any): void {
    if (!request.newName) {
      throw new Error('New name is required for rename refactoring');
    }
    if (request.newName.trim().length === 0) {
      throw new Error('New name cannot be empty');
    }
    if (request.oldName === request.newName) {
      throw new Error('New name must be different from old name');
    }
    if (request.newName.includes(' ')) {
      throw new Error('Rename target cannot contain spaces');
    }
    if (!this.isValidIdentifier(request.newName)) {
      throw new Error('New name must be a valid identifier');
    }
  }

  /**
   * Validate extract method/function request
   */
  private static validateExtractMethodRequest(context: RefactoringContext, request: any): void {
    if (!context.selection) {
      throw new Error('Selection is required for extract method refactoring');
    }

    const selection = context.selection;
    const linesInSelection = selection.end.line - selection.start.line;

    if (linesInSelection < 1) {
      throw new Error('Selection must include at least one complete line for extract method');
    }

    if (!request.methodName || request.methodName.trim().length === 0) {
      throw new Error('Method name is required for extract method refactoring');
    }

    if (!this.isValidIdentifier(request.methodName)) {
      throw new Error('Method name must be a valid identifier');
    }

    // Check if selection contains incomplete statements
    if (this.containsIncompleteStatements(context, request)) {
      throw new Error('Selection contains incomplete statements that cannot be safely extracted');
    }
  }

  /**
   * Validate extract variable request
   */
  private static validateExtractVariableRequest(context: RefactoringContext, request: any): void {
    if (!context.selection) {
      throw new Error('Selection is required for extract variable refactoring');
    }

    if (!request.variableName || request.variableName.trim().length === 0) {
      throw new Error('Variable name is required for extract variable refactoring');
    }

    if (!this.isValidIdentifier(request.variableName)) {
      throw new Error('Variable name must be a valid identifier');
    }

    // Check if selection contains function calls (not always suitable for extraction)
    if (request.generateTests && this.containsSideEffects(context)) {
      throw new Error('Selection contains code with side effects. Consider creation of additional tests.');
    }
  }

  /**
   * Validate extract interface request
   */
  private static validateExtractInterfaceRequest(request: any): void {
    if (!request.interfaceName || request.interfaceName.trim().length === 0) {
      throw new Error('Interface name is required for extract interface refactoring');
    }

    if (!this.isValidIdentifier(request.interfaceName)) {
      throw new Error('Interface name must be a valid identifier');
    }

    // Check for naming conventions
    if (!request.interfaceName.endsWith('Interface') && !request.interfaceName.startsWith('I')) {
      console.warn('Interface names typically end with "Interface" or start with "I"');
    }
  }

  /**
   * Validate move operation request
   */
  private static validateMoveRequest(context: RefactoringContext, request: any, operationType: RefactoringType): void {
    if (!request.targetPath && !request.targetName) {
      throw new Error(`${operationType} requires a target location or name`);
    }

    if (request.targetPath && request.targetPath.trim().length === 0) {
      throw new Error('Target path cannot be empty');
    }

    if (request.targetName && !this.isValidIdentifier(request.targetName)) {
      throw new Error('Target name must be a valid identifier');
    }

    // Check for circular dependencies in move operations
    if (this.wouldCreateCircularDependency(context.filePath, request.targetPath)) {
      throw new Error(`Moving ${operationType} would create circular dependency`);
    }
  }

  /**
   * Validate inline operation request
   */
  private static validateInlineRequest(context: RefactoringContext, operationType: RefactoringType): void {
    if (operationType === 'inline-method' || operationType === 'inline-function') {
      // Check if method has multiple usages
      if (context.usages && context.usages.length > 1) {
        console.warn('Method is used in multiple places. Inlining may impact maintainability.');
      }
    }
  }

  /**
   * Validate parameter operation request
   */
  private static validateParameterRequest(context: RefactoringContext, request: any, operationType: RefactoringType): void {
    if (operationType === 'introduce-parameter') {
      if (!request.parameterName || !this.isValidIdentifier(request.parameterName)) {
        throw new Error('Valid parameter name is required');
      }
      if (!request.parameterType) {
        throw new Error('Parameter type is required');
      }
    }

    if (operationType === 'remove-parameter') {
      if (!request.parameterName) {
        throw new Error('Parameter to remove must be specified');
      }
    }

    if (operationType === 'change-signature') {
      if (!request.newParameters || !Array.isArray(request.newParameters)) {
        throw new Error('New parameter list must be provided');
      }
    }
  }

  /**
   * Validate replacement operations
   */
  private static validateReplaceRequest(context: RefactoringContext, operationType: RefactoringType): void {
    // These operations typically require more complex validation
    if (!context.selection && operationType !== 'replace-conditionals') {
      throw new Error(`${operationType} requires a specific target to replace`);
    }
  }

  /**
   * Validate class operations
   */
  private static validateClassOperationRequest(context: RefactoringContext, request: any, operationType: RefactoringType): void {
    if (operationType === 'split-class') {
      if (!request.classNames || !Array.isArray(request.classNames)) {
        throw new Error('Target class names are required for split operation');
      }

      if (!request.fieldsToMove || !Array.isArray(request.fieldsToMove)) {
        throw new Error('Fields to move must be specified for split operation');
      }

      if (!request.methodsToMove || !Array.isArray(request.methodsToMove)) {
        throw new Error('Methods to move must be specified for split operation');
      }
    }

    if (operationType === 'merge-classes') {
      if (!request.targetClassName) {
        throw new Error('Target class name is required for merge operation');
      }

      if (!request.sourceFiles || !Array.isArray(request.sourceFiles)) {
        throw new Error('Source files must be specified for merge operation');
      }
    }
  }

  /**
   * Validate encapsulation operations
   */
  private static validateEncapsulationRequest(context: RefactoringContext, operationType: RefactoringType): void {
    // Most encapsulation operations work on fields/variables
    if (operationType === 'encapsulate-field') {
      if (!context.symbolName) {
        throw new Error('Field name must be specified for encapsulation');
      }
    }
  }

  /**
   * Validate utility operations
   */
  private static validateUtilityRequest(operationType: RefactoringType): void {
    // Utility operations typically only need basic validation
    // Most work on the entire file or don't need special context
    if (!['add-missing-imports', 'sort-imports', 'convert-to-async', 'generate-getters-setters'].includes(operationType)) {
      throw new Error(`Unknown utility operation: ${operationType}`);
    }
  }

  /**
   * Validate advanced operations
   */
  private static validateAdvancedRequest(context: RefactoringContext, request: any, operationType: RefactoringType): void {
    if (operationType === 'interface-extraction' || operationType === 'batch-interface-extraction') {
      if (!request.methods || !Array.isArray(request.methods)) {
        throw new Error('Method list is required for interface extraction');
      }

      if (request.methods.length === 0) {
        throw new Error('At least one method must be selected for interface extraction');
      }
    }

    if (operationType.includes('async-await') && !this.containsSyncOperations(context)) {
      throw new Error('No synchronous operations found to convert to async/await');
    }
  }

  /**
   * Helper methods
   */
  private static isValidIdentifier(name: string): boolean {
    const validIdentifier = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/;
    return validIdentifier.test(name);
  }

  private static getFileExtension(filePath: string): string | null {
    const parts = filePath.split('.');
    return parts.length > 1 ? parts[parts.length - 1] : null;
  }

  private static isSupportedLanguage(extension: string, operationType: RefactoringType): boolean {
    const languageSupport = {
      'rust': ['rename', 'extract-function', 'extract-variable', 'inline-function'] as RefactoringType[],
      'ts': ['rename', 'extract-function', 'extract-variable', 'convert-to-async'] as RefactoringType[],
      'js': ['rename', 'extract-function', 'extract-variable', 'convert-to-async'] as RefactoringType[],
      'cpp': ['rename', 'extract-function'] as RefactoringType[],
      'c': ['rename', 'extract-function'] as RefactoringType[],
      'java': ['rename', 'extract-function', 'extract-variable'] as RefactoringType[],
      'python': ['rename', 'extract-function', 'extract-variable'] as RefactoringType[],
    };

    const supportedTypes = languageSupport[extension as keyof typeof languageSupport];
    return !supportedTypes || supportedTypes.includes(operationType);
  }

  private static containsIncompleteStatements(context: RefactoringContext, request: any): boolean {
    // Placeholder for complex AST analysis
    // This would check if the selected code contains incomplete statements
    return false; // Always return false for now
  }

  private static containsSideEffects(context: RefactoringContext): boolean {
    // Placeholder for analyzing side effects in selected code
    return false; // Always return false for now
  }

  private static wouldCreateCircularDependency(sourcePath: string, targetPath: string): boolean {
    // Placeholder for dependency analysis
    // This would use Cargo.toml or other project files to check for cycles
    return false; // Always return false for now
  }

  private static containsSyncOperations(context: RefactoringContext): boolean {
    // Placeholder for detecting synchronous operations that can be made async
    return true; // Always return true for now
  }
}