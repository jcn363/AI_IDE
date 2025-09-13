import { invoke } from '@tauri-apps/api/core';

/**
 * WebAuthn Service Layer - provides typed interface to Tauri WebAuthn commands
 * Handles passwordless authentication using FIDO2/WebAuthn standard
 */

// Type definitions for WebAuthn operations

export interface WebAuthnCredential {
  credential_id: string;
  user_id: string;
  created_at: string;
  last_used_at: string;
  counter: number;
  device_info: Record<string, string>;
}

export interface StartRegistrationRequest {
  user_display_name: string;
  user_name: string;
}

export interface FinishRegistrationRequest {
  challenge_id: string;
  registration_response: any; // WebAuthn registration response
}

export interface StartAuthenticationRequest {
  user_id: string;
}

export interface FinishAuthenticationRequest {
  challenge_id: string;
  authentication_response: any; // WebAuthn authentication response
}

export interface DeleteCredentialRequest {
  credential_id: string;
}

export interface WebAuthnChallengeResponse {
  challenge_id: string;
  challenge: any; // WebAuthn challenge data
}

export interface WebAuthnResult {
  success: boolean;
  credential_id?: string;
  user_id?: string;
  authenticated?: boolean;
  credentials?: WebAuthnCredential[];
  status?: string;
  error?: string;
}

/**
 * WebAuthn Service - handles passwordless authentication operations
 */
class WebAuthnService {
  /**
   * Start WebAuthn registration ceremony
   * Initiates the process of registering a new WebAuthn credential
   */
  async startRegistration(request: StartRegistrationRequest): Promise<WebAuthnChallengeResponse> {
    try {
      console.log('Starting WebAuthn registration for user:', request.user_name);
      const result = await invoke<WebAuthnChallengeResponse>('webauthn_start_registration', {
        input: request
      });
      console.log('WebAuthn registration challenge created');
      return result;
    } catch (error) {
      console.error('WebAuthn registration start failed:', error);
      throw new Error(`Failed to start WebAuthn registration: ${error}`);
    }
  }

  /**
   * Finish WebAuthn registration ceremony
   * Completes the registration by verifying the authenticator's response
   */
  async finishRegistration(request: FinishRegistrationRequest): Promise<WebAuthnResult> {
    try {
      console.log('Finishing WebAuthn registration for challenge:', request.challenge_id);
      const result = await invoke<WebAuthnResult>('webauthn_finish_registration', {
        input: request
      });
      console.log('WebAuthn registration completed successfully');
      return result;
    } catch (error) {
      console.error('WebAuthn registration finish failed:', error);
      throw new Error(`Failed to finish WebAuthn registration: ${error}`);
    }
  }

  /**
   * Start WebAuthn authentication ceremony
   * Initiates the process of authenticating with an existing WebAuthn credential
   */
  async startAuthentication(request: StartAuthenticationRequest): Promise<WebAuthnChallengeResponse> {
    try {
      console.log('Starting WebAuthn authentication for user:', request.user_id);
      const result = await invoke<WebAuthnChallengeResponse>('webauthn_start_authentication', {
        input: request
      });
      console.log('WebAuthn authentication challenge created');
      return result;
    } catch (error) {
      console.error('WebAuthn authentication start failed:', error);
      throw new Error(`Failed to start WebAuthn authentication: ${error}`);
    }
  }

  /**
   * Finish WebAuthn authentication ceremony
   * Completes the authentication by verifying the authenticator's response
   */
  async finishAuthentication(request: FinishAuthenticationRequest): Promise<WebAuthnResult> {
    try {
      console.log('Finishing WebAuthn authentication for challenge:', request.challenge_id);
      const result = await invoke<WebAuthnResult>('webauthn_finish_authentication', {
        input: request
      });
      console.log('WebAuthn authentication completed successfully');
      return result;
    } catch (error) {
      console.error('WebAuthn authentication finish failed:', error);
      throw new Error(`Failed to finish WebAuthn authentication: ${error}`);
    }
  }

  /**
   * List user's WebAuthn credentials
   * Returns all registered WebAuthn credentials for the current user
   */
  async listCredentials(): Promise<WebAuthnCredential[]> {
    try {
      console.log('Listing WebAuthn credentials');
      const result = await invoke<WebAuthnResult>('webauthn_list_credentials', {});
      console.log(`Found ${result.credentials?.length || 0} WebAuthn credentials`);
      return result.credentials || [];
    } catch (error) {
      console.error('Failed to list WebAuthn credentials:', error);
      throw new Error(`Failed to list WebAuthn credentials: ${error}`);
    }
  }

