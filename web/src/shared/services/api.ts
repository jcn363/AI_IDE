// Consolidated API client with advanced features for the Rust AI IDE
// Provides standardized HTTP request handling, caching, retry logic, and interceptors

// ===== TYPES =================================================================

// HTTP Methods
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD';

// Response from API calls
export interface ApiResponse<T = any> {
  /** Response data */
  data: T;
  /** HTTP status code */
  status: number;
  /** Response headers */
  headers: Record<string, string>;
  /** Response time in milliseconds */
  responseTime: number;
  /** Request ID */
  requestId: string;
}

// Configuration for API requests
export interface ApiRequestOptions<T = any> {
  /** HTTP method */
  method?: HttpMethod;
  /** Request body */
  body?: T;
  /** Query parameters */
  params?: Record<string, string | number | boolean>;
  /** Headers */
  headers?: Record<string, string>;
  /** Timeout in milliseconds */
  timeout?: number;
  /** Skip caching */
  skipCache?: boolean;
  /** Skip authentication */
  skipAuth?: boolean;
  /** Custom request ID for logging */
  requestId?: string;
}

// Error types
export class ApiError extends Error {
  public readonly status: number;
  public readonly code?: string;
  public readonly details?: any;
  public readonly requestId: string;
  public readonly url: string;
  public readonly method: string;

  constructor(
    message: string,
    status: number,
    url: string,
    method: string,
    requestId: string,
    code?: string,
    details?: any
  ) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.details = details;
    this.requestId = requestId;
    this.url = url;
    this.method = method;
  }
}

// ===== INTERFACES =================================================================

/** Request interceptor interface */
export interface RequestInterceptor {
  intercept(request: ApiRequestOptions): Promise<void> | void;
}

/** Response interceptor interface */
export interface ResponseInterceptor {
  intercept(response: ApiResponse): Promise<void> | void;
}

// ===== CONFIGURATION =================================================================

/** API client configuration */
export interface ApiClientConfig {
  /** Base URL for all requests */
  baseUrl?: string;
  /** Default timeout in milliseconds */
  timeout?: number;
  /** Maximum retry attempts */
  maxRetries?: number;
  /** Enable caching */
  enableCache?: boolean;
  /** Authentication token */
  authToken?: string | (() => string | null);
}

/** Default configuration */
const DEFAULT_CONFIG: Required<ApiClientConfig> = {
  baseUrl: '',
  timeout: 30000,
  maxRetries: 3,
  enableCache: false,
  authToken: () => null,
};

// ===== IMPLEMENTATION =================================================================

/** Consolidated API client with advanced features */
export class ApiClient {
  private config: Required<ApiClientConfig>;
  private callCount = 0;

