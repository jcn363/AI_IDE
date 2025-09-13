import React, { useState, useEffect } from 'react';

import webauthnService, {
  WebAuthnCredential,
  WebAuthnChallengeResponse,
  WebAuthnResult,
  webauthnApiCall,
  checkWebAuthnSupport,
  base64UrlToUint8Array,
  uint8ArrayToBase64Url
} from '../../services/webauthnService';

interface WebAuthnAuthenticationProps {
  userId: string;
  onSuccess?: (userId: string) => void;
  onError?: (error: string) => void;
  autoStart?: boolean;
}

interface AuthenticationState {
  step: 'idle' | 'checking-support' | 'starting' | 'authenticating' | 'verifying' | 'completed' | 'error';
  challenge?: WebAuthnChallengeResponse;
  authenticatedUser?: string;
  error?: string;
  supported: boolean;
  credentials: WebAuthnCredential[];
}

export const WebAuthnAuthentication: React.FC<WebAuthnAuthenticationProps> = ({
  userId,
  onSuccess,
  onError,
  autoStart = false
}) => {
  const [state, setState] = useState<AuthenticationState>({
    step: 'idle',
    supported: false,
    credentials: []
  });

  // Check WebAuthn support and load credentials on mount
  useEffect(() => {
    const initialize = async () => {
      setState(prev => ({ ...prev, step: 'checking-support' }));

      try {
        const [support, credentials] = await Promise.all([
          checkWebAuthnSupport(),
          webauthnApiCall(() => webauthnService.listCredentials())
        ]);

        setState(prev => ({
          ...prev,
          step: 'idle',
          supported: support.supported,
          credentials
        }));

        if (autoStart && support.supported && credentials.length > 0) {
          startAuthentication();
        }
      } catch (error) {
        console.error('Failed to initialize WebAuthn:', error);
        setState(prev => ({
          ...prev,
          step: 'error',
          error: 'Failed to initialize WebAuthn',
          supported: false,
          credentials: []
        }));
      }
    };

    initialize();
  }, [userId, autoStart]);

  const startAuthentication = async () => {
    if (!state.supported) {
      const error = 'WebAuthn is not supported in this browser';
      setState(prev => ({ ...prev, step: 'error', error }));
      onError?.(error);
      return;
    }

    if (state.credentials.length === 0) {
      const error = 'No WebAuthn credentials found. Please register a credential first.';
      setState(prev => ({ ...prev, step: 'error', error }));
      onError?.(error);
      return;
    }

    setState(prev => ({ ...prev, step: 'starting' }));

    try {
      const challenge = await webauthnApiCall(
        () => webauthnService.startAuthentication({ user_id: userId })
      );

      setState(prev => ({
        ...prev,
        step: 'authenticating',
        challenge
      }));

      // Start the browser WebAuthn authentication
      await performBrowserAuthentication(challenge);

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Authentication failed';
      setState(prev => ({ ...prev, step: 'error', error: errorMsg }));
      onError?.(errorMsg);
    }
  };

  const performBrowserAuthentication = async (challenge: WebAuthnChallengeResponse) => {
    try {
      // Convert challenge data for browser WebAuthn API
      const publicKeyCredentialRequestOptions = {
        challenge: base64UrlToUint8Array(challenge.challenge.publicKey.challenge),
        rpId: challenge.challenge.publicKey.rpId,
        allowCredentials: challenge.challenge.publicKey.allowCredentials?.map(cred => ({
          id: base64UrlToUint8Array(cred.id),
          type: cred.type,
          transports: cred.transports
        })),
        timeout: challenge.challenge.publicKey.timeout,
        userVerification: challenge.challenge.publicKey.userVerification
      };

      setState(prev => ({ ...prev, step: 'authenticating' }));

      // Get the credential
      const credential = await navigator.credentials.get({
        publicKey: publicKeyCredentialRequestOptions
      }) as PublicKeyCredential;

      setState(prev => ({ ...prev, step: 'verifying' }));

      // Convert credential for backend
      const authenticationResponse = {
        id: credential.id,
        rawId: uint8ArrayToBase64Url(new Uint8Array(credential.rawId)),
        type: credential.type,
        response: {
          clientDataJSON: uint8ArrayToBase64Url(new Uint8Array(credential.response.clientDataJSON)),
          authenticatorData: uint8ArrayToBase64Url(new Uint8Array((credential.response as AuthenticatorAssertionResponse).authenticatorData)),
          signature: uint8ArrayToBase64Url(new Uint8Array((credential.response as AuthenticatorAssertionResponse).signature)),
          userHandle: (credential.response as AuthenticatorAssertionResponse).userHandle ?
            uint8ArrayToBase64Url(new Uint8Array((credential.response as AuthenticatorAssertionResponse).userHandle!)) : null
        }
      };

      // Finish authentication with backend
      const result = await webauthnApiCall(
        () => webauthnService.finishAuthentication({
          challenge_id: challenge.challenge_id,
          authentication_response: authenticationResponse
        })
      );

      if (result.success && result.authenticated && result.user_id) {
        setState(prev => ({
          ...prev,
          step: 'completed',
          authenticatedUser: result.user_id
        }));
        onSuccess?.(result.user_id);
      } else {
        throw new Error(result.error || 'Authentication verification failed');
      }

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Browser authentication failed';
      setState(prev => ({ ...prev, step: 'error', error: errorMsg }));
      onError?.(errorMsg);
    }
  };

  const resetAuthentication = () => {
    setState(prev => ({
      ...prev,
      step: 'idle',
      challenge: undefined,
      authenticatedUser: undefined,
      error: undefined
    }));
  };

  const renderContent = () => {
    switch (state.step) {
      case 'checking-support':
        return (
          <div className="webauthn-authentication">
            <div className="status-message">
              <div className="spinner"></div>
              Checking WebAuthn support...
            </div>
          </div>
        );

      case 'idle':
        return (
          <div className="webauthn-authentication">
            <div className="authentication-header">
              <h3>Authenticate with WebAuthn</h3>
              <p>Use your registered biometric or security key to sign in.</p>
            </div>

            {!state.supported && (
              <div className="warning-message">
                WebAuthn is not supported in this browser. Please use a modern browser with WebAuthn support.
              </div>
            )}

            {state.credentials.length === 0 && (
              <div className="info-message">
                No WebAuthn credentials registered. Please register a credential first.
              </div>
            )}

            <div className="credential-list">
              <h4>Available Credentials</h4>
              {state.credentials.length > 0 ? (
                <ul>
                  {state.credentials.map((cred) => (
                    <li key={cred.credential_id}>
                      <div className="credential-item">
                        <span className="credential-id">
                          {cred.credential_id.substring(0, 8)}...
                        </span>
                        <span className="credential-date">
                          Created: {new Date(cred.created_at).toLocaleDateString()}
                        </span>
                        <span className="credential-device">
                          {cred.device_info?.type || 'Unknown device'}
                        </span>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No credentials available</p>
              )}
            </div>

            <div className="authentication-actions">
              <button
                onClick={startAuthentication}
                disabled={!state.supported || state.credentials.length === 0}
                className="btn-primary"
              >
                Authenticate
              </button>
            </div>
          </div>
        );

      case 'starting':
        return (
          <div className="webauthn-authentication">
            <div className="status-message">
              <div className="spinner"></div>
              Preparing authentication challenge...
            </div>
          </div>
        );

      case 'authenticating':
        return (
          <div className="webauthn-authentication">
            <div className="status-message">
              <div className="spinner"></div>
              Follow the instructions on your device to complete authentication...
            </div>
            <div className="authenticator-instructions">
              <ul>
                <li>For biometric: Use fingerprint, face, or voice recognition</li>
                <li>For security key: Insert and tap your security key</li>
                <li>For platform: Use Windows Hello, Touch ID, or similar</li>
              </ul>
            </div>
          </div>
        );

      case 'verifying':
        return (
          <div className="webauthn-authentication">
            <div className="status-message">
              <div className="spinner"></div>
              Verifying authentication...
            </div>
          </div>
        );

      case 'completed':
        return (
          <div className="webauthn-authentication">
            <div className="success-message">
              <div className="success-icon">✓</div>
              <h3>Authentication Successful!</h3>
              <p>You have been successfully authenticated.</p>
              {state.authenticatedUser && (
                <div className="user-info">
                  <p><strong>User:</strong> {state.authenticatedUser}</p>
                </div>
              )}
            </div>
            <div className="authentication-actions">
              <button onClick={resetAuthentication} className="btn-secondary">
                Authenticate Again
              </button>
            </div>
          </div>
        );

      case 'error':
        return (
          <div className="webauthn-authentication">
            <div className="error-message">
              <div className="error-icon">✕</div>
              <h3>Authentication Failed</h3>
              <p>{state.error}</p>
            </div>
            <div className="authentication-actions">
              <button onClick={startAuthentication} className="btn-primary">
                Try Again
              </button>
              <button onClick={resetAuthentication} className="btn-secondary">
                Cancel
              </button>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="webauthn-authentication-container">
      {renderContent()}

      <style jsx>{`
        .webauthn-authentication-container {
          max-width: 500px;
          margin: 0 auto;
          padding: 20px;
          border: 1px solid #ddd;
          border-radius: 8px;
          background: #fff;
        }

        .webauthn-authentication {
          text-align: center;
        }

        .authentication-header h3 {
          margin: 0 0 10px 0;
          color: #333;
        }

        .authentication-header p {
          color: #666;
          margin-bottom: 20px;
        }

        .status-message {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 15px;
          padding: 20px;
        }

        .spinner {
          width: 40px;
          height: 40px;
          border: 4px solid #f3f3f3;
          border-top: 4px solid #3498db;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .authenticator-instructions {
          background: #f8f9fa;
          padding: 15px;
          border-radius: 6px;
          text-align: left;
          max-width: 400px;
        }

        .authenticator-instructions ul {
          margin: 0;
          padding-left: 20px;
        }

        .authenticator-instructions li {
          margin-bottom: 8px;
          color: #555;
          font-size: 14px;
        }

        .credential-list {
          margin: 20px 0;
          text-align: left;
        }

        .credential-list h4 {
          margin: 0 0 10px 0;
          color: #333;
        }

        .credential-list ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }

        .credential-list li {
          padding: 10px;
          border: 1px solid #eee;
          border-radius: 4px;
          margin-bottom: 8px;
          background: #f9f9f9;
        }

        .credential-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          flex-wrap: wrap;
          gap: 10px;
        }

        .credential-id {
          font-family: monospace;
          font-size: 12px;
          color: #666;
        }

        .credential-date, .credential-device {
          font-size: 12px;
          color: #888;
        }

        .success-message, .error-message {
          padding: 20px;
        }

        .success-icon, .error-icon {
          font-size: 48px;
          margin-bottom: 15px;
        }

        .success-icon {
          color: #28a745;
        }

        .error-icon {
          color: #dc3545;
        }

        .success-message h3, .error-message h3 {
          margin: 0 0 10px 0;
          color: #333;
        }

        .success-message p, .error-message p {
          color: #666;
          margin-bottom: 15px;
        }

        .user-info {
          background: #f8f9fa;
          padding: 15px;
          border-radius: 6px;
          margin: 15px 0;
        }

        .user-info p {
          margin: 5px 0;
          font-size: 14px;
        }

        .warning-message, .info-message {
          padding: 12px;
          border-radius: 6px;
          margin-bottom: 20px;
        }

        .warning-message {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          color: #856404;
        }

        .info-message {
          background: #d1ecf1;
          border: 1px solid #bee5eb;
          color: #0c5460;
        }

        .authentication-actions {
          display: flex;
          gap: 10px;
          justify-content: center;
          margin-top: 20px;
        }

        .btn-primary {
          background: #007bff;
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 16px;
        }

        .btn-primary:hover {
          background: #0056b3;
        }

        .btn-primary:disabled {
          background: #6c757d;
          cursor: not-allowed;
        }

        .btn-secondary {
          background: #6c757d;
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 16px;
        }

        .btn-secondary:hover {
          background: #545b62;
        }
      `}</style>
    </div>
  );
};