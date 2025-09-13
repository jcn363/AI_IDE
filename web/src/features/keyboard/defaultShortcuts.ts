import type { ShortcutAction, KeyCombination } from './types';

export const DEFAULT_KEY_COMBINATIONS: Record<string, KeyCombination[]> = {
  'editor.save': [{ key: 's', ctrlKey: true }],
  'editor.save-all': [{ key: 's', ctrlKey: true, shiftKey: true }],
  'editor.find': [{ key: 'f', ctrlKey: true }],
  'editor.replace': [{ key: 'h', ctrlKey: true }],
  'editor.find-in-files': [{ key: 'f', ctrlKey: true, shiftKey: true }],
  'editor.replace-in-files': [{ key: 'h', ctrlKey: true, shiftKey: true }],
  'editor.undo': [{ key: 'z', ctrlKey: true }],
  'editor.redo': [
    { key: 'y', ctrlKey: true },
    { key: 'z', ctrlKey: true, shiftKey: true },
  ],
  'editor.copy': [{ key: 'c', ctrlKey: true }],
  'editor.cut': [{ key: 'x', ctrlKey: true }],
  'editor.paste': [{ key: 'v', ctrlKey: true }],
  'editor.select-all': [{ key: 'a', ctrlKey: true }],
  'editor.toggle-comment': [{ key: '/', ctrlKey: true }],
  'editor.format-document': [{ key: 'f', ctrlKey: true, altKey: true }],
  'editor.go-to-line': [{ key: 'g', ctrlKey: true }],
  'editor.toggle-word-wrap': [{ key: 'z', ctrlKey: true, altKey: true }],
  'editor.increase-font-size': [{ key: '=', ctrlKey: true }],
  'editor.decrease-font-size': [{ key: '-', ctrlKey: true }],
  'editor.reset-font-size': [{ key: '0', ctrlKey: true }],
  'editor.multi-cursor': [{ key: 'd', ctrlKey: true }],
  'editor.next-match': [{ key: 'd', ctrlKey: true }],

  'editor.toggle-terminal': [{ key: '`', ctrlKey: true }],
  'editor.split-vertical': [{ key: '\\', ctrlKey: true }],
  'editor.split-horizontal': [{ key: '\\', ctrlKey: true, shiftKey: true }],
  'editor.close-tab': [{ key: 'w', ctrlKey: true }],

  'terminal.new': [{ key: 't', ctrlKey: true, shiftKey: true }],
  'terminal.close': [{ key: 'w', ctrlKey: true, shiftKey: true }],
  'terminal.focus-next': [{ key: ']', ctrlKey: true }],
  'terminal.focus-previous': [{ key: '[', ctrlKey: true }],

  'file-explorer.toggle': [{ key: 'b', ctrlKey: true }],
  'file-explorer.new-file': [{ key: 'n', ctrlKey: true }],
  'file-explorer.new-folder': [{ key: 'f', ctrlKey: true, shiftKey: true }],
  'file-explorer.rename': [{ key: 'r', ctrlKey: true }],
  'file-explorer.delete': [{ key: 'd', ctrlKey: true, shiftKey: true }],

  'command-palette.show': [{ key: 'p', ctrlKey: true, shiftKey: true }],
  'command-palette.quick-open': [{ key: 'p', ctrlKey: true }],
  'command-palette.command-mode': [{ key: '.', ctrlKey: true }],

  'search.focus': [{ key: 'k', ctrlKey: true }],
  'search.global-replace': [{ key: 'k', ctrlKey: true, shiftKey: true }],
  'search.clear': [{ key: 'c', ctrlKey: true, shiftKey: true, altKey: true }],

  'git.status': [{ key: 'g', ctrlKey: true, shiftKey: true }],
  'git.commit': [{ key: 'c', ctrlKey: true, shiftKey: true }],
  'git.push': [{ key: 'p', ctrlKey: true, shiftKey: true }],
  'git.pull': [{ key: 'l', ctrlKey: true, shiftKey: true }],

  'debugger.toggle': [{ key: 'f5' }],
  'debugger.step-over': [{ key: 'f10' }],
  'debugger.step-into': [{ key: 'f11' }],
  'debugger.step-out': [{ key: 'f11', shiftKey: true }],
  'debugger.continue': [{ key: 'f5' }],
  'debugger.stop': [{ key: 'f5', shiftKey: true }],

  'cargo.build': [{ key: 'b', ctrlKey: true, altKey: true }],
  'cargo.run': [{ key: 'r', ctrlKey: true, altKey: true }],
  'cargo.test': [{ key: 't', ctrlKey: true, altKey: true }],
  'cargo.clean': [{ key: 'c', ctrlKey: true, altKey: true }],
  'cargo.check': [{ key: 'k', ctrlKey: true, altKey: true }],
  'cargo.doc': [{ key: 'd', ctrlKey: true, altKey: true }],

  'ai-assistant.toggle': [{ key: 'a', ctrlKey: true, altKey: true }],
  'ai-assistant.quick-suggestion': [{ key: 'i', ctrlKey: true, shiftKey: true }],
};
