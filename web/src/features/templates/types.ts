export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  complexity: 'beginner' | 'intermediate' | 'advanced';
  tags: string[];
  author: string;
  version: string;
  lastUpdated: number;
  featured?: boolean;

  // Template structure
  files: TemplateFile[];
  folders: TemplateFolder[];

  // Configuration
  config: TemplateConfig;

  // Automation metadata
  automation?: TemplateAutomation;

  // Preview
  preview?: {
    screenshot?: string;
    description: string;
    features: string[];
  };
}

export type TemplateCategory =
  | 'web-development'
  | 'mobile-development'
  | 'desktop-application'
  | 'library'
  | 'cli-tool'
  | 'game-development'
  | 'ai-ml'
  | 'system-programming'
  | 'cloud-infrastructure'
  | 'data-analysis';

export interface TemplateFile {
  path: string;
  content: string;
  executable?: boolean;
  variables?: Record<string, string>; // Template variables to replace
}

export interface TemplateFolder {
  path: string;
  children?: TemplateFolder[];
}

export interface TemplateConfig {
  // Package management
  packageManager: 'cargo' | 'npm' | 'yarn' | 'pnpm';
  cargo?: {
    name?: string;
    version?: string;
    edition?: string;
    dependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
    features?: string[];
  };
  npm?: {
    name?: string;
    version?: string;
    scripts?: Record<string, string>;
    dependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
  };

  // Build configuration
  buildSystem?: 'cargo-make' | 'just' | 'make' | 'webpack' | 'vite' | 'rollup';
  buildConfig?: Record<string, any>;

  // Testing
  testFramework?: 'cargo-test' | 'jest' | 'vitest' | 'tape';
  testConfig?: Record<string, any>;

  // Linting
  linter: 'clippy' | 'eslint' | 'prettier';
  lintConfig?: Record<string, any>;

  // Development tools
  devTools?: {
    hotReload?: boolean;
    devServer?: {
      port?: number;
      host?: string;
    };
  };
}

export interface TemplateAutomation {
  // Git hooks
  gitHooks?: {
    preCommit?: string[];
    prePush?: string[];
    postMerge?: string[];
  };

  // CI/CD
  ci?: {
    provider: 'github-actions' | 'gitlab-ci' | 'travis-ci' | 'circle-ci';
    config: Record<string, any>;
  };

  // Deployment
  deployment?: {
    provider: 'vercel' | 'netlify' | 'heroku' | 'aws' | 'docker';
    config: Record<string, any>;
  };

  // Code quality
  codeQuality?: {
    coverage: {
      provider: 'codecov' | 'coveralls' | 'sonarcloud';
      threshold: number;
    };
    security: {
      sast: boolean;
      dast: boolean;
    };
  };
}

export interface BootstrapOptions {
  // Project metadata
  projectName: string;
  description?: string;
  author?: string;
  license?: string;
  repository?: string;

  // Template-specific variables
  variables?: Record<string, string>;

  // Paths
  targetPath: string;

  // Options
  includeAutomation?: boolean;
  includeCI?: boolean;
  includeDeployment?: boolean;
  skipDependencies?: boolean;
  useLatestVersions?: boolean;
}

export interface BootstrapResult {
  success: boolean;
  projectPath: string;
  filesCreated: string[];
  dependencies: string[];
  errors?: string[];
  warnings?: string[];
  nextSteps?: string[];
}

export interface TemplateMarketplaceItem {
  template: ProjectTemplate;
  downloads: number;
  rating: number;
  reviews: number;
  author: AuthorInfo;
  repository?: string;
  issues?: string;
  compatibleVersions?: string[];
  updatedAt: number;
  tags: string[];
}

export interface AuthorInfo {
  name: string;
  email?: string;
  website?: string;
  avatar?: string;
}

export interface TemplateFilter {
  category?: TemplateCategory[];
  complexity?: ('beginner' | 'intermediate' | 'advanced')[];
  tags?: string[];
  author?: string;
  searchTerm?: string;
  featured?: boolean;
}

export interface BootstrapService {
  getTemplates(): ProjectTemplate[];
  filterTemplates(filter: TemplateFilter): ProjectTemplate[];
  getTemplate(id: string): ProjectTemplate | null;
  getMarketplaceTemplates(): TemplateMarketplaceItem[];
  downloadTemplate(item: TemplateMarketplaceItem): Promise<void>;
  uploadTemplate(template: ProjectTemplate): Promise<void>;

  // Bootstrap functionality
  validateBootstrapOptions(options: BootstrapOptions): {
    isValid: boolean;
    errors: string[];
    warnings: string[];
  };
  bootstrapTemplate(templateId: string, options: BootstrapOptions): Promise<BootstrapResult>;
  previewTemplate(
    templateId: string,
    options?: Partial<BootstrapOptions>
  ): { files: string[]; dependencies: string[] };

  // Workflow automation
  setupAutomation(
    templateId: string,
    options: BootstrapOptions
  ): Promise<{ setupCommands: string[]; configFiles: string[] }>;
  generateWorkflows(templateId: string, workflows: string[]): Promise<Record<string, string>>;
  configureCI(templateId: string, ciProvider: string): Promise<string>;
}
