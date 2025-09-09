import { useCallback, useEffect, useState } from "react";
import { useCargoTomlEditor } from "./useCargoTomlEditor";
import { DependencyGraphRenderer } from "./dependencyGraph";
import EnhancedDependencyGraph from "./EnhancedDependencyGraph";
import WorkspaceInheritanceGraph from "./WorkspaceInheritanceGraph";
import DependencyUpdater from "./DependencyUpdater";
import { CargoLockVisualization } from "./lockfile/CargoLockVisualization";
import {
  WorkspaceAnalysis,
  analyzeWorkspaceInheritance,
} from "./workspaceAnalyzer";

interface CargoTomlEditorProps {
  initialToml: string;
  onSave?: (toml: string) => void;
  className?: string;
}

const CargoTomlEditor: React.FC<CargoTomlEditorProps> = ({
  initialToml,
  onSave,
  className = "",
}) => {
  const [activeTab, setActiveTab] = useState<
    | "editor"
    | "dependencies"
    | "features"
    | "security"
    | "licenses"
    | "graph"
    | "enhanced-graph"
    | "workspace"
    | "updates"
    | "lockfile"
  >("editor");
  const [isSaving, setIsSaving] = useState(false);

  const {
    manifest,
    isDirty,
    isLoading,
    error,
    featureFlags,
    featureFlagSuggestions,
    projectPath,
    vulnerabilities,
    licenseInfo,
    licenseSummary,
    licenseCompatibility,
    updateToml,
    optimizeFeatures,
    updateDependency,
    addDependency,
    removeDependency,
    reload,
  } = useCargoTomlEditor(initialToml);

  // Workspace analysis state
  const [workspaceAnalysis, setWorkspaceAnalysis] =
    useState<WorkspaceAnalysis | null>(null);
  const [isAnalyzingWorkspace, setIsAnalyzingWorkspace] = useState(false);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);

  // Analyze workspace when the workspace tab is selected
  useEffect(() => {
    if (
      activeTab === "workspace" &&
      !workspaceAnalysis &&
      !isAnalyzingWorkspace
    ) {
      const analyzeWorkspace = async () => {
        if (!manifest) return;

        setIsAnalyzingWorkspace(true);
        setWorkspaceError(null);

        try {
          // In a real implementation, this would call the Rust backend
          // For now, we'll simulate the response
          const analysis = await analyzeWorkspaceInheritance(
            ".", // Current directory - would be provided by the backend
            manifest,
          );

          setWorkspaceAnalysis(analysis);
        } catch (err) {
          console.error("Error analyzing workspace:", err);
          setWorkspaceError("Failed to analyze workspace. Please try again.");
        } finally {
          setIsAnalyzingWorkspace(false);
        }
      };

      analyzeWorkspace();
    }
  }, [activeTab, manifest, workspaceAnalysis, isAnalyzingWorkspace]);

  const handleDependencyUpdate = useCallback(
    (name: string, version: string) => {
      return updateDependency([{ name, version }]);
    },
    [updateDependency],
  );

  const handleSave = useCallback(async () => {
    if (!isDirty || !onSave) return;

    try {
      setIsSaving(true);
      await onSave(initialToml);
      // Force a reload to ensure everything is in sync
      reload();
    } catch (err) {
      console.error("Failed to save:", err);
    } finally {
      setIsSaving(false);
    }
  }, [isDirty, onSave, initialToml, reload]);

  const renderEditorTab = () => {
    // Custom TOML syntax highlighting with CSS
    const [highlightedContent, setHighlightedContent] = useState("");

    useEffect(() => {
      // Basic TOML syntax highlighting using regex
      const highlightTOML = (text: string) => {
        // Escape HTML
        let html = text.replace(/&/g, '&').replace(/</g, '<').replace(/>/g, '>');

        // Highlight comments
        html = html.replace(/(#.*$)/gm, '<span class="text-green-400">$1</span>');

        // Highlight strings in quotes
        html = html.replace(/"([^"]*)"/g, '<span class="text-amber-400">"$1"</span>');
        html = html.replace(/'([^']*)'/g, '<span class="text-amber-400">\'$1\'</span>');

        // Highlight section headers
        html = html.replace(/\[([^\]]+)\]/g, '<span class="text-purple-400">[$1]</span>');
        html = html.replace(/\[\[([^\]]+)\]\]/g, '<span class="text-purple-400">[[$1]]</span>');

        // Highlight array items
        html = html.replace(/\b\d+(\.\d+)?|[tf]rue|[mf]alse\b/g, '<span class="text-cyan-400">$&</span>');

        // Preserve line breaks
        html = html.replace(/\n/g, '<br>');

        return html;
      };

      setHighlightedContent(highlightTOML(initialToml));
    }, [initialToml]);

    return (
      <div className="h-full flex flex-col">
        <div className="flex-1">
          <div className="h-full border border-gray-600 rounded relative bg-gray-900">
            {/* Line numbers */}
            <div className="absolute left-0 top-0 w-10 bg-gray-800 border-r border-gray-600 py-2 px-2 font-mono text-sm text-gray-400 select-none">
              {initialToml.split('\n').map((_, i) => (
                <div key={i} className="leading-5">{i + 1}</div>
              ))}
            </div>

            {/* Editor content */}
            <textarea
              className="w-full h-full pl-12 pr-4 py-2 font-mono text-sm bg-transparent text-gray-100 focus:outline-none resize-none"
              value={initialToml}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => updateToml(e.target.value)}
              disabled={isLoading}
              spellCheck={false}
              style={{
                tabSize: 2,
                fontFamily: 'Monaco, Consolas, "Liberation Mono", "Courier New", monospace'
              }}
            />

            {/* Status indicator */}
            <div className="absolute bottom-0 left-12 right-0 px-4 py-1 bg-gray-800 border-t border-gray-600">
              <div className="flex items-center space-x-4 text-xs text-gray-400">
                <span>TOML</span>
                <span>{initialToml.split('\n').length} lines</span>
                {isDirty && <span className="text-yellow-400">Modified</span>}
                {isLoading && <span className="text-blue-400">Loading...</span>}
              </div>
            </div>
          </div>
        </div>
        <div className="mt-4 flex justify-end space-x-2">
          <button
            onClick={reload}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            disabled={!isDirty || isLoading}
          >
            Reset
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            disabled={!isDirty || isLoading || isSaving}
          >
            {isSaving ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </div>
    );
  };

  const renderDependenciesTab = () => (
    <div className="space-y-4">
      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <ul className="divide-y divide-gray-200">
          {manifest?.dependencies &&
            Object.entries(manifest.dependencies).map(([name, dep]) => (
              <DependencyItem
                key={`dep-${name}`}
                name={name}
                dep={dep}
                onUpdate={handleDependencyUpdate}
                onRemove={removeDependency}
              />
            ))}
        </ul>
      </div>
    </div>
  );

  const renderFeaturesTab = () => (
    <div className="space-y-6">
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6 flex justify-between items-center">
          <h3 className="text-lg leading-6 font-medium text-gray-900">
            Feature Flags
          </h3>
          <button
            onClick={optimizeFeatures}
            className="inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Optimize Features
          </button>
        </div>

        {featureFlagSuggestions.length > 0 && (
          <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 mb-4">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg
                  className="h-5 w-5 text-yellow-400"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-yellow-800">
                  Feature Flag Suggestions
                </h3>
                <div className="mt-2 text-sm text-yellow-700">
                  <ul className="list-disc pl-5 space-y-1">
                    {featureFlagSuggestions.map((suggestion, i) => (
                      <li key={`suggestion-${i}`}>{suggestion}</li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          </div>
        )}

        <div className="border-t border-gray-200 px-4 py-5 sm:p-0">
          <dl className="sm:divide-y sm:divide-gray-200">
            {featureFlags.map((feature) => (
              <div
                key={feature.name}
                className="py-4 sm:py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6"
              >
                <dt className="text-sm font-medium text-gray-500 flex items-center">
                  {feature.name}
                  {feature.enabledByDefault && (
                    <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      Default
                    </span>
                  )}
                </dt>
                <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                  {feature.usedBy.length > 0 ? (
                    <div>
                      <p className="text-sm text-gray-500">Used by:</p>
                      <ul className="list-disc pl-5 mt-1">
                        {feature.usedBy.map((dep: string) => (
                          <li
                            key={`${feature.name}-${dep}`}
                            className="text-sm"
                          >
                            {dep}
                          </li>
                        ))}
                      </ul>
                    </div>
                  ) : (
                    <span className="text-gray-400">
                      Not used by any dependencies
                    </span>
                  )}
                </dd>
              </div>
            ))}
          </dl>
        </div>
      </div>
    </div>
  );

  const renderSecurityTab = () => (
    <div className="space-y-6">
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">
            Security Vulnerabilities
          </h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500">
            Known security issues in your dependencies
          </p>
        </div>

        {vulnerabilities.length === 0 ? (
          <div className="bg-green-50 border-l-4 border-green-400 p-4">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg
                  className="h-5 w-5 text-green-400"
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414-1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <div className="ml-3">
                <p className="text-sm text-green-700">
                  No known security vulnerabilities found in your dependencies.
                </p>
              </div>
            </div>
          </div>
        ) : (
          <div className="border-t border-gray-200">
            <ul className="divide-y divide-gray-200">
              {vulnerabilities.map((vuln) => (
                <li key={vuln.id} className="px-4 py-4 sm:px-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center">
                      <span
                        className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                          vuln.severity === "critical"
                            ? "bg-red-100 text-red-800"
                            : vuln.severity === "high"
                              ? "bg-orange-100 text-orange-800"
                              : vuln.severity === "medium"
                                ? "bg-yellow-100 text-yellow-800"
                                : "bg-blue-100 text-blue-800"
                        }`}
                      >
                        {vuln.severity.charAt(0).toUpperCase() +
                          vuln.severity.slice(1)}
                      </span>
                      <p className="ml-2 text-sm font-medium text-gray-900">
                        {vuln.package}@{vuln.version}
                      </p>
                    </div>
                    <a
                      href={vuln.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm font-medium text-blue-600 hover:text-blue-500"
                    >
                      Details
                    </a>
                  </div>
                  <div className="mt-2">
                    <p className="text-sm text-gray-900 font-medium">
                      {vuln.title}
                    </p>
                    <p className="mt-1 text-sm text-gray-500">
                      {vuln.description}
                    </p>
                    {vuln.patched_versions && (
                      <div className="mt-2">
                        <span className="text-sm font-medium text-gray-500">
                          Fixed in:{" "}
                        </span>
                        <span className="text-sm text-green-600">
                          {vuln.patched_versions}
                        </span>
                      </div>
                    )}
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );

  const renderLicensesTab = () => (
    <div className="space-y-6">
      {licenseSummary && (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900">
              License Compliance
            </h3>
            <div className="mt-2 grid grid-cols-1 gap-5 sm:grid-cols-4">
              <div className="bg-white overflow-hidden shadow rounded-lg">
                <div className="px-4 py-5 sm:p-6">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      Total Dependencies
                    </dt>
                    <dd className="mt-1 text-3xl font-semibold text-gray-900">
                      {licenseSummary.total}
                    </dd>
                  </dl>
                </div>
              </div>

              <div className="bg-white overflow-hidden shadow rounded-lg">
                <div className="px-4 py-5 sm:p-6">
                  <dl>
                    <dt className="text-sm font-medium text-green-600 truncate">
                      Approved Licenses
                    </dt>
                    <dd className="mt-1 text-3xl font-semibold text-green-600">
                      {licenseSummary.approved}
                    </dd>
                  </dl>
                </div>
              </div>

              <div className="bg-white overflow-hidden shadow rounded-lg">
                <div className="px-4 py-5 sm:p-6">
                  <dl>
                    <dt className="text-sm font-medium text-yellow-600 truncate">
                      Copyleft Licenses
                    </dt>
                    <dd className="mt-1 text-3xl font-semibold text-yellow-600">
                      {licenseSummary.copyleft}
                    </dd>
                  </dl>
                </div>
              </div>

              <div className="bg-white overflow-hidden shadow rounded-lg">
                <div className="px-4 py-5 sm:p-6">
                  <dl>
                    <dt className="text-sm font-medium text-red-600 truncate">
                      Banned Licenses
                    </dt>
                    <dd className="mt-1 text-3xl font-semibold text-red-600">
                      {licenseSummary.banned}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>

            {licenseCompatibility && !licenseCompatibility.compatible && (
              <div className="mt-4 bg-red-50 border-l-4 border-red-400 p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg
                      className="h-5 w-5 text-red-400"
                      xmlns="http://www.w3.org/2000/svg"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fillRule="evenodd"
                        d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414-1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">
                      License Incompatibility Detected
                    </h3>
                    <div className="mt-2 text-sm text-red-700">
                      <p>
                        The following dependencies have incompatible licenses
                        with your project:
                      </p>
                      <ul className="list-disc pl-5 mt-1">
                        {licenseCompatibility.conflicts.map((conflict: { package: string; license: string }, i: number) => (
                          <li key={`conflict-${i}`}>
                            {conflict.package} ({conflict.license})
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="border-t border-gray-200">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Package
                  </th>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Version
                  </th>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    License
                  </th>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {licenseInfo.map((license) => (
                  <tr key={`${license.package}-${license.version}`}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {license.package}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {license.version}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {license.license || "Unknown"}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {license.isBanned ? (
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">
                          Banned
                        </span>
                      ) : license.copyleft ? (
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800">
                          Copyleft
                        </span>
                      ) : license.isApproved ? (
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                          Approved
                        </span>
                      ) : (
                        <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800">
                          Unknown
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );

  const renderDependencyGraphTab = () => {
    if (!manifest) return null;
    return (
      <div className="h-[600px] bg-white shadow overflow-hidden sm:rounded-lg">
        <DependencyGraphRenderer manifest={manifest} />
      </div>
    );
  };

  const renderEnhancedDependencyGraphTab = () => {
    if (!projectPath) return null;
    return (
      <div className="h-[600px] bg-white shadow overflow-hidden sm:rounded-lg">
        <EnhancedDependencyGraph 
          projectPath={projectPath}
          width="100%"
          height="100%"
          showControls={true}
        />
      </div>
    );
  };

  const renderWorkspaceTab = () => {
    if (!projectPath) return null;
    
    // Create mock manifest data
    const createMockManifest = (name: string) => ({
      package: {
        name,
        version: '0.1.0',
        edition: '2021',
      },
      dependencies: {},
      'dev-dependencies': {},
      'build-dependencies': {}
    });
    
    // Create mock workspace members
    const createMockMember = (name: string) => ({
      name,
      path: `${projectPath}/${name}`,
      manifest: createMockManifest(name),
      dependencies: [],
      inheritedDependencies: {},
      directDependencies: {} as Record<string, string>
    });
    
    // Create a mock analysis for demonstration
    const mockAnalysis: WorkspaceAnalysis = {
      root: createMockManifest('root-crate'),
      members: [
        createMockMember('crate1'),
        createMockMember('crate2')
      ],
      workspaceDependencies: {},
      inheritanceGraph: {}
    };

    return (
      <div className="h-[600px] bg-white shadow overflow-hidden sm:rounded-lg p-4">
        <WorkspaceInheritanceGraph 
          analysis={mockAnalysis} 
          width={800}
          height={550}
        />
      </div>
    );
  };

  const handleUpdateDependency = async (updates: Array<{ name: string; version: string }>) => {
    // Implement dependency update logic here
    console.log('Updating dependencies:', updates);
    // You would typically call an API or update the TOML file here
  };

  const renderUpdatesTab = () => {
    if (!projectPath) return null;
    
    // Create mock manifest data
    const createMockManifest = (name: string) => ({
      package: {
        name,
        version: '0.1.0',
        edition: '2021',
      },
      dependencies: {},
      'dev-dependencies': {},
      'build-dependencies': {}
    });
    
    // Create mock workspace members
    const createMockMember = (name: string) => ({
      name,
      path: `${projectPath}/${name}`,
      manifest: createMockManifest(name),
      dependencies: [],
      inheritedDependencies: {},
      directDependencies: {} as Record<string, string>
    });
    
    // Create a mock analysis for demonstration
    const mockAnalysis: WorkspaceAnalysis = {
      root: createMockManifest('root-crate'),
      members: [
        createMockMember('crate1'),
        createMockMember('crate2')
      ],
      workspaceDependencies: {},
      inheritanceGraph: {}
    };

    return (
      <div className="h-[600px] bg-white shadow overflow-hidden sm:rounded-lg p-4">
        <DependencyUpdater
          analysis={mockAnalysis}
          projectPath={projectPath || "."}
          onUpdateDependency={handleUpdateDependency}
        />
      </div>
    );
  };

  const renderLockfileTab = () => {
    if (!projectPath) return null;
    return (
      <div className="h-[600px] bg-white shadow overflow-hidden sm:rounded-lg p-4">
        <CargoLockVisualization projectPath={projectPath} />
      </div>
    );
  };

  const renderTabContent = () => {
    switch (activeTab) {
      case 'editor':
        return renderEditorTab();
      case 'dependencies':
        return renderDependenciesTab();
      case 'features':
        return renderFeaturesTab();
      case 'security':
        return renderSecurityTab();
      case 'licenses':
        return renderLicensesTab();
      case 'graph':
        return renderDependencyGraphTab();
      case 'enhanced-graph':
        return renderEnhancedDependencyGraphTab();
      case 'workspace':
        return renderWorkspaceTab();
      case 'updates':
        return renderUpdatesTab();
      case 'lockfile':
        return renderLockfileTab();
      default:
        return null;
    }
  };

interface DependencyItemProps {
  name: string;
  dep: any;
  onUpdate: (name: string, version: string) => void;
  onRemove: (name: string) => void;
}

function DependencyItem({
  name,
  dep,
  onUpdate,
  onRemove,
}: DependencyItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [version, setVersion] = useState(
    typeof dep === "string" ? dep : dep.version || "",
  );

  const handleSave = () => {
    onUpdate(name, version);
    setIsEditing(false);
  };

  return (
    <li className="px-4 py-4 sm:px-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <p className="text-sm font-medium text-blue-600 truncate">{name}</p>
          {isEditing ? (
            <div className="ml-4 flex items-center">
              <input
                type="text"
                className="shadow-sm focus:ring-blue-500 focus:border-blue-500 block w-32 sm:text-sm border-gray-300 rounded-md"
                value={version}
                onChange={(e) => setVersion((e.target as any).value)}
              />
              <button
                onClick={handleSave}
                className="ml-2 inline-flex items-center px-2.5 py-1.5 border border-transparent text-xs font-medium rounded text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                Save
              </button>
              <button
                onClick={() => setIsEditing(false)}
                className="ml-1 inline-flex items-center px-2.5 py-1.5 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                Cancel
              </button>
            </div>
          ) : (
            <span className="ml-2 text-sm text-gray-500">{version}</span>
          )}
        </div>
        <div className="ml-2 flex-shrink-0 flex">
          <button
            onClick={() => setIsEditing(true)}
            className="mr-2 inline-flex items-center px-2.5 py-1.5 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Edit
          </button>
          <button
            onClick={() => onRemove(name)}
            className="inline-flex items-center px-2.5 py-1.5 border border-transparent text-xs font-medium rounded text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
          >
            Remove
          </button>
        </div>
      </div>

      {typeof dep === "object" && dep.features && (
        <div className="mt-2">
          <span className="text-xs text-gray-500">Features: </span>
          <div className="mt-1 flex flex-wrap gap-1">
            {dep.features.map((feature: string) => (
              <span
                key={`${name}-${feature}`}
                className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800"
              >
                {feature}
              </span>
            ))}
          </div>
        </div>
      )}
    </li>
  );
}

  return (
    <div className="flex flex-col h-full">
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8 overflow-x-auto">
          <button
            onClick={() => setActiveTab('editor')}
            className={`${activeTab === 'editor' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Editor
          </button>
          <button
            onClick={() => setActiveTab('dependencies')}
            className={`${activeTab === 'dependencies' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Dependencies
          </button>
          <button
            onClick={() => setActiveTab('features')}
            className={`${activeTab === 'features' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Features
          </button>
          <button
            onClick={() => setActiveTab('security')}
            className={`${activeTab === 'security' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Security
          </button>
          <button
            onClick={() => setActiveTab('licenses')}
            className={`${activeTab === 'licenses' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Licenses
          </button>
          <button
            onClick={() => setActiveTab('graph')}
            className={`${activeTab === 'graph' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Dependency Graph
          </button>
          <button
            onClick={() => setActiveTab('enhanced-graph')}
            className={`${activeTab === 'enhanced-graph' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm flex items-center`}
          >
            Enhanced Graph
            <span className="ml-1.5 px-1.5 py-0.5 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
              New
            </span>
          </button>
          <button
            onClick={() => setActiveTab('workspace')}
            className={`${activeTab === 'workspace' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Workspace
          </button>
          <button
            onClick={() => setActiveTab('updates')}
            className={`${activeTab === 'updates' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Updates
          </button>
          <button
            onClick={() => setActiveTab('lockfile')}
            className={`${activeTab === 'lockfile' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
          >
            Lockfile
          </button>
        </nav>
      </div>
      
      <div className="flex-1 overflow-auto p-4">
        {renderTabContent()}
      </div>
    </div>
  );
}
