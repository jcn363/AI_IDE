import type {
  ProjectTemplate,
  BootstrapOptions,
  BootstrapResult,
  TemplateMarketplaceItem,
  TemplateFilter,
  BootstrapService as IBootstrapService,
} from './types';

export class BootstrapServiceImpl implements IBootstrapService {
  private templates: Map<string, ProjectTemplate> = new Map();

  constructor() {
    this.initializeBuiltinTemplates();
  }

  getTemplates(): ProjectTemplate[] {
    return Array.from(this.templates.values());
  }

  filterTemplates(filter: TemplateFilter): ProjectTemplate[] {
    return this.getTemplates().filter(template => {
      if (filter.category?.length && !filter.category.includes(template.category)) {
        return false;
      }
      if (filter.complexity?.length && !filter.complexity.includes(template.complexity)) {
        return false;
      }
      if (filter.tags?.length && !filter.tags.some(tag => template.tags.includes(tag))) {
        return false;
      }
      if (filter.searchTerm) {
        const search = filter.searchTerm.toLowerCase();
        if (!template.name.toLowerCase().includes(search) &&
            !template.description.toLowerCase().includes(search) &&
            !template.tags.some(tag => tag.toLowerCase().includes(search))) {
          return false;
        }
      }
      if (filter.featured !== undefined && template.featured !== filter.featured) {
        return false;
      }
      return true;
    });
  }

  getTemplate(id: string): ProjectTemplate | null {
    return this.templates.get(id) || null;
  }

  getMarketplaceTemplates(): TemplateMarketplaceItem[] {
    // Mock marketplace data
    return [];
  }

  async downloadTemplate(item: TemplateMarketplaceItem): Promise<void> {
    this.templates.set(item.template.id, item.template);
  }

  async uploadTemplate(template: ProjectTemplate): Promise<void> {
    this.templates.set(template.id, template);
  }

  validateBootstrapOptions(options: BootstrapOptions): { isValid: boolean; errors: string[]; warnings: string[] } {
    const errors: string[] = [];
    const warnings: string[] = [];

    if (!options.projectName || options.projectName.trim().length === 0) {
      errors.push('Project name is required');
    }

    if (!this.isValidProjectName(options.projectName)) {
      errors.push('Project name contains invalid characters');
    }

    if (!options.targetPath) {
      errors.push('Target path is required');
    }

    // Check if directory already exists
    if (this.directoryExists(options.targetPath + '/' + options.projectName)) {
      warnings.push('Target directory already exists and will be overwritten');
    }

    return { isValid: errors.length === 0, errors, warnings };
  }

  async bootstrapTemplate(templateId: string, options: BootstrapOptions): Promise<BootstrapResult> {
    const validation = this.validateBootstrapOptions(options);
    if (!validation.isValid) {
      return {
        success: false,
        projectPath: options.targetPath,
        filesCreated: [],
        dependencies: [],
        errors: validation.errors,
      };
    }

    const template = this.getTemplate(templateId);
    if (!template) {
      return {
        success: false,
        projectPath: options.targetPath,
        filesCreated: [],
        dependencies: [],
        errors: ['Template not found'],
      };
    }

    try {
      const result = await this.createProject(template, options);
      return result;
    } catch (error) {
      return {
        success: false,
        projectPath: options.targetPath,
        filesCreated: [],
        dependencies: [],
        errors: [error instanceof Error ? error.message : 'Unknown error occurred'],
      };
    }
  }

  previewTemplate(templateId: string, options?: Partial<BootstrapOptions>): { files: string[]; dependencies: string[] } {
    const template = this.getTemplate(templateId);
    if (!template) {
      return { files: [], dependencies: [] };
    }

    const files = template.files.map(file => file.path);
    const dependencies = this.extractDependencies(template);

    return { files, dependencies };
  }

  async setupAutomation(templateId: string, options: BootstrapOptions): Promise<{ setupCommands: string[]; configFiles: string[] }> {
    const template = this.getTemplate(templateId);
    if (!template?.automation) {
      return { setupCommands: [], configFiles: [] };
    }

    const setupCommands: string[] = [];
    const configFiles: string[] = [];

    // Setup git hooks
    if (template.automation.gitHooks) {
      setupCommands.push('git init');
      // Hook setup would be done via file creation
    }

    // Setup CI
    if (template.automation.ci && options.includeCI !== false) {
      setupCommands.push('echo "CI setup would be configured here"');
    }

    return { setupCommands, configFiles };
  }

  async generateWorkflows(templateId: string, workflows: string[]): Promise<Record<string, string>> {
    const workflowsConfig: Record<string, string> = {};

    // Generate basic workflows
    workflows.forEach(workflow => {
      switch (workflow) {
        case 'build':
          workflowsConfig['build.yml'] = this.generateBuildWorkflow();
          break;
        case 'test':
          workflowsConfig['test.yml'] = this.generateTestWorkflow();
          break;
        case 'deploy':
          workflowsConfig['deploy.yml'] = this.generateDeployWorkflow();
          break;
      }
    });

    return workflowsConfig;
  }

  async configureCI(templateId: string, ciProvider: string): Promise<string> {
    // Generate CI configuration
    switch (ciProvider) {
      case 'github-actions':
        return this.generateGitHubActionsConfig();
      case 'gitlab-ci':
        return this.generateGitLabCIConfig();
      default:
        return '';
    }
  }

