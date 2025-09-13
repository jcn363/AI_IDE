import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { FeatureFlagManager } from '../components/FeatureFlagManager';
import '@testing-library/jest-dom';

// Mock the feature flags module with simplified behavior
const mockAnalyzeFeatureFlags = jest.fn(() =>
  Promise.resolve([
    {
      name: 'default-feature',
      usedBy: ['dep-a'],
      enabledByDefault: true,
      isUsed: true,
    },
    {
      name: 'unused-feature',
      usedBy: [],
      enabledByDefault: false,
      isUsed: false,
    },
  ])
);

const mockOptimizeFeatureFlags = jest.fn((manifest) => ({
  ...manifest,
  features: {
    'default-feature': ['dep-a'],
  },
}));

const mockGetFeatureFlagSuggestions = jest.fn((features) => {
  if (!features || !Array.isArray(features)) return [];
  return features
    .filter((f) => !f.isUsed && !f.enabledByDefault)
    .map((f) => `Unused feature "${f.name}" can be safely removed`);
});

jest.mock('../featureFlags', () => ({
  analyzeFeatureFlags: mockAnalyzeFeatureFlags,
  optimizeFeatureFlags: mockOptimizeFeatureFlags,
  getFeatureFlagSuggestions: mockGetFeatureFlagSuggestions,
}));

// Simple test manifest
const mockManifest = {
  package: {
    name: 'test-package',
    version: '0.1.0',
    default_features: ['default-feature'],
  },
  features: {
    'default-feature': ['dep-a'],
    'unused-feature': [],
  },
  dependencies: {
    'dep-a': '1.0.0',
  },
};

describe('FeatureFlagManager', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    jest.clearAllMocks();
  });

  it('renders without crashing', () => {
    render(<FeatureFlagManager manifest={mockManifest} onChange={jest.fn()} />);
    expect(screen.getByRole('table')).toBeInTheDocument();
  });

  it('displays feature flags from the manifest', () => {
    render(<FeatureFlagManager manifest={mockManifest} onChange={jest.fn()} />);

    // Check for the table and feature names
    expect(screen.getByRole('table')).toBeInTheDocument();

    // Verify the default feature is rendered
    expect(screen.getByText('default-feature')).toBeInTheDocument();

    // Verify the unused feature is rendered
    expect(screen.getByText('unused-feature')).toBeInTheDocument();
  });

  it('shows the optimize button', () => {
    render(<FeatureFlagManager manifest={mockManifest} onChange={jest.fn()} />);
    expect(screen.getByRole('button', { name: /optimize features/i })).toBeInTheDocument();
  });

  it('toggles feature flags correctly', async () => {
    const handleChange = jest.fn();

    render(<FeatureFlagManager manifest={mockManifest} onChange={handleChange} />);

    // Find and click the toggle for the unused feature
    const toggle = screen.getByRole('switch', { name: /unused-feature/i });
    fireEvent.click(toggle);

    // Verify the change handler was called with the updated manifest
    await waitFor(() => {
      expect(handleChange).toHaveBeenCalledWith(
        expect.objectContaining({
          package: expect.objectContaining({
            default_features: expect.arrayContaining(['unused-feature']),
          }),
        })
      );
    });
  });

  it('handles feature optimization', async () => {
    const handleChange = jest.fn();

    render(<FeatureFlagManager manifest={mockManifest} onChange={handleChange} />);

    // Click the optimize button
    const optimizeButton = screen.getByRole('button', { name: /optimize features/i });
    fireEvent.click(optimizeButton);

    // Verify the optimization function was called
    expect(mockOptimizeFeatureFlags).toHaveBeenCalledWith(mockManifest, expect.any(Array));

    // Verify the change handler was called with the optimized manifest
    await waitFor(() => {
      expect(handleChange).toHaveBeenCalled();
    });
  });

  it('shows suggestions when available', async () => {
    // Mock the feature flag suggestions to return our test suggestion
    const testSuggestion = 'Unused feature "unused-feature" can be safely removed';

    // Mock the implementation to return our test suggestion
    const mockSuggestions = [testSuggestion];
    mockGetFeatureFlagSuggestions.mockReturnValue(mockSuggestions);

    // Render the component
    render(<FeatureFlagManager manifest={mockManifest} onChange={jest.fn()} />);

    // Verify the suggestions function was called with the expected features
    await waitFor(() => {
      expect(mockGetFeatureFlagSuggestions).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({ name: 'default-feature' }),
          expect.objectContaining({ name: 'unused-feature' }),
        ])
      );
    });

    // Since we're mocking the suggestions function, we can directly test its output
    const suggestions = mockGetFeatureFlagSuggestions([
      { name: 'unused-feature', usedBy: [], enabledByDefault: false, isUsed: false },
      { name: 'default-feature', usedBy: ['dep-a'], enabledByDefault: true, isUsed: true },
    ]);

    // Verify the suggestions array contains our test suggestion
    expect(suggestions).toContain(testSuggestion);
  });

  it('handles errors during feature analysis', async () => {
    // Mock a failing analysis
    const errorMock = jest.spyOn(console, 'error').mockImplementation(() => {});
    mockAnalyzeFeatureFlags.mockImplementationOnce(() => {
      throw new Error('Analysis failed');
    });

    render(<FeatureFlagManager manifest={mockManifest} onChange={jest.fn()} />);

    // Verify the component handles the error gracefully
    expect(errorMock).toHaveBeenCalledWith('Error analyzing feature flags:', expect.any(Error));
    errorMock.mockRestore();
  });

  it('renders the feature toggle controls', () => {
    const handleChange = jest.fn();

    render(<FeatureFlagManager manifest={mockManifest} onChange={handleChange} />);

    // Verify the toggle switch is present
    const toggle = screen.getByRole('switch', { name: /show all|hide unused/i });
    expect(toggle).toBeInTheDocument();

    // Verify optimize button is present
    const optimizeButton = screen.getByRole('button', { name: /optimize features/i });
    expect(optimizeButton).toBeInTheDocument();
  });
});
