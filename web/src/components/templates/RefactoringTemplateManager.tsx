import React from 'react';
import { invoke } from '@tauri-apps/api/core';

interface RefactoringTemplate {
  id: string;
  name: string;
  description: string;
  category: 'cleanup' | 'modernization' | 'performance' | 'maintenance' | 'architecture' | 'custom';
  operations: string[];
  config: {
    aiEnabled: boolean;
    confidenceThreshold: number;
    safetyLevel: 'low' | 'medium' | 'high';
    maxFileChanges: number;
    testGeneration: boolean;
    formatting: boolean;
  };
  metadata: {
    created: number;
    lastModified: number;
    author: string;
    version: string;
    tags: string[];
    usageCount: number;
    difficulty: 'beginner' | 'intermediate' | 'advanced';
    estimatedTimeMin: number;
    successRate: number; // Based on historical usage
  };
}

interface TemplatePreset {
  id: string;
  name: string;
  description: string;
  templates: string[]; // Template IDs
  conditions: {
    fileTypes: string[];
    minConfidence: number;
    maxFileSize: number;
    excludePatterns: string[];
  };
  priority: number;
  autoApply: boolean;
}

interface TemplateManagerState {
  builtInTemplates: RefactoringTemplate[];
  customTemplates: RefactoringTemplate[];
  presets: TemplatePreset[];
  currentEditingTemplate: RefactoringTemplate | null;
  isCreatingTemplate: boolean;
  isCreatingPreset: boolean;
  searchQuery: string;
  selectedCategory: string | null;
  showImportExport: boolean;
  templateUsageStats: { [key: string]: number };
}

class RefactoringTemplateManager extends React.Component<{}, TemplateManagerState> {
  constructor(props: {}) {
    super(props);

    this.state = {
      builtInTemplates: this.getBuiltInTemplates(),
      customTemplates: [],
      presets: [],
      currentEditingTemplate: null,
      isCreatingTemplate: false,
      isCreatingPreset: false,
      searchQuery: '',
      selectedCategory: null,
      showImportExport: false,
      templateUsageStats: {},
    };

    this.loadCustomData();
  }

  componentDidMount() {
    this.loadCustomData();
    this.trackUsageStats();
  }

