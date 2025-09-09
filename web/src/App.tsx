import { CssBaseline, ThemeProvider, createTheme } from '@mui/material';
import { Navigate, Route, BrowserRouter as Router, Routes } from 'react-router-dom';
import { Provider } from 'react-redux';
import { store } from './store';
import { Layout } from './components/Layout';
import { Home } from './pages/Home';
import EditorPage from './pages/EditorPage';
import { Settings } from './pages/Settings';
import BuildPage from './pages/BuildPage';
import { FileExplorer } from './components/FileExplorer/index';
import DebuggerPanel from './components/DebuggerPanel';
import TestingPage from './pages/TestingPage';
import DocsPage from './pages/DocsPage';
import DependencyGraphPage from './pages/DependencyGraphPage';
import VersionAlignmentPage from './pages/VersionAlignmentPage';

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
        </Router>
      </ThemeProvider>
    </Provider>
  );
}

export default App;
