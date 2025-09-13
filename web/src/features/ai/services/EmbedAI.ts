// Embed AI services integration
import type { CombinedAnalysisResult, AIAnalysisConfig } from '../types';

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export interface CodeAnalysisRequest {
  code: string;
  context?: string;
  analysisType: 'full' | 'syntax' | 'semantic' | 'security';
}

export class EmbedAIService {
  private static instance: EmbedAIService;

  static getInstance(): EmbedAIService {
    if (!EmbedAIService.instance) {
      EmbedAIService.instance = new EmbedAIService();
    }
    return EmbedAIService.instance;
  }

  async analyzeCode(request: CodeAnalysisRequest): Promise<CombinedAnalysisResult> {
    try {
      return await window.electron.invoke<CombinedAnalysisResult>('analyze_code_embed', {
        request,
      });
    } catch (error) {
      console.error('EmbedAI analysis failed:', error);
      throw error;
    }
  }

  async chat(messages: ChatMessage[], config: AIAnalysisConfig): Promise<string> {
    try {
      return await window.electron.invoke<string>('chat_with_context', {
        messages,
        config,
      });
    } catch (error) {
      console.error('EmbedAI chat failed:', error);
      throw error;
    }
  }

  async getCompletion(
    prompt: string,
    context: string[],
    config: AIAnalysisConfig
  ): Promise<string> {
    try {
      return await window.electron.invoke<string>('get_completion', {
        prompt,
        context,
        config,
      });
    } catch (error) {
      console.error('EmbedAI completion failed:', error);
      throw error;
    }
  }

  async explainError(
    error: string,
    code: string,
    context: string[],
    config: AIAnalysisConfig
  ): Promise<string> {
    try {
      return await window.electron.invoke<string>('explain_error', {
        error,
        code,
        context,
        config,
      });
    } catch (error) {
      console.error('EmbedAI error explanation failed:', error);
      throw error;
    }
  }

  async suggestRefactoring(code: string, context: string, config: AIAnalysisConfig): Promise<any> {
    try {
      return await window.electron.invoke('suggest_refactoring', {
        code,
        context,
        config,
      });
    } catch (error) {
      console.error('EmbedAI refactoring suggestion failed:', error);
      throw error;
    }
  }

  async generateTests(code: string, context: string, config: AIAnalysisConfig): Promise<string[]> {
    try {
      return await window.electron.invoke<string[]>('generate_tests', {
        code,
        context,
        config,
      });
    } catch (error) {
      console.error('EmbedAI test generation failed:', error);
      throw error;
    }
  }

  async detectPatterns(code: string, patterns: string[], config: AIAnalysisConfig): Promise<any> {
    try {
      return await window.electron.invoke('detect_patterns', {
        code,
        patterns,
        config,
      });
    } catch (error) {
      console.error('EmbedAI pattern detection failed:', error);
      throw error;
    }
  }
}
