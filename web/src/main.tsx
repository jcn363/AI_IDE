import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { CssBaseline } from '@mui/material';
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import './styles/debugger.css';

const rootEl = (document as any).getElementById('root');
ReactDOM.createRoot(rootEl!).render(
  <React.StrictMode>
    <CssBaseline />
    <App />
  </React.StrictMode>
);
