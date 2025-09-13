import React, { useState, useCallback } from 'react';
import { useAppDispatch } from '../../store/hooks';
import { AIFeedback, AIFeedbackType } from '../../types/ai';

/**
 * AI Feedback Panel Component
 * Collects user feedback on AI-generated code for continuous learning
 */
interface AIFeedbackPanelProps {
  generationId: string;
  onSubmitFeedback: (feedback: AIFeedback) => void;
  onClose: () => void;
}

const AIFeedbackPanel: React.FC<AIFeedbackPanelProps> = ({
  generationId,
  onSubmitFeedback,
  onClose,
}) => {
  const [rating, setRating] = useState<number>(3);
  const [feedbackType, setFeedbackType] = useState<AIFeedbackType>('general');
  const [corrections, setCorrections] = useState<string[]>(['']);
  const [suggestions, setSuggestions] = useState<string[]>(['']);
  const [comments, setComments] = useState('');
  const [wouldUseAgain, setWouldUseAgain] = useState<boolean | null>(null);

  const handleRatingChange = useCallback((newRating: number) => {
    setRating(newRating);
  }, []);

  const handleCorrectionChange = useCallback(
    (index: number, value: string) => {
      const newCorrections = [...corrections];
      newCorrections[index] = value;
      setCorrections(newCorrections);
    },
    [corrections]
  );

  const addCorrection = useCallback(() => {
    setCorrections([...corrections, '']);
  }, [corrections]);

  const removeCorrection = useCallback(
    (index: number) => {
      if (corrections.length > 1) {
        const newCorrections = corrections.filter((_, i) => i !== index);
        setCorrections(newCorrections);
      }
    },
    [corrections]
  );

  const handleSuggestionChange = useCallback(
    (index: number, value: string) => {
      const newSuggestions = [...suggestions];
      newSuggestions[index] = value;
      setSuggestions(newSuggestions);
    },
    [suggestions]
  );

  const addSuggestion = useCallback(() => {
    setSuggestions([...suggestions, '']);
  }, [suggestions]);

  const removeSuggestion = useCallback(
    (index: number) => {
      if (suggestions.length > 1) {
        const newSuggestions = suggestions.filter((_, i) => i !== index);
        setSuggestions(newSuggestions);
      }
    },
    [suggestions]
  );

  const handleSubmit = useCallback(() => {
    const feedback: AIFeedback = {
      id: `feedback_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      generationId,
      timestamp: Date.now(),
      rating,
      feedbackType,
      corrections: corrections.filter((c) => c.trim()),
      suggestions: suggestions.filter((s) => s.trim()),
      comments: comments.trim(),
      wouldUseAgain,
      userContext: {
        userId: 'anonymous', // Would be populated from auth system
        sessionId: 'current_session',
        environment: 'ide',
      },
      metadata: {
        browser: navigator.userAgent,
        platform: navigator.platform,
        language: navigator.language,
      },
    };

    onSubmitFeedback(feedback);
    onClose();
  }, [
    generationId,
    rating,
    feedbackType,
    corrections,
    suggestions,
    comments,
    wouldUseAgain,
    onSubmitFeedback,
    onClose,
  ]);

  const getRatingColor = (starRating: number): string => {
    if (starRating >= 4) return '#10b981';
    if (starRating >= 3) return '#f59e0b';
    return '#ef4444';
  };

  const getRatingEmoji = (starRating: number): string => {
    if (starRating >= 4) return '😊';
    if (starRating >= 3) return '😐';
    return '😞';
  };

  return (
    <div className="ai-feedback-panel">
      <div className="feedback-header">
        <h3>Help Improve AI Code Generation</h3>
        <p>Your feedback helps make the AI smarter!</p>
        <button className="close-btn" onClick={onClose}>
          ×
        </button>
      </div>

      <div className="feedback-content">
        {/* Overall Rating */}
        <div className="feedback-section">
          <h4>Overall Rating</h4>
          <div className="rating-container">
            <div className="stars">
              {[1, 2, 3, 4, 5].map((star) => (
                <button
                  key={star}
                  className={`star ${star <= rating ? 'active' : ''}`}
                  onClick={() => handleRatingChange(star)}
                  style={{ color: star <= rating ? getRatingColor(star) : '#d1d5db' }}
                >
                  ★
                </button>
              ))}
            </div>
            <div className="rating-text">
              <span className="rating-emoji">{getRatingEmoji(rating)}</span>
              <span className="rating-label">
                {rating === 5
                  ? 'Excellent'
                  : rating === 4
                    ? 'Good'
                    : rating === 3
                      ? 'Okay'
                      : rating === 2
                        ? 'Poor'
                        : 'Very Poor'}
              </span>
            </div>
          </div>
        </div>

        {/* Feedback Type */}
        <div className="feedback-section">
          <h4>Feedback Type</h4>
          <div className="feedback-type-selector">
            {[
              { value: 'general', label: 'General Feedback', icon: '💬' },
              { value: 'correction', label: 'Code Corrections', icon: '🔧' },
              { value: 'suggestion', label: 'Suggestions', icon: '💡' },
              { value: 'bug', label: 'Bug Report', icon: '🐛' },
            ].map((type) => (
              <button
                key={type.value}
                className={`type-btn ${feedbackType === type.value ? 'active' : ''}`}
                onClick={() => setFeedbackType(type.value as AIFeedbackType)}
              >
                <span className="type-icon">{type.icon}</span>
                <span className="type-label">{type.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Corrections */}
        <div className="feedback-section">
          <div className="section-header">
            <h4>Corrections Made</h4>
            <button className="add-btn" onClick={addCorrection}>
              + Add Correction
            </button>
          </div>
          <div className="corrections-list">
            {corrections.map((correction, index) => (
              <div key={index} className="correction-item">
                <textarea
                  value={correction}
                  onChange={(e) => handleCorrectionChange(index, e.target.value)}
                  placeholder="Describe the correction you made..."
                  rows={2}
                  className="correction-input"
                />
                {corrections.length > 1 && (
                  <button
                    className="remove-btn"
                    onClick={() => removeCorrection(index)}
                    title="Remove correction"
                  >
                    ×
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Suggestions */}
        <div className="feedback-section">
          <div className="section-header">
            <h4>Suggestions for Improvement</h4>
            <button className="add-btn" onClick={addSuggestion}>
              + Add Suggestion
            </button>
          </div>
          <div className="suggestions-list">
            {suggestions.map((suggestion, index) => (
              <div key={index} className="suggestion-item">
                <textarea
                  value={suggestion}
                  onChange={(e) => handleSuggestionChange(index, e.target.value)}
                  placeholder="What could be improved..."
                  rows={2}
                  className="suggestion-input"
                />
                {suggestions.length > 1 && (
                  <button
                    className="remove-btn"
                    onClick={() => removeSuggestion(index)}
                    title="Remove suggestion"
                  >
                    ×
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Comments */}
        <div className="feedback-section">
          <h4>Additional Comments</h4>
          <textarea
            value={comments}
            onChange={(e) => setComments(e.target.value)}
            placeholder="Any additional thoughts or context..."
            rows={3}
            className="comments-input"
          />
        </div>

        {/* Would Use Again */}
        <div className="feedback-section">
          <h4>Would you use this generated code?</h4>
          <div className="usage-options">
            <button
              className={`usage-btn ${wouldUseAgain === true ? 'active' : ''}`}
              onClick={() => setWouldUseAgain(true)}
            >
              ✅ Yes, as-is
            </button>
            <button
              className={`usage-btn ${wouldUseAgain === false ? 'active' : ''}`}
              onClick={() => setWouldUseAgain(false)}
            >
              🔧 With modifications
            </button>
            <button
              className={`usage-btn ${wouldUseAgain === null ? 'active' : ''}`}
              onClick={() => setWouldUseAgain(null)}
            >
              ❌ No, prefer manual
            </button>
          </div>
        </div>

        {/* Submit Actions */}
        <div className="feedback-actions">
          <button className="submit-btn" onClick={handleSubmit}>
            🚀 Submit Feedback
          </button>
          <button className="skip-btn" onClick={onClose}>
            Skip for now
          </button>
        </div>
      </div>

      <style jsx>{`
        .ai-feedback-panel {
          position: fixed;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          background: white;
          border-radius: 12px;
          box-shadow:
            0 20px 25px -5px rgba(0, 0, 0, 0.1),
            0 10px 10px -5px rgba(0, 0, 0, 0.04);
          max-width: 600px;
          width: 90%;
          max-height: 80vh;
          overflow-y: auto;
          z-index: 1000;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .feedback-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          padding: 24px 24px 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .feedback-header h3 {
          margin: 0 0 8px 0;
          color: #111827;
          font-size: 20px;
        }

        .feedback-header p {
          margin: 0;
          color: #6b7280;
          font-size: 14px;
        }

        .close-btn {
          background: none;
          border: none;
          font-size: 24px;
          cursor: pointer;
          color: #6b7280;
          padding: 4px;
          border-radius: 4px;
        }

        .close-btn:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .feedback-content {
          padding: 24px;
        }

        .feedback-section {
          margin-bottom: 24px;
        }

        .feedback-section h4 {
          margin: 0 0 12px 0;
          color: #111827;
          font-size: 16px;
        }

        .rating-container {
          display: flex;
          align-items: center;
          gap: 16px;
        }

        .stars {
          display: flex;
          gap: 4px;
        }

        .star {
          font-size: 24px;
          background: none;
          border: none;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .star:hover {
          transform: scale(1.1);
        }

        .star.active {
          color: #fbbf24;
        }

        .rating-text {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .rating-emoji {
          font-size: 20px;
        }

        .rating-label {
          font-weight: 500;
          color: #374151;
        }

        .feedback-type-selector {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
          gap: 8px;
        }

        .type-btn {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          background: white;
          cursor: pointer;
          transition: all 0.2s;
        }

        .type-btn:hover:not(.active) {
          border-color: #d1d5db;
          background: #f9fafb;
        }

        .type-btn.active {
          border-color: #3b82f6;
          background: #eff6ff;
          color: #1d4ed8;
        }

        .type-icon {
          font-size: 16px;
        }

        .type-label {
          font-size: 14px;
          font-weight: 500;
        }

        .section-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .add-btn {
          padding: 6px 12px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
        }

        .add-btn:hover {
          background: #2563eb;
        }

        .corrections-list,
        .suggestions-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .correction-item,
        .suggestion-item {
          display: flex;
          gap: 8px;
          align-items: flex-start;
        }

        .correction-input,
        .suggestion-input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
          resize: vertical;
        }

        .comments-input {
          width: 100%;
          padding: 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
          resize: vertical;
        }

        .remove-btn {
          padding: 8px;
          background: #ef4444;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          width: 32px;
          height: 32px;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }

        .remove-btn:hover {
          background: #dc2626;
        }

        .usage-options {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }

        .usage-btn {
          padding: 8px 16px;
          border: 2px solid #e5e7eb;
          border-radius: 6px;
          background: white;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.2s;
        }

        .usage-btn:hover:not(.active) {
          border-color: #d1d5db;
          background: #f9fafb;
        }

        .usage-btn.active {
          border-color: #10b981;
          background: #f0fdf4;
          color: #059669;
        }

        .feedback-actions {
          display: flex;
          gap: 12px;
          justify-content: flex-end;
          margin-top: 32px;
          padding-top: 24px;
          border-top: 1px solid #e5e7eb;
        }

        .submit-btn {
          padding: 12px 24px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          border-radius: 6px;
          font-size: 16px;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .submit-btn:hover {
          transform: translateY(-1px);
        }

        .skip-btn {
          padding: 12px 24px;
          background: #f3f4f6;
          color: #374151;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 16px;
          cursor: pointer;
        }

        .skip-btn:hover {
          background: #e5e7eb;
        }

        @media (max-width: 640px) {
          .ai-feedback-panel {
            width: 95%;
            margin: 20px;
          }

          .feedback-header {
            flex-direction: column;
            gap: 12px;
          }

          .feedback-type-selector {
            grid-template-columns: 1fr;
          }

          .usage-options {
            flex-direction: column;
          }

          .feedback-actions {
            flex-direction: column;
          }

          .rating-container {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }
        }
      `}</style>
    </div>
  );
};

export default AIFeedbackPanel;
