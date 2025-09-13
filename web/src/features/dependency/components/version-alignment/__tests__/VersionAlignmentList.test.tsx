import React from 'react';
import { render, screen } from '../../../../../test/test-utils';
import { describe, expect, it, vi } from 'vitest';
import VersionAlignmentList from '../VersionAlignmentList';
import { VersionAlignment } from '../../../types';
import { ThemeProvider, createTheme } from '@mui/material/styles';

// Create a mock theme
const theme = createTheme();

// Wrapper component to provide theme context
const renderWithTheme = (component: React.ReactElement) => {
  return render(<ThemeProvider theme={theme}>{component}</ThemeProvider>);
};

const mockAlignments: VersionAlignment[] = [
  {
    id: 'serde',
    dependencyName: 'serde',
    currentVersions: { crate1: '1.0.0' },
    suggestedVersion: '1.2.0',
    severity: 'medium',
    affectedPackages: ['crate1'],
  },
  {
    id: 'tokio',
    dependencyName: 'tokio',
    currentVersions: { crate2: '0.2.0', crate3: '0.3.0' },
    suggestedVersion: '1.0.0',
    severity: 'high',
    affectedPackages: ['crate2', 'crate3'],
  },
];

describe('VersionAlignmentList', () => {
  it('renders the list of version alignments', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();

    renderWithTheme(
      <VersionAlignmentList
        alignments={mockAlignments}
        selectedIds={[]}
        loading={false}
        onSelect={() => {}}
        onSelectOne={() => {}}
        onApplyAlignment={onApply}
        onIgnoreAlignment={onIgnore}
      />
    );

    expect(screen.getByText('Version Alignment Issues')).toBeInTheDocument();
    expect(screen.getByText('serde')).toBeInTheDocument();
    expect(screen.getByText('tokio')).toBeInTheDocument();
    expect(screen.getByText('→ 1.2.0')).toBeInTheDocument();
    expect(screen.getByText('→ 1.0.0')).toBeInTheDocument();
  });

  it('shows a message when there are no alignments', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();

    renderWithTheme(
      <VersionAlignmentList
        alignments={[]}
        selectedIds={[]}
        loading={false}
        onSelect={() => {}}
        onSelectOne={() => {}}
        onApplyAlignment={onApply}
        onIgnoreAlignment={onIgnore}
      />
    );

    expect(screen.getByText('No version alignment issues found.')).toBeInTheDocument();
  });

  it('renders the correct number of list items', () => {
    const onApply = vi.fn();
    const onIgnore = vi.fn();

    const { container } = renderWithTheme(
      <VersionAlignmentList
        alignments={mockAlignments}
        selectedIds={[]}
        loading={false}
        onSelect={() => {}}
        onSelectOne={() => {}}
        onApplyAlignment={onApply}
        onIgnoreAlignment={onIgnore}
      />
    );

    const listItems = container.querySelectorAll('li');
    // Should be 2 items + 1 divider
    expect(listItems.length).toBe(3);
  });
});