  private getBuiltInTemplates(): RefactoringTemplate[] {
    return [
      {
        id: 'cleanup_basic',
        name: 'Basic Code Cleanup',
        description: 'Remove unused code, optimize imports, and apply basic formatting',
        category: 'cleanup',
        operations: ['removeDeadCode', 'organizeImports', 'formatCode', 'extractVariable'],
        config: {
          aiEnabled: false,
          confidenceThreshold: 0.8,
          safetyLevel: 'high',
          maxFileChanges: 100,
          testGeneration: false,
          formatting: true,
        },
        metadata: {
          created: Date.now() - 30 * 24 * 60 * 60 * 1000, // 30 days ago
          lastModified: Date.now(),
          author: 'System',
          version: '2.0.0',
          tags: ['cleanup', 'imports', 'formatting', 'safe'],
          usageCount: 0,
          difficulty: 'beginner',
          estimatedTimeMin: 15,
          successRate: 0.95,
        },
      },
      {
        id: 'modernize_api',
        name: 'API Modernization',
        description: 'Convert deprecated APIs to modern Rust patterns',
        category: 'modernization',
        operations: [
          'convertToAsync',
          'replaceDeprecatedApis',
          'updatePatterns',
          'changeSignature',
        ],
        config: {
          aiEnabled: true,
          confidenceThreshold: 0.7,
          safetyLevel: 'medium',
          maxFileChanges: 50,
          testGeneration: true,
          formatting: true,
        },
        metadata: {
          created: Date.now() - 15 * 24 * 60 * 60 * 1000, // 15 days ago
          lastModified: Date.now(),
          author: 'System',
          version: '1.0.0',
          tags: ['modernization', 'async', 'api', 'breaking'],
          usageCount: 0,
          difficulty: 'advanced',
          estimatedTimeMin: 45,
          successRate: 0.85,
        },
      },
      {
        id: 'performance_opt',
        name: 'Performance Optimization',
        description:
          'Optimize code for better performance with memory and algorithmic improvements',
        category: 'performance',
        operations: [
          'optimizeMemory',
          'algorithmRefinement',
          'inlineCriticalPaths',
          'reduceAllocations',
        ],
        config: {
          aiEnabled: true,
          confidenceThreshold: 0.9,
          safetyLevel: 'high',
          maxFileChanges: 25,
          testGeneration: true,
          formatting: false,
        },
        metadata: {
          created: Date.now() - 7 * 24 * 60 * 60 * 1000, // 7 days ago
          lastModified: Date.now(),
          author: 'System',
          version: '1.5.0',
          tags: ['performance', 'optimization', 'memory', 'speed'],
          usageCount: 0,
          difficulty: 'advanced',
          estimatedTimeMin: 60,
          successRate: 0.78,
        },
      },
      {
        id: 'architecture_refactor',
        name: 'Architecture Refactoring',
        description: 'Major architectural changes like class splitting and interface extraction',
        category: 'architecture',
        operations: ['extractInterface', 'splitClass', 'mergeClasses', 'moveMethod'],
        config: {
          aiEnabled: true,
          confidenceThreshold: 0.6,
          safetyLevel: 'low', // Architectural changes are risky
          maxFileChanges: 20,
          testGeneration: true,
          formatting: true,
        },
        metadata: {
          created: Date.now() - 3 * 24 * 60 * 60 * 1000, // 3 days ago
          lastModified: Date.now(),
          author: 'System',
          version: '1.2.0',
          tags: ['architecture', 'interfaces', 'classes', 'breaking'],
          usageCount: 0,
          difficulty: 'advanced',
          estimatedTimeMin: 120,
          successRate: 0.7,
        },
      },
      {
        id: 'maintenance_mode',
        name: 'Maintenance Mode',
        description: 'Light refactoring suitable for active development branches',
        category: 'maintenance',
        operations: ['extractFunction', 'extractVariable', 'rename', 'inlineVariable'],
        config: {
          aiEnabled: true,
          confidenceThreshold: 0.8,
          safetyLevel: 'high',
          maxFileChanges: 75,
          testGeneration: false,
          formatting: true,
        },
        metadata: {
          created: Date.now() - 1 * 24 * 60 * 60 * 1000, // Yesterday
          lastModified: Date.now(),
          author: 'System',
          version: '1.0.0',
          tags: ['maintenance', 'safe', 'incremental', 'development'],
          usageCount: 0,
          difficulty: 'intermediate',
          estimatedTimeMin: 30,
          successRate: 0.9,
        },
      },
    ];
  }

  private async loadCustomData() {
    try {
      const customTemplates = await invoke<string>('get_custom_templates', {});
      if (customTemplates) {
        this.setState({ customTemplates: JSON.parse(customTemplates) });
      }

      const presets = await invoke<string>('get_presets', {});
      if (presets) {
        this.setState({ presets: JSON.parse(presets) });
      }

      const usageStats = await invoke<string>('get_template_usage_stats', {});
      if (usageStats) {
        this.setState({ templateUsageStats: JSON.parse(usageStats) });
      }
    } catch (error) {
      console.log('Failed to load custom data:', error);
    }
  }

  private async saveCustomData() {
    try {
      await invoke('save_custom_templates', {
        templates: JSON.stringify(this.state.customTemplates),
      });
      await invoke('save_presets', { presets: JSON.stringify(this.state.presets) });
    } catch (error) {
      console.error('Failed to save custom data:', error);
    }
  }

  private trackUsageStats() {
    // Track real-time usage stats
    setInterval(async () => {
      const updatedStats = { ...this.state.templateUsageStats };
      // Implementation would get real stats from backend
      this.setState({ templateUsageStats: updatedStats });
    }, 60000); // Update every minute
  }