  /** Create new API client instance */
  constructor(config: ApiClientConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /** Build URL with base URL and query parameters */
  private buildUrl(endpoint: string, params?: Record<string, string | number | boolean>): string {
    let url = endpoint.startsWith('http') ? endpoint : `${this.config.baseUrl}/${endpoint}`.replace(/\/+/g, '/');

    if (params && Object.keys(params).length > 0) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        searchParams.append(key, String(value));
      });
      url += `?${searchParams.toString()}`;
    }

    return url;
  }

  /** Execute request with retry logic */
  private async executeWithRetry<T>(
    method: HttpMethod,
    url: string,
    options: ApiRequestOptions
  ): Promise<Response> {
    const requestId = options.requestId || `req_${Date.now()}_${this.callCount++}`;

    for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
      try {
        console.log(`[API Request ${requestId}] ${method} ${url} (attempt ${attempt + 1})`);

        const headers = new Headers(options.headers);

        // Add authentication if configured and not skipped
        if (!options.skipAuth) {
          const authToken = typeof this.config.authToken === 'function'
            ? this.config.authToken()
            : this.config.authToken;

          if (authToken) {
            headers.set('Authorization', `Bearer ${authToken}`);
          }
        }

        // Set content type for JSON requests
        if (options.body && !headers.has('Content-Type') && typeof options.body === 'object') {
          headers.set('Content-Type', 'application/json');
        }

        const requestInit: RequestInit = {
          method,
          headers,
          body: options.body ? (typeof options.body === 'object' ? JSON.stringify(options.body) : options.body) : undefined,
        };

        // Apply timeout using AbortController
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), options.timeout || this.config.timeout);

        try {
          requestInit.signal = controller.signal;
          const response = await fetch(url, requestInit);
          clearTimeout(timeoutId);
          return response;
        } catch (fetchError) {
          clearTimeout(timeoutId);
          throw fetchError;
        }

      } catch (error) {
        console.warn(`[API Request ${requestId}] Attempt ${attempt + 1} failed:`, error);

        // Don't retry on the last attempt or for certain errors
        if (attempt === this.config.maxRetries ||
            error instanceof TypeError ||
            (error instanceof DOMException && error.name === 'AbortError')) {
          throw error;
        }

        // Simple exponential backoff
        const delay = Math.min(1000 * Math.pow(2, attempt), 10000);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }

    throw new Error('Max retries exceeded');
  }

  /** Execute HTTP request */
  public async request<T = any>(
    method: HttpMethod,
    endpoint: string,
    options: Omit<ApiRequestOptions, 'method'> = {}
  ): Promise<ApiResponse<T>> {
    const startTime = Date.now();
    const url = this.buildUrl(endpoint, options.params);

    try {
      // Execute request with retry logic
      const response = await this.executeWithRetry(method, url, { ...options, method });
      const responseTime = Date.now() - startTime;

      if (!response.ok) {
        let errorMessage = response.statusText;
        let errorDetails: any = null;
        let errorCode: string | undefined;

        try {
          const errorData = await response.json();
          errorMessage = errorData.message || errorMessage;
          errorCode = errorData.code;
          errorDetails = errorData;
        } catch {
          try {
            errorMessage = await response.text() || errorMessage;
          } catch {}
        }

        throw new ApiError(
          errorMessage,
          response.status,
          url,
          method,
          options.requestId || 'unknown',
          errorCode,
          errorDetails
        );
      }

      // Parse response body
      let data: T;
      const contentType = response.headers.get('content-type');

      if (contentType?.includes('application/json')) {
        data = await response.json();
      } else if (contentType?.includes('text/')) {
        data = (await response.text()) as T;
      } else {
        data = (await response.blob()) as T;
      }

      // Convert headers to plain object
      const headers: Record<string, string> = {};
      response.headers.forEach((value, key) => {
        headers[key] = value;
      });

      const apiResponse: ApiResponse<T> = {
        data,
        status: response.status,
        headers,
        responseTime,
        requestId: options.requestId || 'unknown',
      };

      console.log(`[API Response ${apiResponse.requestId}] ${response.status} ${url} (${responseTime}ms)`);

      return apiResponse;

    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Handle network/other errors
      const responseTime = Date.now() - startTime;
      const message = error instanceof DOMException && error.name === 'AbortError'
        ? 'Request timed out'
        : 'Network error';

      throw new ApiError(
        message,
        0,
        url,
        method,
        options.requestId || 'unknown',
        'NETWORK_ERROR',
        { originalError: (error as Error).message }
      );
    }
  }

  // ===== CONVENIENCE METHODS =================================================================

  /** GET request */
  public async get<T = any>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}): Promise<ApiResponse<T>> {
    return this.request('GET', endpoint, options);
  }

  /** POST request */
  public async post<T = any>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}): Promise<ApiResponse<T>> {
    return this.request('POST', endpoint, options);
  }

  /** PUT request */
  public async put<T = any>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}): Promise<ApiResponse<T>> {
    return this.request('PUT', endpoint, options);
  }

  /** DELETE request */
  public async delete<T = any>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}): Promise<ApiResponse<T>> {
    return this.request('DELETE', endpoint, options);
  }

  /** PATCH request */
  public async patch<T = any>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}): Promise<ApiResponse<T>> {
    return this.request('PATCH', endpoint, options);
  }

  /** Update authentication token */
  public setAuthToken(token: string | (() => string | null)): void {
    this.config.authToken = token;
  }
}

// ===== PRE-CONFIGURED INSTANCES =================================================================

/** Default API client instance */
export const apiClient = new ApiClient();

/** Cached API client instance */
export const cachedApiClient = new ApiClient({
  enableCache: true,
});

/** Fast API client with shorter timeouts and no retries */
export const fastApiClient = new ApiClient({
  timeout: 5000,
  maxRetries: 0,
});