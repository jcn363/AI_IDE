import {
    Book as BookIcon,
    Build as BuildIcon,
    ChatBubble as ChatBubbleIcon,
    Check as CheckIcon,
    Code as CodeIcon,
    ContentCopy as ContentCopyIcon,
    Search as SearchIcon,
} from '@mui/icons-material';
import {
    Box,
    Button,
    CircularProgress,
    Divider,
    FormControl,
    IconButton,
    InputLabel,
    MenuItem,
    Paper,
    Select,
    SelectChangeEvent,
    TextField,
    Tooltip,
    Typography,
} from '@mui/material';
import React, { useState } from 'react';
import AIContextMenu from '../context/AIContextMenu';
import { useAIAssistant } from '../hooks/useAIAssistant';

const AIFeatureExample: React.FC = () => {
  const [code, setCode] = useState<string>(
    'fn calculate_fibonacci(n: u32) -> u64 {\n    if n == 0 {\n        return 0;\n    } else if n == 1 {\n        return 1;\n    }\n    \n    let mut a = 0;\n    let mut b = 1;\n    \n    for _ in 2..=n {\n        let c = a + b;\n        a = b;\n        b = c;\n    }\n    \n    b\n}',
  );
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  
  const handleContextMenu = (event: React.MouseEvent<HTMLElement>) => {
    event.preventDefault();
    setAnchorEl(event.currentTarget);
  };
  
  const handleClose = () => {
    setAnchorEl(null);
  };

  const [selectedAction, setSelectedAction] = useState<string>('analyze');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [result, setResult] = useState<string>('');
  const [copied, setCopied] = useState<boolean>(false);
  
  const {
    analyzeCurrentFile,
    generateTests,
    generateDocumentation,
    explainCode,
    refactorCode,
  } = useAIAssistant();

  const handleAction = async () => {
    if (!code.trim()) return;
    
    setIsLoading(true);
    setResult('');
    
    try {
      let response;
      
      switch (selectedAction) {
        case 'analyze':
          response = await analyzeCurrentFile(code, 'example.rs');
          break;
        case 'tests':
          response = await generateTests(code, 'example.rs');
          break;
        case 'docs':
          response = await generateDocumentation(code, 'example.rs');
          break;
        case 'explain':
          response = await explainCode(code);
          break;
        case 'refactor':
          response = await refactorCode(code, 'example.rs');
          break;
        default:
          return;
      }
      
      setResult(JSON.stringify(response, null, 2));
    } catch (error) {
      setResult(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleActionChange = (event: SelectChangeEvent<string>) => {
    setSelectedAction(event.target.value);
  };

  const actionConfigs = [
    { value: 'analyze', label: 'Analyze Code', icon: <SearchIcon /> },
    { value: 'tests', label: 'Generate Tests', icon: <CodeIcon /> },
    { value: 'docs', label: 'Generate Documentation', icon: <BookIcon /> },
    { value: 'explain', label: 'Explain Code', icon: <ChatBubbleIcon /> },
    { value: 'refactor', label: 'Refactor Code', icon: <BuildIcon /> },
  ];

  return (
    <Paper elevation={3} sx={{ p: 3, maxWidth: 1000, margin: '0 auto' }}>
      <Typography variant="h5" gutterBottom>
        AI Features Example
      </Typography>
      
      <Box sx={{ mb: 3 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
          <Typography variant="subtitle2">Input Code:</Typography>
          <Tooltip title={copied ? 'Copied!' : 'Copy to clipboard'}>
            <IconButton size="small" onClick={handleCopy}>
              {copied ? <CheckIcon fontSize="small" /> : <ContentCopyIcon fontSize="small" />}
            </IconButton>
          </Tooltip>
        </Box>
        <TextField
          multiline
          fullWidth
          minRows={10}
          maxRows={20}
          variant="outlined"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          sx={{ fontFamily: 'monospace', mb: 2 }}
        />
        
        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
          <FormControl sx={{ minWidth: 200 }}>
            <InputLabel>Select Action</InputLabel>
            <Select
              value={selectedAction}
              onChange={handleActionChange}
              label="Select Action"
              size="small"
            >
              {actionConfigs.map((action) => (
                <MenuItem key={action.value} value={action.value}>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    {action.icon}
                    {action.label}
                  </Box>
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          
          <Button
            variant="contained"
            onClick={handleAction}
            disabled={isLoading}
            startIcon={isLoading ? <CircularProgress size={20} /> : null}
          >
            {isLoading ? 'Processing...' : 'Run'}
          </Button>
        </Box>
      </Box>
      
      <Divider sx={{ my: 2 }} />
      
      <Box>
        <Typography variant="subtitle2" gutterBottom>
          Result:
        </Typography>
        <Paper
          variant="outlined"
          sx={{
            p: 2,
            minHeight: 200,
            maxHeight: 400,
            overflow: 'auto',
            bgcolor: 'background.default',
            whiteSpace: 'pre-wrap',
            fontFamily: 'monospace',
          }}
        >
          {isLoading ? 'Processing your request...' : result || 'Results will appear here'}
        </Paper>
      </Box>
      
      <Box sx={{ mt: 3 }}>
        <Typography variant="subtitle2" gutterBottom>
          Try right-clicking in the code area:
        </Typography>
        <AIContextMenu
  anchorEl={anchorEl}
  onClose={handleClose}
          selectedText={code}
          filePath="example.rs"
          onGenerateCode={(generatedCode) => {
            setCode(generatedCode);
          }}
        >
          <Paper
            variant="outlined"
            onContextMenu={handleContextMenu}
            sx={{
              p: 2,
              minHeight: 100,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor: 'context-menu',
              userSelect: 'none',
              '&:hover': {
                bgcolor: 'action.hover',
              },
            }}
          >
            <Typography color="text.secondary">
              Right-click here to access AI context menu
            </Typography>
          </Paper>
        </AIContextMenu>
      </Box>
    </Paper>
  );
};

export default AIFeatureExample;
