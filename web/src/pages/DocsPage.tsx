import * as React from 'react';
import { useState } from 'react';
import { Box, Button, Stack, TextField, Typography, Paper } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import DOMPurify from 'dompurify';

export default function DocsPage() {
  const [projectPath, setProjectPath] = useState<string>('');
  const [indexPath, setIndexPath] = useState<string>('');
  const [docHtml, setDocHtml] = useState<string>('');

  const generateDocs = async () => {
    try {
      const idx = await invoke<string>('doc_generate', { projectPath });
      setIndexPath(idx);
    } catch (e) {
      setIndexPath(String(e));
    }
  };

  const openEmbedded = async () => {
    try {
      if (!indexPath) return;
      const html = await invoke<string>('doc_read_file', { path: indexPath });
      // Sanitize HTML content to prevent XSS attacks
      const sanitizedHtml = DOMPurify.sanitize(html, {
        ALLOWED_TAGS: [
          'p',
          'br',
          'strong',
          'em',
          'u',
          'h1',
          'h2',
          'h3',
          'h4',
          'h5',
          'h6',
          'ul',
          'ol',
          'li',
          'blockquote',
          'code',
          'pre',
          'a',
          'img',
        ],
        ALLOWED_ATTR: ['href', 'src', 'alt', 'title'],
      });
      setDocHtml(sanitizedHtml);
    } catch (e) {
      // Even error messages should be sanitized as they could contain HTML
      const sanitizedError = DOMPurify.sanitize(String(e), {
        ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'code'],
        ALLOWED_ATTR: [],
      });
      setDocHtml(sanitizedError);
    }
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
      <Typography variant="h5">Documentation</Typography>
      <Stack direction="row" spacing={1} alignItems="center">
        <TextField
          size="small"
          label="Project Path"
          sx={{ minWidth: 360 }}
          value={projectPath}
          onChange={(e) => setProjectPath((e.target as any).value)}
        />
        <Button variant="contained" size="small" onClick={generateDocs}>
          Generate Docs
        </Button>
        {indexPath && (
          <Button size="small" onClick={openEmbedded}>
            Open Embedded
          </Button>
        )}
        {indexPath && (
          <Typography variant="body2" sx={{ ml: 1, color: 'text.secondary' }}>
            {indexPath}
          </Typography>
        )}
      </Stack>

      {docHtml && (
        <Paper sx={{ p: 1, height: '70vh', overflow: 'auto' }} variant="outlined">
          {/* Simplified viewer: embedding raw HTML as string; for full docs choose system open */}
          <div dangerouslySetInnerHTML={{ __html: docHtml }} />
        </Paper>
      )}
    </Box>
  );
}
