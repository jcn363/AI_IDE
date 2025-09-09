import { invoke } from '@tauri-apps/api/tauri';
import { useState, useCallback, useEffect } from 'react';
import type {
  ErrorPattern,
  FixSuggestion,
  LearnedPattern,
  LearningPreferences,
  LearningSystemData,
  FixApplicationResult,
  CodeChange,
  LearningSystemRequest,
  AIAnalysisError
} from '../types';

// ============================================================================
// Core Learning System Utilities
// ============================================================================

/**
 * Records a successful fix application for learning purposes
 */
export async function recordSuccessfulFix(
  errorPattern: ErrorPattern,
  appliedFix: FixSuggestion,
  context: string,
  userFeedback?: 'positive' | 'negative' | 'neutral'
): Promise<string> {
  try {
    const request: LearningSystemRequest = {
      errorPattern,
      appliedFix,
      success: true,
      userFeedback,
      context
    };

    const patternId = await invoke<string>('record_successful_fix', { request });
    
    // Update local analytics
    await updateLocalAnalytics('fix_recorded', {
      patternId,
      errorType: errorPattern.errorType,
      fixType: appliedFix.fixType,
      confidence: appliedFix.confidence
    });

    return patternId;
  } catch (error) {
    console.error('Failed to record successful fix:', error);
    throw new AIAnalysisError({
      type: 'internal',
      message: 'Failed to record fix for learning',
      details: error instanceof Error ? error.message : String(error),
      retryable: true,
      timestamp: Date.now(),
      context: {
        operation: 'record_fix',
        errorType: errorPattern.errorType
      }
    });
  }
}

/**
 * Records a failed fix application for learning purposes
 */
export async function recordFailedFix(
  errorPattern: ErrorPattern,
  attemptedFix: FixSuggestion,
  context: string,
  errorMessage: string
): Promise<void> {
  try {
    const request: LearningSystemRequest = {
      errorPattern,
      appliedFix: attemptedFix,
      success: false,
      context
    };

    await invoke('record_failed_fix', { request, errorMessage });
    
    // Update local analytics
    await updateLocalAnalytics('fix_failed', {
      errorType: errorPattern.errorType,
      fixType: attemptedFix.fixType,
      errorMessage
    });
  } catch (error) {
    console.error('Failed to record failed fix:', error);
    // Don't throw here as this is non-critical
  }
}

/**
 * Retrieves learned patterns for similar errors
 */
export async function getLearnedPatterns(
  errorPattern: ErrorPattern,
  maxResults: number = 10
): Promise<LearnedPattern[]> {
  try {
    const patterns = await invoke<LearnedPattern[]>('get_learned_patterns', {
      errorPattern,
      maxResults
    });

    // Sort by confidence and recency
    return patterns.sort((a, b) => {
      const confidenceDiff = b.confidence - a.confidence;
      if (Math.abs(confidenceDiff) > 0.1) {
        return confidenceDiff;
      }
      return new Date(b.lastUsed).getTime() - new Date(a.lastUsed).getTime();
    });
  } catch (error) {
    console.error('Failed to retrieve learned patterns:', error);
    return [];
  }
}

/**
 * Calculates confidence score for a fix suggestion based on learned patterns
 */
export function calculateFixConfidence(
  fixSuggestion: FixSuggestion,
  learnedPatterns: LearnedPattern[],
  contextSimilarity: number = 0.5
): number {
  if (learnedPatterns.length === 0) {
    return fixSuggestion.confidence;
  }

  // Find patterns that match the fix type
  const relevantPatterns = learnedPatterns.filter(pattern => 
    pattern.successfulFix.fixType === fixSuggestion.fixType
  );

  if (relevantPatterns.length === 0) {
    return fixSuggestion.confidence;
  }

  // Calculate weighted confidence based on historical success
  const totalWeight = relevantPatterns.reduce((sum, pattern) => {
    const successRate = pattern.successCount / (pattern.successCount + pattern.failureCount);
    const recencyWeight = calculateRecencyWeight(pattern.lastUsed);
    return sum + (successRate * pattern.confidence * recencyWeight);
  }, 0);

  const averageWeight = totalWeight / relevantPatterns.length;
  
  // Combine original confidence with learned confidence
  const baseWeight = 0.3;
  const learnedWeight = 0.7;
  const contextWeight = contextSimilarity;
  
  return Math.min(1.0, 
    (fixSuggestion.confidence * baseWeight) + 
    (averageWeight * learnedWeight * contextWeight)
  );
}

