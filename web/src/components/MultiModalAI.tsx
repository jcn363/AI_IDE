import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// NOTE: Following project rules - no React hooks for external state libraries
// Using useState as allowed, but no external hooks

// Collaboration interfaces
interface CollaborationSession {
  session_id: string;
  session_name: string;
  participants: Participant[];
  is_active: boolean;
  created_at: number;
}

interface Participant {
  user_id: string;
  username: string;
  status: 'online' | 'away' | 'offline';
  activity: string;
  joined_at: number;
}

interface CollaborativeResult {
  participant_id: string;
  participant_name: string;
  result: AnalysisResponse;
  timestamp: number;
  is_shared: boolean;
}

interface SharedInput {
  participant_id: string;
  participant_name: string;
  modality_type: string;
  content: any;
  timestamp: number;
}

interface MultiModalAIProps {
  onAnalysisComplete?: (result: AnalysisResponse) => void;
  isFullscreen?: boolean;
}

interface AnalysisRequest {
  modality_types: string[];
  text_content?: string;
  image_data?: string;
  audio_data?: string;
}

interface ModalityResult {
  modality_type: string;
  success: boolean;
  confidence: number;
  data: any;
  data_length?: number;
}

interface AnalysisResponse {
  success: boolean;
  results: ModalityResult[];
  processing_time: number;
  error?: string;
}

const MODALITY_TYPES = {
  TEXT: 'text',
  IMAGE: 'image',
  AUDIO: 'audio',
  SCREENSHOT: 'screenshot',
  DIAGRAM: 'diagram',
  CODE: 'code',
  MULTIMODAL: 'multimodal',
};

