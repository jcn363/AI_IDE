import React, { Component, ErrorInfo, ReactNode } from 'react';
import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Box, Typography, Button, Paper, Alert } from './components/shared/MaterialUI';

// Error Boundary Component
interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    if (process.env.NODE_ENV === 'development') {
      console.error('Error Boundary caught an error:', error, errorInfo);
    }

    this.props.onError?.(error, errorInfo);
    this.setState({ error, errorInfo });
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            minHeight: '100vh',
            p: 2,
          }}
        >
          <Paper
            elevation={3}
            sx={{
              maxWidth: 600,
              p: 3,
              textAlign: 'center',
            }}
          >
            <Alert severity="error" sx={{ mb: 2 }}>
              Something went wrong!
            </Alert>
            <Typography variant="h6" gutterBottom>
              An unexpected error occurred
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
              {this.state.error?.message || 'Please try refreshing the page or contact support if the problem persists.'}
            </Typography>
            <Button
              variant="contained"
              color="primary"
              onClick={this.handleRetry}
              sx={{ mr: 1 }}
            >
              Try Again
            </Button>
            <Button
              variant="outlined"
              onClick={() => window.location.reload()}
            >
              Refresh Page
            </Button>
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <Box sx={{ mt: 2, textAlign: 'left' }}>
                <details>
                  <summary>Error Details (Development)</summary>
                  <pre style={{ fontSize: '12px', overflow: 'auto', maxHeight: '200px' }}>
                    {this.state.error.stack}
                  </pre>
                </details>
              </Box>
            )}
          </Paper>
        </Box>
      );
    }

    return this.props.children;
  }
}

import React, { Suspense } from 'react';
import {
  CssBaseline,
  ThemeProvider,
  createTheme,
  Box,
  CircularProgress,
} from '../components/shared/MaterialUI';
import { Navigate, Route, BrowserRouter as Router, Routes } from 'react-router-dom';
import { Provider } from 'react-redux';
import { store } from './store';
import { Layout } from './components/Layout';

// Lazy load all page components for code splitting
const Home = React.lazy(() => import('./pages/Home'));
const EditorPage = React.lazy(() => import('./pages/EditorPage'));
const Settings = React.lazy(() => import('./pages/Settings'));
const BuildPage = React.lazy(() => import('./pages/BuildPage'));
const FileExplorer = React.lazy(() => import('./components/FileExplorer/index'));
const DebuggerPanel = React.lazy(() => import('./components/DebuggerPanel'));
const TestingPage = React.lazy(() => import('./pages/TestingPage'));
const DocsPage = React.lazy(() => import('./pages/DocsPage'));
const DependencyGraphPage = React.lazy(() => import('./pages/DependencyGraphPage'));
const VersionAlignmentPage = React.lazy(() => import('./pages/VersionAlignmentPage'));

// Loading component for Suspense fallback
const LoadingFallback = () => (
  <Box
    sx={{
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      height: '100vh',
    }}
  >
    <CircularProgress />
  </Box>
);

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#90caf9',
    },
    secondary: {
      main: '#f48fb1',
    },
    background: {
      default: '#121212',
      paper: '#1e1e1e',
    },
  },
  typography: {
    fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
    h5: {
      fontWeight: 500,
    },
  },
  components: {
    MuiAppBar: {
      styleOverrides: {
        root: {
          backgroundColor: '#1e1e1e',
          borderBottom: '1px solid #333',
        },
      },
    },
    MuiDrawer: {
      styleOverrides: {
        paper: {
          backgroundColor: '#252526',
          color: '#e0e0e0',
        },
      },
    },
  },
});

function App() {
  return (
    <Provider store={store}>
      <ThemeProvider theme={darkTheme}>
        <CssBaseline />
        <Router>
          <Suspense fallback={<LoadingFallback />}>
            <Routes>
              <Route path="/" element={<Layout />}>
                <Route index element={<Navigate to="/home" replace />} />
                <Route path="home" element={<Home />} />
                <Route path="editor" element={<EditorPage />} />
                <Route path="build" element={<BuildPage />} />
                <Route path="explorer" element={<FileExplorer />} />
                <Route path="settings" element={<Settings />} />
                <Route path="debugger" element={<DebuggerPanel />} />
                <Route path="testing" element={<TestingPage />} />
                <Route path="docs" element={<DocsPage />} />
                <Route path="dependencies" element={<DependencyGraphPage />} />
                <Route path="version-alignment" element={<VersionAlignmentPage />} />
              </Route>
            </Routes>
          </Suspense>
        </Router>
      </ThemeProvider>
    </Provider>
  );
}

export default App;
