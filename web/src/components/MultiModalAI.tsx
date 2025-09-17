import React, { useState, useCallback, useEffect, Suspense } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Styles
import './MultiModalAI.css';

// Security imports
import DOMPurify from 'dompurify';

// Webview compatibility utilities
const isWebviewEnvironment = (): boolean => {
  return typeof window !== 'undefined' && (
    window.navigator.userAgent.includes('Tauri') ||
    window.navigator.userAgent.includes('WebView') ||
    typeof (window as any).__TAURI__ !== 'undefined'
  );
};

const hasPerformanceMemoryAPI = (): boolean => {
  return typeof performance !== 'undefined' &&
         'memory' in performance &&
         isWebviewEnvironment(); // Only use in webview to avoid security issues
};

// Performance monitoring hook with webview compatibility
const usePerformanceMonitor = (componentName: string) => {
  const [renderCount, setRenderCount] = useState(0);
  const [renderTimes, setRenderTimes] = useState<number[]>([]);
  const [memoryUsage, setMemoryUsage] = useState<number | null>(null);

  useEffect(() => {
    const startTime = performance.now();
    setRenderCount(prev => prev + 1);

    return () => {
      const endTime = performance.now();
      const renderTime = endTime - startTime;
      setRenderTimes(prev => [...prev.slice(-9), renderTime]); // Keep last 10 render times

      // Monitor memory usage if available and in webview environment
      if (hasPerformanceMemoryAPI()) {
        const memory = (performance as { memory: { usedJSHeapSize: number } }).memory;
        setMemoryUsage(memory.usedJSHeapSize);
      }

      // Log performance metrics only in development
      if (process.env.NODE_ENV === 'development' && renderTime > 16.67) { // More than one frame at 60fps
        console.warn(`${componentName} render took ${renderTime.toFixed(2)}ms - potential performance issue`);
      }
    };
  });

  const getAverageRenderTime = () => {
    if (renderTimes.length === 0) return 0;
    return renderTimes.reduce((a, b) => a + b, 0) / renderTimes.length;
  };

  const getPerformanceMetrics = () => ({
    renderCount,
    averageRenderTime: getAverageRenderTime(),
    lastRenderTime: renderTimes[renderTimes.length - 1] || 0,
    memoryUsage,
  });

  return { getPerformanceMetrics };
};

// Lazy loaded components for better performance
const CollaborationPanel = React.lazy(() =>
  import('./CollaborationPanel').catch(() => ({
    default: () => <div>Collaboration panel not available</div>
  }))
);

const ResultsSection = React.lazy(() =>
  import('./ResultsSection').catch(() => ({
    default: () => <div>Results section not available</div>
  }))
);

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

// Security constants
const SECURITY_CONFIG = {
  MAX_FILE_SIZE: 10 * 1024 * 1024, // 10MB
  ALLOWED_FILE_TYPES: ['image/jpeg', 'image/png', 'image/gif', 'image/webp'],
  MAX_INPUT_LENGTH: 10000,
  GENERIC_ERROR_MESSAGE: 'An error occurred. Please try again.',
};

const MODALITY_TYPES = {
  TEXT: 'text',
  IMAGE: 'image',
  AUDIO: 'audio',
  SCREENSHOT: 'screenshot',
  DIAGRAM: 'diagram',
  CODE: 'code',
  MULTIMODAL: 'multimodal',
};