const MultiModalAI: React.FC<MultiModalAIProps> = ({
  onAnalysisComplete,
  isFullscreen = false,
}) => {
  const [loading, setLoading] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [selectedModalities, setSelectedModalities] = useState<Set<string>>(
    new Set([MODALITY_TYPES.TEXT])
  );
  const [inputText, setInputText] = useState('');
  const [inputImage, setInputImage] = useState<File | null>(null);
  const [audioRecording, setAudioRecording] = useState(false);

  // Collaboration state
  const [collaborationSession, setCollaborationSession] = useState<CollaborationSession | null>(
    null
  );
  const [collaborativeResults, setCollaborativeResults] = useState<CollaborativeResult[]>([]);
  const [sharedInputs, setSharedInputs] = useState<SharedInput[]>([]);
  const [userId] = useState(() => `user_${Math.random().toString(36).substr(2, 9)}`);
  const [username] = useState(() => `User_${Math.random().toString(36).substr(2, 4)}`);
  const [showCollaborationPanel, setShowCollaborationPanel] = useState(false);
  const [sessionNameInput, setSessionNameInput] = useState('');
  const [joinSessionId, setJoinSessionId] = useState('');

  const handleModalityToggle = useCallback(
    (modality: string) => {
      const newSelected = new Set(selectedModalities);
      if (newSelected.has(modality)) {
        newSelected.delete(modality);
      } else {
        newSelected.add(modality);
      }
      setSelectedModalities(newSelected);
    },
    [selectedModalities]
  );

  const handleFileUpload = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      if (file.type.startsWith('image/')) {
        setInputImage(file);
      }
    }
  }, []);

  const startAudioRecording = useCallback(() => {
    setAudioRecording(true);
    // TODO: Implement audio recording
    // Note: Webview restrictions prevent full access to navigator.mediaDevices
    console.log('Audio recording started');
  }, []);

  const stopAudioRecording = useCallback(() => {
    setAudioRecording(false);
    // TODO: Stop recording and process audio
    console.log('Audio recording stopped');
  }, []);

  const processAnalysis = useCallback(async () => {
    if (selectedModalities.size === 0) return;

    setLoading(true);
    try {
      const requestData: AnalysisRequest = {
        modality_types: Array.from(selectedModalities),
      };

      // Add text content if text modality selected
      if (selectedModalities.has(MODALITY_TYPES.TEXT) && inputText.trim()) {
        requestData.text_content = inputText.trim();
        // Share text input in collaboration session
        if (collaborationSession) {
          await shareMultimodalInput(MODALITY_TYPES.TEXT, inputText.trim());
        }
      }

      // Add image data if image uploaded
      if (inputImage && selectedModalities.has(MODALITY_TYPES.IMAGE)) {
        // Convert file to base64 (simple approach - in reality might chunk large files)
        const imageData = await fileToBase64(inputImage);
        requestData.image_data = imageData;

        // Share image input in collaboration session
        if (collaborationSession) {
          await shareMultimodalInput(MODALITY_TYPES.IMAGE, {
            filename: inputImage.name,
            size: inputImage.size,
            data: imageData,
          });
        }
      }

      // Add audio data if screenshot requested (placeholder)
      if (selectedModalities.has(MODALITY_TYPES.SCREENSHOT)) {
        // Camera/screenshot API limited in webview
        // Would need backend to capture screenshot
      }

      console.log('Processing multimodal analysis:', requestData);

      // Call backend multimodal AI service
      const result: AnalysisResponse = await invoke('process_multimodal_analysis', {
        requestData,
      });

      setAnalysisResult(result);
      onAnalysisComplete?.(result);

      // Share result in collaboration session
      if (collaborationSession) {
        await shareAnalysisResult(result);
      }
    } catch (error) {
      console.error('Multi-modal analysis failed:', error);
      const errorResult: AnalysisResponse = {
        success: false,
        results: [],
        processing_time: 0,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
      setAnalysisResult(errorResult);
      onAnalysisComplete?.(errorResult);

      // Share error result in collaboration session
      if (collaborationSession) {
        await shareAnalysisResult(errorResult);
      }
    } finally {
      setLoading(false);
    }
  }, [
    selectedModalities,
    inputText,
    inputImage,
    onAnalysisComplete,
    collaborationSession,
    shareMultimodalInput,
    shareAnalysisResult,
  ]);

  // Collaboration functions
  const createCollaborationSession = useCallback(async () => {
    if (!sessionNameInput.trim()) return;

    try {
      const result = await invoke('collaboration_create_authenticated_session', {
        userId,
        username,
        documentId: `multimodal_${Date.now()}`,
        userRole: 'Editor',
      });

      const sessionData = JSON.parse(result as string);
      const newSession: CollaborationSession = {
        session_id: sessionData.session_id,
        session_name: sessionNameInput,
        participants: [
          {
            user_id: userId,
            username,
            status: 'online',
            activity: 'Session created',
            joined_at: Date.now(),
          },
        ],
        is_active: true,
        created_at: Date.now(),
      };

      setCollaborationSession(newSession);
      setSessionNameInput('');
      setShowCollaborationPanel(false);

      // Update presence
      await updatePresence('Online', 'Session created');
    } catch (error) {
      console.error('Failed to create collaboration session:', error);
    }
  }, [sessionNameInput, userId, username]);

  const joinCollaborationSession = useCallback(async () => {
    if (!joinSessionId.trim()) return;

    try {
      // For now, simulate joining - in real implementation would call backend
      const newSession: CollaborationSession = {
        session_id: joinSessionId,
        session_name: `Session ${joinSessionId.slice(-4)}`,
        participants: [
          {
            user_id: 'host',
            username: 'Host User',
            status: 'online',
            activity: 'Hosting session',
            joined_at: Date.now() - 300000,
          },
          {
            user_id: userId,
            username,
            status: 'online',
            activity: 'Joined session',
            joined_at: Date.now(),
          },
        ],
        is_active: true,
        created_at: Date.now() - 300000,
      };

      setCollaborationSession(newSession);
      setJoinSessionId('');
      setShowCollaborationPanel(false);

      await updatePresence('Online', 'Joined collaborative session');
    } catch (error) {
      console.error('Failed to join collaboration session:', error);
    }
  }, [joinSessionId, userId, username]);

  const leaveCollaborationSession = useCallback(async () => {
    if (!collaborationSession) return;

    try {
      await invoke('collaboration_end_session', {
        sessionId: collaborationSession.session_id,
        serviceType: 'collaboration',
      });

      setCollaborationSession(null);
      setCollaborativeResults([]);
      setSharedInputs([]);
    } catch (error) {
      console.error('Failed to leave collaboration session:', error);
    }
  }, [collaborationSession]);

  const updatePresence = useCallback(
    async (status: string, activity: string) => {
      if (!collaborationSession) return;

      try {
        await invoke('collaboration_update_presence', {
          userId,
          username,
          status,
          sessionId: collaborationSession.session_id,
          activityType: 'SessionJoined',
        });
      } catch (error) {
        console.error('Failed to update presence:', error);
      }
    },
    [collaborationSession, userId, username]
  );

  const shareMultimodalInput = useCallback(
    async (modalityType: string, content: any) => {
      if (!collaborationSession) return;

      const sharedInput: SharedInput = {
        participant_id: userId,
        participant_name: username,
        modality_type: modalityType,
        content,
        timestamp: Date.now(),
      };

      setSharedInputs((prev) => [...prev, sharedInput]);

      // In a real implementation, this would send to all session participants
      console.log('Shared input with session:', sharedInput);
    },
    [collaborationSession, userId, username]
  );

  const shareAnalysisResult = useCallback(
    async (result: AnalysisResponse) => {
      if (!collaborationSession || !result) return;

      const collaborativeResult: CollaborativeResult = {
        participant_id: userId,
        participant_name: username,
        result,
        timestamp: Date.now(),
        is_shared: true,
      };

      setCollaborativeResults((prev) => [...prev, collaborativeResult]);
    },
    [collaborationSession, userId, username]
  );

  const requestCollaborativeCoaching = useCallback(async () => {
    if (!collaborationSession || !analysisResult) return;

    try {
      const result = await invoke('collaboration_request_coaching', {
        sessionId: collaborationSession.session_id,
        contextCode: inputText || 'No text context',
        filePath: 'multimodal_analysis',
        cursorPosition: [0, inputText.length],
        language: 'multimodal',
        projectContext: `Multimodal AI analysis with modalities: ${Array.from(selectedModalities).join(', ')}`,
        userActions: [
          {
            action_type: 'analysis_request',
            content: `Analyzed with modalities: ${Array.from(selectedModalities).join(', ')}`,
            timestamp: Date.now(),
          },
        ],
      });

      const coachingData = JSON.parse(result as string);
      console.log('Collaborative coaching received:', coachingData);
      // In a real implementation, this would be displayed in the UI
    } catch (error) {
      console.error('Failed to get collaborative coaching:', error);
    }
  }, [collaborationSession, analysisResult, inputText, selectedModalities]);

  const fileToBase64 = useCallback((file: File): Promise<string> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        // Remove data URL prefix if present
        const base64Data = result.split(',')[1] || result;
        resolve(base64Data);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }, []);

  const getModalityColor = (modality: string): string => {
    switch (modality) {
      case MODALITY_TYPES.TEXT:
        return '#3182ce';
      case MODALITY_TYPES.IMAGE:
        return '#38a169';
      case MODALITY_TYPES.AUDIO:
        return '#dd6b20';
      case MODALITY_TYPES.SCREENSHOT:
        return '#9f7aea';
      case MODALITY_TYPES.DIAGRAM:
        return '#e53e3e';
      case MODALITY_TYPES.CODE:
        return '#805ad5';
      default:
        return '#4a5568';
    }
  };

  return (
    <div className={`multimodal-ai ${isFullscreen ? 'fullscreen' : ''}`}>
      <div className="multimodal-header">
        <div className="header-content">
          <div>
            <h2>Multi-Modal AI Analysis</h2>
            <p>Analyze text, images, audio, and screenshots with AI</p>
          </div>
          <div className="collaboration-controls">
            <button
              className={`collaboration-toggle ${collaborationSession ? 'active' : ''}`}
              onClick={() => setShowCollaborationPanel(!showCollaborationPanel)}
              title={collaborationSession ? 'Collaboration Active' : 'Start Collaboration'}
            >
              👥 {collaborationSession ? 'Collaborating' : 'Collaborate'}
            </button>
            {collaborationSession && (
              <button
                className="leave-session-btn"
                onClick={leaveCollaborationSession}
                title="Leave Collaboration Session"
              >
                🚪 Leave
              </button>
            )}
          </div>
        </div>
        {collaborationSession && (
          <div className="session-info">
            <span className="session-name">Session: {collaborationSession.session_name}</span>
            <span className="participant-count">
              {collaborationSession.participants.length} participant
              {collaborationSession.participants.length !== 1 ? 's' : ''}
            </span>
          </div>
        )}
      </div>

      {/* Modality Selection */}
      <div className="modality-selection">
        <h3>Choose Analysis Types</h3>
        <div className="modality-buttons">
          {Object.entries(MODALITY_TYPES).map(
            ([key, value]) =>
              key !== 'MULTIMODAL' && (
                <button
                  key={value}
                  className={`modality-btn ${selectedModalities.has(value) ? 'active' : ''}`}
                  onClick={() => handleModalityToggle(value)}
                  style={{
                    backgroundColor: selectedModalities.has(value)
                      ? getModalityColor(value)
                      : undefined,
                  }}
                >
                  {key}
                </button>
              )
          )}
        </div>
      </div>

      {/* Collaboration Panel */}
      {showCollaborationPanel && (
        <div className="collaboration-panel">
          <h3>Collaboration Session</h3>
          {!collaborationSession ? (
            <div className="session-setup">
              <div className="setup-section">
                <h4>Create New Session</h4>
                <div className="input-group">
                  <input
                    type="text"
                    placeholder="Session name..."
                    value={sessionNameInput}
                    onChange={(e) => setSessionNameInput(e.target.value)}
                  />
                  <button
                    className="create-btn"
                    onClick={createCollaborationSession}
                    disabled={!sessionNameInput.trim()}
                  >
                    Create Session
                  </button>
                </div>
              </div>
              <div className="setup-section">
                <h4>Join Existing Session</h4>
                <div className="input-group">
                  <input
                    type="text"
                    placeholder="Session ID..."
                    value={joinSessionId}
                    onChange={(e) => setJoinSessionId(e.target.value)}
                  />
                  <button
                    className="join-btn"
                    onClick={joinCollaborationSession}
                    disabled={!joinSessionId.trim()}
                  >
                    Join Session
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="active-session">
              <div className="participants-section">
                <h4>Participants ({collaborationSession.participants.length})</h4>
                <div className="participants-list">
                  {collaborationSession.participants.map((participant) => (
                    <div key={participant.user_id} className="participant-item">
                      <div className={`status-indicator ${participant.status}`}></div>
                      <span className="participant-name">
                        {participant.username}
                        {participant.user_id === userId && ' (You)'}
                      </span>
                      <span className="participant-activity">{participant.activity}</span>
                    </div>
                  ))}
                </div>
              </div>
              {sharedInputs.length > 0 && (
                <div className="shared-inputs-section">
                  <h4>Shared Inputs</h4>
                  <div className="shared-inputs-list">
                    {sharedInputs.slice(-5).map((input, index) => (
                      <div key={index} className="shared-input-item">
                        <span className="input-participant">{input.participant_name}</span>
                        <span className="input-type">{input.modality_type}</span>
                        <span className="input-time">
                          {new Date(input.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Input Section */}
      <div className="input-section">
        {selectedModalities.has(MODALITY_TYPES.TEXT) && (
          <div className="input-group">
            <label>Text Input</label>
            <textarea
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Enter text to analyze..."
              rows={4}
            />
          </div>
        )}

        {selectedModalities.has(MODALITY_TYPES.IMAGE) && (
          <div className="input-group">
            <label>Image Upload</label>
            <input type="file" accept="image/*" onChange={handleFileUpload} />
            {inputImage && (
              <div className="image-preview">
                <img
                  src={URL.createObjectURL(inputImage)}
                  alt="Preview"
                  style={{ maxWidth: '200px', maxHeight: '200px' }}
                />
                <p>{inputImage.name}</p>
              </div>
            )}
          </div>
        )}

        {selectedModalities.has(MODALITY_TYPES.AUDIO) && (
          <div className="input-group">
            <label>Audio Input</label>
            <div className="audio-controls">
              <button
                className={`audio-btn ${audioRecording ? 'recording' : ''}`}
                onClick={audioRecording ? stopAudioRecording : startAudioRecording}
              >
                {audioRecording ? '⏹️ Stop Recording' : '🎤 Start Recording'}
              </button>
              <p>Audio processing limited by webview constraints</p>
            </div>
          </div>
        )}

        {selectedModalities.has(MODALITY_TYPES.SCREENSHOT) && (
          <div className="input-group">
            <label>Screenshot Analysis</label>
            <p>Capture screenshot for analysis (will be handled by backend)</p>
          </div>
        )}
      </div>

      {/* Analysis Button */}
      <div className="action-section">
        <button
          className="analyze-btn"
          onClick={processAnalysis}
          disabled={loading || selectedModalities.size === 0}
        >
          {loading ? '🔄 Analyzing...' : '⚡ Analyze with AI'}
        </button>
        {collaborationSession && analysisResult && (
          <button
            className="coaching-btn"
            onClick={requestCollaborativeCoaching}
            disabled={loading}
            title="Get collaborative AI coaching suggestions"
          >
            🎓 Get Coaching
          </button>
        )}
      </div>

      {/* Results Section */}
      {analysisResult && (
        <div className="results-section">
          <h3>Your Analysis Results</h3>

          {analysisResult.success ? (
            <div className="success-results">
              <div className="success-header">
                <span className="success-icon">✅</span>
                <span>Analysis completed successfully</span>
              </div>
              <p>Processing time: {analysisResult.processing_time.toFixed(2)}ms</p>

              <div className="results-grid">
                {analysisResult.results.map((result, index) => (
                  <div key={index} className="result-card">
                    <div className="result-header">
                      <h4>{result.modality_type}</h4>
                      <span
                        className={`confidence ${result.confidence > 0.8 ? 'high' : result.confidence > 0.6 ? 'medium' : 'low'}`}
                      >
                        {Math.round(result.confidence * 100)}%
                      </span>
                    </div>

                    {result.success ? (
                      <div className="result-content">
                        {result.modality_type === 'text' &&
                          typeof result.data === 'object' &&
                          'content' in result.data && (
                            <p>{(result.data as { content: string }).content}</p>
                          )}
                        {result.modality_type === 'image' &&
                          typeof result.data === 'object' &&
                          'description' in result.data && (
                            <p>{(result.data as { description: string }).description}</p>
                          )}
                        {result.data_length && <p>Data processed: {result.data_length} bytes</p>}
                      </div>
                    ) : (
                      <p className="error-text">Processing failed for this modality</p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="error-results">
              <div className="error-header">
                <span className="error-icon">❌</span>
                <span>Analysis failed</span>
              </div>
              <p className="error-text">{analysisResult.error}</p>
            </div>
          )}
        </div>
      )}

      {/* Collaborative Results Section */}
      {collaborativeResults.length > 0 && (
        <div className="collaborative-results-section">
          <h3>Collaborative Results</h3>
          <div className="collaborative-results-grid">
            {collaborativeResults.slice(-6).map((collabResult, index) => (
              <div key={index} className="collaborative-result-card">
                <div className="collaborative-result-header">
                  <div className="participant-info">
                    <span className="participant-name">{collabResult.participant_name}</span>
                    <span className="result-time">
                      {new Date(collabResult.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                  {collabResult.result.success ? (
                    <span className="collab-success-icon">✅</span>
                  ) : (
                    <span className="collab-error-icon">❌</span>
                  )}
                </div>

                {collabResult.result.success ? (
                  <div className="collaborative-result-content">
                    <p>Processing time: {collabResult.result.processing_time.toFixed(2)}ms</p>
                    <div className="collab-results-summary">
                      {collabResult.result.results.map((result, idx) => (
                        <div key={idx} className="collab-modality-summary">
                          <span className="modality-name">{result.modality_type}:</span>
                          <span
                            className={`modality-confidence ${result.confidence > 0.8 ? 'high' : result.confidence > 0.6 ? 'medium' : 'low'}`}
                          >
                            {Math.round(result.confidence * 100)}%
                          </span>
                          {result.success ? ' ✓' : ' ✗'}
                        </div>
                      ))}
                    </div>
                  </div>
                ) : (
                  <div className="collaborative-error-content">
                    <p className="error-text">{collabResult.result.error}</p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      <style jsx>{`
        .multimodal-ai {
          padding: 24px;
          border-radius: 12px;
          background: white;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          max-width: 800px;
          margin: 0 auto;
        }

        .multimodal-ai.fullscreen {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          z-index: 1000;
          margin: 0;
          border-radius: 0;
          overflow-y: auto;
        }

        .multimodal-header {
          text-align: center;
          margin-bottom: 32px;
        }

        .multimodal-header h2 {
          margin: 0 0 8px 0;
          color: #2d3748;
          font-size: 24px;
        }

        .multimodal-header p {
          margin: 0;
          color: #718096;
          font-size: 14px;
        }

        .modality-selection h3 {
          margin: 0 0 16px 0;
          color: #4a5568;
          font-size: 16px;
        }

        .modality-buttons {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-bottom: 24px;
        }

        .modality-btn {
          padding: 8px 16px;
          border: 2px solid #e1e5e9;
          border-radius: 8px;
          background: white;
          color: #4a5568;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .modality-btn:hover:not(.active) {
          border-color: #3182ce;
        }

        .modality-btn.active {
          color: white;
          border-color: transparent;
          font-weight: 600;
        }

        .input-section {
          display: flex;
          flex-direction: column;
          gap: 20px;
          margin-bottom: 24px;
        }

        .input-group label {
          display: block;
          margin-bottom: 8px;
          font-weight: 500;
          color: #4a5568;
        }

        .input-group textarea {
          width: 100%;
          padding: 12px;
          border: 1px solid #e1e5e9;
          border-radius: 6px;
          resize: vertical;
          font-size: 14px;
        }

        .input-group input[type='file'] {
          padding: 8px;
          border: 1px solid #e1e5e9;
          border-radius: 6px;
        }

        .image-preview {
          margin-top: 12px;
          text-align: center;
        }

        .image-preview p {
          margin: 8px 0 0 0;
          font-size: 12px;
          color: #718096;
        }

        .audio-controls {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .audio-btn {
          padding: 10px 16px;
          border: 1px solid #e1e5e9;
          border-radius: 6px;
          background: #f7fafc;
          cursor: pointer;
          font-size: 14px;
        }

        .audio-btn.recording {
          background: #fed7d7;
          border-color: #e53e3e;
        }

        .action-section {
          text-align: center;
          margin-bottom: 24px;
          display: flex;
          justify-content: center;
          gap: 12px;
          flex-wrap: wrap;
        }

        .analyze-btn {
          padding: 14px 32px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 16px;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .analyze-btn:hover:not(:disabled) {
          transform: translateY(-1px);
        }

        .analyze-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .coaching-btn {
          padding: 14px 24px;
          background: linear-gradient(135deg, #38a169 0%, #2f855a 100%);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .coaching-btn:hover:not(:disabled) {
          transform: translateY(-1px);
        }

        .coaching-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .results-section {
          border-top: 1px solid #e1e5e9;
          padding-top: 24px;
        }

        .results-section h3 {
          margin: 0 0 16px 0;
          color: #4a5568;
          font-size: 18px;
        }

        .success-results {
          background: #f0fff4;
          border: 1px solid #c6f6d5;
          border-radius: 8px;
          padding: 16px;
        }

        .success-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .success-header span:first-child {
          font-size: 18px;
        }

        .success-header span:last-child {
          color: #276749;
          font-weight: 500;
        }

        .error-results {
          background: #fed7d7;
          border: 1px solid #feb2b2;
          border-radius: 8px;
          padding: 16px;
        }

        .error-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .error-text {
          margin: 8px 0 0 0;
          color: #c53030;
          font-size: 14px;
        }

        .results-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 16px;
          margin-top: 16px;
        }

        .result-card {
          background: white;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          padding: 16px;
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .result-header h4 {
          margin: 0;
          color: #2d3748;
          text-transform: capitalize;
        }

        .confidence {
          padding: 4px 8px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 500;
        }

        .confidence.high {
          background: #c6f6d5;
          color: #276749;
        }

        .confidence.medium {
          background: #fef5e7;
          color: #92400e;
        }

        .confidence.low {
          background: #fed7d7;
          color: #c53030;
        }

        .result-content p {
          margin: 0;
          color: #4a5568;
          line-height: 1.5;
        }

        /* Collaboration Styles */
        .multimodal-header {
          position: relative;
        }

        .header-content {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 20px;
        }

        .collaboration-controls {
          display: flex;
          gap: 8px;
          align-items: center;
        }

        .collaboration-toggle {
          padding: 8px 16px;
          border: 2px solid #e1e5e9;
          border-radius: 20px;
          background: white;
          color: #4a5568;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .collaboration-toggle:hover:not(.active) {
          border-color: #3182ce;
        }

        .collaboration-toggle.active {
          background: #3182ce;
          color: white;
          border-color: #3182ce;
        }

        .leave-session-btn {
          padding: 8px 12px;
          border: 1px solid #e53e3e;
          border-radius: 16px;
          background: #fed7d7;
          color: #c53030;
          cursor: pointer;
          font-size: 12px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .leave-session-btn:hover {
          background: #feb2b2;
        }

        .session-info {
          margin-top: 12px;
          padding: 8px 12px;
          background: #f7fafc;
          border-radius: 8px;
          border-left: 4px solid #3182ce;
        }

        .session-name {
          font-weight: 600;
          color: #2d3748;
        }

        .participant-count {
          color: #718096;
          font-size: 14px;
        }

        .collaboration-panel {
          background: #f8f9fa;
          border: 1px solid #e1e5e9;
          border-radius: 12px;
          padding: 20px;
          margin-bottom: 24px;
        }

        .collaboration-panel h3 {
          margin: 0 0 16px 0;
          color: #2d3748;
          font-size: 18px;
        }

        .session-setup {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
        }

        .setup-section h4 {
          margin: 0 0 12px 0;
          color: #4a5568;
          font-size: 16px;
        }

        .setup-section .input-group {
          display: flex;
          gap: 8px;
        }

        .setup-section input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #e1e5e9;
          border-radius: 6px;
          font-size: 14px;
        }

        .create-btn,
        .join-btn {
          padding: 8px 16px;
          border: none;
          border-radius: 6px;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .create-btn {
          background: #3182ce;
          color: white;
        }

        .create-btn:hover:not(:disabled) {
          background: #2c5282;
        }

        .join-btn {
          background: #38a169;
          color: white;
        }

        .join-btn:hover:not(:disabled) {
          background: #2f855a;
        }

        .create-btn:disabled,
        .join-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .active-session .participants-section {
          margin-bottom: 20px;
        }

        .participants-section h4 {
          margin: 0 0 12px 0;
          color: #4a5568;
        }

        .participants-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .participant-item {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          background: white;
          border-radius: 6px;
          border: 1px solid #e1e5e9;
        }

        .status-indicator {
          width: 8px;
          height: 8px;
          border-radius: 50%;
        }

        .status-indicator.online {
          background: #38a169;
        }

        .status-indicator.away {
          background: #dd6b20;
        }

        .status-indicator.offline {
          background: #a0aec0;
        }

        .participant-name {
          flex: 1;
          font-weight: 500;
          color: #2d3748;
        }

        .participant-activity {
          font-size: 12px;
          color: #718096;
        }

        .shared-inputs-section h4 {
          margin: 0 0 12px 0;
          color: #4a5568;
        }

        .shared-inputs-list {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }

        .shared-input-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 6px 12px;
          background: white;
          border-radius: 4px;
          border: 1px solid #e1e5e9;
          font-size: 12px;
        }

        .input-participant {
          font-weight: 500;
          color: #3182ce;
        }

        .input-type {
          background: #edf2f7;
          padding: 2px 6px;
          border-radius: 10px;
          color: #4a5568;
        }

        .input-time {
          color: #718096;
        }

        .collaborative-results-section {
          border-top: 1px solid #e1e5e9;
          padding-top: 24px;
          margin-top: 24px;
        }

        .collaborative-results-section h3 {
          margin: 0 0 16px 0;
          color: #4a5568;
          font-size: 18px;
        }

        .collaborative-results-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
          gap: 16px;
        }

        .collaborative-result-card {
          background: #f8f9fa;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          padding: 16px;
        }

        .collaborative-result-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .participant-info {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .participant-info .participant-name {
          font-weight: 600;
          color: #2d3748;
        }

        .result-time {
          font-size: 12px;
          color: #718096;
        }

        .collab-success-icon {
          font-size: 16px;
          color: #38a169;
        }

        .collab-error-icon {
          font-size: 16px;
          color: #e53e3e;
        }

        .collaborative-result-content p {
          margin: 0 0 8px 0;
          font-size: 14px;
          color: #4a5568;
        }

        .collab-results-summary {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .collab-modality-summary {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 12px;
        }

        .modality-name {
          font-weight: 500;
          color: #4a5568;
        }

        .modality-confidence {
          padding: 2px 6px;
          border-radius: 8px;
          font-size: 11px;
          font-weight: 500;
        }

        .modality-confidence.high {
          background: #c6f6d5;
          color: #276749;
        }

        .modality-confidence.medium {
          background: #fef5e7;
          color: #92400e;
        }

        .modality-confidence.low {
          background: #fed7d7;
          color: #c53030;
        }

        .collaborative-error-content {
          background: #fed7d7;
          border: 1px solid #feb2b2;
          border-radius: 6px;
          padding: 12px;
        }

        @media (max-width: 768px) {
          .multimodal-ai {
            padding: 16px;
          }

          .header-content {
            flex-direction: column;
            align-items: stretch;
            gap: 12px;
          }

          .collaboration-controls {
            justify-content: center;
          }

          .modality-buttons {
            justify-content: center;
          }

          .results-grid {
            grid-template-columns: 1fr;
          }

          .session-setup {
            grid-template-columns: 1fr;
            gap: 16px;
          }

          .collaborative-results-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default MultiModalAI;
