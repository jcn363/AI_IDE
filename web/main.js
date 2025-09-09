// Monaco is loaded dynamically to enable code-splitting and reduce initial bundle size
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile, exists, mkdir, remove } from '@tauri-apps/plugin-fs';
import { listen } from '@tauri-apps/api/event';

class RustAIIDE {
    constructor() {
        this.editor = null;
        this.monaco = null;
        this.currentFile = null;
        this.openFiles = new Map();
        this.activeTab = null;
        this.aiPanelOpen = false;
        this.currentWorkspace = null;
        this.fileTree = null;
        this.searchResults = [];
        this.projectConfig = null;
        
        this.init();
    }
    
    async init() {
        // Initialize Monaco Editor
        await this.setupMonacoEditor();
        
        // Initialize settings system
        this.initializeSettings();
        
        // Initialize command palette
        this.initializeCommandPalette();
        
        // Setup event listeners
        this.setupEventListeners();
        
        // Setup keyboard shortcuts
        this.setupKeyboardShortcuts();
        
        // Setup LSP communication
        this.setupLSP();
        
        console.log('Rust AI IDE initialized');
    }
    
    async setupMonacoEditor() {
        // Dynamically import Monaco editor on demand
        const monaco = this.monaco ?? (this.monaco = await import('monaco-editor'));

        // Configure Monaco for Rust
        monaco.languages.register({ id: 'rust' });
        
        // Set up Rust language configuration
        monaco.languages.setLanguageConfiguration('rust', {
            comments: {
                lineComment: '//',
                blockComment: ['/*', '*/']
            },
            brackets: [
                ['{', '}'],
                ['[', ']'],
                ['(', ')']
            ],
            autoClosingPairs: [
                { open: '{', close: '}' },
                { open: '[', close: ']' },
                { open: '(', close: ')' },
                { open: '"', close: '"', notIn: ['string'] },
                { open: "'", close: "'", notIn: ['string', 'comment'] }
            ],
            surroundingPairs: [
                { open: '{', close: '}' },
                { open: '[', close: ']' },
                { open: '(', close: ')' },
                { open: '"', close: '"' },
                { open: "'", close: "'" }
            ]
        });
        
        // Create the editor
        const editorContainer = document.getElementById('monaco-editor');
        this.editor = monaco.editor.create(editorContainer, {
            value: '// Welcome to Rust AI IDE\n// Start coding in Rust with AI assistance!\n\nfn main() {\n    println!("Hello, Rust AI IDE!");\n}',
            language: 'rust',
            theme: 'vs-dark',
            automaticLayout: true,
            fontSize: 14,
            lineNumbers: 'on',
            minimap: { enabled: true },
            scrollBeyondLastLine: false,
            folding: true,
            lineDecorationsWidth: 10,
            renderWhitespace: 'selection',
            wordWrap: 'on'
        });
        
        // Listen for cursor position changes
        this.editor.onDidChangeCursorPosition((e) => {
            this.updateCursorPosition(e.position);
        });
        
        // Listen for content changes
        this.editor.onDidChangeModelContent(() => {
            if (this.currentFile) {
                this.markFileAsModified(this.currentFile);
            }
        });
    }
    
