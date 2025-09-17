import React from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface TestCoverage {
  file: string;
  lines: Array<{
    line: number;
    covered: boolean;
    count?: number;
    branches?: Array<{ taken: boolean; count?: number }>;
  }>;
  functions: Array<{
    name: string;
    start: number;
    end: number;
    executed: boolean;
    count?: number;
  }>;
  percentage: number;
  totalLines: number;
  coveredLines: number;
}

interface TestResult {
  test: string;
  module: string;
  duration: number;
  status: "passed" | "failed" | "ignored";
  stdout?: string;
  stderr?: string;
}

interface CoverageAnalysis {
  overallCoverage: number;
  filesCoverage: TestCoverage[];
  missingCoverage: Array<{
    file: string;
    lines: number[];
    reason: "not_tested" | "complex_logic" | "edge_case";
    priority: "high" | "medium" | "low";
  }>;
  testResults: TestResult[];
  generationRecommendations: Array<{
    type: "unit_test" | "integration_test" | "property_test";
    location: string;
    description: string;
    template: string;
    priority: number;
  }>;
}

interface CodeCoverageIntegrationState {
  coverage: CoverageAnalysis | null;
  isAnalyzing: boolean;
  isGeneratingTests: boolean;
  autoMode: boolean;
  coverageThreshold: number;
  lastAnalysis: number;
  generatedTests: Array<{
    id: string;
    file: string;
    content: string;
    applied: boolean;
    coverageImpact: number;
  }>;
}

class CodeCoverageIntegration extends React.Component<
  {
    currentFile?: string;
    onCoverageUpdate?: (coverage: CoverageAnalysis) => void;
    onTestGenerated?: (testFile: string) => void;
  },
  CodeCoverageIntegrationState
