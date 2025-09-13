<template>
  <div class="plugin-manager">
    <div class="plugin-manager-header">
      <h2>Plugin Manager</h2>
      <button @click="loadInstalledPlugins">Refresh</button>
    </div>

    <div class="search-section">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search marketplace..."
        @keyup.enter="searchMarketplace"
      />
      <button @click="searchMarketplace">Search</button>
    </div>

    <div class="plugin-list">
      <div v-if="loading" class="loading">Loading...</div>

      <div v-else-if="plugins.length === 0" class="empty-state">
        No plugins found. Try searching the marketplace.
      </div>

      <div v-else class="plugin-item" v-for="plugin in plugins" :key="plugin.id">
        <div class="plugin-info">
          <h3>{{ plugin.name }}</h3>
          <p>{{ plugin.description }}</p>
          <small>{{ plugin.version }}</small>
        </div>

        <div class="plugin-actions">
          <button
            v-if="!plugin.installed"
            @click="installPlugin(plugin.id)"
            :disabled="loading"
          >
            Install
          </button>

          <button
            v-else
            @click="plugin.enabled ? deactivatePlugin(plugin.id) : activatePlugin(plugin.id)"
            :disabled="loading"
          >
            {{ plugin.enabled ? 'Deactivate' : 'Activate' }}
          </button>

          <button
            v-if="plugin.installed"
            @click="uninstallPlugin(plugin.id)"
            :disabled="loading"
            class="danger"
          >
            Uninstall
          </button>
        </div>
      </div>
    </div>

    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Reactive data
const plugins = ref([])
const loading = ref(false)
const errorMessage = ref('')
const searchQuery = ref('')

// Load installed plugins
const loadInstalledPlugins = async () => {
  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('list_installed_plugins', {
      includeMetadata: true
    })

    if (response.success) {
      plugins.value = response.data || []
    } else {
      errorMessage.value = response.message || 'Failed to load plugins'
    }
  } catch (error) {
    console.error('Failed to load installed plugins:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Search marketplace
const searchMarketplace = async () => {
  if (!searchQuery.value.trim()) {
    errorMessage.value = 'Please enter a search query'
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('search_marketplace', {
      query: searchQuery.value,
      limit: 20
    })

    if (response.success) {
      plugins.value = response.data || []
      // Mark non-installed plugins
      plugins.value = plugins.value.map(plugin => ({
        ...plugin,
        installed: plugin.installed || false,
        enabled: plugin.enabled || false
      }))
    } else {
      errorMessage.value = response.message || 'Failed to search marketplace'
    }
  } catch (error) {
    console.error('Failed to search marketplace:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Install plugin
const installPlugin = async (pluginId) => {
  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('install_plugin', {
      pluginId,
      version: null,
      sourceUrl: null
    })

    if (response.success) {
      await loadInstalledPlugins() // Refresh list
    } else {
      errorMessage.value = response.message || 'Failed to install plugin'
    }
  } catch (error) {
    console.error('Failed to install plugin:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Activate plugin
const activatePlugin = async (pluginId) => {
  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('activate_plugin', {
      pluginId
    })

    if (response.success) {
      await loadInstalledPlugins() // Refresh list
    } else {
      errorMessage.value = response.message || 'Failed to activate plugin'
    }
  } catch (error) {
    console.error('Failed to activate plugin:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Deactivate plugin
const deactivatePlugin = async (pluginId) => {
  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('deactivate_plugin', {
      pluginId
    })

    if (response.success) {
      await loadInstalledPlugins() // Refresh list
    } else {
      errorMessage.value = response.message || 'Failed to deactivate plugin'
    }
  } catch (error) {
    console.error('Failed to deactivate plugin:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Uninstall plugin
const uninstallPlugin = async (pluginId) => {
  if (!confirm('Are you sure you want to uninstall this plugin?')) {
    return
  }

  try {
    loading.value = true
    errorMessage.value = ''

    const response = await invoke('uninstall_plugin', {
      pluginId
    })

    if (response.success) {
      await loadInstalledPlugins() // Refresh list
    } else {
      errorMessage.value = response.message || 'Failed to uninstall plugin'
    }
  } catch (error) {
    console.error('Failed to uninstall plugin:', error)
    errorMessage.value = 'Failed to communicate with the backend'
  } finally {
    loading.value = false
  }
}

// Initialize component
onMounted(() => {
  loadInstalledPlugins()
})
</script>

<style scoped>
.plugin-manager {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.plugin-manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.plugin-manager-header h2 {
  margin: 0;
}

.search-section {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.search-section input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.plugin-list {
  border: 1px solid #eee;
  border-radius: 8px;
}

.plugin-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #eee;
}

.plugin-item:last-child {
  border-bottom: none;
}

.plugin-info {
  flex: 1;
}

.plugin-info h3 {
  margin: 0 0 4px 0;
  font-size: 18px;
}

.plugin-info p {
  margin: 0 0 8px 0;
  color: #666;
}

.plugin-info small {
  color: #999;
  font-family: monospace;
}

.plugin-actions {
  display: flex;
  gap: 8px;
}

.plugin-actions button {
  padding: 8px 16px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: white;
  cursor: pointer;
}

.plugin-actions button:hover {
  background: #f8f8f8;
}

.plugin-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.plugin-actions button.danger {
  background: #ffdddd;
  border-color: #ffaaaa;
}

.plugin-actions button.danger:hover {
  background: #ffcccc;
}

.loading, .empty-state {
  text-align: center;
  padding: 40px;
  color: #666;
}

.error-message {
  margin-top: 20px;
  padding: 12px;
  background: #ffdddd;
  border: 1px solid #ffaaaa;
  border-radius: 4px;
  color: #c00;
}
</style>