  private initializeBuiltinTemplates(): void {
    // Basic Rust CLI template
    const rustCliTemplate: ProjectTemplate = {
      id: 'rust-cli-basic',
      name: 'Basic Rust CLI Tool',
      description: 'A simple command-line application template with basic structure and dependencies.',
      category: 'cli-tool',
      complexity: 'beginner',
      tags: ['rust', 'cli', 'beginner', 'tool'],
      author: 'RUST_AI_IDE',
      version: '1.0.0',
      lastUpdated: Date.now(),
      featured: true,
      files: [
        {
          path: 'Cargo.toml',
          content: `[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"
authors = ["{{author}}"]
description = "{{description}}"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
`,
          variables: {
            project_name: '{{project_name}}',
            author: '{{author}}',
            description: '{{description}}',
          },
        },
        {
          path: 'src/main.rs',
          content: `use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "{{project_name}}")]
#[command(author = "{{author}}")]
#[command(version = "1.0")]
#[command(about = "{{description}}")]
struct Args {
    /// Input file
    #[arg(short, long)]
    input: Option<String>,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("{{project_name}} v1.0");
    println!("Input: {{}}, Output: {{}}",
             args.input.as_deref().unwrap_or("none"),
             args.output.as_deref().unwrap_or("none"));

    // Your application logic here

    Ok(())
}
`,
          variables: {
            project_name: '{{project_name}}',
            author: '{{author}}',
            description: '{{description}}',
          },
        },
        {
          path: 'README.md',
          content: `# {{project_name}}

{{description}}

## Usage

\`\`\`bash
cargo run -- --input input.txt --output output.txt
\`\`\`

## Development

\`\`\`bash
cargo build
cargo test
cargo run
\`\`\`

## License

{{license}}
`,
          variables: {
            project_name: '{{project_name}}',
            description: '{{description}}',
            license: '{{license}}',
          },
        },
      ],
      folders: [
        { path: 'src' },
        { path: 'tests' },
        { path: 'docs' },
      ],
      config: {
        packageManager: 'cargo',
        testFramework: 'cargo-test',
        linter: 'clippy',
      },
      automation: {
        gitHooks: {
          preCommit: ['cargo clippy'],
          prePush: ['cargo test'],
        },
        ci: {
          provider: 'github-actions',
          config: {
            rust_toolchain: 'stable',
            rustfmt: true,
            clippy: true,
          },
        },
      },
      preview: {
        description: 'Perfect for building command-line utilities, automation scripts, and system tools.',
        features: [
          'CLI argument parsing with clap',
          'Error handling with anyhow',
          'Async support with tokio',
          'Ready-to-develop structure',
        ],
      },
    };

    this.templates.set(rustCliTemplate.id, rustCliTemplate);
  }

  private isValidProjectName(name: string): boolean {
    // Basic validation for project names
    const valid = /^[a-zA-Z][a-zA-Z0-9_-]*$/.test(name);
    return valid && name.length <= 50;
  }

  private directoryExists(path: string): boolean {
    // In a real implementation, this would check filesystem
    return false;
  }

  private async createProject(template: ProjectTemplate, options: BootstrapOptions): Promise<BootstrapResult> {
    const filesCreated: string[] = [];
    const dependencies: string[] = [];
    const nextSteps: string[] = [];

    try {
      // Replace template variables
      const variableReplacements = {
        project_name: options.projectName,
        author: options.author || 'Developer',
        description: options.description || 'A new project',
        license: options.license || 'MIT',
        ...options.variables,
      };

      // Create directories
      if (template.folders) {
        for (const folder of template.folders) {
          // Create directory - in real implementation
          console.log(`Creating directory: ${folder.path}`);
        }
      }

      // Create files
      for (const file of template.files) {
        let content = file.content;

        // Replace variables
        if (file.variables) {
          Object.entries(file.variables).forEach(([key, value]) => {
            content = content.replace(new RegExp(value, 'g'), variableReplacements[key] || value);
          });
        }

        // Write file - in real implementation
        console.log(`Creating file: ${file.path}`);
        filesCreated.push(file.path);
      }

      // Setup dependencies
      if (template.config.cargo?.dependencies) {
        dependencies.push(...Object.keys(template.config.cargo.dependencies));
      }

      nextSteps.push(
        'Navigate to the project directory',
        'Run cargo build to build the project',
        'Run cargo run to execute the application',
        'Run cargo test to run tests'
      );

    } catch (error) {
      throw new Error('Failed to create project structure');
    }

    return {
      success: true,
      projectPath: `${options.targetPath}/${options.projectName}`,
      filesCreated,
      dependencies,
      nextSteps,
    };
  }

  private extractDependencies(template: ProjectTemplate): string[] {
    const dependencies: string[] = [];

    if (template.config.cargo?.dependencies) {
      dependencies.push(...Object.keys(template.config.cargo.dependencies));
    }

    if (template.config.npm?.dependencies) {
      dependencies.push(...Object.keys(template.config.npm.dependencies));
    }

    return dependencies;
  }

  private generateBuildWorkflow(): string {
    return `
name: Build
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Use latest stable
      uses: dtolnay/rust-toolchain@stable
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
`;
  }

  private generateTestWorkflow(): string {
    return `
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Use latest stable
      uses: dtolnay/rust-toolchain@stable
    - name: Test
      run: cargo test --verbose
    - name: Check formatting
      run: cargo fmt --check
    - name: Lint with clippy
      run: cargo clippy -- -D warnings
`;
  }

  private generateDeployWorkflow(): string {
    return `
name: Deploy
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Deploy to production
      run: echo "Deployment would happen here"
`;
  }

  private generateGitHubActionsConfig(): string {
    return `
name: Rust

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:

    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
`;
  }

  private generateGitLabCIConfig(): string {
    return `
stages:
  - build
  - test

build:
  stage: build
  script:
    - cargo build --verbose

test:
  stage: test
  script:
    - cargo test --verbose
    - cargo clippy -- -D warnings
`;
  }
}

// Create singleton instance
export const bootstrapService = new BootstrapServiceImpl();