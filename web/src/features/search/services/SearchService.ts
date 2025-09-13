import { invoke } from '@tauri-apps/api/core';
import {
  SearchOptions,
  SearchResult,
  SymbolInfo,
  SymbolSearchResult,
  NavigationLocation,
  NavigationPath,
  SearchFilesRequest,
  SearchSymbolsRequest,
  NavigateToSymbolRequest,
  GetBreadcrumbsRequest,
  GoToDefinitionRequest,
  FindReferencesRequest,
} from '../types';

/**
 * Advanced Search and Navigation Service
 * Provides comprehensive search and navigation functionality integrated with backend
 */
export class SearchService {
  private static instance: SearchService;

  private constructor() {}

  public static getInstance(): SearchService {
    if (!SearchService.instance) {
      SearchService.instance = new SearchService();
    }
    return SearchService.instance;
  }

  /**
   * Perform advanced text search across files
   */
  async searchFiles(
    searchOptions: SearchOptions,
    workspacePath: string
  ): Promise<{ results: SearchResult[]; total_count: number }> {
    try {
      const request: SearchFilesRequest = {
        search_options: searchOptions,
        workspace_path: workspacePath,
      };

      const response = (await invoke('search_files', { request })) as any;

      if (response.results && Array.isArray(response.results)) {
        return {
          results: response.results,
          total_count: response.results.length,
        };
      }

      return { results: [], total_count: 0 };
    } catch (error) {
      console.error('Search failed:', error);
      throw new Error(`Search failed: ${error}`);
    }
  }

  /**
   * Search for symbols across the workspace
   */
  async searchSymbols(query: string, workspacePath: string): Promise<SymbolSearchResult> {
    try {
      const request: SearchSymbolsRequest = {
        query,
        workspace_path: workspacePath,
      };

      const response = (await invoke('search_symbols', { request })) as any;

      return {
        symbols: response.symbols || [],
        total_count: response.total_count || 0,
        search_time_ms: response.search_time_ms || 0,
      };
    } catch (error) {
      console.error('Symbol search failed:', error);
      throw new Error(`Symbol search failed: ${error}`);
    }
  }

  /**
   * Navigate to a specific symbol
   */
  async navigateToSymbol(symbolName: string): Promise<NavigationLocation> {
    try {
      const request: NavigateToSymbolRequest = {
        symbol_name: symbolName,
      };

      const response = (await invoke('navigate_to_symbol', { request })) as any;

      return {
        file_path: response.file_path,
        line_number: response.line_number,
        column: response.column,
        context: response.context,
        timestamp: response.timestamp,
      };
    } catch (error) {
      console.error('Navigate to symbol failed:', error);
      throw new Error(`Navigate to symbol failed: ${error}`);
    }
  }

  /**
   * Get breadcrumb navigation for a file location
   */
  async getBreadcrumbs(filePath: string, line: number, column: number): Promise<NavigationPath> {
    try {
      const request: GetBreadcrumbsRequest = {
        file_path: filePath,
        line,
        column,
      };

      const response = (await invoke('get_breadcrumbs', { request })) as any;

      return {
        path: response.path,
        parts: response.parts || [],
      };
    } catch (error) {
      console.error('Get breadcrumbs failed:', error);
      throw new Error(`Get breadcrumbs failed: ${error}`);
    }
  }

  /**
   * Go to definition of symbol at location
   */
  async goToDefinition(
    filePath: string,
    line: number,
    column: number
  ): Promise<NavigationLocation> {
    try {
      const request: GoToDefinitionRequest = {
        file_path: filePath,
        line,
        column,
      };

      const response = (await invoke('go_to_definition', { request })) as any;

      return {
        file_path: response.file_path,
        line_number: response.line_number,
        column: response.column,
        context: response.context,
        timestamp: response.timestamp,
      };
    } catch (error) {
      console.error('Go to definition failed:', error);
      throw new Error(`Go to definition failed: ${error}`);
    }
  }

  /**
   * Find all references to symbol at location
   */
  async findReferences(
    filePath: string,
    line: number,
    column: number
  ): Promise<NavigationLocation[]> {
    try {
      const request: FindReferencesRequest = {
        file_path: filePath,
        line,
        column,
      };

      const response = (await invoke('find_references', { request })) as NavigationLocation[];

      return response;
    } catch (error) {
      console.error('Find references failed:', error);
      throw new Error(`Find references failed: ${error}`);
    }
  }

  /**
   * Get navigation history
   */
  async getNavigationHistory(): Promise<NavigationLocation[]> {
    try {
      const response = (await invoke('get_navigation_history')) as NavigationLocation[];
      return response;
    } catch (error) {
      console.error('Get navigation history failed:', error);
      throw new Error(`Get navigation history failed: ${error}`);
    }
  }

  /**
   * Get search history
   */
  async getSearchHistory(): Promise<string[]> {
    try {
      const response = (await invoke('get_search_history')) as string[];
      return response;
    } catch (error) {
      console.error('Get search history failed:', error);
      throw new Error(`Get search history failed: ${error}`);
    }
  }

  /**
   * Validate search query and options
   */
  validateSearchOptions(options: SearchOptions): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!options.query.trim()) {
      errors.push('Search query cannot be empty');
    }

    if (options.max_results && options.max_results < 0) {
      errors.push('Max results cannot be negative');
    }

    if (options.context_lines < 0) {
      errors.push('Context lines cannot be negative');
    }

    if (options.regex) {
      try {
        new RegExp(options.query);
      } catch {
        errors.push('Invalid regular expression');
      }
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Get symbol kind icon
   */
  static getSymbolIcon(kind: string): string {
    const iconMap: Record<string, string> = {
      function: '📄',
      struct: '🏗️',
      enum: '📋',
      trait: '🔧',
      module: '📦',
      constant: '🔢',
      field: '📍',
      method: '⚙️',
      type: '🏷️',
      macro: '🎭',
      variable: '💾',
      class: '🏛️',
    };

    return iconMap[kind.toLowerCase()] || '🔍';
  }

  /**
   * Format file path for display
   */
  static formatFilePath(filePath: string, workspacePath: string): string {
    if (filePath.startsWith(workspacePath)) {
      return filePath.substring(workspacePath.length + 1);
    }
    return filePath;
  }

  /**
   * Highlight search matches in text
   */
  static highlightMatches(
    text: string,
    query: string,
    caseSensitive: boolean = false,
    regex: boolean = false
  ): { text: string; highlights: { start: number; end: number }[] } {
    const highlights: { start: number; end: number }[] = [];

    if (!query.trim()) {
      return { text, highlights: [] };
    }

    const flags = caseSensitive ? 'g' : 'gi';
    let pattern: RegExp;

    if (regex) {
      try {
        pattern = new RegExp(query, flags);
      } catch {
        return { text, highlights: [] };
      }
    } else {
      pattern = new RegExp(this.escapeRegExp(query), flags);
    }

    let match;
    while ((match = pattern.exec(text)) !== null) {
      highlights.push({
        start: match.index,
        end: match.index + match[0].length,
      });
    }

    return { text, highlights };
  }

  /**
   * Escape special regex characters
   */
  private static escapeRegExp(text: string): string {
    return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }
}