  /**
   * Delete a WebAuthn credential
   * Removes a specific WebAuthn credential from the user's account
   */
  async deleteCredential(request: DeleteCredentialRequest): Promise<WebAuthnResult> {
    try {
      console.log('Deleting WebAuthn credential:', request.credential_id);
      const result = await invoke<WebAuthnResult>('webauthn_delete_credential', {
        input: request
      });
      console.log('WebAuthn credential deleted successfully');
      return result;
    } catch (error) {
      console.error('Failed to delete WebAuthn credential:', error);
      throw new Error(`Failed to delete WebAuthn credential: ${error}`);
    }
  }

  /**
   * Get WebAuthn service status
   * Returns the current health status of the WebAuthn service
   */
  async getStatus(): Promise<{ status: string; service: string }> {
    try {
      console.log('Checking WebAuthn service status');
      const result = await invoke<{ status: string; service: string }>('webauthn_get_status', {});
      console.log('WebAuthn service status:', result.status);
      return result;
    } catch (error) {
      console.error('Failed to get WebAuthn service status:', error);
      throw new Error(`Failed to get WebAuthn service status: ${error}`);
    }
  }

  /**
   * Clean up expired WebAuthn challenges
   * Removes expired registration and authentication challenges
   */
  async cleanupExpiredChallenges(): Promise<{ cleaned_challenges: number; status: string }> {
    try {
      console.log('Cleaning up expired WebAuthn challenges');
      const result = await invoke<{ cleaned_challenges: number; status: string }>('webauthn_cleanup_expired_challenges', {});
      console.log(`Cleaned up ${result.cleaned_challenges} expired challenges`);
      return result;
    } catch (error) {
      console.error('Failed to cleanup expired challenges:', error);
      throw new Error(`Failed to cleanup expired challenges: ${error}`);
    }
  }

  /**
   * Check if WebAuthn is supported in the current browser/environment
   */
  async isSupported(): Promise<boolean> {
    try {
      // Check if WebAuthn is available
      if (typeof window !== 'undefined' && window.PublicKeyCredential) {
        // Check if the current context supports WebAuthn
        const available = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
        console.log('WebAuthn support detected:', available);
        return available;
      }
      return false;
    } catch (error) {
      console.warn('WebAuthn support check failed:', error);
      return false;
    }
  }

  /**
   * Get available authenticators
   */
  async getAvailableAuthenticators(): Promise<string[]> {
    const authenticators = [];

    try {
      // Check for platform authenticator (Windows Hello, Touch ID, etc.)
      if (typeof window !== 'undefined' && window.PublicKeyCredential) {
        const platformAvailable = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
        if (platformAvailable) {
          authenticators.push('platform'); // Windows Hello, Touch ID, Face ID
        }
      }

      // Assume cross-platform authenticators are available (YubiKey, etc.)
      authenticators.push('cross-platform');

      console.log('Available WebAuthn authenticators:', authenticators);
      return authenticators;
    } catch (error) {
      console.warn('Failed to detect available authenticators:', error);
      return ['cross-platform']; // Fallback
    }
  }
}

// Singleton instance
export const webauthnService = new WebAuthnService();

// Default export for convenience
export default webauthnService;

/**
 * WebAuthn utility functions for React integration
 */

/**
 * Wrap WebAuthn API calls with consistent error handling
 */
export async function webauthnApiCall<T>(
  operation: () => Promise<T>,
  onSuccess?: (result: T) => void,
  onError?: (error: Error) => void
): Promise<T> {
  try {
    const result = await operation();
    onSuccess?.(result);
    return result;
  } catch (error) {
    console.error('WebAuthn API call failed:', error);
    onError?.(error as Error);
    throw error;
  }
}

/**
 * Create a standardized WebAuthn result
 */
export function createWebAuthnResult(
  success: boolean,
  data?: any,
  error?: string
): WebAuthnResult {
  return {
    success,
    ...data,
    error,
  };
}

/**
 * Browser WebAuthn API helpers (for frontend WebAuthn operations)
 */

export interface WebAuthnBrowserSupport {
  supported: boolean;
  platformAuthenticator: boolean;
  crossPlatformAuthenticator: boolean;
}

/**
 * Check browser WebAuthn support
 */
export async function checkWebAuthnSupport(): Promise<WebAuthnBrowserSupport> {
  const support: WebAuthnBrowserSupport = {
    supported: false,
    platformAuthenticator: false,
    crossPlatformAuthenticator: false,
  };

  try {
    if (typeof window !== 'undefined' && window.PublicKeyCredential) {
      support.supported = true;

      // Check platform authenticator
      support.platformAuthenticator = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();

      // Cross-platform is always assumed available if WebAuthn is supported
      support.crossPlatformAuthenticator = true;
    }
  } catch (error) {
    console.warn('WebAuthn support check failed:', error);
  }

  return support;
}

/**
 * Convert base64url to Uint8Array
 */
export function base64UrlToUint8Array(base64Url: string): Uint8Array {
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Convert Uint8Array to base64url
 */
export function uint8ArrayToBase64Url(uint8Array: Uint8Array): string {
  const binary = String.fromCharCode(...uint8Array);
  const base64 = btoa(binary);
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}