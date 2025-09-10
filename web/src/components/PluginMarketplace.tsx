import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  category: string;
  tags: string[];
  downloadCount: number;
  rating?: number;
  isInstalled: boolean;
  isInstalling: boolean;
  license: string;
}

interface PluginMarketplaceProps {
  className?: string;
}

interface SearchPayload {
  query: string;
  category?: string;
  limit?: number;
}

interface InstallPayload {
  pluginId: string;
  version?: string;
}

interface RatePayload {
  pluginId: string;
  rating: number;
  review: string;
}

const PluginMarketplace: React.FC<PluginMarketplaceProps> = ({ className = '' }) => {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentTab, setCurrentTab] = useState<'discover' | 'installed'>('discover');
  const [installingPlugins, setInstallingPlugins] = useState<Set<string>>(new Set());

  const categories = ['all', 'editor', 'language', 'theme', 'tooling', 'integration'];

  useEffect(() => {
    loadPlugins();
  }, [searchQuery, selectedCategory, currentTab]);

  const loadPlugins = async () => {
    setIsLoading(true);
    setError(null);

    try {
      let result: any;

      if (currentTab === 'discover') {
        const payload: SearchPayload = {
          query: searchQuery,
          category: selectedCategory === 'all' ? undefined : selectedCategory,
          limit: 50,
        };
        result = await invoke('search_marketplace', { payload });
      } else {
        result = await invoke('list_installed_plugins', {
          payload: { includeMetadata: true }
        });
      }

      if (result.success && result.data) {
        setPlugins(result.data);
      } else {
        setError(result.message || 'Failed to load plugins');
      }
    } catch (err) {
      console.error('Plugin loading error:', err);
      setError('Failed to connect to plugin marketplace');
    } finally {
      setIsLoading(false);
    }
  };

  const installPlugin = async (pluginId: string, version?: string) => {
    if (installingPlugins.has(pluginId)) return;

    setInstallingPlugins(prev => new Set(prev).add(pluginId));
    setError(null);

    try {
      const payload: InstallPayload = { pluginId, version };
      const result: any = await invoke('install_plugin', { payload });

      if (result.success) {
        // Update plugin list to show installed status
        await loadPlugins();
      } else {
        setError(result.message || 'Failed to install plugin');
      }
    } catch (err) {
      console.error('Plugin installation error:', err);
      setError('Failed to install plugin');
    } finally {
      setInstallingPlugins(prev => {
        const newSet = new Set(prev);
        newSet.delete(pluginId);
        return newSet;
      });
    }
  };

  const uninstallPlugin = async (pluginId: string) => {
    try {
      const result: any = await invoke('uninstall_plugin', {
        payload: { pluginId }
      });

      if (result.success) {
        await loadPlugins();
      } else {
        setError(result.message || 'Failed to uninstall plugin');
      }
    } catch (err) {
      console.error('Plugin uninstall error:', err);
      setError('Failed to uninstall plugin');
    }
  };

  const executePluginCommand = async (pluginId: string, command: string, args: any) => {
    try {
      const result: any = await invoke('execute_plugin_command', {
        payload: { pluginId, command, args }
      });

      if (!result.success) {
        setError(result.message || 'Failed to execute plugin command');
      }
    } catch (err) {
      console.error('Plugin command execution error:', err);
      setError('Failed to execute plugin command');
    }
  };

  const ratePlugin = async (pluginId: string, rating: number, review: string) => {
    try {
      const payload: RatePayload = { pluginId, rating, review };
      const result: any = await invoke('rate_plugin', { payload });

      if (!result.success) {
        setError(result.message || 'Failed to submit rating');
      }
    } catch (err) {
      console.error('Plugin rating error:', err);
      setError('Failed to submit rating');
    }
  };

  const scanPluginSecurity = async (pluginId: string) => {
    try {
      const result: any = await invoke('scan_plugin_security', {
        payload: { pluginId }
      });

      if (result.success && result.data) {
        return result.data;
      } else {
        setError(result.message || 'Security scan failed');
      }
    } catch (err) {
      console.error('Plugin security scan error:', err);
      setError('Failed to perform security scan');
    }
    return null;
  };

  const renderPluginCard = (plugin: PluginInfo) => (
    <div key={plugin.id} className="plugin-card bg-gray-800 rounded-lg p-4 hover:bg-gray-700 transition-colors">
      <div className="flex justify-between items-start mb-3">
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-white">{plugin.name}</h3>
          <p className="text-sm text-gray-400">by {plugin.author} • v{plugin.version}</p>
        </div>
        {plugin.rating && (
          <div className="flex items-center">
            <span className="text-yellow-400 mr-1">★</span>
            <span className="text-sm text-gray-300">{plugin.rating.toFixed(1)}</span>
          </div>
        )}
      </div>

      <p className="text-gray-300 text-sm mb-3 line-clamp-2">{plugin.description}</p>

      <div className="flex flex-wrap gap-1 mb-3">
        {plugin.tags.slice(0, 3).map(tag => (
          <span key={tag} className="px-2 py-1 bg-blue-600 text-xs text-white rounded">
            {tag}
          </span>
        ))}
      </div>

      <div className="flex justify-between items-center">
        <div className="flex items-center gap-4 text-sm text-gray-400">
          <span>📥 {plugin.downloadCount.toLocaleString()} downloads</span>
          <span className={`px-2 py-1 rounded text-xs ${
            plugin.license.includes('MIT') ? 'bg-green-600' : 'bg-orange-600'
          } text-white`}>
            {plugin.license}
          </span>
        </div>

        <div className="flex gap-2">
          {plugin.isInstalled ? (
            <>
              <button
                onClick={() => executePluginCommand(plugin.id, 'show_config', {})}
                className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700"
              >
                Configure
              </button>
              <button
                onClick={() => uninstallPlugin(plugin.id)}
                className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700"
              >
                Uninstall
              </button>
            </>
          ) : installingPlugins.has(plugin.id) ? (
            <div className="px-3 py-1 bg-gray-600 text-white text-sm rounded flex items-center">
              <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-white mr-2"></div>
              Installing...
            </div>
          ) : (
            <>
              <button
                onClick={() => scanPluginSecurity(plugin.id)}
                className="px-3 py-1 bg-gray-600 text-white text-sm rounded hover:bg-gray-700"
              >
                🔒 Scan
              </button>
              <button
                onClick={() => installPlugin(plugin.id)}
                className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700"
              >
                Install
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );

  return (
    <div className={`plugin-marketplace ${className}`}>
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white mb-4">Plugin Marketplace</h1>

        {error && (
          <div className="bg-red-900 border border-red-600 text-red-200 px-4 py-3 rounded mb-4">
            {error}
          </div>
        )}

        <div className="flex gap-1 mb-4">
          <button
            onClick={() => setCurrentTab('discover')}
            className={`px-4 py-2 rounded ${
              currentTab === 'discover'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            Discover Plugins
          </button>
          <button
            onClick={() => setCurrentTab('installed')}
            className={`px-4 py-2 rounded ${
              currentTab === 'installed'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            Installed Plugins
          </button>
        </div>

        {currentTab === 'discover' && (
          <div className="flex gap-4 mb-4">
            <div className="flex-1">
              <input
                type="search"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search plugins..."
                className="w-full px-4 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              />
            </div>
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="px-4 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
            >
              {categories.map(category => (
                <option key={category} value={category}>
                  {category.charAt(0).toUpperCase() + category.slice(1)}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      {isLoading ? (
        <div className="flex justify-center items-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
          <span className="ml-3 text-gray-400">Loading plugins...</span>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {plugins.map(renderPluginCard)}
          {!plugins.length && !isLoading && (
            <div className="col-span-full text-center py-12 text-gray-400">
              {currentTab === 'discover'
                ? 'No plugins found matching your search.'
                : 'No plugins installed yet.'
              }
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default PluginMarketplace;