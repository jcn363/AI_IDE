// Advanced Search and Navigation Types

export interface SearchResult {
  id: string;
  file_path: string;
  line_number: number;
  column_start: number;
  column_end: number;
  content: string;
  match_type: 'text' | 'regex' | 'symbol' | 'definition';
  score: number;
  context_before: string[];
  context_after: string[];
}

export interface SearchResponse {
  results: SearchResult[];
  total_count: number;
  search_time_ms: number;
  has_more?: boolean;
}

export interface SearchOptions {
  query: string;
  case_sensitive: boolean;
  whole_word: boolean;
  regex: bool;
  include_hidden: boolean;
  include_binary: boolean;
  file_patterns: string[];
  exclude_patterns: string[];
  max_results?: number;
  context_lines: number;
}

export interface SymbolInfo {
  name: string;
  kind: 'function' | 'struct' | 'enum' | 'trait' | 'module' | 'constant' | 'field' | 'method' | 'type' | 'macro' | 'variable';
  location: {
    file_path: string;
    line: number;
    column: number;
  };
  container_name?: string;
  documentation?: string;
  scope?: 'public' | 'private' | 'protected';
}

export interface SymbolSearchResult {
  symbols: SymbolInfo[];
  total_count: number;
  search_time_ms: number;
}

export interface NavigationLocation {
  file_path: string;
  line_number: number;
  column: number;
  context: string;
  timestamp: number;
  description?: string;
}

export interface BreadcrumbItem {
  name: string;
  kind: 'file' | 'directory' | 'function' | 'struct' | 'enum' | 'trait' | 'module' | 'class' | 'method';
  location: NavigationLocation;
}

export interface NavigationPath {
  path: string;
  parts: BreadcrumbItem[];
}

export interface SearchHistoryItem {
  query: string;
  timestamp: number;
  result_count: number;
}

export interface SearchState {
  current_query: string;
  results: SearchResult[];
  loading: boolean;
  error?: string;
  history: SearchHistoryItem[];
  active_result_index: number;
}

export interface SymbolNavigationState {
  symbols: SymbolInfo[];
  loading: boolean;
  error?: string;
  selected_symbol?: SymbolInfo;
  search_query: string;
}

export interface NavigationState {
  current_location?: NavigationLocation;
  history: NavigationLocation[];
  bookmarks: NavigationLocation[];
  breadcrumbs: BreadcrumbItem[];
}

export interface SearchFilters {
  file_types: string[];
  time_range?: {
    from: number;
    to: number;
  };
  size_range?: {
    min?: number;
    max?: number;
  };
}

// Search command request/response types
export interface SearchFilesRequest {
  search_options: SearchOptions;
  workspace_path: string;
}

export interface SearchSymbolsRequest {
  query: string;
  workspace_path: string;
}

export interface NavigateToSymbolRequest {
  symbol_name: string;
}

export interface GetBreadcrumbsRequest {
  file_path: string;
  line: number;
  column: number;
}

export interface GoToDefinitionRequest {
  file_path: string;
  line: number;
  column: number;
}

export interface FindReferencesRequest {
  file_path: string;
  line: number;
  column: number;
}

// Event types for search communication
export interface SearchEvents {
  'search:started': SearchFilesRequest;
  'search:completed': SearchResponse;
  'search:error': { error: string };
  'navigation:location-changed': NavigationLocation;
  'navigation:bookmarked': NavigationLocation;
  'symbol:selected': SymbolInfo;
}

// Search configuration
export interface SearchConfig {
  max_history_size: number;
  max_preview_lines: number;
  highlight_colors: {
    match: string;
    context: string;
    selected: string;
  };
  keyboard_shortcuts: {
    next_result: string;
    prev_result: string;
    toggle_case: string;
    toggle_regex: string;
    focus: string;
  };
}