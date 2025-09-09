import React from 'react';
import { useAIAssistant } from '../hooks/useAIAssistant';
import { CommandPaletteItem } from '../../command-palette/types';

export const useAICommands = () => {
  const {
    analyzeCurrentFile,
    generateTests,
    generateDocumentation,
    explainCode,
    refactorCode,
    hasSuggestions,
    showPanel,
    hidePanel,
    isPanelVisible,
  } = useAIAssistant();

  const commands: CommandPaletteItem[] = [
    {
      id: 'ai.analyze',
      name: 'AI: Analyze Code',
      description: 'Analyze the current file for issues and improvements',
      icon: 'search',
      handler: async (editor) => {
        const code = editor.getValue();
        const path = editor.getModel()?.uri?.path || 'current_file.rs';
        await analyzeCurrentFile(code, path);
      },
    },
    {
      id: 'ai.generate-tests',
      name: 'AI: Generate Tests',
      description: 'Generate test cases for the current file or selection',
      icon: 'test',
      handler: async (editor) => {
        const selection = editor.getSelection();
        const code = selection
          ? editor.getModel()?.getValueInRange(selection) || ''
          : editor.getValue();
        const path = editor.getModel()?.uri?.path || 'current_file.rs';
        await generateTests(code, path);
      },
    },
    {
      id: 'ai.generate-docs',
      name: 'AI: Generate Documentation',
      description: 'Generate documentation for the current file or selection',
      icon: 'book',
      handler: async (editor) => {
        const selection = editor.getSelection();
        const code = selection
          ? editor.getModel()?.getValueInRange(selection) || ''
          : editor.getValue();
        const path = editor.getModel()?.uri?.path || 'current_file.rs';
        await generateDocumentation(code, path);
      },
    },
    {
      id: 'ai.explain',
      name: 'AI: Explain Code',
      description: 'Get an explanation for the selected code',
      icon: 'comment-discussion',
      handler: async (editor) => {
        const selection = editor.getSelection();
        if (!selection) {
          console.warn('No code selected');
          return;
        }
        const code = editor.getModel()?.getValueInRange(selection) || '';
        await explainCode(code);
      },
    },
    {
      id: 'ai.refactor',
      name: 'AI: Refactor Code',
      description: 'Refactor the selected code',
      icon: 'git-compare',
      handler: async (editor) => {
        const selection = editor.getSelection();
        if (!selection) {
          console.warn('No code selected');
          return;
        }
        const code = editor.getModel()?.getValueInRange(selection) || '';
        const path = editor.getModel()?.uri?.path || 'current_file.rs';
        await refactorCode(code, path);
      },
    },
    {
      id: 'ai.toggle-suggestions',
      name: isPanelVisible ? 'AI: Hide Suggestions' : 'AI: Show Suggestions',
      description: isPanelVisible 
        ? 'Hide the AI suggestions panel' 
        : 'Show the AI suggestions panel',
      icon: 'lightbulb',
      handler: async () => {
        if (isPanelVisible) {
          await hidePanel();
        } else {
          await showPanel();
        }
        return Promise.resolve();
      },
    },
  ];

  // Only include the toggle command if there are suggestions
  return hasSuggestions 
    ? commands 
    : commands.filter(cmd => cmd.id !== 'ai.toggle-suggestions');
};

export default useAICommands;
