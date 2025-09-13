import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { WorkspaceAnalysis } from './workspaceAnalyzer';

type UpdateType = 'major' | 'minor' | 'patch';

interface DependencyUpdate {
  name: string;
  currentVersion: string;
  latestVersion: string;
  updateType: UpdateType;
  usedIn: Array<{ member: string; version: string }>;
  changelogUrl?: string;
  isUpdating: boolean;
  updateError?: string;
}

interface DependencyUpdaterProps {
  analysis: WorkspaceAnalysis;
  projectPath: string;
  onUpdateDependency: (updates: Array<{ name: string; version: string }>) => Promise<void>;
}

const DependencyUpdater: React.FC<DependencyUpdaterProps> = ({
  analysis,
  projectPath,
  onUpdateDependency,
}) => {
  const [updates, setUpdates] = useState<DependencyUpdate[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedUpdates, setSelectedUpdates] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState<UpdateType | 'all'>('all');
  const [searchTerm, setSearchTerm] = useState('');

  // Check for updates when the component mounts
  useEffect(() => {
    const checkForUpdates = async () => {
      setIsLoading(true);
      setError(null);

      try {
        // Call the Rust backend to check for dependency updates
        const dependencyUpdates = await invoke<
          {
            name: string;
            current_version: string;
            latest_version: string;
            update_type: string;
            changelog_url?: string;
            is_direct: boolean;
            used_in: string[];
          }[]
        >('check_dependency_updates', {
          project_path: projectPath,
        });

        // Transform the Rust responses to our frontend format
        setUpdates(
          dependencyUpdates.map((u: any) => ({
            name: u.name,
            currentVersion: u.current_version,
            latestVersion: u.latest_version,
            updateType: getUpdateType(u.current_version, u.latest_version),
            usedIn: u.used_in.map((member: string) => ({ member, version: u.current_version })),
            changelogUrl: u.changelog_url,
            isUpdating: false,
          }))
        );
      } catch (err) {
        console.error('Error checking for updates:', err);
        setError('Failed to check for updates. Please try again.');
      } finally {
        setIsLoading(false);
      }
    };

    checkForUpdates();
  }, [analysis, projectPath]);

  // Helper function to determine update type
  const getUpdateType = (current: string, latest: string): UpdateType => {
    if (!current || !latest || current === latest) return 'patch';

    const currentParts = current.split('.').map(Number);
    const latestParts = latest.split('.').map(Number);

    if (currentParts[0] < latestParts[0]) return 'major';
    if (currentParts[1] < latestParts[1]) return 'minor';
    return 'patch';
  };

  // Filter updates based on selected filter and search term
  const filteredUpdates = updates.filter((update) => {
    const matchesFilter = filter === 'all' || update.updateType === filter;
    const matchesSearch =
      update.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      update.currentVersion.includes(searchTerm) ||
      update.latestVersion.includes(searchTerm);
    return matchesFilter && matchesSearch;
  });

  // Toggle selection of an update
  const toggleUpdate = (name: string) => {
    const newSelection = new Set(selectedUpdates);
    if (newSelection.has(name)) {
      newSelection.delete(name);
    } else {
      newSelection.add(name);
    }
    setSelectedUpdates(newSelection);
  };

  // Select all filtered updates
  const selectAll = () => {
    const allFiltered = new Set(filteredUpdates.map((u) => u.name));
    setSelectedUpdates(allFiltered);
  };

  // Clear all selections
  const clearSelection = () => {
    setSelectedUpdates(new Set());
  };

  // Apply selected updates
  const applyUpdates = async () => {
    if (selectedUpdates.size === 0) return;

    const updatesToApply = updates
      .filter((update) => selectedUpdates.has(update.name))
      .map((update) => ({
        name: update.name,
        version: update.latestVersion,
      }));

    // Mark updates as in progress
    setUpdates(
      updates.map((update) =>
        selectedUpdates.has(update.name)
          ? { ...update, isUpdating: true, updateError: undefined }
          : update
      )
    );

    try {
      await onUpdateDependency(updatesToApply);

      // Remove successfully updated dependencies
      setUpdates((prev) => prev.filter((update) => !selectedUpdates.has(update.name)));

      setSelectedUpdates(new Set());
    } catch (error) {
      console.error('Error applying updates:', error);

      // Mark updates with error
      setUpdates(
        updates.map((update) =>
          selectedUpdates.has(update.name)
            ? {
                ...update,
                isUpdating: false,
                updateError: 'Failed to update. Please try again.',
              }
            : update
        )
      );
    }
  };

  // Get badge color based on update type
  const getBadgeColor = (type: UpdateType) => {
    switch (type) {
      case 'major':
        return 'bg-red-100 text-red-800';
      case 'minor':
        return 'bg-yellow-100 text-yellow-800';
      case 'patch':
        return 'bg-green-100 text-green-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  // Get badge label based on update type
  const getBadgeLabel = (type: UpdateType) => {
    switch (type) {
      case 'major':
        return 'Major';
      case 'minor':
        return 'Minor';
      case 'patch':
        return 'Patch';
      default:
        return 'Unknown';
    }
  };

  return (
    <div className="space-y-4">
      {/* Filters and actions */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-2 sm:space-y-0">
        <div className="flex space-x-2">
          <select
            value={filter}
            onChange={(e) => {
              const { value } = e.target as unknown as { value: UpdateType | 'all' };
              setFilter(value);
            }}
            className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
          >
            <option value="all">All Updates</option>
            <option value="major">Major</option>
            <option value="minor">Minor</option>
            <option value="patch">Patch</option>
          </select>

          <input
            type="text"
            placeholder="Search dependencies..."
            value={searchTerm}
            onChange={(e) => {
              const { value } = e.target as unknown as { value: string };
              setSearchTerm(value);
            }}
            className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 text-sm"
          />
        </div>

        <div className="flex space-x-2">
          <button
            onClick={selectAll}
            className="px-3 py-1.5 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            Select All
          </button>
          <button
            onClick={clearSelection}
            className="px-3 py-1.5 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            Clear
          </button>
          <button
            onClick={applyUpdates}
            disabled={selectedUpdates.size === 0}
            className={`px-3 py-1.5 rounded-md text-sm font-medium text-white ${
              selectedUpdates.size > 0
                ? 'bg-blue-600 hover:bg-blue-700'
                : 'bg-blue-300 cursor-not-allowed'
            }`}
          >
            Update Selected ({selectedUpdates.size})
          </button>
        </div>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="flex justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="bg-red-50 border-l-4 border-red-400 p-4">
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
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm text-red-700">{error}</p>
            </div>
          </div>
        </div>
      )}

      {/* No updates available */}
      {!isLoading && !error && filteredUpdates.length === 0 && (
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
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm text-green-700">All dependencies are up to date!</p>
            </div>
          </div>
        </div>
      )}

      {/* Updates list */}
      {!isLoading && filteredUpdates.length > 0 && (
        <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 rounded-lg">
          <table className="min-w-full divide-y divide-gray-300">
            <thead className="bg-gray-50">
              <tr>
                <th scope="col" className="relative w-12 px-6 sm:w-16 sm:px-8">
                  <input
                    type="checkbox"
                    className="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 sm:left-6"
                    checked={
                      selectedUpdates.size === filteredUpdates.length && filteredUpdates.length > 0
                    }
                    onChange={() =>
                      selectedUpdates.size === filteredUpdates.length
                        ? clearSelection()
                        : selectAll()
                    }
                  />
                </th>
                <th
                  scope="col"
                  className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Dependency
                </th>
                <th
                  scope="col"
                  className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Current
                </th>
                <th
                  scope="col"
                  className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Latest
                </th>
                <th
                  scope="col"
                  className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Type
                </th>
                <th
                  scope="col"
                  className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Used In
                </th>
                <th scope="col" className="relative py-3.5 pl-3 pr-4 sm:pr-6">
                  <span className="sr-only">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 bg-white">
              {filteredUpdates.map((update) => (
                <tr
                  key={update.name}
                  className={selectedUpdates.has(update.name) ? 'bg-blue-50' : 'hover:bg-gray-50'}
                >
                  <td className="relative w-12 px-6 sm:w-16 sm:px-8">
                    {selectedUpdates.has(update.name) && (
                      <div className="absolute inset-y-0 left-0 w-0.5 bg-blue-600"></div>
                    )}
                    <input
                      type="checkbox"
                      className="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 sm:left-6"
                      checked={selectedUpdates.has(update.name)}
                      onChange={() => toggleUpdate(update.name)}
                    />
                  </td>
                  <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                    <div className="flex items-center">
                      <span className="font-mono">{update.name}</span>
                      {update.changelogUrl && (
                        <a
                          href={update.changelogUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="ml-2 text-blue-600 hover:text-blue-800"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <span className="sr-only">Changelog</span>
                          <svg
                            className="h-4 w-4"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                          </svg>
                        </a>
                      )}
                    </div>
                  </td>
                  <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                    <span className="font-mono">{update.currentVersion}</span>
                  </td>
                  <td className="whitespace-nowrap px-3 py-4 text-sm font-medium">
                    <span className="font-mono text-green-600">{update.latestVersion}</span>
                  </td>
                  <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getBadgeColor(update.updateType)}`}
                    >
                      {getBadgeLabel(update.updateType)}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                    <div className="flex flex-wrap gap-1 max-w-xs">
                      {update.usedIn.slice(0, 3).map(({ member }) => (
                        <span
                          key={member}
                          className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800"
                        >
                          {member}
                        </span>
                      ))}
                      {update.usedIn.length > 3 && (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-500">
                          +{update.usedIn.length - 3} more
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6">
                    {update.isUpdating ? (
                      <span className="text-gray-500">Updating...</span>
                    ) : update.updateError ? (
                      <span className="text-red-600">{update.updateError}</span>
                    ) : (
                      <button
                        onClick={() =>
                          onUpdateDependency([{ name: update.name, version: update.latestVersion }])
                        }
                        className="text-blue-600 hover:text-blue-900"
                      >
                        Update
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Batch update info */}
      {selectedUpdates.size > 0 && (
        <div className="fixed bottom-4 right-4 bg-white rounded-lg shadow-lg p-4 border border-gray-200 z-10 max-w-md">
          <div className="flex items-start">
            <div className="flex-shrink-0 pt-0.5">
              <svg
                className="h-6 w-6 text-blue-600"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-gray-900">
                Ready to update {selectedUpdates.size} dependencies
              </h3>
              <div className="mt-1 text-sm text-gray-500">
                <p>This will update the selected dependencies to their latest versions.</p>
              </div>
              <div className="mt-4 flex">
                <button
                  type="button"
                  className="inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  onClick={applyUpdates}
                >
                  Apply Updates
                </button>
                <button
                  type="button"
                  className="ml-3 inline-flex items-center px-3 py-1.5 border border-gray-300 shadow-sm text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  onClick={clearSelection}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default DependencyUpdater;
