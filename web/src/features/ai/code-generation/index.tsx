import React, { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { CodeGenerator, GeneratedCode, CodeGenerationOptions } from './CodeGenerator';
import type { AICollaborator, CollaborationSession, CodeReview, UserPresence } from '../types';

// Import collaboration types and services
import type { CollaborationRoom, EditingSession } from '../../collaboration/types';
import { CollaborationManager } from '../../collaboration/CollaborationManager';

// Styles - using CSS modules or inline styles due to webview isolation
const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100%',
    padding: '16px',
    backgroundColor: '#1e1e1e',
    color: '#ffffff',
    fontFamily: 'system-ui, -apple-system, sans-serif',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
    paddingBottom: '8px',
    borderBottom: '1px solid #404040',
  },
  title: {
    fontSize: '18px',
    fontWeight: '600',
    margin: 0,
  },
  sessionInfo: {
    fontSize: '12px',
    color: '#cccccc',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  mainContent: {
    display: 'flex',
    flex: 1,
    gap: '16px',
    overflow: 'hidden',
  },
  leftPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
  },
  rightPanel: {
    width: '300px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
  },
  promptSection: {
    backgroundColor: '#2d2d2d',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #404040',
  },
  sectionTitle: {
    fontSize: '14px',
    fontWeight: '500',
    marginBottom: '12px',
    marginTop: 0,
  },
  promptInput: {
    width: '100%',
    minHeight: '100px',
    backgroundColor: '#1e1e1e',
    border: '1px solid #555555',
    borderRadius: '4px',
    color: '#ffffff',
    padding: '8px',
    fontFamily: 'monospace',
    fontSize: '14px',
    resize: 'vertical' as const,
  },
  generationControls: {
    display: 'flex',
    gap: '8px',
    alignItems: 'center',
  },
  button: {
    padding: '8px 16px',
    borderRadius: '4px',
    border: 'none',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: '500',
    transition: 'all 0.2s',
  },
  primaryButton: {
    backgroundColor: '#007acc',
    color: '#ffffff',
  },
  secondaryButton: {
    backgroundColor: '#5f5f5f',
    color: '#ffffff',
  },
  disabledButton: {
    backgroundColor: '#404040',
    color: '#888888',
    cursor: 'not-allowed',
  },
  resultSection: {
    flex: 1,
    backgroundColor: '#2d2d2d',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #404040',
    display: 'flex',
    flexDirection: 'column' as const,
  },
  resultContent: {
    flex: 1,
    backgroundColor: '#1e1e1e',
    border: '1px solid #555555',
    borderRadius: '4px',
    padding: '12px',
    fontFamily: 'monospace',
    fontSize: '14px',
    color: '#ffffff',
    overflow: 'auto',
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
  },
  collaboratorsSection: {
    backgroundColor: '#2d2d2d',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #404040',
  },
  collaboratorList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  collaborator: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '6px',
    backgroundColor: '#1e1e1e',
    borderRadius: '4px',
    border: '1px solid #404040',
  },
  collaboratorAvatar: {
    width: '24px',
    height: '24px',
    borderRadius: '50%',
    backgroundColor: '#007acc',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '12px',
    fontWeight: 'bold',
    color: '#ffffff',
  },
  collaboratorInfo: {
    flex: 1,
  },
  collaboratorName: {
    fontSize: '14px',
    fontWeight: '500',
    margin: 0,
  },
  collaboratorStatus: {
    fontSize: '12px',
    color: '#cccccc',
    margin: 0,
  },
  statusIndicator: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
  },
  statusOnline: {
    backgroundColor: '#4caf50',
  },
  statusAway: {
    backgroundColor: '#ff9800',
  },
  statusOffline: {
    backgroundColor: '#f44336',
  },
  sharedPromptsSection: {
    backgroundColor: '#2d2d2d',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #404040',
    flex: 1,
  },
  sharedPromptList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxHeight: '200px',
    overflowY: 'auto' as const,
  },
  sharedPrompt: {
    padding: '8px',
    backgroundColor: '#1e1e1e',
    border: '1px solid #404040',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  sharedPromptHover: {
    borderColor: '#007acc',
  },
  sharedPromptAuthor: {
    fontSize: '12px',
    color: '#cccccc',
    marginBottom: '4px',
  },
  sharedPromptText: {
    fontSize: '14px',
    color: '#ffffff',
    margin: 0,
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  loadingIndicator: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '20px',
    color: '#cccccc',
  },
  errorMessage: {
    backgroundColor: '#f44336',
    color: '#ffffff',
    padding: '8px',
    borderRadius: '4px',
    marginBottom: '8px',
    fontSize: '14px',
  },
  reviewSection: {
    backgroundColor: '#2d2d2d',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #404040',
  },
  reviewList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxHeight: '150px',
    overflowY: 'auto' as const,
  },
  review: {
    padding: '8px',
    backgroundColor: '#1e1e1e',
    border: '1px solid #404040',
    borderRadius: '4px',
  },
  reviewHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '4px',
  },
  reviewAuthor: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#ffffff',
  },
  reviewTimestamp: {
    fontSize: '12px',
    color: '#cccccc',
  },
  reviewComment: {
    fontSize: '14px',
    color: '#ffffff',
    margin: 0,
  },
  reviewActions: {
    display: 'flex',
    gap: '4px',
    marginTop: '8px',
  },
  reviewButton: {
    padding: '4px 8px',
    borderRadius: '3px',
    border: 'none',
    cursor: 'pointer',
    fontSize: '12px',
    fontWeight: '500',
  },
  reviewAccept: {
    backgroundColor: '#4caf50',
    color: '#ffffff',
  },
  reviewReject: {
    backgroundColor: '#f44336',
    color: '#ffffff',
  },
};