    setupEventListeners() {
        // File operations
        document.getElementById('new-file-btn').addEventListener('click', () => this.newFile());
        document.getElementById('open-folder-btn').addEventListener('click', () => this.openFolder());
        
        // AI Assistant
        document.getElementById('ai-assistant-btn').addEventListener('click', () => this.toggleAIPanel());
        document.getElementById('close-ai-panel').addEventListener('click', () => this.toggleAIPanel());
        document.getElementById('send-ai-message').addEventListener('click', () => this.sendAIMessage());
        
        // AI input keyboard shortcuts
        document.getElementById('ai-input').addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && e.ctrlKey) {
                e.preventDefault();
                this.sendAIMessage();
            }
        });
        
        // Settings modal
        const settingsBtn = document.getElementById('settings-btn');
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => this.openSettings());
        }
        
        const saveSettingsBtn = document.getElementById('save-settings');
        if (saveSettingsBtn) {
            saveSettingsBtn.addEventListener('click', () => this.applySettingsChanges());
        }

        const closeSettingsBtn = document.getElementById('close-settings');
        if (closeSettingsBtn) {
            closeSettingsBtn.addEventListener('click', () => this.closeSettings());
        }
        
        // Font size slider
        const fontSizeSlider = document.getElementById('font-size');
        if (fontSizeSlider) {
            fontSizeSlider.addEventListener('input', (e) => {
                document.getElementById('font-size-value').textContent = e.target.value + 'px';
            });
        }
        
        // Command palette input
        const commandInput = document.getElementById('command-input');
        if (commandInput) {
            commandInput.addEventListener('input', (e) => {
                this.updateCommandList(e.target.value);
            });
        }
        
        // Modal backdrop clicks to close
        const settingsModal = document.getElementById('settings-modal');
        if (settingsModal) {
            settingsModal.addEventListener('click', (e) => {
                if (e.target === settingsModal || e.target.classList.contains('modal-backdrop')) {
                    this.closeSettings();
                }
            });
        }
        
        const commandPalette = document.getElementById('command-palette');
        if (commandPalette) {
            commandPalette.addEventListener('click', (e) => {
                if (e.target === commandPalette || e.target.classList.contains('command-palette-backdrop')) {
                    this.closeCommandPalette();
                }
            });
        }
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey) {
                switch (e.key) {
                    case 'n':
                        e.preventDefault();
                        this.newFile();
                        break;
                    case 'o':
                        e.preventDefault();
                        this.openFolder();
                        break;
                    case 's':
                        e.preventDefault();
                        this.saveCurrentFile();
                        break;
                    case '`':
                        e.preventDefault();
                        this.toggleAIPanel();
                        break;
                }
            }
        });
    }
    
    async setupLSP() {
        try {
            // Initialize LSP connection through Tauri backend
            await invoke('init_lsp');
            console.log('LSP initialized');
            
            // Listen for LSP diagnostics
            await listen('lsp-diagnostics', (event) => {
                this.handleLSPDiagnostics(event.payload);
            });
            
            // Listen for LSP hover information
            await listen('lsp-hover', (event) => {
                this.handleLSPHover(event.payload);
            });
            
        } catch (error) {
            console.error('Failed to initialize LSP:', error);
        }
    }
    
    async newFile() {
        const fileName = `untitled-${Date.now()}.rs`;
        const content = '// New Rust file\n\nfn main() {\n    println!("Hello, world!");\n}\n';
        
        this.openFiles.set(fileName, {
            path: null,
            content: content,
            modified: false
        });
        
        this.createTab(fileName);
        this.switchToFile(fileName);
    }
    
    async openFolder() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Open Folder'
            });
            
            if (selected) {
                await this.loadFolder(selected);
            }
        } catch (error) {
            console.error('Failed to open folder:', error);
        }
    }
    
    async loadFolder(folderPath) {
        try {
            this.currentWorkspace = folderPath;
            const files = await invoke('list_files', { path: folderPath });
            
            // Check if this is a Rust project
            await this.detectProjectType(folderPath, files);
            
            this.populateFileExplorer(files, folderPath);
            
            // Update window title with workspace name
            const workspaceName = folderPath.split('/').pop() || folderPath.split('\\').pop();
            document.title = `${workspaceName} - Rust AI IDE`;
            
        } catch (error) {
            console.error('Failed to load folder:', error);
        }
    }
    
    async detectProjectType(folderPath, files) {
        const cargoToml = files.find(f => f.name === 'Cargo.toml' && !f.is_directory);
        if (cargoToml) {
            this.projectConfig = {
                type: 'rust',
                root: folderPath,
                manifestPath: cargoToml.path
            };
            
            // Add project commands to the interface
            this.addProjectCommands();
            
            try {
                // Load Cargo.toml to get project info
                const cargoContent = await readTextFile(cargoToml.path);
                console.log('Detected Rust project:', cargoContent.substring(0, 200));
            } catch (error) {
                console.warn('Could not read Cargo.toml:', error);
            }
        }
    }
    
    addProjectCommands() {
        const header = document.querySelector('.header-right');
        
        // Remove existing project buttons
        const existingBtns = header.querySelectorAll('.project-btn');
        existingBtns.forEach(btn => btn.remove());
        
        if (this.projectConfig?.type === 'rust') {
            const buildBtn = document.createElement('button');
            buildBtn.className = 'btn project-btn';
            buildBtn.textContent = 'Build';
            buildBtn.addEventListener('click', () => this.buildProject());
            
            const runBtn = document.createElement('button');
            runBtn.className = 'btn project-btn';
            runBtn.textContent = 'Run';
            runBtn.addEventListener('click', () => this.runProject());
            
            const testBtn = document.createElement('button');
            testBtn.className = 'btn project-btn';
            testBtn.textContent = 'Test';
            testBtn.addEventListener('click', () => this.testProject());
            
            // Insert before AI Assistant button
            const aiBtn = document.getElementById('ai-assistant-btn');
            header.insertBefore(buildBtn, aiBtn);
            header.insertBefore(runBtn, aiBtn);
            header.insertBefore(testBtn, aiBtn);
        }
    }
    
    async buildProject() {
        if (!this.projectConfig) return;
        
        try {
            this.showStatusMessage('Building project...', 'info');
            const result = await invoke('build_project', { project_path: this.projectConfig.root });
            this.showStatusMessage('Build completed successfully', 'success');
            console.log('Build result:', result);
        } catch (error) {
            this.showStatusMessage('Build failed', 'error');
            console.error('Build error:', error);
        }
    }
    
    async runProject() {
        if (!this.projectConfig) return;
        
        try {
            this.showStatusMessage('Running project...', 'info');
            const result = await invoke('run_project', { project_path: this.projectConfig.root });
            this.showStatusMessage('Project started', 'success');
            console.log('Run result:', result);
        } catch (error) {
            this.showStatusMessage('Failed to run project', 'error');
            console.error('Run error:', error);
        }
    }
    
    async testProject() {
        if (!this.projectConfig) return;
        
        try {
            this.showStatusMessage('Running tests...', 'info');
            const result = await invoke('test_project', { project_path: this.projectConfig.root });
            this.showStatusMessage('Tests completed', 'success');
            console.log('Test result:', result);
        } catch (error) {
            this.showStatusMessage('Tests failed', 'error');
            console.error('Test error:', error);
        }
    }
    
    showStatusMessage(message, type = 'info') {
        const statusBar = document.querySelector('.status-bar');
        let messageElement = statusBar.querySelector('.status-message');
        
        if (!messageElement) {
            messageElement = document.createElement('span');
            messageElement.className = 'status-message';
            statusBar.querySelector('.status-left').appendChild(messageElement);
        }
        
        messageElement.textContent = message;
        messageElement.className = `status-message ${type}`;
        
        // Clear message after 3 seconds
        setTimeout(() => {
            messageElement.textContent = '';
            messageElement.className = 'status-message';
        }, 3000);
    }
    
    populateFileExplorer(files, rootPath) {
        const explorer = document.getElementById('file-explorer');
        explorer.innerHTML = '';
        
        // Add workspace header
        const workspaceHeader = document.createElement('div');
        workspaceHeader.className = 'workspace-header';
        workspaceHeader.innerHTML = `
            <strong>${rootPath.split('/').pop() || rootPath.split('\\').pop()}</strong>
            <div class="workspace-actions">
                <button class="btn-icon" onclick="ide.refreshWorkspace()" title="Refresh">🔄</button>
                <button class="btn-icon" onclick="ide.showNewFileDialog()" title="New File">📄</button>
                <button class="btn-icon" onclick="ide.showNewFolderDialog()" title="New Folder">📁</button>
            </div>
        `;
        explorer.appendChild(workspaceHeader);
        
        // Create file tree
        const fileTree = document.createElement('div');
        fileTree.className = 'file-tree';
        
        this.renderFileTree(files, fileTree, 0);
        explorer.appendChild(fileTree);
    }
    
    renderFileTree(files, container, depth) {
        files.forEach(file => {
            const item = document.createElement('div');
            item.className = 'file-tree-item';
            item.style.paddingLeft = `${depth * 16 + 8}px`;
            
            const icon = file.is_directory ? '📁' : this.getFileIcon(file.name);
            item.innerHTML = `
                <span class="file-icon">${icon}</span>
                <span class="file-name">${file.name}</span>
            `;
            
            if (file.is_directory) {
                item.classList.add('directory');
                item.addEventListener('click', async () => {
                    if (item.classList.contains('expanded')) {
                        // Collapse directory
                        item.classList.remove('expanded');
                        const children = container.querySelectorAll(`[data-parent="${file.path}"]`);
                        children.forEach(child => child.remove());
                    } else {
                        // Expand directory
                        item.classList.add('expanded');
                        try {
                            const subFiles = await invoke('list_files', { path: file.path });
                            const subContainer = document.createElement('div');
                            subContainer.setAttribute('data-parent', file.path);
                            this.renderFileTree(subFiles, subContainer, depth + 1);
                            
                            // Insert after current item
                            item.insertAdjacentElement('afterend', subContainer);
                        } catch (error) {
                            console.error('Failed to load directory:', error);
                        }
                    }
                });
            } else {
                item.addEventListener('click', () => this.openFile(file.path));
                item.addEventListener('contextmenu', (e) => {
                    e.preventDefault();
                    this.showFileContextMenu(e, file);
                });
            }
            
            container.appendChild(item);
        });
    }
    
    getFileIcon(fileName) {
        const extension = fileName.split('.').pop()?.toLowerCase();
        switch (extension) {
            case 'rs': return '🦀';
            case 'toml': return '⚙️';
            case 'md': return '📝';
            case 'json': return '📋';
            case 'yaml': case 'yml': return '📄';
            case 'lock': return '🔒';
            case 'txt': return '📄';
            default: return '📄';
        }
    }
    
    showFileContextMenu(event, file) {
        const menu = document.createElement('div');
        menu.className = 'context-menu';
        menu.style.left = `${event.clientX}px`;
        menu.style.top = `${event.clientY}px`;
        
        menu.innerHTML = `
            <div class="context-menu-item" onclick="ide.deleteFile('${file.path}')">Delete</div>
            <div class="context-menu-item" onclick="ide.renameFile('${file.path}')">Rename</div>
            <div class="context-menu-item" onclick="ide.copyPath('${file.path}')">Copy Path</div>
        `;
        
        document.body.appendChild(menu);
        
        // Remove menu on click outside
        setTimeout(() => {
            document.addEventListener('click', () => {
                if (menu.parentNode) {
                    menu.parentNode.removeChild(menu);
                }
            }, { once: true });
        }, 100);
    }
    
    async refreshWorkspace() {
        if (this.currentWorkspace) {
            await this.loadFolder(this.currentWorkspace);
        }
    }
    
    async showNewFileDialog() {
        const fileName = prompt('Enter file name:');
        if (fileName && this.currentWorkspace) {
            try {
                const filePath = `${this.currentWorkspace}/${fileName}`;
                await writeTextFile(filePath, '');
                await this.refreshWorkspace();
            } catch (error) {
                this.showStatusMessage('Failed to create file', 'error');
                console.error('Create file error:', error);
            }
        }
    }
    
    async showNewFolderDialog() {
        const folderName = prompt('Enter folder name:');
        if (folderName && this.currentWorkspace) {
            try {
                const folderPath = `${this.currentWorkspace}/${folderName}`;
                await mkdir(folderPath);
                await this.refreshWorkspace();
            } catch (error) {
                this.showStatusMessage('Failed to create folder', 'error');
                console.error('Create folder error:', error);
            }
        }
    }
    
    async deleteFile(filePath) {
        if (confirm('Are you sure you want to delete this file?')) {
            try {
                await remove(filePath);
                await this.refreshWorkspace();
                this.showStatusMessage('File deleted', 'success');
            } catch (error) {
                this.showStatusMessage('Failed to delete file', 'error');
                console.error('Delete error:', error);
            }
        }
    }
    
    copyPath(filePath) {
        navigator.clipboard.writeText(filePath).then(() => {
            this.showStatusMessage('Path copied to clipboard', 'success');
        }).catch(error => {
            console.error('Copy error:', error);
        });
    }
    
    async openFile(filePath) {
        try {
            const content = await readTextFile(filePath);
            const fileName = filePath.split('/').pop() || filePath.split('\\\\').pop();
            
            this.openFiles.set(fileName, {
                path: filePath,
                content: content,
                modified: false
            });
            
            this.createTab(fileName);
            this.switchToFile(fileName);
            
        } catch (error) {
            console.error('Failed to open file:', error);
        }
    }
    
    createTab(fileName) {
        const tabBar = document.getElementById('tab-bar');
        
        // Remove existing tab if it exists
        const existingTab = document.querySelector(`[data-file="${fileName}"]`);
        if (existingTab) {
            existingTab.remove();
        }
        
        const tab = document.createElement('div');
        tab.className = 'tab';
        tab.setAttribute('data-file', fileName);
        tab.innerHTML = `
            <span class="tab-title">${fileName}</span>
            <button class="tab-close" onclick="event.stopPropagation()">×</button>
        `;
        
        tab.addEventListener('click', () => this.switchToFile(fileName));
        tab.querySelector('.tab-close').addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTab(fileName);
        });
        
        tabBar.appendChild(tab);
    }
    
    switchToFile(fileName) {
        const fileData = this.openFiles.get(fileName);
        if (!fileData) return;
        
        // Update active tab
        document.querySelectorAll('.tab').forEach(tab => tab.classList.remove('active'));
        document.querySelector(`[data-file="${fileName}"]`).classList.add('active');
        
        // Update editor content
        this.editor.setValue(fileData.content);
        this.currentFile = fileName;
        
        // Update status bar
        document.getElementById('file-path').textContent = fileData.path || fileName;
        
        // Update language mode based on file extension
        const extension = fileName.split('.').pop();
        let language = 'plaintext';
        if (extension === 'rs') language = 'rust';
        else if (extension === 'toml') language = 'toml';
        else if (extension === 'md') language = 'markdown';
        
        if (this.monaco) {
            this.monaco.editor.setModelLanguage(this.editor.getModel(), language);
        }
        document.getElementById('language-mode').textContent = language.charAt(0).toUpperCase() + language.slice(1);
    }
    
    async closeTab(fileName) {
        const fileData = this.openFiles.get(fileName);
        if (fileData && fileData.modified) {
            const shouldSave = confirm(`Save changes to ${fileName}?`);
            if (shouldSave) {
                try {
                    await this.saveSpecificFile(fileName);
                } catch (e) {
                    console.error('Failed to save before closing tab:', e);
                    return; // abort close on save failure
                }
            }
        }
        
        this.openFiles.delete(fileName);
        const tabEl = document.querySelector(`[data-file="${fileName}"]`);
        if (tabEl) tabEl.remove();
        
        // Switch to another tab if this was the active one
        if (this.currentFile === fileName) {
            const remainingTabs = document.querySelectorAll('.tab');
            if (remainingTabs.length > 0) {
                const nextFileName = remainingTabs[0].getAttribute('data-file');
                this.switchToFile(nextFileName);
            } else {
                this.currentFile = null;
                this.editor.setValue('');
                document.getElementById('file-path').textContent = 'No file selected';
            }
        }
    }
    
    async saveCurrentFile() {
        if (!this.currentFile) return;
        await this.saveSpecificFile(this.currentFile);
    }
    
    async saveSpecificFile(fileName, forceSaveAs = false) {
        const fileData = this.openFiles.get(fileName);
        if (!fileData) return;
        try {
            const content = (this.currentFile === fileName && this.editor) ? this.editor.getValue() : (fileData.content ?? '');
            let targetPath = fileData.path;
            if (!targetPath || forceSaveAs) {
                const defaultExt = (fileName.split('.').pop() || 'rs');
                const dlgPath = await save({
                    defaultPath: targetPath || this.currentWorkspace || undefined,
                    filters: [{ name: 'Files', extensions: [defaultExt] }],
                });
                if (!dlgPath) return; // user cancelled
                targetPath = dlgPath;
            }
            await writeTextFile(targetPath, content);
            fileData.path = targetPath;
            fileData.content = content;
            fileData.modified = false;
            if (fileName === this.currentFile) {
                this.updateTabTitle(fileName);
                document.getElementById('file-path').textContent = targetPath;
            } else {
                this.updateTabTitle(fileName);
            }
            this.showNotification('File saved', 'success');
        } catch (error) {
            console.error('Failed to save file:', error);
            this.showNotification('Failed to save file', 'error');
            throw error;
        }
    }
    
    markFileAsModified(fileName) {
        const fileData = this.openFiles.get(fileName);
        if (fileData) {
            fileData.modified = true;
            this.updateTabTitle(fileName);
        }
    }
    
    updateTabTitle(fileName) {
        const tab = document.querySelector(`[data-file="${fileName}"]`);
        const fileData = this.openFiles.get(fileName);
        if (tab && fileData) {
            const title = tab.querySelector('.tab-title');
            title.textContent = fileName + (fileData.modified ? ' •' : '');
        }
    }
    
    updateCursorPosition(position) {
        document.getElementById('cursor-position').textContent = 
            `Ln ${position.lineNumber}, Col ${position.column}`;
    }
    
    toggleAIPanel() {
        const aiPanel = document.getElementById('ai-panel');
        this.aiPanelOpen = !this.aiPanelOpen;
        
        if (this.aiPanelOpen) {
            aiPanel.style.display = 'flex';
            document.getElementById('ai-input').focus();
        } else {
            aiPanel.style.display = 'none';
        }
    }
    
    async sendAIMessage() {
        const input = document.getElementById('ai-input');
        const message = input.value.trim();
        
        if (!message) return;
        
        // Add user message to chat
        this.addAIMessage(message, 'user');
        input.value = '';
        
        try {
            // Get current code context
            const currentCode = this.editor.getValue();
            const cursorPosition = this.editor.getPosition();
            
            // Send message to AI through Tauri backend
            const response = await invoke('send_ai_message', {
                message: message,
                context: {
                    code: currentCode,
                    fileName: this.currentFile,
                    cursorLine: cursorPosition.lineNumber,
                    cursorColumn: cursorPosition.column
                }
            });
            
            // Add AI response to chat
            this.addAIMessage(response, 'assistant');
            
        } catch (error) {
            console.error('Failed to send AI message:', error);
            this.addAIMessage('Sorry, I encountered an error processing your request.', 'assistant');
        }
    }
    
    addAIMessage(content, role) {
        const messagesContainer = document.getElementById('ai-messages');
        const messageDiv = document.createElement('div');
        messageDiv.className = `ai-message ${role}`;
        messageDiv.textContent = content;
        
        messagesContainer.appendChild(messageDiv);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
    
    handleLSPDiagnostics(diagnostics) {
        // Convert LSP diagnostics to Monaco markers
        if (!this.monaco || !this.editor) return;

        const markers = diagnostics.map(diagnostic => ({
            severity: diagnostic.severity === 1 ? this.monaco.MarkerSeverity.Error : 
                     diagnostic.severity === 2 ? this.monaco.MarkerSeverity.Warning :
                     this.monaco.MarkerSeverity.Info,
            startLineNumber: diagnostic.range.start.line + 1,
            startColumn: diagnostic.range.start.character + 1,
            endLineNumber: diagnostic.range.end.line + 1,
            endColumn: diagnostic.range.end.character + 1,
            message: diagnostic.message
        }));

        this.monaco.editor.setModelMarkers(this.editor.getModel(), 'rust-analyzer', markers);
    }
    
    handleLSPHover(hoverInfo) {
        // Handle hover information from LSP
        console.log('LSP Hover:', hoverInfo);
    }

    // Settings management
    initializeSettings() {
        this.settings = {
            theme: localStorage.getItem('ide-theme') || 'dark',
            fontSize: parseInt(localStorage.getItem('ide-font-size')) || 14,
            fontFamily: localStorage.getItem('ide-font-family') || 'Monaco, Consolas, monospace',
            minimap: localStorage.getItem('ide-minimap') !== 'false',
            wordWrap: localStorage.getItem('ide-word-wrap') === 'true',
            lineNumbers: localStorage.getItem('ide-line-numbers') !== 'false',
            tabSize: parseInt(localStorage.getItem('ide-tab-size')) || 4,
            aiProvider: localStorage.getItem('ide-ai-provider') || 'openai',
            aiModel: localStorage.getItem('ide-ai-model') || 'gpt-3.5-turbo',
            autoSave: localStorage.getItem('ide-auto-save') === 'true',
            autoSaveDelay: parseInt(localStorage.getItem('ide-auto-save-delay')) || 1000,
            formatOnSave: localStorage.getItem('ide-format-on-save') === 'true'
        };
        this.applySettings();
    }

    applySettings() {
        // Apply theme
        document.documentElement.setAttribute('data-theme', this.settings.theme);
        
        // Apply editor settings
        if (this.editor) {
            this.editor.updateOptions({
                fontSize: this.settings.fontSize,
                fontFamily: this.settings.fontFamily,
                minimap: { enabled: this.settings.minimap },
                wordWrap: this.settings.wordWrap ? 'on' : 'off',
                lineNumbers: this.settings.lineNumbers ? 'on' : 'off',
                tabSize: this.settings.tabSize,
                insertSpaces: true
            });
        }
        
        // Apply auto-save
        if (this.settings.autoSave) {
            this.setupAutoSave();
        }
    }

    saveSettings() {
        for (const [key, value] of Object.entries(this.settings)) {
            localStorage.setItem(`ide-${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`, value.toString());
        }
        this.applySettings();
    }

    openSettings() {
        const modal = document.getElementById('settings-modal');
        if (modal) {
            // Populate settings form
            document.getElementById('theme-select').value = this.settings.theme;
            document.getElementById('font-size').value = this.settings.fontSize;
            document.getElementById('font-size-value').textContent = this.settings.fontSize + 'px';
            document.getElementById('font-family').value = this.settings.fontFamily;
            const minimapEl = document.getElementById('minimap-enabled');
            if (minimapEl) minimapEl.checked = this.settings.minimap;
            document.getElementById('word-wrap').checked = this.settings.wordWrap;
            document.getElementById('line-numbers').checked = this.settings.lineNumbers;
            document.getElementById('tab-size').value = this.settings.tabSize;
            document.getElementById('ai-provider').value = this.settings.aiProvider;
            document.getElementById('ai-model').value = this.settings.aiModel;
            document.getElementById('auto-save').checked = this.settings.autoSave;
            document.getElementById('auto-save-delay').value = this.settings.autoSaveDelay;
            document.getElementById('format-on-save').checked = this.settings.formatOnSave;

            // Show modal
            modal.style.display = 'block';
        }
    }

    closeSettings() {
        const modal = document.getElementById('settings-modal');
        if (modal) {
            modal.style.display = 'none';
        }
    }

    applySettingsChanges() {
        // Gather settings from form
        this.settings.theme = document.getElementById('theme-select').value;
        this.settings.fontSize = parseInt(document.getElementById('font-size').value);
        this.settings.fontFamily = document.getElementById('font-family').value;
        const minimapEl = document.getElementById('minimap-enabled');
        this.settings.minimap = minimapEl ? minimapEl.checked : this.settings.minimap;
        this.settings.wordWrap = document.getElementById('word-wrap').checked;
        this.settings.lineNumbers = document.getElementById('line-numbers').checked;
        this.settings.tabSize = parseInt(document.getElementById('tab-size').value);
        this.settings.aiProvider = document.getElementById('ai-provider').value;
        this.settings.aiModel = document.getElementById('ai-model').value;
        this.settings.autoSave = document.getElementById('auto-save').checked;
        this.settings.autoSaveDelay = parseInt(document.getElementById('auto-save-delay').value);
        this.settings.formatOnSave = document.getElementById('format-on-save').checked;
        
        this.saveSettings();
        this.closeSettings();
        this.showNotification('Settings saved successfully', 'success');
    }

    // Command palette
    initializeCommandPalette() {
        this.commands = [
            { id: 'new-file', name: 'New File', description: 'Create a new file', shortcut: 'Ctrl+N' },
            { id: 'open-file', name: 'Open File', description: 'Open an existing file', shortcut: 'Ctrl+O' },
            { id: 'save-file', name: 'Save File', description: 'Save current file', shortcut: 'Ctrl+S' },
            { id: 'save-as', name: 'Save As', description: 'Save file with new name', shortcut: 'Ctrl+Shift+S' },
            { id: 'close-file', name: 'Close File', description: 'Close current file', shortcut: 'Ctrl+W' },
            { id: 'refresh', name: 'Refresh Workspace', description: 'Refresh file explorer', shortcut: 'F5' },
            { id: 'toggle-ai', name: 'Toggle AI Panel', description: 'Show/hide AI assistant', shortcut: 'Ctrl+`' },
            { id: 'settings', name: 'Open Settings', description: 'Configure IDE preferences', shortcut: 'Ctrl+,' },
            { id: 'build', name: 'Build Project', description: 'Build the current Rust project', shortcut: 'Ctrl+B' },
            { id: 'run', name: 'Run Project', description: 'Run the current Rust project', shortcut: 'Ctrl+R' },
            { id: 'test', name: 'Test Project', description: 'Run tests for current project', shortcut: 'Ctrl+T' },
        ];
        this.selectedCommandIndex = 0;
    }

    openCommandPalette() {
        const palette = document.getElementById('command-palette');
        const input = document.getElementById('command-input');
        const list = document.querySelector('.command-list');
        
        if (palette && input && list) {
            palette.classList.remove('hidden');
            input.value = '';
            input.focus();
            this.updateCommandList('');
            this.selectedCommandIndex = 0;
        }
    }

    closeCommandPalette() {
        const palette = document.getElementById('command-palette');
        if (palette) {
            palette.classList.add('hidden');
        }
    }

    updateCommandList(filter) {
        const list = document.querySelector('.command-list');
        if (!list) return;
        
        const filteredCommands = this.commands.filter(cmd => 
            cmd.name.toLowerCase().includes(filter.toLowerCase()) ||
            cmd.description.toLowerCase().includes(filter.toLowerCase())
        );
        
        list.innerHTML = filteredCommands.map((cmd, index) => `
            <div class="command-item ${index === this.selectedCommandIndex ? 'selected' : ''}" data-command="${cmd.id}">
                <div class="command-icon">⚡</div>
                <div class="command-details">
                    <div class="command-name">${cmd.name}</div>
                    <div class="command-description">${cmd.description}</div>
                </div>
                <div class="command-shortcut">${cmd.shortcut || ''}</div>
            </div>
        `).join('');
        
        // Add click handlers
        list.querySelectorAll('.command-item').forEach((item, index) => {
            item.addEventListener('click', () => {
                this.executeCommand(filteredCommands[index].id);
                this.closeCommandPalette();
            });
        });
    }

    executeCommand(commandId) {
        switch (commandId) {
            case 'new-file':
                this.newFile();
                break;
            case 'open-file':
                this.openFile();
                break;
            case 'save-file':
                this.saveCurrentFile();
                break;
            case 'save-as':
                this.saveFileAs();
                break;
            case 'close-file':
                this.closeCurrentFile();
                break;
            case 'refresh':
                this.refreshWorkspace();
                break;
            case 'toggle-ai':
                this.toggleAIPanel();
                break;
            case 'settings':
                this.openSettings();
                break;
            case 'build':
                this.buildProject();
                break;
            case 'run':
                this.runProject();
                break;
            case 'test':
                this.testProject();
                break;
        }
    }

    // Auto-save functionality
    setupAutoSave() {
        if (this.autoSaveTimeout) {
            clearTimeout(this.autoSaveTimeout);
        }
        
        if (this.settings.autoSave && this.editor) {
            this.editor.onDidChangeModelContent(() => {
                if (this.autoSaveTimeout) {
                    clearTimeout(this.autoSaveTimeout);
                }
                this.autoSaveTimeout = setTimeout(() => {
                    if (this.currentFile && this.openFiles.get(this.currentFile)?.modified) {
                        this.saveCurrentFile(); // Silent save
                    }
                }, this.settings.autoSaveDelay);
            });
        }
    }

    // Enhanced keyboard shortcuts
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (event) => {
            // Check if we're in an input field
            if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') {
                // Handle command palette navigation
                if (event.target.id === 'command-input') {
                    if (event.key === 'ArrowDown') {
                        event.preventDefault();
                        this.selectedCommandIndex = Math.min(this.selectedCommandIndex + 1, this.commands.length - 1);
                        this.updateCommandList(event.target.value);
                    } else if (event.key === 'ArrowUp') {
                        event.preventDefault();
                        this.selectedCommandIndex = Math.max(this.selectedCommandIndex - 1, 0);
                        this.updateCommandList(event.target.value);
                    } else if (event.key === 'Enter') {
                        event.preventDefault();
                        const filteredCommands = this.commands.filter(cmd => 
                            cmd.name.toLowerCase().includes(event.target.value.toLowerCase()) ||
                            cmd.description.toLowerCase().includes(event.target.value.toLowerCase())
                        );
                        if (filteredCommands[this.selectedCommandIndex]) {
                            this.executeCommand(filteredCommands[this.selectedCommandIndex].id);
                            this.closeCommandPalette();
                        }
                    } else if (event.key === 'Escape') {
                        this.closeCommandPalette();
                    }
                }
                return;
            }
            
            if (event.ctrlKey || event.metaKey) {
                switch (event.key) {
                    case 'n':
                        event.preventDefault();
                        this.newFile();
                        break;
                    case 'o':
                        event.preventDefault();
                        this.openFile();
                        break;
                    case 's':
                        event.preventDefault();
                        if (event.shiftKey) {
                            this.saveFileAs();
                        } else {
                            this.saveCurrentFile();
                        }
                        break;
                    case 'w':
                        event.preventDefault();
                        this.closeCurrentFile();
                        break;
                    case '`':
                        event.preventDefault();
                        this.toggleAIPanel();
                        break;
                    case ',':
                        event.preventDefault();
                        this.openSettings();
                        break;
                    case 'p':
                        if (event.shiftKey) {
                            event.preventDefault();
                            this.openCommandPalette();
                        }
                        break;
                    case 'b':
                        event.preventDefault();
                        this.buildProject();
                        break;
                    case 'r':
                        event.preventDefault();
                        this.runProject();
                        break;
                    case 't':
                        event.preventDefault();
                        this.testProject();
                        break;
                }
            } else {
                switch (event.key) {
                    case 'F5':
                        event.preventDefault();
                        this.refreshWorkspace();
                        break;
                    case 'Escape':
                        // Close modals on escape
                        this.closeSettings();
                        this.closeCommandPalette();
                        break;
                }
            }
        });
    }

    // Enhanced notification system
    showNotification(message, type = 'info', duration = 4000) {
        const container = document.getElementById('notifications');
        if (!container) {
            // Fallback to simple notification
            console.log(`${type.toUpperCase()}: ${message}`);
            return;
        }
        
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        
        const iconMap = {
            success: '✓',
            error: '✗',
            warning: '⚠',
            info: 'ℹ'
        };
        
        notification.innerHTML = `
            <div class="notification-icon">${iconMap[type] || iconMap.info}</div>
            <div class="notification-content">${message}</div>
            <button class="notification-close">×</button>
        `;
        
        // Add close handler
        notification.querySelector('.notification-close').addEventListener('click', () => {
            notification.remove();
        });
        
        container.appendChild(notification);
        
        // Auto remove
        if (duration > 0) {
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.remove();
                }
            }, duration);
        }
    }

    // Helper methods for missing functionality
    newFile() {
        const fileName = 'untitled.rs';
        let counter = 1;
        let finalName = fileName;
        
        while (this.openFiles.has(finalName)) {
            finalName = `untitled${counter}.rs`;
            counter++;
        }
        
        this.openFiles.set(finalName, {
            path: null,
            content: '',
            modified: false
        });
        
        this.createTab(finalName);
        this.switchToFile(finalName);
        this.showNotification('New file created', 'success');
    }

    closeCurrentFile() {
        if (this.currentFile) {
            this.closeTab(this.currentFile);
        }
    }

    async saveFileAs() {
        if (!this.currentFile) return;
        await this.saveSpecificFile(this.currentFile, true);
    }
}

// Global IDE instance
let ide;

// Initialize the IDE when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    ide = new RustAIIDE();
    
    // Make it available globally for button onclick handlers
    window.ide = ide;
});
