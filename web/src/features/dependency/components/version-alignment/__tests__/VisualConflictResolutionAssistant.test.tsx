import { describe, expect, it, vi } from 'vitest';
import React from 'react';
import { fireEvent, render, screen } from '../../../../../test/test-utils';
import { VisualConflictResolutionAssistant } from '../VisualConflictResolutionAssistant';
import { VersionAlignment } from '../../../types';

describe('VisualConflictResolutionAssistant', () => {
  const mockAlignment: VersionAlignment = {
    id: 'test-id',
    dependencyName: 'test-dependency',
    currentVersions: {
      'package-1': '1.0.0',
      'package-2': '2.0.0',
    },
    suggestedVersion: '2.0.0',
    severity: 'high',
    affectedPackages: ['package-1', 'package-2'],
  };

  it('renders correctly with version options', () => {
    const onResolve = vi.fn();
    const onCancel = vi.fn();

    render(
      <VisualConflictResolutionAssistant
        alignment={mockAlignment}
        onResolve={onResolve}
        onCancel={onCancel}
      />,
    );

    expect(screen.getByText('Resolve Version Conflict: test-dependency')).toBeInTheDocument();
    expect(screen.getByText('package-1')).toBeInTheDocument();
    expect(screen.getByText('Version: 1.0.0')).toBeInTheDocument();
    expect(screen.getByText('package-2')).toBeInTheDocument();
    expect(screen.getByText('Version: 2.0.0')).toBeInTheDocument();
    expect(screen.getByText('Cancel')).toBeInTheDocument();
    expect(screen.getByText('Apply Resolution')).toBeInTheDocument();
  });

  it('calls onResolve with suggested version when apply is clicked', () => {
    const onResolve = vi.fn();
    const onCancel = vi.fn();

    render(
      <VisualConflictResolutionAssistant
        alignment={mockAlignment}
        onResolve={onResolve}
        onCancel={onCancel}
      />,
    );
    
    // Click apply - should use suggested version (2.0.0) by default
    fireEvent.click(screen.getByText('Apply Resolution'));
    
    expect(onResolve).toHaveBeenCalledWith({
      selectedVersion: '2.0.0',
    });
  });
  
  it('calls onResolve with suggested version when no version is explicitly selected', () => {
    const onResolve = vi.fn();
    const onCancel = vi.fn();

    render(
      <VisualConflictResolutionAssistant
        alignment={mockAlignment}
        onResolve={onResolve}
        onCancel={onCancel}
      />,
    );
    
    // Click apply without explicitly selecting a version
    fireEvent.click(screen.getByText('Apply Resolution'));
    
    // Should use the suggested version (2.0.0) by default
    expect(onResolve).toHaveBeenCalledWith({
      selectedVersion: '2.0.0',
    });
  });

  it('calls onCancel when cancel button is clicked', () => {
    const onResolve = vi.fn();
    const onCancel = vi.fn();

    render(
      <VisualConflictResolutionAssistant
        alignment={mockAlignment}
        onResolve={onResolve}
        onCancel={onCancel}
      />,
    );

    fireEvent.click(screen.getByText('Cancel'));
    expect(onCancel).toHaveBeenCalled();
  });
});
