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
