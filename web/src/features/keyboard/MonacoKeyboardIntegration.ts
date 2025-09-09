import type * as monaco from 'monaco-editor';
import type { ShortcutAction, KeyCombination, MonacoKeyboardIntegration } from './types';

export class MonacoKeyboardIntegrationImpl implements MonacoKeyboardIntegration {
  private editorKeybindings: Map<string, string[]> = new Map();

  registerMonacoAction(
    editor: monaco.editor.IStandaloneCodeEditor,
    action: ShortcutAction,
    keys?: KeyCombination[]
  ): void {
    // Unregister existing action if it exists
    this.unregisterMonacoAction(editor, action.id);

    // Convert our key combinations to Monaco's format
    const monacoKeybindings = keys || this.getKeysFromShortcut(action);

    if (monacoKeybindings.length > 0) {
      const monacoBinding = monacoKeybindings[0]; // Take the first binding for Monaco
      const monacoKey = this.keyCombinationToMonacoKey(monacoBinding);

      if (monacoKey !== null) {
        // Create Monaco action
        const id = `custom.${action.id}`;
        const disposable = editor.addAction({
          id,
          label: action.name,
          keybindings: [monacoKey],
          contextMenuGroupId: action.context === 'editor' ? 'navigation' : undefined,
          contextMenuOrder: 1.5,
          run: () => {
            action.action();
          },
        });

        // Store the binding for cleanup
        const existingBindings = this.editorKeybindings.get(action.id) || [];
        existingBindings.push(disposable);
        this.editorKeybindings.set(action.id, existingBindings);
      }
    }
  }

  unregisterMonacoAction(editor: monaco.editor.IStandaloneCodeEditor, actionId: string): void {
    const bindings = this.editorKeybindings.get(actionId);
    if (bindings) {
      bindings.forEach(binding => binding.dispose());
      this.editorKeybindings.delete(actionId);
    }
  }

  updateMonacoKeybindings(editor: monaco.editor.IStandaloneCodeEditor): void {
    // Clear all existing bindings
    this.editorKeybindings.forEach(bindings => {
      bindings.forEach(binding => binding.dispose());
    });
    this.editorKeybindings.clear();

    // TODO: Re-register all actions with current shortcuts
    // This would require access to the registered actions from KeyboardService
  }

  private getKeysFromShortcut(action: ShortcutAction): KeyCombination[] {
    // This would typically come from KeyboardService, but we don't have access here
    // For now, return empty array
    return [];
  }

  private keyCombinationToMonacoKey(combo: KeyCombination): number | null {
    let monacoKey = 0;

    if (combo.ctrlKey) monacoKey |= monaco.KeyMod.CtrlCmd;
    if (combo.altKey) monacoKey |= monaco.KeyMod.Alt;
    if (combo.shiftKey) monacoKey |= monaco.KeyMod.Shift;
    if (combo.metaKey) monacoKey |= monaco.KeyMod.WinCtrl;

    // Add the key code
    const keyCode = this.keyToMonacoKeyCode(combo.key);
    if (keyCode === null) return null;

    return monacoKey | keyCode;
  }

  private keyToMonacoKeyCode(key: string): number | null {
    const keyMap: Record<string, number> = {
      'a': monaco.KeyCode.KeyA,
      'b': monaco.KeyCode.KeyB,
      'c': monaco.KeyCode.KeyC,
      'd': monaco.KeyCode.KeyD,
      'e': monaco.KeyCode.KeyE,
      'f': monaco.KeyCode.KeyF,
      'g': monaco.KeyCode.KeyG,
      'h': monaco.KeyCode.KeyH,
      'i': monaco.KeyCode.KeyI,
      'j': monaco.KeyCode.KeyJ,
      'k': monaco.KeyCode.KeyK,
      'l': monaco.KeyCode.KeyL,
      'm': monaco.KeyCode.KeyM,
      'n': monaco.KeyCode.KeyN,
      'o': monaco.KeyCode.KeyO,
      'p': monaco.KeyCode.KeyP,
      'q': monaco.KeyCode.KeyQ,
      'r': monaco.KeyCode.KeyR,
      's': monaco.KeyCode.KeyS,
      't': monaco.KeyCode.KeyT,
      'u': monaco.KeyCode.KeyU,
      'v': monaco.KeyCode.KeyV,
      'w': monaco.KeyCode.KeyW,
      'x': monaco.KeyCode.KeyX,
      'y': monaco.KeyCode.KeyY,
      'z': monaco.KeyCode.KeyZ,
      '0': monaco.KeyCode.Digit0,
      '1': monaco.KeyCode.Digit1,
      '2': monaco.KeyCode.Digit2,
      '3': monaco.KeyCode.Digit3,
      '4': monaco.KeyCode.Digit4,
      '5': monaco.KeyCode.Digit5,
      '6': monaco.KeyCode.Digit6,
      '7': monaco.KeyCode.Digit7,
      '8': monaco.KeyCode.Digit8,
      '9': monaco.KeyCode.Digit9,
      'f1': monaco.KeyCode.F1,
      'f2': monaco.KeyCode.F2,
      'f3': monaco.KeyCode.F3,
      'f4': monaco.KeyCode.F4,
      'f5': monaco.KeyCode.F5,
      'f6': monaco.KeyCode.F6,
      'f7': monaco.KeyCode.F7,
      'f8': monaco.KeyCode.F8,
      'f9': monaco.KeyCode.F9,
      'f10': monaco.KeyCode.F10,
      'f11': monaco.KeyCode.F11,
      'f12': monaco.KeyCode.F12,
      '[equal]': monaco.KeyCode.Equal,
      'enter': monaco.KeyCode.Enter,
      'escape': monaco.KeyCode.Escape,
      'space': monaco.KeyCode.Space,
      'backslash': monaco.KeyCode.Backslash,
      'slash': monaco.KeyCode.Slash,
      'period': monaco.KeyCode.Period,
      'comma': monaco.KeyCode.Comma,
      'bracketleft': monaco.KeyCode.BracketLeft,
      'bracketright': monaco.KeyCode.BracketRight,
    };

    const lowerKey = key.toLowerCase();
    return keyMap[lowerKey] || null;
  }
}

// Create singleton instance
export const monacoKeyboardIntegration = new MonacoKeyboardIntegrationImpl();