  createTemplate = async (template: Omit<RefactoringTemplate, 'id' | 'metadata'>) => {
    this.setState({ isCreatingTemplate: true });

    const newTemplate: RefactoringTemplate = {
      ...template,
      id: `custom_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      metadata: {
        created: Date.now(),
        lastModified: Date.now(),
        author: 'User', // Would get from current user profile
        version: '1.0.0',
        tags: [],
        usageCount: 0,
        difficulty: 'intermediate', // Default, can be adjusted
        estimatedTimeMin: 30,
        successRate: 0.8, // Initial estimate
      },
    };

    this.setState((prevState) => ({
      customTemplates: [...prevState.customTemplates, newTemplate],
      isCreatingTemplate: false,
    }));

    await this.saveCustomData();
  };

  updateTemplate = async (templateId: string, updates: Partial<RefactoringTemplate>) => {
    this.setState((prevState) => ({
      customTemplates: prevState.customTemplates.map((template) =>
        template.id === templateId
          ? {
              ...template,
              ...updates,
              metadata: { ...template.metadata, lastModified: Date.now() },
            }
          : template
      ),
    }));

    await this.saveCustomData();
  };

  deleteTemplate = async (templateId: string) => {
    this.setState((prevState) => ({
      customTemplates: prevState.customTemplates.filter((template) => template.id !== templateId),
    }));

    await this.saveCustomData();
  };

  createPreset = async (preset: Omit<TemplatePreset, 'id'>) => {
    this.setState({ isCreatingPreset: true });

    const newPreset: TemplatePreset = {
      ...preset,
      id: `preset_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };

    this.setState((prevState) => ({
      presets: [...prevState.presets, newPreset],
      isCreatingPreset: false,
    }));

    await this.saveCustomData();
  };

  applyTemplate = async (
    template: RefactoringTemplate,
    context?: {
      filePath?: string;
      selection?: { start: number; end: number };
      workspaceFilters?: any[];
    }
  ) => {
    try {
      // Track usage
      await invoke('track_template_usage', {
        templateId: template.id,
        context: {
          timestamp: Date.now(),
          filePath: context?.filePath,
          workspaceInfo: {}, // Would include workspace context
        },
      });

      // Create workflow from template
      const workflowRequest = {
        name: `Apply ${template.name}`,
        description: template.description,
        operations: template.operations,
        config: template.config,
        context,
      };

      const workflowId = await invoke<string>('create_workflow_from_template', workflowRequest);

      return workflowId;
    } catch (error) {
      console.error('Failed to apply template:', error);
      throw error;
    }
  };

