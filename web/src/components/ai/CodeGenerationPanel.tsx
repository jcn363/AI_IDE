import React, { useState, useCallback, useEffect } from 'react';
import { useAppDispatch } from '../../store/hooks';
import { codegenActions } from '../../store/slices/codegenSlice';
import { codegenService } from '../../services/codegenService';
import type {
  GenerateFunctionRequest,
  GenerateFunctionResponse,
  ValidationResult,
  SupportedLanguageResult,
  GenerationTemplate,
  GenerationHistoryItem,
  AIFeedback,
  LearningMetrics,
} from '../../services/codegenService';
import GenerationHistory from './GenerationHistory';
import AIFeedbackPanel from './AIFeedbackPanel';
import LearningProgressIndicator from './LearningProgressIndicator';

/**
 * Code Generation Panel Component
 * Main UI for AI-powered function generation with validation
 */
const CodeGenerationPanel: React.FC = () => {
  const dispatch = useAppDispatch();

  // Form state
  const [functionPurpose, setFunctionPurpose] = useState('');
  const [targetLanguage, setTargetLanguage] = useState<
    'Rust' | 'Python' | 'TypeScript' | 'JavaScript' | 'Go' | 'Java' | 'C++' | 'SQL' | 'HTML' | 'CSS'
  >('Rust');
  const [parameters, setParameters] = useState<string[]>(['']);
  const [returnType, setReturnType] = useState('');
  const [similarFunctions, setSimilarFunctions] = useState<string[]>(['']);
  const [errorHandling, setErrorHandling] = useState(true);
  const [performanceReq, setPerformanceReq] = useState('');
  const [safetyReq, setSafetyReq] = useState('');

  // UI state
  const [loading, setLoading] = useState(false);
  const [generationResult, setGenerationResult] = useState<GenerateFunctionResponse | null>(null);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [supportedLanguages, setSupportedLanguages] = useState<string[]>([]);
  const [generationTemplates, setGenerationTemplates] = useState<GenerationTemplate[]>([]);
  const [activeTab, setActiveTab] = useState<
    | 'generate'
    | 'validate'
    | 'templates'
    | 'history'
    | 'learning'
    | 'classes'
    | 'interfaces'
    | 'api'
    | 'database'
    | 'utilities'
  >('generate');
  const [generationType, setGenerationType] = useState<
    'function' | 'class' | 'interface' | 'api_endpoint' | 'database_schema' | 'utility'
  >('function');

  // AI Learning state
  const [showFeedbackPanel, setShowFeedbackPanel] = useState(false);
  const [learningMetrics, setLearningMetrics] = useState<LearningMetrics | null>(null);
  const [currentGenerationId, setCurrentGenerationId] = useState<string>('');

  // Load supported languages on mount
  useEffect(() => {
    loadSupportedLanguages();
    loadGenerationTemplates();
  }, [targetLanguage]);

  const loadSupportedLanguages = useCallback(async () => {
    try {
      const result = await codegenService.getSupportedLanguages();
      if (result.success && result.supported_languages) {
        setSupportedLanguages(result.supported_languages);
      }
    } catch (error) {
      console.error('Failed to load supported languages:', error);
    }
  }, []);

  const loadGenerationTemplates = useCallback(async () => {
    try {
      const result = await codegenService.getGenerationTemplates(targetLanguage);
      if (result.success && result.templates) {
        setGenerationTemplates(result.templates);
      }
    } catch (error) {
      console.error('Failed to load generation templates:', error);
      setGenerationTemplates([]);
    }
  }, [targetLanguage]);

  const handleParameterChange = useCallback(
    (index: number, value: string) => {
      const newParams = [...parameters];
      newParams[index] = value;
      setParameters(newParams);
    },
    [parameters]
  );

  const addParameter = useCallback(() => {
    setParameters([...parameters, '']);
  }, [parameters]);

  const removeParameter = useCallback(
    (index: number) => {
      if (parameters.length > 1) {
        const newParams = parameters.filter((_, i) => i !== index);
        setParameters(newParams);
      }
    },
    [parameters]
  );

  const handleSimilarFunctionChange = useCallback(
    (index: number, value: string) => {
      const newFunctions = [...similarFunctions];
      newFunctions[index] = value;
      setSimilarFunctions(newFunctions);
    },
    [similarFunctions]
  );

  const addSimilarFunction = useCallback(() => {
    setSimilarFunctions([...similarFunctions, '']);
  }, [similarFunctions]);

  const removeSimilarFunction = useCallback(
    (index: number) => {
      if (similarFunctions.length > 1) {
        const newFunctions = similarFunctions.filter((_, i) => i !== index);
        setSimilarFunctions(newFunctions);
      }
    },
    [similarFunctions]
  );

  const generateFunction = useCallback(async () => {
    if (!functionPurpose.trim()) return;

    setLoading(true);
    try {
      const request: GenerateFunctionRequest = {
        function_purpose: functionPurpose.trim(),
        target_language: targetLanguage,
        parameters: parameters.filter((p) => p.trim()),
        return_type: returnType.trim() || undefined,
        similar_functions: similarFunctions.filter((f) => f.trim()),
        error_handling: errorHandling,
        performance_requirements: performanceReq.trim() || undefined,
        safety_requirements: safetyReq.trim() || undefined,
      };

      const result = await codegenService.generateFunction(request);
      setGenerationResult(result);

      // Save to history
      const historyItem: GenerationHistoryItem = {
        id: `gen_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        request,
        result: {
          success: result.success,
          generated_function: result.generated_function || undefined,
          error: result.error || undefined,
        },
        validation: undefined, // Will be updated after validation
        timestamp: Date.now(),
        isFavorite: false,
        tags: [],
      };

      dispatch(codegenActions.addToHistory(historyItem));

      // Auto-validate the generated code if successful
      if (result.success && result.generated_function?.code) {
        const validation = await validateGeneratedCode(
          result.generated_function.code,
          targetLanguage
        );
        if (validation) {
          // Update the history item with validation results
          const updatedHistoryItem = {
            ...historyItem,
            validation: {
              overall_score: validation.overall_score,
              readability_score: validation.readability_score,
              maintainability_score: validation.maintainability_score,
              performance_score: validation.performance_score,
              security_score: validation.security_score,
              compliance_score: validation.compliance_score,
              issues: validation.issues,
            },
          };
          dispatch(codegenActions.addToHistory(updatedHistoryItem));
        }
      }
    } catch (error) {
      console.error('Generation failed:', error);
      setGenerationResult({
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        timestamp: Date.now(),
      });
    } finally {
      setLoading(false);
    }
  }, [
    functionPurpose,
    targetLanguage,
    parameters,
    returnType,
    similarFunctions,
    errorHandling,
    performanceReq,
    safetyReq,
  ]);

  const validateGeneratedCode = useCallback(
    async (code: string, language: typeof targetLanguage) => {
      try {
        const result = await codegenService.validateCode({ code, language });
        setValidationResult(result);
      } catch (error) {
        console.error('Validation failed:', error);
      }
    },
    []
  );

  const resetForm = useCallback(() => {
    setFunctionPurpose('');
    setTargetLanguage('Rust');
    setParameters(['']);
    setReturnType('');
    setSimilarFunctions(['']);
    setErrorHandling(true);
    setPerformanceReq('');
    setSafetyReq('');
    setGenerationResult(null);
    setValidationResult(null);
  }, []);

  const getLanguageColor = (lang: string): string => {
    switch (lang.toLowerCase()) {
      case 'rust':
        return '#000000';
      case 'python':
        return '#3776ab';
      case 'typescript':
        return '#3178c6';
      case 'javascript':
        return '#f7df1e';
      case 'go':
        return '#00add8';
      case 'java':
        return '#007396';
      case 'c++':
        return '#00599c';
      case 'sql':
        return '#336791';
      case 'html':
        return '#e34f26';
      case 'css':
        return '#1572b6';
      default:
        return '#6b7280';
    }
  };

  const mapLanguageToMonaco = (lang: string): string => {
    switch (lang.toLowerCase()) {
      case 'rust':
        return 'rust';
      case 'python':
        return 'python';
      case 'typescript':
        return 'typescript';
      case 'javascript':
        return 'javascript';
      case 'go':
        return 'go';
      case 'java':
        return 'java';
      case 'c++':
        return 'cpp';
      case 'sql':
        return 'sql';
      case 'html':
        return 'html';
      case 'css':
        return 'css';
      default:
        return 'plaintext';
    }
  };

  const getScoreColor = (score: number): string => {
    if (score >= 0.8) return '#10b981';
    if (score >= 0.6) return '#f59e0b';
    return '#ef4444';
  };

  return (
    <div className="code-generation-panel">
      {/* Header */}
      <div className="panel-header">
        <div className="header-content">
          <h2>AI Function Generator</h2>
          <p>Generate high-quality functions with AI assistance and validation</p>
        </div>
        <div className="header-actions">
          <button className="reset-btn" onClick={resetForm} title="Reset Form">
            🔄 Reset
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="tab-navigation">
        <button
          className={`tab-btn ${activeTab === 'generate' ? 'active' : ''}`}
          onClick={() => setActiveTab('generate')}
        >
          ⚡ Functions
        </button>
        <button
          className={`tab-btn ${activeTab === 'classes' ? 'active' : ''}`}
          onClick={() => setActiveTab('classes')}
        >
          🏗️ Classes
        </button>
        <button
          className={`tab-btn ${activeTab === 'interfaces' ? 'active' : ''}`}
          onClick={() => setActiveTab('interfaces')}
        >
          🔌 Interfaces
        </button>
        <button
          className={`tab-btn ${activeTab === 'api' ? 'active' : ''}`}
          onClick={() => setActiveTab('api')}
        >
          🌐 API Endpoints
        </button>
        <button
          className={`tab-btn ${activeTab === 'database' ? 'active' : ''}`}
          onClick={() => setActiveTab('database')}
        >
          🗄️ Database
        </button>
        <button
          className={`tab-btn ${activeTab === 'utilities' ? 'active' : ''}`}
          onClick={() => setActiveTab('utilities')}
        >
          🔧 Utilities
        </button>
        <button
          className={`tab-btn ${activeTab === 'validate' ? 'active' : ''}`}
          onClick={() => setActiveTab('validate')}
        >
          ✅ Validate
        </button>
        <button
          className={`tab-btn ${activeTab === 'learning' ? 'active' : ''}`}
          onClick={() => setActiveTab('learning')}
        >
          🧠 Learning
        </button>
        <button
          className={`tab-btn ${activeTab === 'templates' ? 'active' : ''}`}
          onClick={() => setActiveTab('templates')}
        >
          📋 Templates
        </button>
        <button
          className={`tab-btn ${activeTab === 'history' ? 'active' : ''}`}
          onClick={() => setActiveTab('history')}
        >
          📚 History
        </button>
      </div>

      {/* Main Content */}
      <div className="panel-content">
        {activeTab === 'generate' && (
          <div className="generate-tab">
            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages.map((lang) => (
                  <button
                    key={lang}
                    className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                    onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                    style={{
                      borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                      backgroundColor:
                        targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                    }}
                  >
                    {lang}
                  </button>
                ))}
              </div>
            </div>

            {/* Function Purpose */}
            <div className="form-section">
              <h3>Function Purpose</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe what the function should do..."
                rows={3}
                className="purpose-input"
              />
            </div>

            {/* Parameters */}
            <div className="form-section">
              <div className="section-header">
                <h3>Parameters</h3>
                <button className="add-btn" onClick={addParameter}>
                  + Add Parameter
                </button>
              </div>
              <div className="parameters-list">
                {parameters.map((param, index) => (
                  <div key={index} className="parameter-item">
                    <input
                      type="text"
                      value={param}
                      onChange={(e) => handleParameterChange(index, e.target.value)}
                      placeholder={`Parameter ${index + 1}`}
                      className="parameter-input"
                    />
                    {parameters.length > 1 && (
                      <button
                        className="remove-btn"
                        onClick={() => removeParameter(index)}
                        title="Remove parameter"
                      >
                        ×
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* Return Type */}
            <div className="form-section">
              <h3>Return Type</h3>
              <input
                type="text"
                value={returnType}
                onChange={(e) => setReturnType(e.target.value)}
                placeholder="e.g., String, i32, Result<T, E>"
                className="return-type-input"
              />
            </div>

            {/* Similar Functions */}
            <div className="form-section">
              <div className="section-header">
                <h3>Similar Functions (Optional)</h3>
                <button className="add-btn" onClick={addSimilarFunction}>
                  + Add Reference
                </button>
              </div>
              <div className="similar-functions-list">
                {similarFunctions.map((func, index) => (
                  <div key={index} className="similar-function-item">
                    <input
                      type="text"
                      value={func}
                      onChange={(e) => handleSimilarFunctionChange(index, e.target.value)}
                      placeholder="Function signature or name..."
                      className="similar-function-input"
                    />
                    {similarFunctions.length > 1 && (
                      <button
                        className="remove-btn"
                        onClick={() => removeSimilarFunction(index)}
                        title="Remove reference"
                      >
                        ×
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* Options */}
            <div className="form-section">
              <h3>Options</h3>
              <div className="options-grid">
                <label className="option-item">
                  <input
                    type="checkbox"
                    checked={errorHandling}
                    onChange={(e) => setErrorHandling(e.target.checked)}
                  />
                  <span>Include Error Handling</span>
                </label>
              </div>
            </div>

            {/* Requirements */}
            <div className="form-section">
              <h3>Requirements (Optional)</h3>
              <div className="requirements-grid">
                <div className="requirement-item">
                  <label>Performance Requirements</label>
                  <input
                    type="text"
                    value={performanceReq}
                    onChange={(e) => setPerformanceReq(e.target.value)}
                    placeholder="e.g., O(n), memory efficient"
                    className="requirement-input"
                  />
                </div>
                <div className="requirement-item">
                  <label>Safety Requirements</label>
                  <input
                    type="text"
                    value={safetyReq}
                    onChange={(e) => setSafetyReq(e.target.value)}
                    placeholder="e.g., thread-safe, no panics"
                    className="requirement-input"
                  />
                </div>
              </div>
            </div>

            {/* Generate Button */}
            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🚀 Generate Function'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'classes' && (
          <div className="classes-tab">
            <div className="tab-header">
              <h3>Generate Class</h3>
              <p>AI-powered class generation with inheritance and method support</p>
            </div>

            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages
                  .filter((lang) =>
                    ['TypeScript', 'JavaScript', 'Java', 'Python', 'C++', 'Go'].includes(lang)
                  )
                  .map((lang) => (
                    <button
                      key={lang}
                      className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                      onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                      style={{
                        borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                        backgroundColor:
                          targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                      }}
                    >
                      {lang}
                    </button>
                  ))}
              </div>
            </div>

            {/* Class Description */}
            <div className="form-section">
              <h3>Class Description</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe what the class should do..."
                rows={3}
                className="purpose-input"
              />
            </div>

            {/* Class Name */}
            <div className="form-section">
              <h3>Class Name</h3>
              <input
                type="text"
                value={functionPurpose.split(' ')[0] || ''}
                onChange={(e) => {
                  /* Extract class name from description */
                }}
                placeholder="e.g., UserService, DataProcessor"
                className="return-type-input"
              />
            </div>

            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🏗️ Generate Class'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'interfaces' && (
          <div className="interfaces-tab">
            <div className="tab-header">
              <h3>Generate Interface</h3>
              <p>AI-powered interface generation with type safety</p>
            </div>

            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages
                  .filter((lang) => ['TypeScript', 'Java', 'Go'].includes(lang))
                  .map((lang) => (
                    <button
                      key={lang}
                      className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                      onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                      style={{
                        borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                        backgroundColor:
                          targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                      }}
                    >
                      {lang}
                    </button>
                  ))}
              </div>
            </div>

            {/* Interface Description */}
            <div className="form-section">
              <h3>Interface Description</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe the interface contract..."
                rows={3}
                className="purpose-input"
              />
            </div>

            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🔌 Generate Interface'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'api' && (
          <div className="api-tab">
            <div className="tab-header">
              <h3>Generate API Endpoint</h3>
              <p>RESTful and GraphQL API endpoint generation</p>
            </div>

            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages
                  .filter((lang) =>
                    ['TypeScript', 'JavaScript', 'Python', 'Java', 'Go'].includes(lang)
                  )
                  .map((lang) => (
                    <button
                      key={lang}
                      className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                      onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                      style={{
                        borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                        backgroundColor:
                          targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                      }}
                    >
                      {lang}
                    </button>
                  ))}
              </div>
            </div>

            {/* API Description */}
            <div className="form-section">
              <h3>API Endpoint Description</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe the API endpoint functionality..."
                rows={3}
                className="purpose-input"
              />
            </div>

            {/* HTTP Method */}
            <div className="form-section">
              <h3>HTTP Method</h3>
              <select className="method-select">
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
                <option value="PATCH">PATCH</option>
              </select>
            </div>

            {/* Endpoint Path */}
            <div className="form-section">
              <h3>Endpoint Path</h3>
              <input type="text" placeholder="/api/users/:id" className="return-type-input" />
            </div>

            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🌐 Generate API Endpoint'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'database' && (
          <div className="database-tab">
            <div className="tab-header">
              <h3>Generate Database Schema</h3>
              <p>Database schema generation with relationships</p>
            </div>

            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages
                  .filter((lang) => ['SQL', 'TypeScript', 'Python', 'Java'].includes(lang))
                  .map((lang) => (
                    <button
                      key={lang}
                      className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                      onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                      style={{
                        borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                        backgroundColor:
                          targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                      }}
                    >
                      {lang}
                    </button>
                  ))}
              </div>
            </div>

            {/* Schema Description */}
            <div className="form-section">
              <h3>Schema Description</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe the database schema..."
                rows={3}
                className="purpose-input"
              />
            </div>

            {/* Table Name */}
            <div className="form-section">
              <h3>Table Name</h3>
              <input
                type="text"
                placeholder="users, products, orders"
                className="return-type-input"
              />
            </div>

            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🗄️ Generate Database Schema'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'utilities' && (
          <div className="utilities-tab">
            <div className="tab-header">
              <h3>Generate Utility Functions</h3>
              <p>Common helper functions and utilities</p>
            </div>

            {/* Language Selection */}
            <div className="form-section">
              <h3>Target Language</h3>
              <div className="language-selector">
                {supportedLanguages
                  .filter((lang) => !['SQL', 'HTML', 'CSS'].includes(lang))
                  .map((lang) => (
                    <button
                      key={lang}
                      className={`language-btn ${targetLanguage === lang ? 'active' : ''}`}
                      onClick={() => setTargetLanguage(lang as typeof targetLanguage)}
                      style={{
                        borderColor: targetLanguage === lang ? getLanguageColor(lang) : '#e5e7eb',
                        backgroundColor:
                          targetLanguage === lang ? `${getLanguageColor(lang)}10` : 'white',
                      }}
                    >
                      {lang}
                    </button>
                  ))}
              </div>
            </div>

            {/* Utility Description */}
            <div className="form-section">
              <h3>Utility Description</h3>
              <textarea
                value={functionPurpose}
                onChange={(e) => setFunctionPurpose(e.target.value)}
                placeholder="Describe the utility function..."
                rows={3}
                className="purpose-input"
              />
            </div>

            {/* Utility Type */}
            <div className="form-section">
              <h3>Utility Type</h3>
              <select className="method-select">
                <option value="string">String Utilities</option>
                <option value="array">Array Utilities</option>
                <option value="date">Date/Time Utilities</option>
                <option value="math">Math Utilities</option>
                <option value="validation">Validation Utilities</option>
                <option value="formatting">Formatting Utilities</option>
              </select>
            </div>

            <div className="action-section">
              <button
                className="generate-btn"
                onClick={generateFunction}
                disabled={loading || !functionPurpose.trim()}
              >
                {loading ? '🤖 Generating...' : '🔧 Generate Utility'}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'validate' && (
          <div className="validate-tab">
            <div className="validation-placeholder">
              <h3>Code Validation</h3>
              <p>Validation results will appear here after generation</p>
              {validationResult && (
                <div className="validation-results">
                  <div className="validation-header">
                    <h4>Validation Score: {Math.round(validationResult.overall_score * 100)}%</h4>
                    <div
                      className="score-indicator"
                      style={{ backgroundColor: getScoreColor(validationResult.overall_score) }}
                    />
                  </div>
                  <div className="validation-details">
                    <div className="score-breakdown">
                      <div className="score-item">
                        <span>Readability</span>
                        <span>{Math.round(validationResult.readability_score * 100)}%</span>
                      </div>
                      <div className="score-item">
                        <span>Maintainability</span>
                        <span>{Math.round(validationResult.maintainability_score * 100)}%</span>
                      </div>
                      <div className="score-item">
                        <span>Performance</span>
                        <span>{Math.round(validationResult.performance_score * 100)}%</span>
                      </div>
                      <div className="score-item">
                        <span>Security</span>
                        <span>{Math.round(validationResult.security_score * 100)}%</span>
                      </div>
                      <div className="score-item">
                        <span>Compliance</span>
                        <span>{Math.round(validationResult.compliance_score * 100)}%</span>
                      </div>
                    </div>
                    {validationResult.issues.length > 0 && (
                      <div className="validation-issues">
                        <h5>Issues Found:</h5>
                        {validationResult.issues.map((issue, index) => (
                          <div key={index} className={`issue-item ${issue.severity}`}>
                            <span className="issue-category">{issue.category}</span>
                            <span className="issue-message">{issue.message}</span>
                            {issue.suggestion && (
                              <span className="issue-suggestion">💡 {issue.suggestion}</span>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'learning' && (
          <div className="learning-tab">
            <div className="learning-header">
              <h3>AI Learning Dashboard</h3>
              <p>Track AI improvement and provide feedback to enhance performance</p>
            </div>

            {learningMetrics ? (
              <div className="learning-content">
                <LearningProgressIndicator metrics={learningMetrics} showDetails={true} />

                <div className="learning-actions">
                  <button
                    className="learning-btn primary"
                    onClick={() => setShowFeedbackPanel(true)}
                    disabled={!currentGenerationId}
                  >
                    📝 Provide Feedback
                  </button>
                  <button
                    className="learning-btn secondary"
                    onClick={() => {
                      /* Navigate to learning history */
                    }}
                  >
                    📊 View Learning History
                  </button>
                </div>

                <div className="learning-insights">
                  <h4>Recent Insights</h4>
                  <div className="insights-list">
                    <div className="insight-item">
                      <span className="insight-icon">🎯</span>
                      <span>
                        Pattern accuracy improved by{' '}
                        {(learningMetrics.patternAccuracy * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div className="insight-item">
                      <span className="insight-icon">📈</span>
                      <span>
                        Context understanding enhanced by{' '}
                        {(learningMetrics.contextImprovement * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div className="insight-item">
                      <span className="insight-icon">⭐</span>
                      <span>
                        Average user satisfaction: {learningMetrics.averageRating.toFixed(1)}/5
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="learning-placeholder">
                <div className="placeholder-icon">🧠</div>
                <h4>AI Learning Not Yet Active</h4>
                <p>Generate some code and provide feedback to start the learning process!</p>
                <button className="start-learning-btn" onClick={() => setActiveTab('generate')}>
                  🚀 Start Generating Code
                </button>
              </div>
            )}
          </div>
        )}

        {activeTab === 'templates' && (
          <div className="templates-tab">
            <div className="templates-header">
              <h3>Generation Templates for {targetLanguage}</h3>
              <p>Available templates and patterns</p>
            </div>
            <div className="templates-grid">
              {generationTemplates.length > 0 ? (
                generationTemplates.map((template, index) => (
                  <div key={index} className="template-card">
                    <div className="template-header">
                      <h4>{template.name}</h4>
                      <span className="template-version">v{template.version}</span>
                    </div>
                    <p className="template-description">{template.description}</p>
                    <div className="template-languages">
                      {template.language_support.map((lang) => (
                        <span key={lang} className="language-tag">
                          {lang}
                        </span>
                      ))}
                    </div>
                  </div>
                ))
              ) : (
                <div className="no-templates">
                  <p>No templates available for {targetLanguage}</p>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'history' && (
          <div className="history-tab">
            <GenerationHistory
              onLoadGeneration={(item) => {
                // Load the generation parameters back into the form
                setFunctionPurpose(item.request.function_purpose);
                setTargetLanguage(item.request.target_language as typeof targetLanguage);
                setParameters(item.request.parameters.length > 0 ? item.request.parameters : ['']);
                setReturnType(item.request.return_type || '');
                setSimilarFunctions(
                  item.request.similar_functions.length > 0 ? item.request.similar_functions : ['']
                );
                setErrorHandling(item.request.error_handling);
                setPerformanceReq(item.request.performance_requirements || '');
                setSafetyReq(item.request.safety_requirements || '');
                setActiveTab('generate');
              }}
            />
          </div>
        )}
      </div>

      {/* Generation Results */}
      {generationResult && (
        <div className="results-section">
          <h3>Generation Results</h3>
          {generationResult.success ? (
            <div className="success-results">
              <div className="result-header">
                <span className="success-icon">✅</span>
                <span>Function generated successfully!</span>
              </div>
              {generationResult.generated_function && (
                <div className="generated-function">
                  <div className="function-header">
                    <h4>{generationResult.generated_function.name}</h4>
                    <div className="function-meta">
                      <span className="confidence">
                        Confidence:{' '}
                        {Math.round(generationResult.generated_function.confidence_score * 100)}%
                      </span>
                      <span className="complexity">
                        Complexity: {generationResult.generated_function.complexity}/10
                      </span>
                    </div>
                  </div>
                  <div className="function-signature">
                    <pre>{generationResult.generated_function.signature}</pre>
                  </div>
                  <div className="function-code">
                    <pre>{generationResult.generated_function.code}</pre>
                  </div>
                  {generationResult.generated_function.documentation && (
                    <div className="function-docs">
                      <h5>Documentation</h5>
                      <p>{generationResult.generated_function.documentation}</p>
                    </div>
                  )}
                  {generationResult.generated_function.tests && (
                    <div className="function-tests">
                      <h5>Tests</h5>
                      <pre>{generationResult.generated_function.tests}</pre>
                    </div>
                  )}
                  {generationResult.generated_function.imports.length > 0 && (
                    <div className="function-imports">
                      <h5>Imports</h5>
                      <pre>{generationResult.generated_function.imports.join('\n')}</pre>
                    </div>
                  )}

                  {/* AI Feedback Button */}
                  <div className="generation-actions">
                    <button
                      className="feedback-btn"
                      onClick={() => {
                        setCurrentGenerationId(
                          generationResult.generated_function.id || `gen_${Date.now()}`
                        );
                        setShowFeedbackPanel(true);
                      }}
                      title="Help improve AI by providing feedback"
                    >
                      💬 Provide Feedback
                    </button>
                    <LearningProgressIndicator
                      metrics={{
                        totalGenerations: 1,
                        averageRating: 0,
                        patternAccuracy: 0,
                        contextImprovement: 0,
                        lastUpdated: new Date().toISOString(),
                        improvementRate: 0,
                      }}
                      compact={true}
                    />
                  </div>
                </div>
              )}
            </div>
          ) : (
            <div className="error-results">
              <div className="error-header">
                <span className="error-icon">❌</span>
                <span>Generation failed</span>
              </div>
              <p className="error-message">{generationResult.error}</p>
            </div>
          )}
        </div>
      )}

      {/* AI Feedback Panel */}
      {showFeedbackPanel && (
        <AIFeedbackPanel
          generationId={currentGenerationId}
          onSubmitFeedback={async (feedback: AIFeedback) => {
            // Handle feedback submission
            try {
              // This would integrate with the backend feedback system
              console.log('Feedback submitted:', feedback);

              // Update learning metrics (placeholder for now)
              setLearningMetrics({
                totalGenerations: (learningMetrics?.totalGenerations || 0) + 1,
                averageRating: feedback.rating,
                patternAccuracy: 0.8,
                contextImprovement: 0.1,
                lastUpdated: new Date().toISOString(),
                improvementRate: 0.05,
              });

              // Close the panel
              setShowFeedbackPanel(false);
            } catch (error) {
              console.error('Failed to submit feedback:', error);
            }
          }}
          onClose={() => setShowFeedbackPanel(false)}
        />
      )}

      <style jsx>{`
        .code-generation-panel {
          padding: 24px;
          border-radius: 12px;
          background: white;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          max-width: 1200px;
          margin: 0 auto;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 24px;
          padding-bottom: 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .header-content h2 {
          margin: 0 0 8px 0;
          color: #111827;
          font-size: 24px;
        }

        .header-content p {
          margin: 0;
          color: #6b7280;
        }

        .header-actions {
          display: flex;
          gap: 8px;
        }

        .reset-btn {
          padding: 8px 16px;
          background: #f3f4f6;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          cursor: pointer;
          font-size: 14px;
          color: #374151;
        }

        .reset-btn:hover {
          background: #e5e7eb;
        }

        .tab-navigation {
          display: flex;
          margin-bottom: 24px;
          border-bottom: 1px solid #e5e7eb;
        }

        .tab-btn {
          padding: 12px 24px;
          border: none;
          background: none;
          cursor: pointer;
          font-size: 16px;
          font-weight: 500;
          color: #6b7280;
          border-bottom: 2px solid transparent;
        }

        .tab-btn.active {
          color: #3b82f6;
          border-bottom-color: #3b82f6;
        }

        .tab-btn:hover:not(.active) {
          color: #374151;
        }

        .panel-content {
          min-height: 400px;
        }

        .form-section {
          margin-bottom: 24px;
        }

        .form-section h3 {
          margin: 0 0 12px 0;
          color: #111827;
          font-size: 18px;
        }

        .language-selector {
          display: flex;
          gap: 12px;
          flex-wrap: wrap;
        }

        .language-btn {
          padding: 8px 16px;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          background: white;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
        }

        .language-btn.active {
          font-weight: 600;
        }

        .purpose-input,
        .return-type-input {
          width: 100%;
          padding: 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          font-size: 14px;
          resize: vertical;
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

        .parameters-list,
        .similar-functions-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .parameter-item,
        .similar-function-item {
          display: flex;
          gap: 8px;
          align-items: center;
        }

        .parameter-input,
        .similar-function-input {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
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
        }

        .options-grid {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .option-item {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }

        .requirements-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
        }

        .requirement-item {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .requirement-input {
          padding: 8px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
        }

        .action-section {
          text-align: center;
          margin-top: 32px;
        }

        .generate-btn {
          padding: 16px 32px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 18px;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .generate-btn:hover:not(:disabled) {
          transform: translateY(-1px);
        }

        .generate-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .results-section {
          margin-top: 32px;
          padding-top: 24px;
          border-top: 1px solid #e5e7eb;
        }

        .results-section h3 {
          margin: 0 0 16px 0;
          color: #111827;
          font-size: 20px;
        }

        .success-results {
          background: #f0fff4;
          border: 1px solid #bbf7d0;
          border-radius: 8px;
          padding: 16px;
        }

        .result-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 16px;
        }

        .success-icon {
          font-size: 20px;
        }

        .generated-function {
          background: white;
          border-radius: 6px;
          padding: 16px;
        }

        .function-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .function-header h4 {
          margin: 0;
          color: #111827;
        }

        .function-meta {
          display: flex;
          gap: 16px;
          font-size: 14px;
          color: #6b7280;
        }

        .function-signature,
        .function-code,
        .function-tests,
        .function-imports {
          margin-bottom: 16px;
        }

        .function-signature pre,
        .function-code pre,
        .function-tests pre,
        .function-imports pre {
          background: #f3f4f6;
          padding: 12px;
          border-radius: 4px;
          overflow-x: auto;
          font-size: 14px;
          margin: 0;
        }

        .function-docs {
          margin-bottom: 16px;
        }

        .function-docs h5 {
          margin: 0 0 8px 0;
          color: #374151;
        }

        .function-docs p {
          margin: 0;
          color: #4b5563;
          line-height: 1.5;
        }

        .error-results {
          background: #fef2f2;
          border: 1px solid #fecaca;
          border-radius: 8px;
          padding: 16px;
        }

        .error-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .error-icon {
          font-size: 20px;
        }

        .error-message {
          margin: 0;
          color: #dc2626;
        }

        .validation-results {
          background: white;
          border-radius: 8px;
          padding: 16px;
          margin-top: 16px;
        }

        .validation-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .validation-header h4 {
          margin: 0;
          color: #111827;
        }

        .score-indicator {
          width: 16px;
          height: 16px;
          border-radius: 50%;
        }

        .score-breakdown {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
          margin-bottom: 16px;
        }

        .score-item {
          display: flex;
          justify-content: space-between;
          padding: 8px 12px;
          background: #f9fafb;
          border-radius: 4px;
        }

        .validation-issues h5 {
          margin: 0 0 12px 0;
          color: #374151;
        }

        .issue-item {
          padding: 8px 12px;
          border-radius: 4px;
          margin-bottom: 8px;
        }

        .issue-item.high {
          background: #fef2f2;
          border-left: 4px solid #ef4444;
        }

        .issue-item.medium {
          background: #fef3c7;
          border-left: 4px solid #f59e0b;
        }

        .issue-item.low {
          background: #f0fdf4;
          border-left: 4px solid #10b981;
        }

        .issue-category {
          font-weight: 600;
          color: #374151;
        }

        .issue-message {
          display: block;
          margin: 4px 0;
          color: #4b5563;
        }

        .issue-suggestion {
          font-size: 12px;
          color: #059669;
          font-style: italic;
        }

        .templates-tab {
          padding: 20px 0;
        }

        .templates-header h3 {
          margin: 0 0 8px 0;
          color: #111827;
        }

        .templates-header p {
          margin: 0;
          color: #6b7280;
        }

        .templates-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
          gap: 16px;
          margin-top: 20px;
        }

        .template-card {
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          background: #f9fafb;
        }

        .template-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .template-header h4 {
          margin: 0;
          color: #111827;
        }

        .template-version {
          font-size: 12px;
          color: #6b7280;
          background: #e5e7eb;
          padding: 2px 6px;
          border-radius: 10px;
        }

        .template-description {
          margin: 0 0 12px 0;
          color: #4b5563;
          font-size: 14px;
        }

        .template-languages {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
        }

        .language-tag {
          font-size: 12px;
          background: #dbeafe;
          color: #1e40af;
          padding: 2px 8px;
          border-radius: 12px;
        }

        .no-templates {
          text-align: center;
          padding: 40px;
          color: #6b7280;
        }

        .validate-tab {
          padding: 20px 0;
        }

        .history-tab {
          padding: 20px 0;
          height: 600px;
          overflow: hidden;
        }

        .learning-tab {
          padding: 20px 0;
        }

        .learning-header h3 {
          margin: 0 0 8px 0;
          color: #111827;
        }

        .learning-header p {
          margin: 0;
          color: #6b7280;
        }

        .learning-content {
          margin-top: 20px;
        }

        .learning-actions {
          display: flex;
          gap: 12px;
          margin-top: 20px;
          flex-wrap: wrap;
        }

        .learning-btn {
          padding: 12px 24px;
          border: none;
          border-radius: 6px;
          font-size: 16px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .learning-btn.primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .learning-btn.primary:hover:not(:disabled) {
          transform: translateY(-1px);
        }

        .learning-btn.secondary {
          background: #f3f4f6;
          color: #374151;
          border: 1px solid #d1d5db;
        }

        .learning-btn.secondary:hover {
          background: #e5e7eb;
        }

        .learning-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .learning-insights {
          margin-top: 24px;
          padding-top: 20px;
          border-top: 1px solid #e5e7eb;
        }

        .learning-insights h4 {
          margin: 0 0 16px 0;
          color: #111827;
          font-size: 18px;
        }

        .insights-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .insight-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px;
          background: #f9fafb;
          border-radius: 8px;
          border-left: 4px solid #3b82f6;
        }

        .insight-icon {
          font-size: 20px;
        }

        .insight-item span:last-child {
          color: #374151;
          font-weight: 500;
        }

        .learning-placeholder {
          text-align: center;
          padding: 60px 20px;
          color: #6b7280;
        }

        .placeholder-icon {
          font-size: 48px;
          margin-bottom: 16px;
        }

        .learning-placeholder h4 {
          margin: 0 0 8px 0;
          color: #111827;
          font-size: 20px;
        }

        .learning-placeholder p {
          margin: 0 0 24px 0;
          font-size: 16px;
        }

        .start-learning-btn {
          padding: 16px 32px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          border-radius: 8px;
          font-size: 18px;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s;
        }

        .start-learning-btn:hover {
          transform: translateY(-1px);
        }

        .generation-actions {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-top: 20px;
          padding-top: 16px;
          border-top: 1px solid #e5e7eb;
        }

        .feedback-btn {
          padding: 10px 20px;
          background: #10b981;
          color: white;
          border: none;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .feedback-btn:hover {
          background: #059669;
          transform: translateY(-1px);
        }

        .validation-placeholder h3 {
          margin: 0 0 8px 0;
          color: #111827;
        }

        .validation-placeholder p {
          margin: 0;
          color: #6b7280;
        }

        @media (max-width: 768px) {
          .code-generation-panel {
            padding: 16px;
          }

          .panel-header {
            flex-direction: column;
            align-items: stretch;
            gap: 16px;
          }

          .language-selector {
            justify-content: center;
          }

          .requirements-grid {
            grid-template-columns: 1fr;
          }

          .function-meta {
            flex-direction: column;
            align-items: flex-start;
            gap: 4px;
          }

          .templates-grid {
            grid-template-columns: 1fr;
          }

          .tab-navigation {
            overflow-x: auto;
          }

          .tab-btn {
            white-space: nowrap;
          }
        }
      `}</style>
    </div>
  );
};

export default CodeGenerationPanel;