interface CodeGenerationSession {
  id: string;
  prompt: string;
  generatedCode: GeneratedCode | null;
  collaborators: AICollaborator[];
  reviews: CodeReview[];
  createdAt: number;
  status: 'idle' | 'generating' | 'completed' | 'error';
  error?: string;
}

interface SharedPrompt {
  id: string;
  prompt: string;
  author: string;
  timestamp: number;
  votes: number;
}

interface CodeGenerationProps {
  filePath?: string;
  fileContent?: string;
  language?: string;
  collaborationRoom?: CollaborationRoom;
  onCodeGenerated?: (code: GeneratedCode) => void;
  onSessionCreated?: (session: CodeGenerationSession) => void;
}

export const CodeGenerationPanel: React.FC<CodeGenerationProps> = ({
  filePath,
  fileContent,
  language = 'rust',
  collaborationRoom,
  onCodeGenerated,
  onSessionCreated,
}) => {
  // State management - using useState instead of external state libraries
  const [currentPrompt, setCurrentPrompt] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [currentSession, setCurrentSession] = useState<CodeGenerationSession | null>(null);
  const [sharedPrompts, setSharedPrompts] = useState<SharedPrompt[]>([]);
  const [collaborators, setCollaborators] = useState<UserPresence[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Refs for managing subscriptions
  const codeGeneratorRef = useRef<CodeGenerator | null>(null);
  const collaborationManagerRef = useRef<CollaborationManager | null>(null);
  const unsubscribeRefs = useRef<(() => void)[]>([]);

  // Initialize services
  useEffect(() => {
    const initializeServices = async () => {
      try {
        // Initialize CodeGenerator
        const codeGenerator = CodeGenerator.getInstance();
        await codeGenerator.initialize();
        codeGeneratorRef.current = codeGenerator;

        // Initialize CollaborationManager if room is provided
        if (collaborationRoom) {
          const collaborationManager = CollaborationManager.getInstance();
          collaborationManagerRef.current = collaborationManager;

          // Join the collaboration room
          await collaborationManager.joinRoom(collaborationRoom.id);

          // Set up collaboration event listeners
          const unsubscribePresence = collaborationManager.onPresenceUpdate(handlePresenceUpdate);
          const unsubscribeSession = collaborationManager.onSessionEvent(handleSessionEvent);

          unsubscribeRefs.current = [unsubscribePresence, unsubscribeSession];
        }

        // Load shared prompts
        await loadSharedPrompts();
      } catch (err) {
        console.error('Failed to initialize services:', err);
        setError('Failed to initialize AI code generation services');
      }
    };

    initializeServices();

    // Cleanup
    return () => {
      unsubscribeRefs.current.forEach((unsubscribe) => unsubscribe());
      unsubscribeRefs.current = [];
    };
  }, []);

  // Collaboration event handlers
  const handlePresenceUpdate = useCallback((presence: UserPresence) => {
    setCollaborators((prev) => {
      const existing = prev.find((c) => c.userId === presence.userId);
      if (existing) {
        return prev.map((c) => (c.userId === presence.userId ? presence : c));
      } else {
        return [...prev, presence];
      }
    });
  }, []);

  const handleSessionEvent = useCallback((event: any) => {
    // Handle collaboration session events for code generation
    if (event.type === 'code_generation_started') {
      setCurrentSession((prev) =>
        prev
          ? {
              ...prev,
              status: 'generating',
            }
          : null
      );
    } else if (event.type === 'code_generation_completed') {
      setCurrentSession((prev) =>
        prev
          ? {
              ...prev,
              status: 'completed',
              generatedCode: event.generatedCode,
            }
          : null
      );
    }
  }, []);

  // Load shared prompts from collaboration
  const loadSharedPrompts = async () => {
    try {
      if (collaborationRoom && collaborationManagerRef.current) {
        // Load shared prompts from the collaboration room
        const prompts = await invoke('get_shared_prompts', {
          room_id: collaborationRoom.id,
        });
        setSharedPrompts(prompts || []);
      }
    } catch (err) {
      console.error('Failed to load shared prompts:', err);
    }
  };

  // Handle code generation
  const handleGenerateCode = async () => {
    if (!currentPrompt.trim() || !codeGeneratorRef.current) return;

    setIsGenerating(true);
    setError(null);

    try {
      // Create new session
      const sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      const newSession: CodeGenerationSession = {
        id: sessionId,
        prompt: currentPrompt,
        generatedCode: null,
        collaborators: collaborators.map((c) => ({
          id: c.userId,
          name: c.name,
          role: 'collaborator',
          capabilities: ['code-review'],
          status: c.status === 'online' ? 'ready' : 'busy',
        })),
        reviews: [],
        createdAt: Date.now(),
        status: 'generating',
      };

      setCurrentSession(newSession);
      onSessionCreated?.(newSession);

      // Generate code options
      const options: CodeGenerationOptions = {
        type: 'boilerplate',
        context: currentPrompt,
        language,
        filePath,
        fileContent,
        maxLength: 2000,
        temperature: 0.7,
        includeExamples: true,
      };

      // Generate code
      const generatedCode = await codeGeneratorRef.current.generateBoilerplate(options);

      // Update session
      const completedSession = {
        ...newSession,
        status: 'completed' as const,
        generatedCode,
      };

      setCurrentSession(completedSession);
      onCodeGenerated?.(generatedCode);

      // Broadcast to collaborators
      if (collaborationManagerRef.current && collaborationRoom) {
        await collaborationManagerRef.current.broadcastEvent({
          type: 'code_generation_completed',
          sessionId: collaborationRoom.id,
          userId: 'current-user', // TODO: Get actual user ID
          timestamp: Date.now(),
          data: { generatedCode, prompt: currentPrompt },
        });
      }
    } catch (err) {
      console.error('Code generation failed:', err);
      setError('Failed to generate code. Please try again.');

      if (currentSession) {
        setCurrentSession({
          ...currentSession,
          status: 'error',
          error: err instanceof Error ? err.message : 'Unknown error',
        });
      }
    } finally {
      setIsGenerating(false);
    }
  };

  // Handle shared prompt selection
  const handleSharedPromptSelect = (prompt: SharedPrompt) => {
    setCurrentPrompt(prompt.prompt);
  };

  // Handle code review
  const handleCodeReview = async (review: CodeReview) => {
    if (!currentSession) return;

    try {
      // Add review to session
      setCurrentSession((prev) =>
        prev
          ? {
              ...prev,
              reviews: [...prev.reviews, review],
            }
          : null
      );

      // Broadcast review to collaborators
      if (collaborationManagerRef.current && collaborationRoom) {
        await collaborationManagerRef.current.broadcastEvent({
          type: 'code_review_added',
          sessionId: collaborationRoom.id,
          userId: review.reviewer,
          timestamp: Date.now(),
          data: { review, sessionId: currentSession.id },
        });
      }
    } catch (err) {
      console.error('Failed to add code review:', err);
    }
  };

  // Share current prompt
  const handleSharePrompt = async () => {
    if (!currentPrompt.trim() || !collaborationRoom) return;

    try {
      const sharedPrompt: SharedPrompt = {
        id: `prompt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        prompt: currentPrompt,
        author: 'current-user', // TODO: Get actual user name
        timestamp: Date.now(),
        votes: 0,
      };

      await invoke('share_prompt', {
        room_id: collaborationRoom.id,
        prompt: sharedPrompt,
      });

      setSharedPrompts((prev) => [sharedPrompt, ...prev]);
    } catch (err) {
      console.error('Failed to share prompt:', err);
      setError('Failed to share prompt');
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h1 style={styles.title}>Collaborative AI Code Generation</h1>
        {collaborationRoom && (
          <div style={styles.sessionInfo}>
            <span>Room: {collaborationRoom.name}</span>
            <span>•</span>
            <span>{collaborators.length} collaborators</span>
          </div>
        )}
      </div>

      <div style={styles.mainContent}>
        <div style={styles.leftPanel}>
          {/* Prompt Input Section */}
          <div style={styles.promptSection}>
            <h3 style={styles.sectionTitle}>Code Generation Prompt</h3>
            <textarea
              style={styles.promptInput}
              value={currentPrompt}
              onChange={(e) => setCurrentPrompt(e.target.value)}
              placeholder="Describe the code you want to generate..."
              disabled={isGenerating}
            />
            <div style={styles.generationControls}>
              <button
                style={{
                  ...styles.button,
                  ...styles.primaryButton,
                  ...(isGenerating ? styles.disabledButton : {}),
                }}
                onClick={handleGenerateCode}
                disabled={isGenerating || !currentPrompt.trim()}
              >
                {isGenerating ? 'Generating...' : 'Generate Code'}
              </button>
              {collaborationRoom && (
                <button
                  style={{
                    ...styles.button,
                    ...styles.secondaryButton,
                    ...(!currentPrompt.trim() ? styles.disabledButton : {}),
                  }}
                  onClick={handleSharePrompt}
                  disabled={!currentPrompt.trim()}
                >
                  Share Prompt
                </button>
              )}
            </div>
            {error && <div style={styles.errorMessage}>{error}</div>}
          </div>

          {/* Generated Code Section */}
          <div style={styles.resultSection}>
            <h3 style={styles.sectionTitle}>
              Generated Code
              {currentSession && (
                <span style={{ fontSize: '12px', color: '#cccccc', marginLeft: '8px' }}>
                  Session: {currentSession.id.slice(-8)}
                </span>
              )}
            </h3>
            {isGenerating ? (
              <div style={styles.loadingIndicator}>Generating code...</div>
            ) : currentSession?.generatedCode ? (
              <pre style={styles.resultContent}>{currentSession.generatedCode.content}</pre>
            ) : (
              <div style={styles.loadingIndicator}>
                Enter a prompt and click "Generate Code" to get started
              </div>
            )}
          </div>
        </div>

        <div style={styles.rightPanel}>
          {/* Collaborators Section */}
          {collaborationRoom && (
            <div style={styles.collaboratorsSection}>
              <h3 style={styles.sectionTitle}>Collaborators</h3>
              <div style={styles.collaboratorList}>
                {collaborators.map((collaborator) => (
                  <div key={collaborator.userId} style={styles.collaborator}>
                    <div
                      style={{
                        ...styles.collaboratorAvatar,
                        backgroundColor: collaborator.color || '#007acc',
                      }}
                    >
                      {collaborator.name.charAt(0).toUpperCase()}
                    </div>
                    <div style={styles.collaboratorInfo}>
                      <p style={styles.collaboratorName}>{collaborator.name}</p>
                      <p style={styles.collaboratorStatus}>
                        {collaborator.currentFile
                          ? `Editing ${collaborator.currentFile}`
                          : 'Available'}
                      </p>
                    </div>
                    <div
                      style={{
                        ...styles.statusIndicator,
                        ...(collaborator.status === 'online'
                          ? styles.statusOnline
                          : collaborator.status === 'away'
                            ? styles.statusAway
                            : styles.statusOffline),
                      }}
                    />
                  </div>
                ))}
                {collaborators.length === 0 && (
                  <div style={{ padding: '8px', color: '#cccccc', fontSize: '14px' }}>
                    No collaborators online
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Shared Prompts Section */}
          {collaborationRoom && (
            <div style={styles.sharedPromptsSection}>
              <h3 style={styles.sectionTitle}>Shared Prompts</h3>
              <div style={styles.sharedPromptList}>
                {sharedPrompts.map((prompt) => (
                  <div
                    key={prompt.id}
                    style={styles.sharedPrompt}
                    onClick={() => handleSharedPromptSelect(prompt)}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.borderColor = '#007acc';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.borderColor = '#404040';
                    }}
                  >
                    <div style={styles.sharedPromptAuthor}>
                      {prompt.author} • {new Date(prompt.timestamp).toLocaleTimeString()}
                    </div>
                    <p style={styles.sharedPromptText}>{prompt.prompt}</p>
                  </div>
                ))}
                {sharedPrompts.length === 0 && (
                  <div style={{ padding: '8px', color: '#cccccc', fontSize: '14px' }}>
                    No shared prompts yet
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Code Reviews Section */}
          {currentSession && currentSession.reviews.length > 0 && (
            <div style={styles.reviewSection}>
              <h3 style={styles.sectionTitle}>Code Reviews</h3>
              <div style={styles.reviewList}>
                {currentSession.reviews.map((review, index) => (
                  <div key={index} style={styles.review}>
                    <div style={styles.reviewHeader}>
                      <span style={styles.reviewAuthor}>{review.reviewer}</span>
                      <span style={styles.reviewTimestamp}>
                        {new Date(review.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <p style={styles.reviewComment}>{review.comment}</p>
                    <div style={styles.reviewActions}>
                      <button style={{ ...styles.reviewButton, ...styles.reviewAccept }}>
                        Accept
                      </button>
                      <button style={{ ...styles.reviewButton, ...styles.reviewReject }}>
                        Reject
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default CodeGenerationPanel;
