import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Box, Typography, Button, Paper, Tooltip, Switch, FormControlLabel, IconButton } from '@mui/material';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import { Clear as ClearIcon } from '@mui/icons-material';
import TabPanel from './TabPanel';
import { useAppDispatch, useAppSelector } from '../../store/store';
import { tabManagementActions } from '../../store/slices/tabManagementSlice';
import { editorActions } from '../../store/slices/editorSlice';
import type { RootState } from '../../store/types';

interface CommandOutputProps {
  activeTab: number;
  commands: Record<string, any>;
  handleClearOutput: () => void;
  parseJson?: boolean;
}

const errorPattern = /error\[[A-Z]?\d{4}\]|panicked at|failed to compile|could not compile|E\d{4}/i;
const locationPatterns = [
  /\s*-->\s+([^:]+):(\d+):(\d+)/, // rustc arrow lines
  /(^|\s)([^\s:]+\.rs):(\d+):(\d+)/, // generic file:line:col
];

function normalizePath(path: string): string {
  const parts = path.split('/');
  const stack: string[] = [];
  for (const part of parts) {
    if (part === '' || part === '.') continue;
    if (part === '..') stack.pop();
    else stack.push(part);
  }
  return '/' + stack.join('/');
}

function resolvePathAgainstRoot(root: string | undefined, p: string): string {
  if (!p) return p;
  if (p.startsWith('/') || /^[A-Za-z]:\\/.test(p)) return p; // absolute (unix/windows)
  if (!root) return p;
  return normalizePath(`${root}/${p}`);
}

function useOpenInEditor() {
  const dispatch = useAppDispatch();
  const activePaneId = useAppSelector((s: RootState) => s.tabManagement.activePaneId);
  const workspaceRoot = useAppSelector((s: RootState) => s.editor.fileTree?.path);
  return useCallback((filePath: string, line?: number, column?: number) => {
    if (!filePath) return;
    const resolved = resolvePathAgainstRoot(workspaceRoot, filePath);
    if (activePaneId) {
      dispatch(tabManagementActions.openFileInPane({ paneId: activePaneId, filePath: resolved }));
    }
    dispatch(editorActions.setCurrentFile(resolved));
    dispatch(editorActions.setNavigationTarget({ filePath: resolved, line, column }));
    // Navigation is handled by EditorPage, which listens to navigationTarget and applies the cursor move in Monaco.
  }, [dispatch, activePaneId, workspaceRoot]);
}

function useRenderHighlighted(parseJsonDiagnostics: boolean) {
  const openInEditor = useOpenInEditor();
  return useCallback((text: string) => {
    const lines = text.split('\n');
    return (
      <>
        {lines.map((line, idx) => {
          // Extract possible file locations
          let clickable: { path: string; line?: number; col?: number } | null = null;
          let display = line;

          // Try JSON diagnostics if enabled
          if (parseJsonDiagnostics) {
            try {
              const obj = JSON.parse(line);
              if (obj && obj.message && obj.spans && obj.spans.length) {
                const span = obj.spans[0];
                if (span && span.file_name && span.line_start && span.column_start) {
                  clickable = {
                    path: span.file_name,
                    line: span.line_start,
                    col: span.column_start,
                  };
                  display = `${(obj.level || 'diagnostic').toUpperCase()}: ${obj.message} (${span.file_name}:${span.line_start}:${span.column_start})`;
                }
              }
            } catch {}
          }

          for (const pat of locationPatterns) {
            const m = line.match(pat as RegExp);
            if (m) {
              // Pattern lengths:
              //  - rustc arrow: [full, path, line, col] => length 4
              //  - generic:     [full, spaceOrStart, path, line, col] => length 5
              let path = '';
              let ln: number | undefined;
              let col: number | undefined;
              if (m.length === 4) {
                path = m[1];
                ln = parseInt(m[2], 10);
                col = parseInt(m[3], 10);
              } else if (m.length === 5) {
                path = m[2];
                ln = parseInt(m[3], 10);
                col = parseInt(m[4], 10);
              }
              if (path) {
                clickable = { path, line: isNaN(ln as any) ? undefined : ln, col: isNaN(col as any) ? undefined : col };
                break;
              }
            }
          }

          const isError = errorPattern.test(line);
          return (
            <Box
              key={idx}
              component="div"
              sx={{
                color: isError ? 'error.main' : 'text.primary',
                whiteSpace: 'pre-wrap',
                fontFamily: 'inherit',
                cursor: clickable ? 'pointer' : 'default',
                textDecoration: clickable ? 'underline' : 'none',
                textUnderlineOffset: clickable ? '2px' : undefined,
              }}
              onClick={() => {
                if (clickable) openInEditor(clickable.path, clickable.line, clickable.col);
              }}
              title={clickable ? `Open ${clickable.path}${clickable.line ? `:${clickable.line}${clickable.col ? `:${clickable.col}` : ''}` : ''}` : undefined}
            >
              {display}
            </Box>
          );
        })}
      </>
    );
  }, [openInEditor, parseJsonDiagnostics]);
}