  exportTemplates = (templateIds: string[]) => {
    // Create a Set of unique template IDs to avoid duplicates
    const uniqueTemplateIds = new Set(templateIds);

    // First collect custom templates, then add built-in templates that aren't overridden by custom ones
    const templatesToExport = [
      ...this.state.customTemplates.filter((template) => uniqueTemplateIds.has(template.id)),
      ...this.state.builtInTemplates.filter(
        (template) =>
          uniqueTemplateIds.has(template.id) &&
          !this.state.customTemplates.some((custom) => custom.id === template.id)
      ),
    ];

    const exportData = {
      templates: templatesToExport,
      presets: this.state.presets,
      exported: Date.now(),
      version: '1.0',
    };

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    // Create download link
    const a = document.createElement('a') as HTMLAnchorElement;
    a.href = url;
    a.download = `refactoring_templates_${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  importTemplates = () => {
    const input = document.createElement('input') as HTMLInputElement;
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = () => {
          try {
            const importData = JSON.parse(reader.result as string);
            if (importData.templates && Array.isArray(importData.templates)) {
              // Filter for custom templates (can't override built-ins)
              const newCustomTemplates = importData.templates.filter(
                (template: RefactoringTemplate) =>
                  !this.state.builtInTemplates.some((builtIn) => builtIn.id === template.id)
              );

              this.setState((prevState) => ({
                customTemplates: [...prevState.customTemplates, ...newCustomTemplates],
              }));

              this.saveCustomData();
            }
          } catch (error) {
            console.error('Failed to import templates:', error);
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  };

  getRecommendedTemplates = (context: {
    filePath?: string;
    fileSize?: number;
    selectionComplexity?: number;
    confidenceNeeded?: number;
  }): RefactoringTemplate[] => {
    const allTemplates = [...this.state.builtInTemplates, ...this.state.customTemplates];
    const contextTemplates = [];

    // Recommend based on file size
    if (context.fileSize) {
      if (context.fileSize < 1000) {
        // Small files - prefer simple, safe operations
        contextTemplates.push(...allTemplates.filter((t) => t.category === 'cleanup'));
      } else {
        // Large files - prefer performance operations
        contextTemplates.push(...allTemplates.filter((t) => t.category === 'performance'));
      }
    }

    // Recommend based on confidence needs
    if (context.confidenceNeeded) {
      contextTemplates.push(
        ...allTemplates.filter(
          (t) => t.config.confidenceThreshold >= (context.confidenceNeeded ?? 0)
        )
      );
    }

    // Remove duplicates and sort by success rate and usage
    const uniqueTemplates = Array.from(new Set(contextTemplates.map((t) => t.id)))
      .map((id) => allTemplates.find((t) => t.id === id))
      .filter((t): t is RefactoringTemplate => t !== undefined)
      .sort((a, b) => {
        const scoreA = this.getTemplateScore(a, context);
        const scoreB = this.getTemplateScore(b, context);
        return scoreB - scoreA;
      });

    return uniqueTemplates.slice(0, 5); // Top 5 recommendations
  };

  private getTemplateScore(template: RefactoringTemplate, context: any): number {
    let score = 0;

    // Success rate coefficient (0-10)
    score += template.metadata.successRate * 10;

    // Usage coefficient (more used = higher score, but with diminishing returns)
    const usageFactor = Math.min(template.metadata.usageCount / 100, 1) * 5;
    score += usageFactor;

    // Difficulty matching (prefer intermediate templates for general use)
    const difficultyBonus =
      template.metadata.difficulty === 'intermediate'
        ? 2
        : template.metadata.difficulty === 'beginner'
          ? 1
          : 0;
    score += difficultyBonus;

    // Context matching
    if (context.fileSize && template.category === 'performance' && context.fileSize > 2000) {
      score += 3; // Boost performance templates for large files
    }

    if (
      context.confidenceNeeded &&
      template.config.confidenceThreshold >= context.confidenceNeeded
    ) {
      score += 2; // Boost templates that meet confidence requirements
    }

    return score;
  }

  getTemplatesByCategory(category: string): RefactoringTemplate[] {
    return [
      ...this.state.builtInTemplates.filter((t) => t.category === category),
      ...this.state.customTemplates.filter((t) => t.category === category),
    ];
  }

  searchTemplates(query: string): RefactoringTemplate[] {
    if (!query.trim()) return [];

    const allTemplates = [...this.state.builtInTemplates, ...this.state.customTemplates];
    const lowercaseQuery = query.toLowerCase();

    return allTemplates.filter(
      (template) =>
        template.name.toLowerCase().includes(lowercaseQuery) ||
        template.description.toLowerCase().includes(lowercaseQuery) ||
        template.metadata.tags.some((tag) => tag.toLowerCase().includes(lowercaseQuery))
    );
  }

  render() {
    return (
      <div className="template-manager">
        {/* This would render the template management UI */}
        <div className="template-content">
          {this.state.builtInTemplates.length > 0 && (
            <div className="built-in-section">
              <h3>Built-in Templates</h3>
              {/* Template grid would go here */}
            </div>
          )}

          {this.state.customTemplates.length > 0 && (
            <div className="custom-templates-section">
              <h3>Custom Templates</h3>
              {/* Custom template grid would go here */}
            </div>
          )}
        </div>
      </div>
    );
  }
}

export default RefactoringTemplateManager;
export type { RefactoringTemplate, TemplatePreset };
