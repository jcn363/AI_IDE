// Simple test utilities for React components
import React from 'react';
import { render as rtlRender } from '@testing-library/react';

// Simple render function that can be extended if needed
export function render(ui: React.ReactElement, options = {}) {
  return rtlRender(ui, options);
}

export * from '@testing-library/react';
export { render as default } from '@testing-library/react';