> {
  constructor(props: any) {
    super(props);

    this.state = {
      coverage: null,
      isAnalyzing: false,
      isGeneratingTests: false,
      autoMode: false,
      coverageThreshold: 80,
      lastAnalysis: 0,
      generatedTests: [],
    };
  }

  componentDidMount() {
    this.loadConfigurations();
    if (this.props.currentFile) {
      this.analyzeCoverageForFile(this.props.currentFile);
    }
  }

  componentDidUpdate(prevProps: any) {
    if (
      prevProps.currentFile !== this.props.currentFile &&
      this.props.currentFile
    ) {
      this.analyzeCoverageForFile(this.props.currentFile);
    }
  }

  private async loadConfigurations() {
    try {
      const config = await invoke<{ autoMode?: boolean; threshold?: number }>(
        "get_coverage_config",
        {},
      );
      if (config) {
        this.setState({
          autoMode: config.autoMode || false,
          coverageThreshold: config.threshold || 80,
        });
      }
    } catch (error) {
      console.log("No saved coverage configuration found");
    }
  }

  private async saveConfigurations() {
    try {
      await invoke("save_coverage_config", {
        autoMode: this.state.autoMode,
        threshold: this.state.coverageThreshold,
      });
    } catch (error) {
      console.error("Failed to save coverage configuration:", error);
    }
  }

  async analyzeCoverage(runTests = false): Promise<CoverageAnalysis> {
    this.setState({ isAnalyzing: true });

    try {
      // Run cargo-tarpaulin for coverage analysis
      const coverageResults = await invoke<CoverageAnalysis>(
        "run_coverage_analysis",
        {
          withTests: runTests,
        },
      );

      this.setState({
        coverage: coverageResults,
        lastAnalysis: Date.now(),
        isAnalyzing: false,
      });

      if (this.props.onCoverageUpdate) {
        this.props.onCoverageUpdate(coverageResults);
      }

      return coverageResults;
    } catch (error) {
      console.error("Coverage analysis failed:", error);
      this.setState({ isAnalyzing: false });

      // Return minimal coverage data
      const fallbackCoverage: CoverageAnalysis = {
        overallCoverage: 0,
        filesCoverage: [],
        missingCoverage: [],
        testResults: [],
        generationRecommendations: [],
      };

      return fallbackCoverage;
    }
  }

  async analyzeCoverageForFile(filePath: string): Promise<TestCoverage | null> {
    try {
      const fileCoverage = await invoke<TestCoverage>(
        "run_file_coverage_analysis",
        {
          filePath,
        },
      );

      // Update the overall coverage data with this file's info
      this.setState((prevState) => {
        if (!prevState.coverage) return prevState;

        const updatedFilesCoverage = prevState.coverage.filesCoverage.map(
          (fc) => (fc.file === filePath ? fileCoverage : fc),
        );

        return {
          coverage: prevState.coverage
            ? {
                ...prevState.coverage,
                filesCoverage: updatedFilesCoverage,
              }
            : null,
        };
      });

      return fileCoverage;
    } catch (error) {
      console.error(`Coverage analysis failed for ${filePath}:`, error);
      return null;
    }
  }

  async generateTestsForCoverage(
    format: "tarpaulin" | "html" | "json" = "json",
  ): Promise<void> {
    if (!this.state.coverage) return;

    this.setState({ isGeneratingTests: true });

    try {
      const generatedTests = await invoke<
        Array<{ file: string; content: string; coverageIncrease: number }>
      >("generate_coverage_tests", {
        missingCoverage: this.state.coverage.missingCoverage,
        recommendations: this.state.coverage.generationRecommendations,
        generateAll: this.state.autoMode,
      });

      const testItems = generatedTests.map((test, index) => ({
        id: `test_${Date.now()}_${index}`,
        file: test.file,
        content: test.content,
        applied: false,
        coverageImpact: test.coverageIncrease,
      }));

      this.setState((prevState) => ({
        generatedTests: [...prevState.generatedTests, ...testItems],
        isGeneratingTests: false,
      }));
    } catch (error) {
      console.error("Test generation failed:", error);
      this.setState({ isGeneratingTests: false });
    }
  }

  async applyGeneratedTest(testId: string): Promise<void> {
    try {
      const test = this.state.generatedTests.find((t) => t.id === testId);
      if (!test) return;

      await invoke("apply_generated_test", {
        testContent: test.content,
        testFile: test.file,
      });

      this.setState((prevState) => ({
        generatedTests: prevState.generatedTests.map((t) =>
          t.id === testId ? { ...t, applied: true } : t,
        ),
      }));

      if (this.props.onTestGenerated) {
        this.props.onTestGenerated(test.file);
      }

      // Re-analyze coverage to see the impact
      await this.analyzeCoverage(true);
    } catch (error) {
      console.error("Failed to apply test:", error);
    }
  }

  async runCoverageEnhancedRefactoring(
    operationType: string,
    targetFile: string,
  ): Promise<void> {
    try {
      const refactoringRequest = {
        operation: operationType,
        file: targetFile,
        coverageContext: this.state.coverage,
        preserveCoverage: true,
      };

      const result = await invoke(
        "refactor_with_coverage_awareness",
        refactoringRequest,
      );

      // Refresh coverage analysis after refactoring
      await this.analyzeCoverage(true);
    } catch (error) {
      console.error("Coverage-enhanced refactoring failed:", error);
    }
  }

  getCoverageDrivenSuggestions(): Array<{
    type: "coverage_gap" | "test_missing" | "edge_case";
    description: string;
    priority: "high" | "medium" | "low";
    action: () => void;
  }> {
    if (!this.state.coverage) return [];

    const suggestions = [];

    // Low overall coverage suggestion
    if (this.state.coverage.overallCoverage < this.state.coverageThreshold) {
      suggestions.push({
        type: "coverage_gap",
        description: `Overall coverage is ${(100 - this.state.coverage.overallCoverage).toFixed(1)}% below target (${this.state.coverageThreshold}%)`,
        priority: "high",
        action: () => this.generateTestsForCoverage(),
      });
    }

    // Files with low coverage
    const lowCoverageFiles = this.state.coverage.filesCoverage.filter(
      (fc) => fc.percentage < this.state.coverageThreshold,
    );

    if (lowCoverageFiles.length > 0) {
      suggestions.push({
        type: "coverage_gap",
        description: `${lowCoverageFiles.length} file(s) below coverage threshold`,
        priority: "medium",
        action: () => {
          // Focus generation on these files
          this.generateTestsForCoverage();
        },
      });
    }

    // Missing test recommendations
    if (this.state.coverage.generationRecommendations.length > 0) {
      suggestions.push({
        type: "test_missing",
        description: `${this.state.coverage.generationRecommendations.length} test cases recommended by coverage analysis`,
        priority: "medium",
        action: () => this.generateTestsForCoverage(),
      });
    }

    return suggestions;
  }

  render() {
    const {
      coverage,
      isAnalyzing,
      isGeneratingTests,
      autoMode,
      coverageThreshold,
      lastAnalysis,
      generatedTests,
    } = this.state;

    const coverageDrivenSuggestions = this.getCoverageDrivenSuggestions();

    return (
      <div className="code-coverage-integration">
        <div className="coverage-header">
          <h4>Code Coverage & Testing</h4>
          <div className="coverage-controls">
            <button
              className={`btn ${isAnalyzing ? "disabled" : "primary"}`}
              onClick={() => this.analyzeCoverage(true)}
              disabled={isAnalyzing}
            >
              {isAnalyzing ? "🔄 Analyzing..." : "📊 Analyze Coverage"}
            </button>
            <button
              className={`btn ${isGeneratingTests ? "disabled" : "secondary"}`}
              onClick={() => this.generateTestsForCoverage()}
              disabled={isGeneratingTests}
            >
              {isGeneratingTests ? "🔄 Generating..." : "🧪 Generate Tests"}
            </button>
            <div className="coverage-threshold-setting">
              <label>Target: </label>
              <input
                type="number"
                min="0"
                max="100"
                value={coverageThreshold}
                onChange={(e) => {
                  const value = parseInt(e.target.value) || 80;
                  this.setState({ coverageThreshold: value }, () => {
                    this.saveConfigurations();
                  });
                }}
                style={{ width: "50px", marginLeft: "5px" }}
              />
              <span>%</span>
            </div>
            <label className="auto-mode-toggle">
              <input
                type="checkbox"
                checked={autoMode}
                onChange={(e) => {
                  this.setState({ autoMode: e.target.checked }, () => {
                    this.saveConfigurations();
                  });
                }}
              />
              Auto-generate
            </label>
          </div>
        </div>

        <div className="coverage-content">
          {this.renderCoverageOverview()}
          {this.renderFileCoverageTable()}
          {this.renderCoverageSuggestions()}
          {this.renderGeneratedTests()}
          {this.renderTestResults()}
        </div>

        <div className="coverage-footer">
          {lastAnalysis > 0 && (
            <div className="analysis-timestamp">
              Last analyzed: {new Date(lastAnalysis).toLocaleString()}
            </div>
          )}
        </div>
      </div>
    );
  }

  private renderCoverageOverview() {
    const { coverage } = this.state;

    if (!coverage) {
      return (
        <div className="coverage-overview">
          <div className="no-coverage-data">
            <div className="no-data-icon">📊</div>
            <h5>No Coverage Data</h5>
            <p>Run coverage analysis to see test coverage information.</p>
          </div>
        </div>
      );
    }

    const coverageColor =
      coverage.overallCoverage >= this.state.coverageThreshold
        ? "success"
        : coverage.overallCoverage >= 50
          ? "warning"
          : "error";

    return (
      <div className="coverage-overview">
        <div className="coverage-summary">
          <div className="coverage-metric">
            <h3 className={coverageColor}>
              {coverage.overallCoverage.toFixed(1)}%
            </h3>
            <p>Overall Coverage</p>
          </div>
          <div className="coverage-metric">
            <h3>{coverage.testResults.length}</h3>
            <p>Total Tests</p>
          </div>
          <div className="coverage-metric">
            <h3>
              {coverage.testResults.filter((t) => t.status === "passed").length}
            </h3>
            <p>Passed Tests</p>
          </div>
          <div className="coverage-metric">
            <h3>{coverage.missingCoverage.length}</h3>
            <p>Coverage Gaps</p>
          </div>
        </div>

        <div className="coverage-bar">
          <div
            className={`coverage-fill ${coverageColor}`}
            style={{ width: `${coverage.overallCoverage}%` }}
          />
          <div
            className="coverage-target-line"
            style={{ left: `${this.state.coverageThreshold}%` }}
          />
        </div>

        <div className="coverage-target-indicator">
          Target: {this.state.coverageThreshold}%
          {coverage.overallCoverage >= this.state.coverageThreshold ? (
            <span className="target-met">🎯 Target met!</span>
          ) : (
            <span className="target-missed">
              {(
                this.state.coverageThreshold - coverage.overallCoverage
              ).toFixed(1)}
              % short
            </span>
          )}
        </div>
      </div>
    );
  }

  private renderFileCoverageTable() {
    const { coverage } = this.state;

    if (!coverage || coverage.filesCoverage.length === 0) {
      return null;
    }

    return (
      <div className="file-coverage-table">
        <h5>File Coverage Details</h5>
        <div className="table-container">
          <table className="coverage-table">
            <thead>
              <tr>
                <th>File</th>
                <th>Coverage</th>
                <th>Lines</th>
                <th>Functions</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {coverage.filesCoverage.map((fileCov, index) => (
                <tr key={index}>
                  <td className="file-name">
                    {fileCov.file.replace(/^.*[\\\/]/, "")}
                  </td>
                  <td>
                    <div className="coverage-cell">
                      <span
                        className={`coverage-percent ${
                          fileCov.percentage >= this.state.coverageThreshold
                            ? "good"
                            : fileCov.percentage >= 50
                              ? "fair"
                              : "poor"
                        }`}
                      >
                        {fileCov.percentage.toFixed(1)}%
                      </span>
                      <div className="coverage-mini-bar">
                        <div
                          className="mini-fill"
                          style={{ width: `${fileCov.percentage}%` }}
                        />
                      </div>
                    </div>
                  </td>
                  <td>
                    {fileCov.coveredLines}/{fileCov.totalLines}
                  </td>
                  <td>
                    {fileCov.functions.filter((f) => f.executed).length}/
                    {fileCov.functions.length}
                  </td>
                  <td>
                    <button
                      className="btn btn-small"
                      onClick={() => this.analyzeCoverageForFile(fileCov.file)}
                      title="Re-analyze this file"
                    >
                      🔄
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  private renderCoverageSuggestions() {
    const suggestions = this.getCoverageDrivenSuggestions();

    if (suggestions.length === 0) return null;

    return (
      <div className="coverage-suggestions">
        <h5>Coverage Improvement Suggestions</h5>
        <div className="suggestions-list">
          {suggestions.map((suggestion, index) => (
            <div
              key={index}
              className={`suggestion-item ${suggestion.priority}`}
            >
              <div className="suggestion-content">
                <div className={`priority-indicator ${suggestion.priority}`}>
                  {suggestion.priority.toUpperCase()}
                </div>
                <p>{suggestion.description}</p>
              </div>
              <button
                className="btn btn-small"
                onClick={suggestion.action}
                title={`Apply ${suggestion.description}`}
              >
                Apply
              </button>
            </div>
          ))}
        </div>
      </div>
    );
  }

  private renderGeneratedTests() {
    const { generatedTests } = this.state;
    const unappliedTests = generatedTests.filter((t) => !t.applied);

    if (unappliedTests.length === 0) return null;

    return (
      <div className="generated-tests">
        <h5>Generated Test Files</h5>
        <div className="test-files-list">
          {unappliedTests.map((test) => (
            <div key={test.id} className="test-file-item">
              <div className="test-file-info">
                <span className="test-file-name">
                  {test.file.replace(/^.*[\\\/]/, "")}
                </span>
                <span
                  className={`test-coverage-impact ${
                    test.coverageImpact > 10
                      ? "high"
                      : test.coverageImpact > 5
                        ? "medium"
                        : "low"
                  }`}
                >
                  +{test.coverageImpact}% coverage
                </span>
              </div>
              <div className="test-file-actions">
                <button
                  className="btn btn-small"
                  onClick={() => this.applyGeneratedTest(test.id)}
                  title="Apply this test file"
                >
                  ✓ Apply
                </button>
                <button
                  className="btn btn-small btn-outline"
                  onClick={() => {
                    // Show test content in modal or preview
                    console.log("Displaying test content:", test.content);
                  }}
                  title="Preview test content"
                >
                  👁️ Preview
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  private renderTestResults() {
    const { coverage } = this.state;

    if (!coverage || coverage.testResults.length === 0) {
      return null;
    }

    const passed = coverage.testResults.filter((t) => t.status === "passed");
    const failed = coverage.testResults.filter((t) => t.status === "failed");
    const ignored = coverage.testResults.filter((t) => t.status === "ignored");

    return (
      <div className="test-results">
        <h5>Test Execution Results</h5>
        <div className="test-summary-cards">
          <div className="test-card passed">
            <h6 className="test-count">{passed.length}</h6>
            <p className="test-label">Passed</p>
          </div>
          <div className="test-card failed">
            <h6 className="test-count">{failed.length}</h6>
            <p className="test-label">Failed</p>
          </div>
          <div className="test-card ignored">
            <h6 className="test-count">{ignored.length}</h6>
            <p className="test-label">Ignored</p>
          </div>
        </div>

        {failed.length > 0 && (
          <div className="failed-tests-section">
            <h6>Failed Tests</h6>
            <div className="failed-tests-list">
              {failed.slice(0, 5).map((test, index) => (
                <div key={index} className="failed-test-item">
                  <div className="failed-test-name">{test.test}</div>
                  <div className="failed-test-details">
                    <span className="test-module">{test.module}</span>
                    <span className="test-duration">
                      {test.duration.toFixed(0)}ms
                    </span>
                  </div>
                  {test.stderr && (
                    <div className="test-error-output">
                      <pre>{test.stderr.slice(0, 200)}...</pre>
                    </div>
                  )}
                </div>
              ))}
              {failed.length > 5 && (
                <div className="more-failed-tests">
                  +{failed.length - 5} more failed tests...
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    );
  }
}

export default CodeCoverageIntegration;
