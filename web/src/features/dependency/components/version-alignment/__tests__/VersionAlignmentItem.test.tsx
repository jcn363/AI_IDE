import { describe, expect, it, vi } from 'vitest';
import React from 'react';
import { fireEvent, render, screen } from '../../../../../test/test-utils';
import VersionAlignmentItem from '../VersionAlignmentItem';
import { VersionAlignment } from '../../../types';
import { ThemeProvider } from '@emotion/react';
import { createTheme } from '@mui/material';

const mockAlignment: VersionAlignment = {
  id: 'serde',
  dependencyName: 'serde',
  currentVersions: {
    'crate1': '1.0.0',
    'crate2': '1.1.0',
  },
  suggestedVersion: '1.2.0',
  severity: 'medium',
  affectedPackages: ['crate1', 'crate2'],
};

const theme = createTheme();

const renderWithTheme = (component: React.ReactElement) => {
  return render(
    <ThemeProvider theme={theme}>
      {component}
    </ThemeProvider>,
  );
};

describe('VersionAlignmentItem', () => {
  it('renders the dependency name and suggested version', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();
    
    renderWithTheme(
      <VersionAlignmentItem 
        alignment={mockAlignment} 
        onApply={onApply} 
        onIgnore={onIgnore} 
      />,
    );

    expect(screen.getByText('serde')).toBeInTheDocument();
    expect(screen.getByText('→ 1.2.0')).toBeInTheDocument();
  });

  it('renders all current versions', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();
    
    renderWithTheme(
      <VersionAlignmentItem 
        alignment={mockAlignment} 
        onApply={onApply} 
        onIgnore={onIgnore} 
      />,
    );

    expect(screen.getByText('crate1: 1.0.0')).toBeInTheDocument();
    expect(screen.getByText('crate2: 1.1.0')).toBeInTheDocument();
  });

  it('calls onApply when apply button is clicked', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();
    
    renderWithTheme(
      <VersionAlignmentItem 
        alignment={mockAlignment} 
        onApply={onApply} 
        onIgnore={onIgnore} 
      />,
    );

    fireEvent.click(screen.getByText('Apply'));
    expect(onApply).toHaveBeenCalledWith(mockAlignment);
  });

  it('calls onIgnore when ignore button is clicked', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();
    
    renderWithTheme(
      <VersionAlignmentItem 
        alignment={mockAlignment} 
        onApply={onApply} 
        onIgnore={onIgnore} 
      />,
    );

    fireEvent.click(screen.getByText('Ignore'));
    expect(onIgnore).toHaveBeenCalledWith(mockAlignment);
  });
});