const MultiModalAI: React.FC<MultiModalAIProps> = React.memo(({
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
  const [imageObjectUrl, setImageObjectUrl] = useState<string | null>(null);
  const [audioRecording, setAudioRecording] = useState(false);

  // Collaboration state
  const [collaborationSession, setCollaborationSession] = useState<CollaborationSession | null>(
    null
  );
  const [collaborativeResults, setCollaborativeResults] = useState<CollaborativeResult[]>([]);
  const [sharedInputs, setSharedInputs] = useState<SharedInput[]>([]);
  // Secure ID generation using crypto.getRandomValues
  const generateSecureId = (length: number): string => {
    const array = new Uint8Array(length);
    crypto.getRandomValues(array);
    return Array.from(array, byte => byte.toString(36)).join('').slice(0, length);
  };

  // Input sanitization and validation functions
  const sanitizeInput = useCallback((input: string): string => {
    // Remove potentially dangerous characters and trim
    return input
      .replace(/[<>\"'&]/g, '') // Remove HTML/XML dangerous chars
      .trim()
      .slice(0, SECURITY_CONFIG.MAX_INPUT_LENGTH);
  }, []);

  const validateInput = useCallback((input: string, fieldName: string): { isValid: boolean; error?: string } => {
    if (!input || input.trim().length === 0) {
      return { isValid: false, error: `${fieldName} cannot be empty` };
    }

    if (input.length > SECURITY_CONFIG.MAX_INPUT_LENGTH) {
      return { isValid: false, error: `${fieldName} exceeds maximum length of ${SECURITY_CONFIG.MAX_INPUT_LENGTH} characters` };
    }

    // Check for potentially malicious patterns
    const maliciousPatterns = [
      /<script/i,
      /javascript:/i,
      /on\w+\s*=/i,
      /data:text\/html/i
    ];

    for (const pattern of maliciousPatterns) {
      if (pattern.test(input)) {
        return { isValid: false, error: `${fieldName} contains invalid content` };
      }
    }

    return { isValid: true };
  }, []);

  const [userId] = useState(() => `user_${generateSecureId(9)}`);
  const [username] = useState(() => `User_${generateSecureId(4)}`);
  const [showCollaborationPanel, setShowCollaborationPanel] = useState(false);
  const [sessionNameInput, setSessionNameInput] = useState('');
  const [joinSessionId, setJoinSessionId] = useState('');
  const [audioData, setAudioData] = useState<string>('');
  const [errorState, setErrorState] = useState<{hasError: boolean, message: string, details: string} | null>(null);

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

  const handleTextInputChange = useCallback((value: string) => {
    const sanitized = sanitizeInput(value);
    const validation = validateInput(sanitized, 'Text input');
    if (validation.isValid) {
      setInputText(sanitized);
      setErrorState(null);
    } else {
      setErrorState({
        hasError: true,
        message: 'Input validation failed',
        details: validation.error || 'Invalid input',
      });
    }
  }, [sanitizeInput, validateInput]);

  const handleSessionNameChange = useCallback((value: string) => {
    const sanitized = sanitizeInput(value);
    const validation = validateInput(sanitized, 'Session name');
    if (validation.isValid) {
      setSessionNameInput(sanitized);
      setErrorState(null);
    } else {
      setErrorState({
        hasError: true,
        message: 'Session name validation failed',
        details: validation.error || 'Invalid session name',
      });
    }
  }, [sanitizeInput, validateInput]);

  const handleJoinSessionIdChange = useCallback((value: string) => {
    const sanitized = sanitizeInput(value);
    const validation = validateInput(sanitized, 'Session ID');
    if (validation.isValid) {
      setJoinSessionId(sanitized);
      setErrorState(null);
    } else {
      setErrorState({
        hasError: true,
        message: 'Session ID validation failed',
        details: validation.error || 'Invalid session ID',
      });
    }
  }, [sanitizeInput, validateInput]);

  const validateFile = useCallback((file: File): { isValid: boolean; error?: string } => {
    // Check file size
    if (file.size > SECURITY_CONFIG.MAX_FILE_SIZE) {
      return { isValid: false, error: `File size exceeds ${SECURITY_CONFIG.MAX_FILE_SIZE / (1024 * 1024)}MB limit` };
    }

    // Check file type
    if (!SECURITY_CONFIG.ALLOWED_FILE_TYPES.includes(file.type)) {
      return { isValid: false, error: 'File type not allowed. Only JPEG, PNG, GIF, and WebP images are permitted' };
    }

    // Basic content validation (check file extension matches type)
    const extension = file.name.toLowerCase().split('.').pop();
    const expectedExtensions: { [key: string]: string[] } = {
      'image/jpeg': ['jpg', 'jpeg'],
      'image/png': ['png'],
      'image/gif': ['gif'],
      'image/webp': ['webp']
    };

    if (!expectedExtensions[file.type]?.includes(extension || '')) {
      return { isValid: false, error: 'File extension does not match file type' };
    }

    return { isValid: true };
  }, []);

  const handleFileUpload = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const validation = validateFile(file);
      if (validation.isValid) {
        setInputImage(file);
        setErrorState(null);
      } else {
        setErrorState({
          hasError: true,
          message: 'File upload rejected',
          details: validation.error || 'Unknown error',
        });
      }
    }
  }, [validateFile]);

  // Manage object URL lifecycle for image preview
  useEffect(() => {
    if (inputImage) {
      const url = URL.createObjectURL(inputImage);
      setImageObjectUrl(url);

      // Cleanup previous URL and create new one
      return () => {
        if (imageObjectUrl) {
          URL.revokeObjectURL(imageObjectUrl);
        }
      };
    } else {
      // Clear object URL when no image
      if (imageObjectUrl) {
        URL.revokeObjectURL(imageObjectUrl);
        setImageObjectUrl(null);
      }
    }
  }, [inputImage, imageObjectUrl]);

  // Cleanup object URL on unmount
  useEffect(() => {
    return () => {
      if (imageObjectUrl) {
        URL.revokeObjectURL(imageObjectUrl);
      }
    };
  }, [imageObjectUrl]);

  const [mediaRecorder, setMediaRecorder] = useState<MediaRecorder | null>(null);
  const [audioChunks, setAudioChunks] = useState<Blob[]>([]);
  const [audioStream, setAudioStream] = useState<MediaStream | null>(null);

  const startAudioRecording = useCallback(async () => {
    try {
      // Request microphone access
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      setAudioStream(stream);

      // Create media recorder
      const recorder = new MediaRecorder(stream);
      const chunks: Blob[] = [];

      // Handle data available event
      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          chunks.push(event.data);
        }
      };

      // Handle recording stop
      recorder.onstop = () => {
        setAudioChunks(chunks);
        const audioBlob = new Blob(chunks, { type: 'audio/wav' });
        // Convert to base64 for backend processing
        const reader = new FileReader();
        reader.onloadend = () => {
          const base64data = reader.result as string;
          // Store the audio data in state for processing
          setAudioData(base64data);
        };
        reader.readAsDataURL(audioBlob);
      };

      // Start recording
      recorder.start();
      setMediaRecorder(recorder);
      setAudioRecording(true);
      console.log('Audio recording started');
    } catch (error) {
      console.error('Error accessing microphone:', error);
      // Handle error state
      setErrorState({
        hasError: true,
        message: 'Failed to access microphone. Please check permissions.',
        details: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  }, []);

  const stopAudioRecording = useCallback(() => {
    if (mediaRecorder && audioRecording) {
      mediaRecorder.stop();
      setAudioRecording(false);

      // Stop all tracks in the stream
      if (audioStream) {
        audioStream.getTracks().forEach((track) => track.stop());
        setAudioStream(null);
      }

      console.log('Audio recording stopped and processed');
    }
  }, [mediaRecorder, audioRecording, audioStream]);

  // Clean up resources on unmount
  useEffect(() => {
    return () => {
      if (audioStream) {
        audioStream.getTracks().forEach((track) => track.stop());
      }
    };
  }, [audioStream]);

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
      // Secure error handling - prevent information disclosure
      console.error('Multi-modal analysis failed:', error);
      const errorResult: AnalysisResponse = {
        success: false,
        results: [],
        processing_time: 0,
        error: SECURITY_CONFIG.GENERIC_ERROR_MESSAGE, // Generic error message to prevent info disclosure
      };
      setAnalysisResult(errorResult);
      onAnalysisComplete?.(errorResult);

      // Share error result in collaboration session
      if (collaborationSession) {
        await shareAnalysisResult(errorResult);
      }

      // Set secure error state
      setErrorState({
        hasError: true,
        message: 'Analysis failed',
        details: SECURITY_CONFIG.GENERIC_ERROR_MESSAGE,
      });
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

  const getModalityColor = useCallback((modality: string): string => {
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
  }, []);

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

      {/* Collaboration Panel - Lazy Loaded */}
      {showCollaborationPanel && (
        <Suspense fallback={<div className="loading-panel">Loading collaboration...</div>}>
          <CollaborationPanel
            collaborationSession={collaborationSession}
            userId={userId}
            sessionNameInput={sessionNameInput}
            joinSessionId={joinSessionId}
            sharedInputs={sharedInputs}
            onSessionNameChange={handleSessionNameChange}
            onJoinSessionIdChange={handleJoinSessionIdChange}
            onCreateSession={createCollaborationSession}
            onJoinSession={joinCollaborationSession}
          />
        </Suspense>
      )}

      {/* Input Section */}
      <div className="input-section">
        {selectedModalities.has(MODALITY_TYPES.TEXT) && (
          <div className="input-group">
            <label>Text Input</label>
            <textarea
              value={inputText}
              onChange={(e) => handleTextInputChange(e.target.value)}
              placeholder="Enter text to analyze..."
              rows={4}
            />
          </div>
        )}

        {selectedModalities.has(MODALITY_TYPES.IMAGE) && (
          <div className="input-group">
            <label>Image Upload</label>
            <input type="file" accept="image/*" onChange={handleFileUpload} />
            {inputImage && imageObjectUrl && (
              <div className="image-preview">
                <img
                  src={imageObjectUrl}
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

      {/* Results Section - Lazy Loaded */}
      {analysisResult && (
        <Suspense fallback={<div className="loading-results">Loading results...</div>}>
          <ResultsSection
            analysisResult={analysisResult}
            collaborativeResults={collaborativeResults}
            getModalityColor={getModalityColor}
          />
        </Suspense>
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

    </div>
  );
});

MultiModalAI.displayName = 'MultiModalAI';

export default MultiModalAI;