/**
 * Calculates recency weight for learned patterns (more recent = higher weight)
 */
function calculateRecencyWeight(lastUsed: string): number {
  const now = new Date().getTime();
  const lastUsedTime = new Date(lastUsed).getTime();
  const daysSince = (now - lastUsedTime) / (1000 * 60 * 60 * 24);
  
  // Exponential decay: weight decreases by half every 30 days
  return Math.exp(-daysSince / 30);
}

/**
 * Calculates context similarity between two error patterns
 */
export function calculateContextSimilarity(
  pattern1: ErrorPattern,
  pattern2: ErrorPattern
): number {
  // Simple similarity based on error type and pattern matching
  if (pattern1.errorType !== pattern2.errorType) {
    return 0;
  }

  // Calculate string similarity for patterns
  const patternSimilarity = calculateStringSimilarity(pattern1.pattern, pattern2.pattern);
  const contextSimilarity = calculateStringSimilarity(pattern1.context, pattern2.context);
  
  return (patternSimilarity * 0.7) + (contextSimilarity * 0.3);
}

/**
 * Simple string similarity calculation using Jaccard similarity
 */
function calculateStringSimilarity(str1: string, str2: string): number {
  const set1 = new Set(str1.toLowerCase().split(/\s+/));
  const set2 = new Set(str2.toLowerCase().split(/\s+/));
  
  const intersection = new Set([...set1].filter(x => set2.has(x)));
  const union = new Set([...set1, ...set2]);
  
  return intersection.size / union.size;
}

// ============================================================================
// Privacy Controls
// ============================================================================

/**
 * Updates user learning preferences
 */
export async function updateLearningPreferences(
  preferences: LearningPreferences
): Promise<void> {
  try {
    await invoke('update_learning_preferences', { preferences });
    
    // Store preferences locally for quick access
    localStorage.setItem('ai_learning_preferences', JSON.stringify(preferences));
    
    await updateLocalAnalytics('preferences_updated', {
      enableLearning: preferences.enableLearning,
      privacyMode: preferences.privacyMode
    });
  } catch (error) {
    console.error('Failed to update learning preferences:', error);
    throw new AIAnalysisError({
      type: 'internal',
      message: 'Failed to update learning preferences',
      details: error instanceof Error ? error.message : String(error),
      retryable: true,
      timestamp: Date.now(),
      context: { operation: 'update_preferences' }
    });
  }
}

/**
 * Gets current learning preferences
 */
export async function getLearningPreferences(): Promise<LearningPreferences> {
  try {
    // Try to get from backend first
    const preferences = await invoke<LearningPreferences>('get_learning_preferences');
    
    // Cache locally
    localStorage.setItem('ai_learning_preferences', JSON.stringify(preferences));
    
    return preferences;
  } catch (error) {
    console.error('Failed to get learning preferences from backend:', error);
    
    // Fallback to local storage
    const cached = localStorage.getItem('ai_learning_preferences');
    if (cached) {
      return JSON.parse(cached);
    }
    
    // Return default preferences
    return {
      enableLearning: false,
      privacyMode: 'opt-in',
      shareAnonymousData: false,
      retainPersonalData: false,
      dataRetentionDays: 30,
      allowModelTraining: false
    };
  }
}

/**
 * Clears all learning data based on privacy preferences
 */
