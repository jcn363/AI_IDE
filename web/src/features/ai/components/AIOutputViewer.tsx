import { Close, Code, Error as ErrorIcon, Lightbulb } from '@mui/icons-material';
import {
  Alert,
  AlertTitle,
  Box,
  CircularProgress,
  IconButton,
  Paper,
  Typography,
} from '@mui/material';
import React from 'react';
import Editor, { type EditorProps } from '@monaco-editor/react';

// Create a typed version of the Editor component
const EditorComponent = Editor as unknown as React.FC<EditorProps>;
import { useAIAssistant } from '../hooks/useAIAssistant';

interface AIOutputViewerProps {
  onClose?: () => void;
  maxHeight?: string | number;
}

const AIOutputViewer: React.FC<AIOutputViewerProps> = ({ onClose, maxHeight = '300px' }) => {
  const { generatedContent, generationError, isGenerating, clearGeneratedContent } =
    useAIAssistant();

  const handleClose = () => {
    clearGeneratedContent();
    if (onClose) onClose();
  };

  if (!generatedContent && !generationError && !isGenerating) {
    return null;
  }

  // Simple check if the content looks like code
  const isLikelyCode = (content: string) => {
    const codeIndicators = ['{', '}', ';', 'fn ', 'let ', 'const ', 'import ', 'export '];
    return codeIndicators.some((indicator) => content.includes(indicator));
  };

  const renderContent = () => {
    if (isGenerating) {
      return (
        <Box display="flex" flexDirection="column" alignItems="center" p={4}>
          <CircularProgress size={24} sx={{ mb: 2 }} />
          <Typography variant="body2" color="text.secondary">
            Generating content...
          </Typography>
        </Box>
      );
    }

    if (generationError) {
      return (
        <Alert
          severity="error"
          sx={{
            borderRadius: 1,
            mb: 2,
            '& .MuiAlert-message': {
              width: '100%',
            },
          }}
        >
          <AlertTitle>Error</AlertTitle>
          {generationError}
        </Alert>
      );
    }

    if (!generatedContent) return null;

    if (isLikelyCode(generatedContent)) {
      return (
        <Box
          sx={{
            position: 'relative',
            height: maxHeight,
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
            overflow: 'hidden',
          }}
        >
          <EditorComponent
            height="100%"
            defaultLanguage="rust"
            value={generatedContent}
            options={{
              readOnly: true,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              fontSize: 14,
              wordWrap: 'on',
              lineNumbers: 'on',
              renderLineHighlight: 'all',
              automaticLayout: true,
              theme: 'vs-dark',
            }}
          />
        </Box>
      );
    }

    // Render as markdown or plain text
    return (
      <Box
        sx={{
          p: 2,
          whiteSpace: 'pre-wrap',
          maxHeight,
          overflow: 'auto',
          '& p': {
            margin: '0 0 8px 0',
          },
          '& code': {
            backgroundColor: 'rgba(0,0,0,0.1)',
            padding: '2px 4px',
            borderRadius: 3,
            fontFamily: 'monospace',
            fontSize: '0.9em',
          },
        }}
      >
        {generatedContent.split('\n').map((line, i) => (
          <p key={i}>{line || <br />}</p>
        ))}
      </Box>
    );
  };

  return (
    <Paper
      elevation={3}
      sx={{
        position: 'relative',
        mt: 2,
        borderRadius: 1,
        overflow: 'hidden',
        border: '1px solid',
        borderColor: 'divider',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          px: 2,
          py: 1,
          bgcolor: 'background.paper',
          borderBottom: '1px solid',
          borderColor: 'divider',
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', flex: 1 }}>
          {generationError ? (
            <ErrorIcon color="error" fontSize="small" sx={{ mr: 1 }} />
          ) : isGenerating ? (
            <CircularProgress size={16} sx={{ mr: 1 }} />
          ) : isLikelyCode(generatedContent || '') ? (
            <Code color="primary" fontSize="small" sx={{ mr: 1 }} />
          ) : (
            <Lightbulb color="primary" fontSize="small" sx={{ mr: 1 }} />
          )}
          <Typography variant="subtitle2" color="text.primary">
            {generationError
              ? 'Error'
              : isGenerating
                ? 'Generating...'
                : isLikelyCode(generatedContent || '')
                  ? 'Generated Code'
                  : 'AI Suggestion'}
          </Typography>
        </Box>
        <IconButton size="small" onClick={handleClose} sx={{ ml: 1 }}>
          <Close fontSize="small" />
        </IconButton>
      </Box>
      {renderContent()}
    </Paper>
  );
};

export default AIOutputViewer;
