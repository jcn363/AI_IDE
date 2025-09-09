import React, { useState, useEffect } from 'react';
import type {
  ArchitecturalRecommendation,
  ArchitecturalDecision,
  RiskAssessment,
  PriorityAction,
  ArchitecturalRoadmap,
  DecisionStatus
} from '../types';

interface ArchitecturalAdvisorPanelProps {
  className?: string;
}

export const ArchitecturalAdvisorPanel: React.FC<ArchitecturalAdvisorPanelProps> = ({ className }) => {
  const [analysis, setAnalysis] = useState<string>('');
  const [recommendations, setRecommendations] = useState<ArchitecturalRecommendation[]>([]);
  const [decisions, setDecisions] = useState<ArchitecturalDecision[]>([]);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [activeTab, setActiveTab] = useState<'input' | 'recommendations' | 'decisions'>('input');
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!analysis.trim()) {
      setError('Please enter code or architecture description for analysis');
      return;
    }

    setIsAnalyzing(true);
    setError(null);

    try {
      // This would call the actual Tauri command
      // For now, create mock recommendations
      await new Promise(resolve => setTimeout(resolve, 1500));

      const mockRecommendations: ArchitecturalRecommendation = {
        primary_recommendations: [
          {
            title: 'Implement Repository Pattern',
            description: 'Separate data access logic into dedicated repository classes to improve testability and maintainability.',
            impact: 'High',
            effort: 'Medium',
            rationale: 'This pattern will decouple business logic from data access, making the system more modular and easier to test.'
          },
          {
            title: 'Add Comprehensive Error Handling',
            description: 'Implement structured error handling with custom error types and proper error propagation.',
            impact: 'Medium',
            effort: 'Low',
            rationale: 'Proper error handling improves system reliability and provides better debugging capabilities.'
          }
        ],
        secondary_suggestions: [
          {
            title: 'Consider adding caching layer',
            description: 'Implement caching for frequently accessed data to improve performance.',
            rationale: 'This will reduce database load and improve response times.'
          }
        ],
        risk_assessment: {
          overall_risk: 0.15,
          risk_factors: [
            'Potential breaking changes during refactoring',
            'Increased complexity with new architectural patterns'
          ],
          mitigation_strategies: [
            'Implement changes incrementally',
            'Maintain backward compatibility during transition'
          ]
        },
        priority_actions: [
          {
            action: 'Create repository interfaces',
            timeline: 'Week 1',
            rationale: 'Foundation for data access abstraction'
          },
          {
            action: 'Implement error handling patterns',
            timeline: 'Week 1-2',
            rationale: 'Improve system reliability immediately'
          }
        ],
        roadmap: {
          short_term: [
            'Implement repository pattern for data access',
            'Add proper error handling throughout the application'
          ],
          medium_term: [
            'Introduce dependency injection container',
            'Implement comprehensive logging and monitoring'
          ],
          long_term: [
            'Consider microservices architecture if system grows significantly',
            'Implement distributed caching and high availability patterns'
          ]
        }
      };

      setRecommendations([mockRecommendations]);
      setActiveTab('recommendations');

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Analysis failed');
    } finally {
      setIsAnalyzing(false);
    }
  };

  const renderInputTab = () => (
    <div className="architecture-input">
      <div className="input-section">
        <h4>Code/Architecture Analysis</h4>
        <textarea
          value={analysis}
          onChange={(e) => setAnalysis(e.target.value)}
          placeholder="Paste your code or describe your current architecture here...

Example:
'We have a monolithic Rust application with the following structure:
- Main application with HTTP server
- Database layer using Diesel ORM
- Business logic mixed with data access
- No testing infrastructure
- Growing user base requiring better architecture

What architectural patterns should we consider?'"
          rows={20}
          disabled={isAnalyzing}
        />
      </div>

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className="action-buttons">
        <button
          onClick={handleAnalyze}
          disabled={isAnalyzing || !analysis.trim()}
          className="analyze-button"
        >
          {isAnalyzing ? 'Analyzing...' : 'Analyze Architecture'}
        </button>
      </div>
    </div>
  );

  const renderRecommendationsTab = () => {
    if (recommendations.length === 0) {
      return <div className="no-recommendations">No recommendations available</div>;
    }

    const recommendation = recommendations[0];

    return (
      <div className="recommendations-content">
        {/* Primary Recommendations */}
        <div className="primary-recommendations">
          <h4>Primary Recommendations</h4>
          {recommendation.primary_recommendations.map((rec, index) => (
            <div key={index} className="recommendation-card">
              <div className="recommendation-header">
                <h5>{rec.title}</h5>
                <div className="badges">
                  <span className={`badge impact-${rec.impact.toLowerCase()}`}>
                    {rec.impact} Impact
                  </span>
                  <span className={`badge effort-${rec.effort.toLowerCase()}`}>
                    {rec.effort} Effort
                  </span>
                </div>
              </div>
              <p className="description">{rec.description}</p>
              <div className="rationale">
                <strong>Rationale:</strong> {rec.rationale}
              </div>
            </div>
          ))}
        </div>

        {/* Secondary Suggestions */}
        <div className="secondary-suggestions">
          <h4>Additional Suggestions</h4>
          {recommendation.secondary_suggestions.map((suggestion, index) => (
            <div key={index} className="suggestion-item">
              <h5>{suggestion.title}</h5>
              <p>{suggestion.description}</p>
              <div className="rationale">
                <strong>Why:</strong> {suggestion.rationale}
              </div>
            </div>
          ))}
        </div>

        {/* Risk Assessment */}
        <div className="risk-assessment">
          <h4>Risk Assessment</h4>
          <div className="risk-summary">
            <div className="overall-risk">
              <span className="risk-label">Overall Risk Level:</span>
              <span className={`risk-value risk-${recommendation.risk_assessment.overall_risk < 0.3 ? 'low' : recommendation.risk_assessment.overall_risk < 0.6 ? 'medium' : 'high'}`}>
                {Math.round(recommendation.risk_assessment.overall_risk * 100)}%
              </span>
            </div>
          </div>

          <div className="risk-details">
            <div className="risk-factors">
              <h5>Risk Factors:</h5>
              <ul>
                {recommendation.risk_assessment.risk_factors.map((factor, index) => (
                  <li key={index}>{factor}</li>
                ))}
              </ul>
            </div>

            <div className="mitigation">
              <h5>Mitigation Strategies:</h5>
              <ul>
                {recommendation.risk_assessment.mitigation_strategies.map((strategy, index) => (
                  <li key={index}>{strategy}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>

        {/* Priority Actions */}
        <div className="priority-actions">
          <h4>Implementation Priority</h4>
          {recommendation.priority_actions.map((action, index) => (
            <div key={index} className="action-item">
              <div className="action-header">
                <span className="action-title">{action.action}</span>
                <span className="timeline">{action.timeline}</span>
              </div>
              <p className="action-rationale">{action.rationale}</p>
            </div>
          ))}
        </div>

        {/* Architectural Roadmap */}
        <div className="roadmap">
          <h4>Architectural Roadmap</h4>

          <div className="roadmap-section">
            <h5>Short-term (1-3 months)</h5>
            <ul>
              {recommendation.roadmap.short_term.map((item, index) => (
                <li key={index}>{item}</li>
              ))}
            </ul>
          </div>

          <div className="roadmap-section">
            <h5>Medium-term (3-6 months)</h5>
            <ul>
              {recommendation.roadmap.medium_term.map((item, index) => (
                <li key={index}>{item}</li>
              ))}
            </ul>
          </div>

          <div className="roadmap-section">
            <h5>Long-term (6+ months)</h5>
            <ul>
              {recommendation.roadmap.long_term.map((item, index) => (
                <li key={index}>{item}</li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    );
  };

  const renderDecisionsTab = () => (
    <div className="decisions-content">
      <div className="decisions-header">
        <h4>Architectural Decisions</h4>
        <button className="new-decision-button">+ New Decision</button>
      </div>

      {decisions.length === 0 ? (
        <div className="no-decisions">
          <p>No architectural decisions recorded yet.</p>
          <p>Use the "New Decision" button to document important architecture choices.</p>
        </div>
      ) : (
        <div className="decisions-list">
          {decisions.map((decision) => (
            <div key={decision.id} className="decision-card">
              <div className="decision-header">
                <h5>{decision.title}</h5>
                <span className={`decision-status status-${decision.status.toLowerCase()}`}>
                  {decision.status}
                </span>
              </div>

              <div className="decision-content">
                <div className="decision-problem">
                  <strong>Problem:</strong> {decision.problem}
                </div>

                <div className="decision-solution">
                  <strong>Solution:</strong> {decision.solution_choice}
                </div>

                {decision.consequences.length > 0 && (
                  <div className="decision-consequences">
                    <strong>Consequences:</strong>
                    <ul>
                      {decision.consequences.map((consequence, index) => (
                        <li key={index}>{consequence}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );

  return (
    <div className={`architectural-advisor-panel ${className || ''}`}>
      <div className="panel-header">
        <h3>Architectural Advisor</h3>
        <p className="panel-subtitle">Get architectural recommendations and document design decisions</p>
      </div>

      <div className="panel-tabs">
        <button
          className={activeTab === 'input' ? 'active' : ''}
          onClick={() => setActiveTab('input')}
        >
          Analysis Input
        </button>
        <button
          className={activeTab === 'recommendations' && recommendations.length > 0 ? 'active' : ''}
          onClick={() => recommendations.length > 0 && setActiveTab('recommendations')}
        >
          Recommendations
        </button>
        <button
          className={activeTab === 'decisions' ? 'active' : ''}
          onClick={() => setActiveTab('decisions')}
        >
          Decisions
        </button>
      </div>

      <div className="panel-content">
        {activeTab === 'input' && renderInputTab()}
        {activeTab === 'recommendations' && renderRecommendationsTab()}
        {activeTab === 'decisions' && renderDecisionsTab()}
      </div>

      <style jsx>{`
        .architectural-advisor-panel {
          padding: 20px;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          background: #ffffff;
          margin: 20px 0;
        }

        .panel-header {
          margin-bottom: 20px;
          padding-bottom: 15px;
          border-bottom: 1px solid #e1e5e9;
        }

        .panel-header h3 {
          margin: 0 0 8px 0;
          color: #2d3748;
          font-size: 24px;
          font-weight: 600;
        }

        .panel-subtitle {
          margin: 0;
          color: #718096;
          font-size: 16px;
        }

        .panel-tabs {
          display: flex;
          margin-bottom: 20px;
        }

        .panel-tabs button {
          padding: 10px 20px;
          border: none;
          background: #f7fafc;
          color: #4a5568;
          cursor: pointer;
          transition: background-color 0.2s, color 0.2s;
        }

        .panel-tabs button.active {
          background: #3182ce;
          color: white;
        }

        .panel-content {
          background: #f7fafc;
          border-radius: 6px;
          padding: 20px;
          min-height: 400px;
        }

        .architecture-input {
          display: flex;
          flex-direction: column;
          gap: 20px;
        }

        .input-section h4 {
          margin: 0 0 12px 0;
          color: #2d3748;
          font-size: 18px;
          font-weight: 600;
        }

        .input-section textarea {
          width: 100%;
          padding: 12px;
          border: 1px solid #e1e5e9;
          border-radius: 4px;
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          font-size: 14px;
          line-height: 1.5;
          resize: vertical;
        }

        .error-message {
          padding: 12px;
          background: #fed7d7;
          border: 1px solid #fc8181;
          border-radius: 4px;
          color: #c53030;
        }

        .action-buttons {
          display: flex;
          justify-content: center;
          padding-top: 20px;
        }

        .analyze-button {
          background: #3182ce;
          color: white;
          border: none;
          padding: 12px 24px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 16px;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .analyze-button:hover:not(:disabled) {
          background: #2c5282;
        }

        .analyze-button:disabled {
          background: #cbd5e0;
          cursor: not-allowed;
        }

        .no-recommendations, .no-decisions {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          color: #718096;
          text-align: center;
        }

        .no-recommendations p, .no-decisions p {
          margin: 0 0 8px 0;
          font-size: 16px;
        }

        .recommendations-content {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }

        .recommendations-content h4 {
          margin: 0 0 16px 0;
          color: #2d3748;
          font-size: 18px;
          border-bottom: 1px solid #e1e5e9;
          padding-bottom: 8px;
        }

        .primary-recommendations, .secondary-suggestions,
        .risk-assessment, .priority-actions, .roadmap {
          background: white;
          padding: 16px;
          border-radius: 6px;
          border: 1px solid #e1e5e9;
        }

        .recommendation-card, .suggestion-item {
          margin-bottom: 16px;
          padding-bottom: 16px;
          border-bottom: 1px solid #f1f5f9;
        }

        .recommendation-card:last-child, .suggestion-item:last-child {
          margin-bottom: 0;
          padding-bottom: 0;
          border-bottom: none;
        }

        .recommendation-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .recommendation-header h5, .suggestion-item h5 {
          margin: 0;
          color: #2d3748;
          font-size: 16px;
          font-weight: 600;
        }

        .badges {
          display: flex;
          gap: 8px;
        }

        .badge {
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
          text-transform: uppercase;
        }

        .badge.impact-high { background: #c6f6d5; color: #276749; }
        .badge.impact-medium { background: #fef5e7; color: #d69e2e; }
        .badge.impact-low { background: #bee3f8; color: #2c3338; }

        .badge.effort-high { background: #fed7d7; color: #c53030; }
        .badge.effort-medium { background: #feebc8; color: #7b341e; }
        .badge.effort-low { background: #c6f6d5; color: #276749; }

        .description, .rationale {
          color: #4a5568;
          line-height: 1.5;
          margin-bottom: 8px;
        }

        .rationale {
          font-style: italic;
          background: #f7fafc;
          padding: 8px 12px;
          border-radius: 4px;
          border-left: 4px solid #3182ce;
        }

        .risk-summary {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .overall-risk {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .risk-label {
          font-weight: 500;
          color: #4a5568;
        }

        .risk-value {
          font-weight: 600;
          font-size: 16px;
        }

        .risk-value.risk-low { color: #38a169; }
        .risk-value.risk-medium { color: #d69e2e; }
        .risk-value.risk-high { color: #e53e3e; }

        .risk-details {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 20px;
        }

        .risk-factors h5, .mitigation h5 {
          margin: 0 0 8px 0;
          color: #2d3748;
          font-size: 14px;
          font-weight: 600;
        }

        .risk-factors ul, .mitigation ul {
          margin: 0;
          padding-left: 20px;
          color: #4a5568;
        }

        .priority-actions .action-item {
          margin-bottom: 12px;
          padding-bottom: 12px;
          border-bottom: 1px solid #f1f5f9;
        }

        .priority-actions .action-item:last-child {
          margin-bottom: 0;
          padding-bottom: 0;
          border-bottom: none;
        }

        .action-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .action-title {
          font-weight: 500;
          color: #2d3748;
        }

        .timeline {
          padding: 4px 8px;
          background: #edf2f7;
          border-radius: 4px;
          font-size: 12px;
          color: #4a5568;
          font-weight: 500;
        }

        .action-rationale {
          color: #718096;
          font-style: italic;
          margin: 0;
        }

        .roadmap .roadmap-section {
          margin-bottom: 16px;
          padding-bottom: 16px;
          border-bottom: 1px solid #f1f5f9;
        }

        .roadmap .roadmap-section:last-child {
          margin-bottom: 0;
          padding-bottom: 0;
          border-bottom: none;
        }

        .roadmap .roadmap-section h5 {
          margin: 0 0 8px 0;
          color: #2d3748;
          font-size: 14px;
          font-weight: 600;
        }

        .roadmap .roadmap-section ul {
          margin: 0;
          padding-left: 20px;
          color: #4a5568;
        }

        /* Decisions Tab Styles */
        .decisions-content {
          display: flex;
          flex-direction: column;
        }

        .decisions-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .decisions-header h4 {
          margin: 0;
          color: #2d3748;
          font-size: 18px;
        }

        .new-decision-button {
          background: #3182ce;
          color: white;
          border: none;
          padding: 8px 16px;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .new-decision-button:hover {
          background: #2c5282;
        }

        .decisions-list {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .decision-card {
          background: white;
          padding: 16px;
          border-radius: 6px;
          border: 1px solid #e1e5e9;
        }

        .decision-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .decision-header h5 {
          margin: 0;
          color: #2d3748;
          font-size: 16px;
          font-weight: 600;
        }

        .decision-status {
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
          text-transform: uppercase;
        }

        .decision-status.status-proposed { background: #e2e8f0; color: #4a5568; }
        .decision-status.status-accepted { background: #c6f6d5; color: #276749; }
        .decision-status.status-implement { background: #fef5e7; color: #d69e2e; }
        .decision-status.status-implemented { background: #c6f6d5; color: #276749; }
        .decision-status.status-rejected { background: #fed7d7; color: #c53030; }
        .decision-status.status-deprecated { background: #e2e8f0; color: #4a5568; }

        .decision-content {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .decision-problem, .decision-solution {
          color: #4a5568;
          line-height: 1.5;
        }

        .decision-consequences {
          color: #718096;
        }

        .decision-consequences ul {
          margin: 4px 0 0 0;
          padding-left: 20px;
          font-size: 14px;
        }
      `}</style>
    </div>
  );
};

export default ArchitecturalAdvisorPanel;