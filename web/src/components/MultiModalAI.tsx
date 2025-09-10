import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// NOTE: Following project rules - no React hooks for external state libraries
// Using useState as allowed, but no external hooks

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
  MULTIMODAL: 'multimodal'
};

const MultiModalAI: React.FC<MultiModalAIProps> = ({
  onAnalysisComplete,
  isFullscreen = false
}) => {
  const [loading, setLoading] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResponse | null>(null);
  const [selectedModalities, setSelectedModalities] = useState<Set<string>>(new Set([MODALITY_TYPES.TEXT]));
  const [inputText, setInputText] = useState('');
  const [inputImage, setInputImage] = useState<File | null>(null);
  const [audioRecording, setAudioRecording] = useState(false);

  const handleModalityToggle = useCallback((modality: string) => {
    const newSelected = new Set(selectedModalities);
    if (newSelected.has(modality)) {
      newSelected.delete(modality);
    } else {
      newSelected.add(modality);
    }
    setSelectedModalities(newSelected);
  }, [selectedModalities]);

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
        modality_types: Array.from(selectedModalities)
      };

      // Add text content if text modality selected
      if (selectedModalities.has(MODALITY_TYPES.TEXT) && inputText.trim()) {
        requestData.text_content = inputText.trim();
      }

      // Add image data if image uploaded
      if (inputImage && selectedModalities.has(MODALITY_TYPES.IMAGE)) {
        // Convert file to base64 (simple approach - in reality might chunk large files)
        const imageData = await fileToBase64(inputImage);
        requestData.image_data = imageData;
      }

      // Add audio data if screenshot requested (placeholder)
      if (selectedModalities.has(MODALITY_TYPES.SCREENSHOT)) {
        // Camera/screenshot API limited in webview
        // Would need backend to capture screenshot
      }

      console.log('Processing multimodal analysis:', requestData);

      // Call backend multimodal AI service
      const result: AnalysisResponse = await invoke('process_multimodal_analysis', {
        requestData
      });

      setAnalysisResult(result);
      onAnalysisComplete?.(result);

    } catch (error) {
      console.error('Multi-modal analysis failed:', error);
      const errorResult: AnalysisResponse = {
        success: false,
        results: [],
        processing_time: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
      setAnalysisResult(errorResult);
      onAnalysisComplete?.(errorResult);
    } finally {
      setLoading(false);
    }
  }, [selectedModalities, inputText, inputImage, onAnalysisComplete]);

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
      case MODALITY_TYPES.TEXT: return '#3182ce';
      case MODALITY_TYPES.IMAGE: return '#38a169';
      case MODALITY_TYPES.AUDIO: return '#dd6b20';
      case MODALITY_TYPES.SCREENSHOT: return '#9f7aea';
      case MODALITY_TYPES.DIAGRAM: return '#e53e3e';
      case MODALITY_TYPES.CODE: return '#805ad5';
      default: return '#4a5568';
    }
  };

  return (
    <div className={`multimodal-ai ${isFullscreen ? 'fullscreen' : ''}`}>
      <div className="multimodal-header">
        <h2>Multi-Modal AI Analysis</h2>
        <p>Analyze text, images, audio, and screenshots with AI</p>
      </div>

      {/* Modality Selection */}
      <div className="modality-selection">
        <h3>Choose Analysis Types</h3>
        <div className="modality-buttons">
          {Object.entries(MODALITY_TYPES).map(([key, value]) => (
            key !== 'MULTIMODAL' && (
              <button
                key={value}
                className={`modality-btn ${selectedModalities.has(value) ? 'active' : ''}`}
                onClick={() => handleModalityToggle(value)}
                style={{
                  backgroundColor: selectedModalities.has(value) ? getModalityColor(value) : undefined,
                }}
              >
                {key}
              </button>
            )
          ))}
        </div>
      </div>

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
            <input
              type="file"
              accept="image/*"
              onChange={handleFileUpload}
            />
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
      </div>

      {/* Results Section */}
      {analysisResult && (
        <div className="results-section">
          <h3>Analysis Results</h3>

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
                      <span className={`confidence ${result.confidence > 0.8 ? 'high' : result.confidence > 0.6 ? 'medium' : 'low'}`}>
                        {Math.round(result.confidence * 100)}%
                      </span>
                    </div>

                    {result.success ? (
                      <div className="result-content">
                        {result.modality_type === 'text' && typeof result.data === 'object' && 'content' in result.data && (
                          <p>{(result.data as { content: string }).content}</p>
                        )}
                        {result.modality_type === 'image' && typeof result.data === 'object' && 'description' in result.data && (
                          <p>{(result.data as { description: string }).description}</p>
                        )}
                        {result.data_length && (
                          <p>Data processed: {result.data_length} bytes</p>
                        )}
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

        .input-group input[type="file"] {
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

        @media (max-width: 768px) {
          .multimodal-ai {
            padding: 16px;
          }

          .modality-buttons {
            justify-content: center;
          }

          .results-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default MultiModalAI;