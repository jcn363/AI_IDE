import { useEffect, useMemo, useRef } from 'react';
import type * as monaco from 'monaco-editor';
import type { OnMount } from '@monaco-editor/react';
import { useAppSelector } from '../store/store';
import type { RootState } from '../store/types';
import { DebuggerService } from '../services/debuggerService';

type Breakpoint = { file: string; line: number; id?: number; enabled?: boolean };

export function useEditorBreakpoints(params: {
  activeFilePath: string | null | undefined;
  currentFileContent: string;
  editorRef: React.RefObject<monaco.editor.IStandaloneCodeEditor | null>;
  monacoRef: React.RefObject<typeof monaco | null>;
}) {
  const { activeFilePath, currentFileContent, editorRef, monacoRef } = params;

  const dbgBreakpoints = useAppSelector(
    (state: RootState) => state.debugger.breakpoints as Array<Breakpoint>
  );

  const breakpointCollectionsRef = useRef<
    Record<number, monaco.editor.IEditorDecorationsCollection>
  >({});
  const breakpointMapRef = useRef<Map<string, number>>(new Map());

  // Maintain file:line -> breakpoint id map
  useEffect(() => {
    const m = new Map<string, number>();
    for (const b of dbgBreakpoints) {
      if (typeof b.id === 'number') {
        m.set(`${b.file}:${b.line}`, b.id);
      }
    }
    breakpointMapRef.current = m;

    return () => {
      breakpointMapRef.current = new Map();
    };
  }, [dbgBreakpoints]);

  // Sync gutter glyphs with debugger breakpoints
  useEffect(() => {
    const editor = editorRef.current;
    const mi = monacoRef.current;
    if (!editor || !mi) return;

    // Clear existing decorations
    Object.values(breakpointCollectionsRef.current).forEach((c) => {
      try {
        c.clear();
      } catch {
        // no-op
      }
    });
    breakpointCollectionsRef.current = {};

    if (!activeFilePath) return;

    // Add current decorations
    (dbgBreakpoints || []).forEach((b) => {
      if (!(b.file === activeFilePath && (b.enabled ?? true))) return;
      const coll = editor.createDecorationsCollection([
        {
          range: new mi.Range(b.line, 1, b.line, 1),
          options: {
            isWholeLine: false,
            glyphMarginClassName: 'breakpoint-glyph',
            glyphMarginHoverMessage: { value: 'Breakpoint' },
          },
        },
      ]);
      breakpointCollectionsRef.current[b.line] = coll;
    });

    return () => {
      Object.values(breakpointCollectionsRef.current).forEach((c) => {
        try {
          c.clear();
        } catch {
          // no-op
        }
      });
      breakpointCollectionsRef.current = {};
    };
  }, [dbgBreakpoints, activeFilePath]);

  const onEditorDidMount: OnMount = (editor, monacoInstance) => {
    // Register basic Rust language if not present
    if (
      !monacoInstance.languages.getLanguages().some((lang: { id: string }) => lang.id === 'rust')
    ) {
      monacoInstance.languages.register({ id: 'rust', extensions: ['.rs'] });
      monacoInstance.languages.setLanguageConfiguration('rust', {
        comments: {
          lineComment: '//',
          blockComment: ['/*', '*/'],
        },
        brackets: [
          ['{', '}'],
          ['[', ']'],
          ['(', ')'],
        ],
        autoClosingPairs: [
          { open: '{', close: '}' },
          { open: '[', close: ']' },
          { open: '(', close: ')' },
          { open: '"', close: '"' },
          { open: "'", close: "'" },
        ],
        surroundingPairs: [
          { open: '{', close: '}' },
          { open: '[', close: ']' },
          { open: '(', close: ')' },
          { open: '"', close: '"' },
          { open: "'", close: "'" },
        ],
      });
    }

    // Set up breakpoint toggling on gutter click
    const disposables: monaco.IDisposable[] = [];
    disposables.push(
      editor.onMouseDown(async (e: monaco.editor.IEditorMouseEvent) => {
        if (e.target?.type !== monacoInstance.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) return;
        const lineNumber = e.target.position?.lineNumber;
        if (!lineNumber || !activeFilePath) return;
        const key = `${activeFilePath}:${lineNumber}`;
        const id = breakpointMapRef.current.get(key);
        try {
          if (id) {
            await DebuggerService.removeBreakpoint(id);
          } else {
            await DebuggerService.setBreakpoint(activeFilePath, lineNumber);
          }
        } catch (err) {
          console.error('Breakpoint toggle failed', err);
        }
      })
    );

    // Clear any existing markers for rust-analyzer
    const model = editor.getModel();
    if (model) {
      monacoInstance.editor.setModelMarkers(model, 'rust-analyzer', []);
    }

    // Initialize content
    if (activeFilePath && currentFileContent) {
      editor.setValue(currentFileContent);
    }

    return () => {
      disposables.forEach((d) => {
        try {
          d.dispose?.();
        } catch {
          // no-op
        }
      });
    };
  };

  return {
    onEditorDidMount,
  };
}