export async function clearLearningData(
  clearType: 'all' | 'personal' | 'anonymous' = 'all'
): Promise<void> {
  try {
    await invoke('clear_learning_data', { clearType });
    
    // Clear local analytics if clearing all data
    if (clearType === 'all') {
      localStorage.removeItem('ai_learning_analytics');
    }
    
    await updateLocalAnalytics('data_cleared', { clearType });
  } catch (error) {
    console.error('Failed to clear learning data:', error);
    throw new AIAnalysisError({
      type: 'internal',
      message: 'Failed to clear learning data',
      details: error instanceof Error ? error.message : String(error),
      retryable: true,
      timestamp: Date.now(),
      context: { operation: 'clear_data', clearType }
    });
  }
}

// ============================================================================
// Data Synchronization
// ============================================================================

/**
 * Synchronizes learning data with backend storage
 */
export async function syncLearningData(): Promise<{
  success: boolean;
  syncedPatterns: number;
  errors: string[];
}> {
  try {
    const result = await invoke<{
      success: boolean;
      syncedPatterns: number;
      errors: string[];
    }>('sync_learning_data');
    
    await updateLocalAnalytics('data_synced', {
      success: result.success,
      syncedPatterns: result.syncedPatterns,
      errorCount: result.errors.length
    });
    
    return result;
  } catch (error) {
    console.error('Failed to sync learning data:', error);
    return {
      success: false,
      syncedPatterns: 0,
      errors: [error instanceof Error ? error.message : String(error)]
    };
  }
}

/**
 * Gets learning system data including patterns and statistics
 */
export async function getLearningSystemData(): Promise<LearningSystemData> {
  try {
    return await invoke<LearningSystemData>('get_learning_system_data');
  } catch (error) {
    console.error('Failed to get learning system data:', error);
    throw new AIAnalysisError({
      type: 'internal',
      message: 'Failed to retrieve learning system data',
      details: error instanceof Error ? error.message : String(error),
      retryable: true,
      timestamp: Date.now(),
      context: { operation: 'get_learning_data' }
    });
  }
}

// ============================================================================
// Analytics and Tracking
// ============================================================================

interface AnalyticsEvent {
  type: string;
  timestamp: number;
  data: Record<string, any>;
}

/**
 * Updates local analytics data
 */
async function updateLocalAnalytics(
  eventType: string,
  data: Record<string, any>
): Promise<void> {
  try {
    const preferences = await getLearningPreferences();
    
    // Only track if user has opted in
    if (!preferences.enableLearning || preferences.privacyMode === 'opt-out') {
      return;
    }
    
    const event: AnalyticsEvent = {
      type: eventType,
      timestamp: Date.now(),
      data: preferences.shareAnonymousData ? data : {}
    };
    
    const existing = localStorage.getItem('ai_learning_analytics');
    const events: AnalyticsEvent[] = existing ? JSON.parse(existing) : [];
    
    events.push(event);
    
    // Keep only last 1000 events
    if (events.length > 1000) {
      events.splice(0, events.length - 1000);
    }
    
    localStorage.setItem('ai_learning_analytics', JSON.stringify(events));
  } catch (error) {
    console.error('Failed to update local analytics:', error);
  }
}

/**
 * Gets learning system effectiveness metrics
 */
export async function getLearningAnalytics(): Promise<{
  totalFixesRecorded: number;
  successRate: number;
  averageConfidenceImprovement: number;
  mostCommonErrorTypes: Array<{ type: string; count: number }>;
  userAdoptionMetrics: {
    learningEnabled: boolean;
    daysActive: number;
    fixesApplied: number;
    feedbackProvided: number;
  };
}> {
  try {
    const backendAnalytics = await invoke<any>('get_learning_analytics');
    const localEvents = getLocalAnalytics();
    
    return {
      ...backendAnalytics,
      userAdoptionMetrics: {
        ...backendAnalytics.userAdoptionMetrics,
        daysActive: calculateActiveDays(localEvents),
        feedbackProvided: localEvents.filter(e => e.type === 'fix_recorded' && e.data.userFeedback).length
      }
    };
  } catch (error) {
    console.error('Failed to get learning analytics:', error);
    
    // Return local analytics as fallback
    const localEvents = getLocalAnalytics();
    return {
      totalFixesRecorded: localEvents.filter(e => e.type === 'fix_recorded').length,
      successRate: calculateLocalSuccessRate(localEvents),
      averageConfidenceImprovement: 0,
      mostCommonErrorTypes: [],
      userAdoptionMetrics: {
        learningEnabled: true,
        daysActive: calculateActiveDays(localEvents),
        fixesApplied: localEvents.filter(e => e.type === 'fix_recorded').length,
        feedbackProvided: localEvents.filter(e => e.type === 'fix_recorded' && e.data.userFeedback).length
      }
    };
  }
}

