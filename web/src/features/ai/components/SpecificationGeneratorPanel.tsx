import React, { useState, useEffect } from 'react';
import type {
  CodeSpecification,
  GeneratedCode,
  GeneratedFile,
  ValidationResult
} from '../types';
import styles from './SpecificationGeneratorPanel.module.css';

interface SpecificationGeneratorPanelProps {
  className?: string;
}

export const SpecificationGeneratorPanel: React.FC<SpecificationGeneratorPanelProps> = ({ className }) => {
  const [specification, setSpecification] = useState('');
  const [language, setLanguage] = useState('rust');
  const [generatedCode, setGeneratedCode] = useState<GeneratedCode | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'input' | 'output'>('input');
  const [outputTab, setOutputTab] = useState<'main' | 'modules' | 'supporting' | 'build'>('main');

  const handleGenerate = async () => {
    if (!specification.trim()) {
      setError('Please enter a specification');
      return;
    }

    setIsGenerating(true);
    setError(null);

    try {
      // This would call the Tauri command for specification-based generation
      // For now, create a mock response
      await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate generation time

      const mockGeneratedCode: GeneratedCode = {
        main_file: {
          path: 'src/main.rs',
          content: `// Generated from specification: ${specification.substring(0, 50)}...
fn main() {
    println!("Generated Rust application");
    println!("Specification: {}", specification);
}`,
          file_type: 'SourceCode',
          description: 'Main application file generated from specification'
        },
        modules: [
          {
            name: 'utils',
            description: 'Utility module',
            files: [{
              path: 'src/utils.rs',
              content: `pub mod utils {
    pub fn helper_function() -> String {
        "Helper function".to_string()
    }
}`,
              file_type: 'SourceCode',
              description: 'Utilities module'
            }]
          }
        ],
        supporting_files: [
          {
            path: 'Cargo.toml',
            content: `[package]
name = "generated_app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"`,
            file_type: 'Configuration',
            description: 'Package configuration'
          },
          {
            path: 'README.md',
            content: `# Generated Application

This application was generated from specification: ${specification}

## Features
- Auto-generated based on requirements
- Ready to build and run
`,
            file_type: 'Documentation',
            description: 'Project documentation'
          }
        ],
        dependencies: ['serde', 'tokio'],
        build_instructions: [
          'cargo build',
          'cargo test',
          'cargo run'
        ],
        validation_report: {
          is_valid: true,
          score: 0.9,
          issues: [],
          suggestions: [
            'Consider adding more error handling',
            'Add comprehensive tests'
          ]
        }
      };

      setGeneratedCode(mockGeneratedCode);
      setOutputTab('main');
      setActiveTab('output');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Generation failed');
    } finally {
      setIsGenerating(false);
    }
  };

  const renderInputTab = () => (
    <div className={styles.specificationInput}>
      <div className={styles.inputControls}>
        <div className={styles.controlGroup}>
          <label htmlFor="language-select">Target Language:</label>
          <select
            id="language-select"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            disabled={isGenerating}
          >
            <option value="rust">Rust</option>
            <option value="javascript">JavaScript/TypeScript</option>
            <option value="python">Python</option>
            <option value="go">Go</option>
          </select>
        </div>
      </div>

      <div className={styles.specificationEditor}>
        <label htmlFor="spec-textarea">Specification:</label>
        <textarea
          id="spec-textarea"
          value={specification}
          onChange={(e) => setSpecification(e.target.value)}
          placeholder={`Enter your ${language} application specification here...

Example for Rust:
"Create a web server that serves a REST API for managing users.
The server should:
- Handle GET/POST/PUT/DELETE for /users endpoint
- Store users in memory (struct User { id: u64, name: String, email: String })
- Return JSON responses
- Handle basic error cases"`}
          rows={20}
          disabled={isGenerating}
        />
      </div>

      {error && (
        <div className={styles.errorMessage}>
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className={styles.actionButtons}>
        <button
          onClick={handleGenerate}
          disabled={isGenerating || !specification.trim()}
          className={styles.generateButton}
        >
          {isGenerating ? 'Generating...' : 'Generate Code'}
        </button>
      </div>
    </div>
  );

  const renderOutputTab = () => {
    if (!generatedCode) {
      return <div className={styles.noOutput}>No generated code available</div>;
    }

    return (
      <div className="generated-output">
        <div className={styles.validationSummary}>
          <div className={`validation-status ${generatedCode.validation_report.is_valid ? 'valid' : 'invalid'}`}>
            <span className="status-icon">{generatedCode.validation_report.is_valid ? '✓' : '⚠'}</span>
            <span>Validation: {generatedCode.validation_report.score * 100}%</span>
          </div>

          {generatedCode.validation_report.suggestions?.length > 0 && (
            <div className={styles.suggestions}>
              <h4>Suggestions:</h4>
              <ul>
                {generatedCode.validation_report.suggestions.map((suggestion: string, index: number) => (
                  <li key={index}>{suggestion}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

        <div className={styles.codeTabs}>
          <div className={styles.tabHeader}>
            <button
              className={outputTab === 'main' ? styles.active : ''}
              onClick={() => setOutputTab('main')}
            >
              Main File
            </button>
            <button
              className={outputTab === 'modules' ? styles.active : ''}
              onClick={() => setOutputTab('modules')}
            >
              Modules ({generatedCode.modules.length})
            </button>
            <button
              className={outputTab === 'supporting' ? styles.active : ''}
              onClick={() => setOutputTab('supporting')}
            >
              Supporting Files ({generatedCode.supporting_files.length})
            </button>
            <button
              className={outputTab === 'build' ? styles.active : ''}
              onClick={() => setOutputTab('build')}
            >
              Build Instructions
            </button>
          </div>

          <div className={styles.tabContent}>
            {outputTab === 'main' && renderFileContent(generatedCode.main_file)}
            {outputTab === 'modules' && renderModules(generatedCode.modules)}
            {outputTab === 'supporting' && renderSupportingFiles(generatedCode.supporting_files)}
            {outputTab === 'build' && renderBuildInstructions(generatedCode.build_instructions)}
          </div>
        </div>

        <div className="dependencies">
          <h4>Dependencies:</h4>
          <ul className="dependency-list">
            {generatedCode.dependencies.map((dep, index) => (
              <li key={index}>
                <code>{dep}</code>
              </li>
            ))}
          </ul>
        </div>
      </div>
    );
  };

  const renderFileContent = (file: GeneratedFile) => (
    <div className="file-content">
      <div className="file-header">
        <span className="file-path">{file.path}</span>
        <span className="file-type">{file.file_type}</span>
      </div>
      <pre className="code-block">
        <code>{file.content}</code>
      </pre>
      <p className="file-description">{file.description}</p>
    </div>
  );

  const renderModules = (modules: any[]) => (
    <div className="modules-list">
      {modules.map((module, index) => (
        <div key={index} className="module">
          <h4>{module.name}</h4>
          <p>{module.description}</p>
          {module.files.map((file: GeneratedFile, fileIndex: number) => (
            <div key={fileIndex} className="module-file">
              {renderFileContent(file)}
            </div>
          ))}
        </div>
      ))}
    </div>
  );

  const renderSupportingFiles = (files: GeneratedFile[]) => (
    <div className="supporting-files-list">
      {files.map((file, index) => (
        <div key={index} className="supporting-file">
          {renderFileContent(file)}
        </div>
      ))}
    </div>
  );

  const renderBuildInstructions = (instructions: string[]) => (
    <div className="build-instructions">
      <ol>
        {instructions.map((instruction, index) => (
          <li key={index}>
            <code>{instruction}</code>
          </li>
        ))}
      </ol>
    </div>
  );

  return (
    <div className={`${styles.specificationGeneratorPanel} ${className || ''}`}>
      <div className={styles.panelHeader}>
        <h3>Specification-Based Code Generation</h3>
      </div>

      <div className="panel-tabs">
        <button
          className={activeTab === 'input' ? 'active' : ''}
          onClick={() => setActiveTab('input')}
        >
          Input Specification
        </button>
        <button
          className={activeTab === 'output' && generatedCode ? 'active' : 'disabled'}
          onClick={() => generatedCode && setActiveTab('output')}
          disabled={!generatedCode}
        >
          Generated Code
        </button>
      </div>

      <div className="panel-content">
        {activeTab === 'input' ? renderInputTab() : renderOutputTab()}
      </div>

    </div>
  );
};

export default SpecificationGeneratorPanel;