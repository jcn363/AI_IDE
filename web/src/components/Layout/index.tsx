import React from 'react';
import {
  Box,
  CssBaseline,
  Divider,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Typography,
  CodeIcon,
  SettingsIcon,
  HomeIcon,
  FolderOpenIcon,
  BuildIcon,
  BugReportIcon,
  ScienceIcon,
  DescriptionIcon,
  AccountTreeIcon,
  CompareArrowsIcon,
} from '../shared/MaterialUI';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useAppSelector } from '../../store/store';
import { tabManagementSelectors } from '../../store/slices/tabManagementSlice';
import { FileExplorer } from '../FileExplorer/index';
import StatusBar from '../StatusBar';
import CargoNotifier from '../Notifications/CargoNotifier';

const drawerWidth = 240;

interface MenuItem {
  text: string;
  icon: React.ReactNode;
  path: string;
}

const menuItems: MenuItem[] = [
  { text: 'Home', icon: <HomeIcon />, path: '/home' },
  { text: 'Editor', icon: <CodeIcon />, path: '/editor' },
  { text: 'Build', icon: <BuildIcon />, path: '/build' },
  { text: 'Explorer', icon: <FolderOpenIcon />, path: '/explorer' },
  { text: 'Debugger', icon: <BugReportIcon />, path: '/debugger' },
  { text: 'Testing', icon: <ScienceIcon />, path: '/testing' },
  { text: 'Docs', icon: <DescriptionIcon />, path: '/docs' },
  { text: 'Dependencies', icon: <AccountTreeIcon />, path: '/dependencies' },
  { text: 'Version Alignment', icon: <CompareArrowsIcon />, path: '/version-alignment' },
  { text: 'Settings', icon: <SettingsIcon />, path: '/settings' },
];

interface LayoutProps {
  children?: React.ReactNode;
}

export function Layout({ children }: Readonly<LayoutProps>) {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Box sx={{ display: 'flex' }}>
      <CssBaseline />

      {/* Sidebar */}
      <Drawer
        variant="permanent"
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          display: 'flex',
          flexDirection: 'column',
          [`& .MuiDrawer-paper`]: {
            width: drawerWidth,
            boxSizing: 'border-box',
            display: 'flex',
            flexDirection: 'column',
          },
        }}
      >
        <Toolbar>
          <Typography variant="h6" noWrap component="div">
            Rust AI IDE
          </Typography>
        </Toolbar>
        <Divider />
        <Box sx={{ overflow: 'auto', flex: 1, display: 'flex', flexDirection: 'column' }}>
          <List sx={{ flexShrink: 0 }}>
            {menuItems.map((item) => (
              <ListItem key={item.text} disablePadding>
                <ListItemButton
                  selected={location.pathname === item.path}
                  onClick={() => navigate(item.path)}
                >
                  <ListItemIcon>{item.icon}</ListItemIcon>
                  <ListItemText primary={item.text} />
                </ListItemButton>
              </ListItem>
            ))}
          </List>
          <Divider />
          {location.pathname === '/explorer' && (
            <Box sx={{ flex: 1, overflow: 'auto' }}>
              <FileExplorer />
            </Box>
          )}
        </Box>
      </Drawer>

      {/* Main content + Status bar */}
      <Box
        component="section"
        sx={{ flexGrow: 1, height: '100vh', display: 'flex', flexDirection: 'column' }}
      >
        <Box component="main" sx={{ p: 3, flex: 1, overflow: 'auto' }}>
          <Toolbar />
          {children || <Outlet />}
        </Box>
        <StatusBar
          activeFilePath={useAppSelector(
            (state) => tabManagementSelectors.selectActivePane(state)?.activeFile || null
          )}
          isSaving={false} // TODO: Implement saving state in tabManagementSlice if needed
          isConnected={useAppSelector((state) => {
            // Check if language server state is available in the store
            if ('languageServer' in state) {
              return (state as any).languageServer?.isConnected || false;
            }
            // Fallback to true if language server state is not available
            return true;
          })}
        />
        <CargoNotifier />
      </Box>
    </Box>
  );
}