function getLocalAnalytics(): AnalyticsEvent[] {
  try {
    const stored = localStorage.getItem('ai_learning_analytics');
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function calculateActiveDays(events: AnalyticsEvent[]): number {
  const days = new Set(
    events.map(e => new Date(e.timestamp).toDateString())
  );
  return days.size;
}

function calculateLocalSuccessRate(events: AnalyticsEvent[]): number {
  const fixEvents = events.filter(e => e.type === 'fix_recorded' || e.type === 'fix_failed');
  if (fixEvents.length === 0) return 0;
  
  const successCount = fixEvents.filter(e => e.type === 'fix_recorded').length;
  return successCount / fixEvents.length;
}

// ============================================================================
// React Hooks for Learning System Integration
// ============================================================================

/**
 * Hook for managing learning system state and operations
 */
export function useLearningSystem() {
  const [preferences, setPreferences] = useState<LearningPreferences | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<AIAnalysisError | null>(null);

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, []);

  const loadPreferences = useCallback(async () => {
    try {
      setIsLoading(true);
      const prefs = await getLearningPreferences();
      setPreferences(prefs);
      setError(null);
    } catch (err) {
      setError(err as AIAnalysisError);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updatePreferences = useCallback(async (newPreferences: LearningPreferences) => {
    try {
      setIsLoading(true);
      await updateLearningPreferences(newPreferences);
      setPreferences(newPreferences);
      setError(null);
    } catch (err) {
      setError(err as AIAnalysisError);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const recordFix = useCallback(async (
    errorPattern: ErrorPattern,
    appliedFix: FixSuggestion,
    context: string,
    userFeedback?: 'positive' | 'negative' | 'neutral'
  ) => {
    if (!preferences?.enableLearning) {
      return null;
    }

    try {
      return await recordSuccessfulFix(errorPattern, appliedFix, context, userFeedback);
    } catch (err) {
      setError(err as AIAnalysisError);
      throw err;
    }
  }, [preferences]);

  const getPatterns = useCallback(async (
    errorPattern: ErrorPattern,
    maxResults?: number
  ) => {
    if (!preferences?.enableLearning) {
      return [];
    }

    try {
      return await getLearnedPatterns(errorPattern, maxResults);
    } catch (err) {
      setError(err as AIAnalysisError);
      return [];
    }
  }, [preferences]);

  const clearData = useCallback(async (clearType?: 'all' | 'personal' | 'anonymous') => {
    try {
      setIsLoading(true);
      await clearLearningData(clearType);
      setError(null);
    } catch (err) {
      setError(err as AIAnalysisError);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    preferences,
    isLoading,
    error,
    loadPreferences,
    updatePreferences,
    recordFix,
    getPatterns,
    clearData,
    isEnabled: preferences?.enableLearning ?? false
  };
}

/**
 * Hook for enhanced fix suggestions with learning integration
 */
export function useEnhancedFixSuggestions(errorPattern: ErrorPattern | null) {
  const [learnedPatterns, setLearnedPatterns] = useState<LearnedPattern[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { isEnabled } = useLearningSystem();

  useEffect(() => {
    if (!errorPattern || !isEnabled) {
      setLearnedPatterns([]);
      return;
    }

    loadLearnedPatterns();
  }, [errorPattern, isEnabled]);

  const loadLearnedPatterns = useCallback(async () => {
    if (!errorPattern) return;

    try {
      setIsLoading(true);
      const patterns = await getLearnedPatterns(errorPattern);
      setLearnedPatterns(patterns);
    } catch (error) {
      console.error('Failed to load learned patterns:', error);
      setLearnedPatterns([]);
    } finally {
      setIsLoading(false);
    }
  }, [errorPattern]);

  const enhanceFixSuggestion = useCallback((
    fixSuggestion: FixSuggestion,
    contextSimilarity: number = 0.5
  ): FixSuggestion => {
    if (!isEnabled || learnedPatterns.length === 0) {
      return fixSuggestion;
    }

    const enhancedConfidence = calculateFixConfidence(
      fixSuggestion,
      learnedPatterns,
      contextSimilarity
    );

    const relevantPattern = learnedPatterns.find(pattern =>
      pattern.successfulFix.fixType === fixSuggestion.fixType
    );

    let successRate: number | undefined = undefined;
    if (relevantPattern) {
      const totalAttempts = relevantPattern.successCount + relevantPattern.failureCount;
      if (totalAttempts > 0) {
        successRate = relevantPattern.successCount / totalAttempts;
      }
    }

    return {
      ...fixSuggestion,
      confidence: enhancedConfidence,
      learnedFrom: relevantPattern?.id,
      successRate
    };
  }, [isEnabled, learnedPatterns]);

  return {
    learnedPatterns,
    isLoading,
    enhanceFixSuggestion,
    reload: loadLearnedPatterns
  };
}

/**
 * Hook for learning system analytics
 */
export function useLearningAnalytics() {
  const [analytics, setAnalytics] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAnalytics = useCallback(async () => {
    try {
      setIsLoading(true);
      const data = await getLearningAnalytics();
      setAnalytics(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAnalytics();
  }, [loadAnalytics]);

  return {
    analytics,
    isLoading,
    error,
    reload: loadAnalytics
  };
}

// ============================================================================
// Utility Functions for Error Pattern Matching
// ============================================================================

/**
 * Creates an error pattern from a compiler diagnostic or error message
 */
export function createErrorPattern(
  errorType: string,
  message: string,
  context: string,
  filePath?: string
): ErrorPattern {
  // Generate a pattern hash for deduplication
  const patternContent = `${errorType}:${message}:${context}`;
  const pattern = btoa(patternContent).slice(0, 32);

  return {
    id: `pattern_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    errorType,
    pattern,
    context: filePath ? `${filePath}:${context}` : context,
    frequency: 1,
    lastSeen: new Date().toISOString(),
    confidence: 0.8
  };
}

/**
 * Matches an error against known patterns
 */
export function matchErrorPattern(
  errorMessage: string,
  errorType: string,
  knownPatterns: ErrorPattern[]
): ErrorPattern | null {
  const candidates = knownPatterns.filter(pattern => 
    pattern.errorType === errorType
  );

  if (candidates.length === 0) {
    return null;
  }

  // Find the best matching pattern
  let bestMatch: ErrorPattern | null = null;
  let bestScore = 0;

  for (const pattern of candidates) {
    const score = calculateStringSimilarity(errorMessage, pattern.pattern);
    if (score > bestScore && score > 0.7) { // Minimum similarity threshold
      bestScore = score;
      bestMatch = pattern;
    }
  }

  return bestMatch;
}

export default {
  recordSuccessfulFix,
  recordFailedFix,
  getLearnedPatterns,
  calculateFixConfidence,
  calculateContextSimilarity,
  updateLearningPreferences,
  getLearningPreferences,
  clearLearningData,
  syncLearningData,
  getLearningSystemData,
  getLearningAnalytics,
  createErrorPattern,
  matchErrorPattern,
  useLearningSystem,
  useEnhancedFixSuggestions,
  useLearningAnalytics
};