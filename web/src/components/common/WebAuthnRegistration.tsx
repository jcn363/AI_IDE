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

interface WebAuthnRegistrationProps {
  userId: string;
  userDisplayName: string;
  userName: string;
  onSuccess?: (credential: WebAuthnCredential) => void;
  onError?: (error: string) => void;
}

interface RegistrationState {
  step: 'idle' | 'checking-support' | 'starting' | 'authenticating' | 'verifying' | 'completed' | 'error';
  challenge?: WebAuthnChallengeResponse;
  credential?: WebAuthnCredential;
  error?: string;
  supported: boolean;
}

export const WebAuthnRegistration: React.FC<WebAuthnRegistrationProps> = ({
  userId,
  userDisplayName,
  userName,
  onSuccess,
  onError
}) => {
  const [state, setState] = useState<RegistrationState>({
    step: 'idle',
    supported: false
  });

  // Check WebAuthn support on mount
  useEffect(() => {
    const checkSupport = async () => {
      setState(prev => ({ ...prev, step: 'checking-support' }));

      try {
        const support = await checkWebAuthnSupport();
        setState(prev => ({
          ...prev,
          step: 'idle',
          supported: support.supported
        }));
      } catch (error) {
        console.error('Failed to check WebAuthn support:', error);
        setState(prev => ({
          ...prev,
          step: 'error',
          error: 'Failed to check WebAuthn support',
          supported: false
        }));
      }
    };

    checkSupport();
  }, []);

  const startRegistration = async () => {
    if (!state.supported) {
      const error = 'WebAuthn is not supported in this browser';
      setState(prev => ({ ...prev, step: 'error', error }));
      onError?.(error);
      return;
    }

    setState(prev => ({ ...prev, step: 'starting' }));

    try {
      const challenge = await webauthnApiCall(
        () => webauthnService.startRegistration({
          user_display_name: userDisplayName,
          user_name: userName
        })
      );

      setState(prev => ({
        ...prev,
        step: 'authenticating',
        challenge
      }));

      // Start the browser WebAuthn registration
      await performBrowserRegistration(challenge);

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Registration failed';
      setState(prev => ({ ...prev, step: 'error', error: errorMsg }));
      onError?.(errorMsg);
    }
  };

  const performBrowserRegistration = async (challenge: WebAuthnChallengeResponse) => {
    try {
      // Convert challenge data for browser WebAuthn API
      const publicKeyCredentialCreationOptions = {
        challenge: base64UrlToUint8Array(challenge.challenge.publicKey.challenge),
        rp: challenge.challenge.publicKey.rp,
        user: {
          id: base64UrlToUint8Array(challenge.challenge.publicKey.user.id),
          name: challenge.challenge.publicKey.user.name,
          displayName: challenge.challenge.publicKey.user.displayName
        },
        pubKeyCredParams: challenge.challenge.publicKey.pubKeyCredParams,
        authenticatorSelection: challenge.challenge.publicKey.authenticatorSelection,
        timeout: challenge.challenge.publicKey.timeout,
        attestation: challenge.challenge.publicKey.attestation
      };

      setState(prev => ({ ...prev, step: 'authenticating' }));

      // Create the credential
      const credential = await navigator.credentials.create({
        publicKey: publicKeyCredentialCreationOptions
      }) as PublicKeyCredential;

      setState(prev => ({ ...prev, step: 'verifying' }));

      // Convert credential for backend
      const registrationResponse = {
        id: credential.id,
        rawId: uint8ArrayToBase64Url(new Uint8Array(credential.rawId)),
        type: credential.type,
        response: {
          clientDataJSON: uint8ArrayToBase64Url(new Uint8Array(credential.response.clientDataJSON)),
          attestationObject: uint8ArrayToBase64Url(new Uint8Array((credential.response as AuthenticatorAttestationResponse).attestationObject))
        }
      };

      // Finish registration with backend
      const result = await webauthnApiCall(
        () => webauthnService.finishRegistration({
          challenge_id: challenge.challenge_id,
          registration_response: registrationResponse
        })
      );

      if (result.success && result.credential_id) {
        setState(prev => ({
          ...prev,
          step: 'completed',
          credential: result as any // This would be properly typed
        }));
        onSuccess?.(result as any);
      } else {
        throw new Error(result.error || 'Registration verification failed');
      }

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Browser registration failed';
      setState(prev => ({ ...prev, step: 'error', error: errorMsg }));
      onError?.(errorMsg);
    }
  };

  const resetRegistration = () => {
    setState(prev => ({
      ...prev,
      step: 'idle',
      challenge: undefined,
      credential: undefined,
      error: undefined
    }));
  };

  const renderContent = () => {
    switch (state.step) {
      case 'checking-support':
        return (
          <div className="webauthn-registration">
            <div className="status-message">
              <div className="spinner"></div>
              Checking WebAuthn support...
            </div>
          </div>
        );

      case 'idle':
        return (
          <div className="webauthn-registration">
            <div className="registration-header">
              <h3>Register WebAuthn Credential</h3>
              <p>Set up passwordless authentication using your device's biometric or security key.</p>
            </div>

            {!state.supported && (
              <div className="warning-message">
                WebAuthn is not supported in this browser. Please use a modern browser with WebAuthn support.
              </div>
            )}

            <div className="registration-actions">
              <button
                onClick={startRegistration}
                disabled={!state.supported}
                className="btn-primary"
              >
                Start Registration
              </button>
            </div>
          </div>
        );

      case 'starting':
        return (
          <div className="webauthn-registration">
            <div className="status-message">
              <div className="spinner"></div>
              Preparing registration challenge...
            </div>
          </div>
        );

      case 'authenticating':
        return (
          <div className="webauthn-registration">
            <div className="status-message">
              <div className="spinner"></div>
              Follow the instructions on your device to complete registration...
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
          <div className="webauthn-registration">
            <div className="status-message">
              <div className="spinner"></div>
              Verifying registration...
            </div>
          </div>
        );

      case 'completed':
        return (
          <div className="webauthn-registration">
            <div className="success-message">
              <div className="success-icon">✓</div>
              <h3>Registration Successful!</h3>
              <p>Your WebAuthn credential has been registered successfully.</p>
              {state.credential && (
                <div className="credential-info">
                  <p><strong>Credential ID:</strong> {state.credential.credential_id}</p>
                  <p><strong>Created:</strong> {new Date(state.credential.created_at).toLocaleString()}</p>
                </div>
              )}
            </div>
            <div className="registration-actions">
              <button onClick={resetRegistration} className="btn-secondary">
                Register Another
              </button>
            </div>
          </div>
        );

      case 'error':
        return (
          <div className="webauthn-registration">
            <div className="error-message">
              <div className="error-icon">✕</div>
              <h3>Registration Failed</h3>
              <p>{state.error}</p>
            </div>
            <div className="registration-actions">
              <button onClick={startRegistration} className="btn-primary">
                Try Again
              </button>
              <button onClick={resetRegistration} className="btn-secondary">
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
    <div className="webauthn-registration-container">
      {renderContent()}

      <style jsx>{`
        .webauthn-registration-container {
          max-width: 500px;
          margin: 0 auto;
          padding: 20px;
          border: 1px solid #ddd;
          border-radius: 8px;
          background: #fff;
        }

        .webauthn-registration {
          text-align: center;
        }

        .registration-header h3 {
          margin: 0 0 10px 0;
          color: #333;
        }

        .registration-header p {
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

        .credential-info {
          background: #f8f9fa;
          padding: 15px;
          border-radius: 6px;
          text-align: left;
          margin: 15px 0;
        }

        .credential-info p {
          margin: 5px 0;
          font-size: 14px;
        }

        .warning-message {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
          color: #856404;
          padding: 12px;
          border-radius: 6px;
          margin-bottom: 20px;
        }

        .registration-actions {
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