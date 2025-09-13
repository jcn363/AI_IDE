// Optimized Material UI imports for better tree shaking
// Import only what you need to reduce bundle size

import React from 'react';

// Core Material UI components - frequently used
export {
  Box,
  Button,
  Card,
  CardContent,
  CardHeader,
  CircularProgress,
  Container,
  CssBaseline,
  Divider,
  Drawer,
  IconButton,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  TextField,
  Toolbar,
  Typography,
  AppBar,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  Avatar,
  Badge,
  Tooltip,
  Fab,
  Menu,
  MenuItem,
  Popover,
  Snackbar,
  Alert,
  Tabs,
  Tab,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Stepper,
  Step,
  StepLabel,
  LinearProgress,
  Grid,
  Stack,
  ThemeProvider,
  createTheme,
  useTheme,
  useMediaQuery,
} from '@mui/material';

// Material UI Icons - commonly used icons
export {
  Home as HomeIcon,
  Code as CodeIcon,
  Settings as SettingsIcon,
  FolderOpen as FolderOpenIcon,
  Build as BuildIcon,
  BugReport as BugReportIcon,
  Science as ScienceIcon,
  Description as DescriptionIcon,
  AccountTree as AccountTreeIcon,
  CompareArrows as CompareArrowsIcon,
  Menu as MenuIcon,
  Close as CloseIcon,
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Save as SaveIcon,
  Search as SearchIcon,
  ArrowBack as ArrowBackIcon,
  ArrowForward as ArrowForwardIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
  Check as CheckIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
} from '@mui/icons-material';

// Material UI System utilities
export { styled } from '@mui/material/styles';
export type { Theme, SxProps, Breakpoint } from '@mui/material/styles';

// Utility functions for performance
export const createOptimizedTheme = (options: any) => createTheme(options);

// Lazy load heavy components that might not be needed immediately
export const LazyTextField = React.lazy(() =>
  import('@mui/material').then((module) => ({ default: module.TextField }))
);
export const LazyDialog = React.lazy(() =>
  import('@mui/material').then((module) => ({ default: module.Dialog }))
);
export const LazyMenu = React.lazy(() =>
  import('@mui/material').then((module) => ({ default: module.Menu }))
);