const CommandOutput: React.FC<CommandOutputProps> = ({
  activeTab,
  commands,
  handleClearOutput,
  parseJson = false,
}) => {
  const bottomRef = useRef<HTMLDivElement>(null);
  const dispatch = useAppDispatch();
  const renderHighlighted = useRenderHighlighted(parseJson);
  const openInEditor = useOpenInEditor();
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  // Memoize sorted commands by timestamp desc
  const commandEntries = useMemo(
    () => Object.entries(commands).sort((a, b) => (b[1]?.timestamp ?? 0) - (a[1]?.timestamp ?? 0)),
    [commands]
  );

  const handleCopyAll = useCallback(() => {
    const all = commandEntries.map(([, cmd]) => `$ cargo ${cmd.command} ${cmd.args?.join(' ')}\n${cmd.output ?? ''}`).join('\n\n');
    const nav = (globalThis as any)?.navigator;
    if (nav && nav.clipboard && typeof nav.clipboard.writeText === 'function') {
      nav.clipboard.writeText(all).catch(() => {});
    }
  }, [commandEntries]);

  useEffect(() => {
    // Use a tolerant call to support TS configs without DOM lib
    const el = bottomRef.current as unknown as { scrollIntoView?: (opts?: any) => void } | null;
    el?.scrollIntoView?.({ behavior: 'smooth' });
  }, [commandEntries]);

  const handleExpandAll = useCallback(() => {
    const next: Record<string, boolean> = { ...collapsed };
    for (const [id, cmd] of commandEntries as [string, any][]) {
      const blocks = (cmd.output as string | undefined)?.split(/\n\s*\n/) || [];
      blocks.forEach((_, idx) => {
        next[`${id}-${idx}`] = false;
      });
    }
    setCollapsed(next);
  }, [collapsed, commandEntries]);

  const handleCollapseAll = useCallback(() => {
    const next: Record<string, boolean> = { ...collapsed };
    for (const [id, cmd] of commandEntries as [string, any][]) {
      const blocks = (cmd.output as string | undefined)?.split(/\n\s*\n/) || [];
      blocks.forEach((block, idx) => {
        const isErrBlock = errorPattern.test(block);
        next[`${id}-${idx}`] = true;
        // Default collapse behavior can prioritize error blocks, but we collapse all here
        void isErrBlock;
      });
    }
    setCollapsed(next);
  }, [collapsed, commandEntries]);

  return (
    <TabPanel value={activeTab} index={2}>
      <Box sx={{ mb: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Typography variant="subtitle1">Command Output</Typography>
        <Button
          size="small"
          onClick={handleClearOutput}
          disabled={Object.keys(commands).length === 0}
          startIcon={<ClearIcon />}
        >
          Clear All
        </Button>
        <Tooltip title="Copy all output">
          <span>
            <Button size="small" onClick={handleCopyAll} disabled={commandEntries.length === 0}>
              Copy All
            </Button>
          </span>
        </Tooltip>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Tooltip title="Expand all blocks">
            <span>
              <Button size="small" onClick={handleExpandAll} disabled={commandEntries.length === 0}>Expand All</Button>
            </span>
          </Tooltip>
          <Tooltip title="Collapse all blocks">
            <span>
              <Button size="small" onClick={handleCollapseAll} disabled={commandEntries.length === 0}>Collapse All</Button>
            </span>
          </Tooltip>
          <FormControlLabel
            sx={{ ml: 1 }}
            control={<Switch size="small" checked={parseJson} onChange={() => { /* controlled by parent */ }} />}
            label={<Typography variant="caption">Parse JSON diagnostics</Typography>}
          />
        </Box>
      </Box>
      
      {commandEntries.map(([id, cmd]: [string, any]) => (
        <Box key={id} sx={{ mb: 2 }}>
          <Typography variant="subtitle2" color="text.secondary">
            $ cargo {cmd.command} {cmd.args?.join(' ')}
          </Typography>
          {parseJson && Array.isArray(cmd.diagnostics) && cmd.diagnostics.length > 0 && (
            <Paper variant="outlined" sx={{ p: 1, bgcolor: 'background.paper', mb: 1 }}>
              <Typography variant="caption" sx={{ fontWeight: 600, display: 'block', mb: 0.5 }}>
                Diagnostics ({cmd.diagnostics.length})
              </Typography>
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.5 }}>
                {cmd.diagnostics.map((d: any, dIdx: number) => {
                  const span = d.spans?.[0];
                  const loc = span ? `${span.file_name}:${span.line_start}:${span.column_start}` : '';
                  const clickable = span && span.file_name;
                  const level = (d.level || '').toUpperCase();
                  return (
                    <Box
                      key={`${id}-diag-${dIdx}`}
                      sx={{
                        color: /ERROR/i.test(level) ? 'error.main' : 'text.primary',
                        cursor: clickable ? 'pointer' : 'default',
                        textDecoration: clickable ? 'underline' : 'none',
                        textUnderlineOffset: clickable ? '2px' : undefined,
                        fontFamily: 'monospace',
                      }}
                      onClick={() => {
                        if (clickable) {
                          // Use first span for navigation
                          const line = span.line_start ? Number(span.line_start) : undefined;
                          const col = span.column_start ? Number(span.column_start) : undefined;
                          openInEditor(span.file_name, line, col);
                        }
                      }}
                      title={clickable ? `Open ${loc}` : undefined}
                    >
                      [{level || 'DIAG'}] {d.message || ''}{loc ? ` (${loc})` : ''}
                    </Box>
                  );
                })}
              </Box>
            </Paper>
          )}
          {cmd.output ? (
            // Collapse by double newlines into blocks
            cmd.output.split(/\n\s*\n/).map((block: string, idx: number) => {
              const key = `${id}-${idx}`;
              const isErrBlock = errorPattern.test(block);
              const isCollapsed = collapsed[key] ?? (isErrBlock ? true : false);
              return (
                <Paper key={key} variant="outlined" sx={{ p: 1, bgcolor: 'background.paper', mb: 1 }}>
                  <Box sx={{ display: 'flex', alignItems: 'center', mb: isCollapsed ? 0 : 1 }}>
                    <IconButton size="small" onClick={() => setCollapsed(prev => ({ ...prev, [key]: !isCollapsed }))}>
                      {isCollapsed ? <ExpandMoreIcon fontSize="small" /> : <ExpandLessIcon fontSize="small" />}
                    </IconButton>
                    <Typography variant="caption" color={isErrBlock ? 'error.main' : 'text.secondary'} sx={{ ml: 0.5 }}>
                      {isErrBlock ? 'Error Block' : 'Output Block'} {isCollapsed && `(${block.split('\n').length} lines)`}
                    </Typography>
                    <Box sx={{ flexGrow: 1 }} />
                    <Tooltip title="Export block">
                      <span>
                        <Button size="small" onClick={() => {
                          const content = block;
                          const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
                          const a = document.createElement('a') as any; // tolerant typing for projects without DOM lib
                          const href = URL.createObjectURL(blob);
                          a.href = href;
                          a.download = `cargo-block-${idx}.txt`;
                          a.style = 'display:none';
                          (document.body as any)?.appendChild?.(a);
                          a.click?.();
                          a.remove?.();
                          URL.revokeObjectURL(href);
                        }}>Save</Button>
                      </span>
                    </Tooltip>
                  </Box>
                  {!isCollapsed && (
                    <Box sx={{ whiteSpace: 'pre-wrap', fontFamily: 'monospace' }}>
                      {renderHighlighted(block)}
                    </Box>
                  )}
                </Paper>
              );
            })
          ) : (
            <Paper variant="outlined" sx={{ p: 1, bgcolor: 'background.paper' }}>No output yet...</Paper>
          )}
        </Box>
      ))}
      <div ref={bottomRef} />
    </TabPanel>
  );
}

export default CommandOutput;
