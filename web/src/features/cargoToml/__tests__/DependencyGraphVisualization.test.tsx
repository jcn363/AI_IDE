import React from 'react';
import { act, render, screen, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import DependencyGraphVisualization from '../DependencyGraphVisualization';
import * as dependencyGraph from '../dependencyGraph';
import type {
  CrateDependencyNode,
  DependencyGraphData,
  DependencyLink,
  FeatureDependencyNode,
  WorkspaceDependencyNode,
} from '../types';

// Mock the useDependencyGraph hook
jest.mock('../dependencyGraph', () => ({
  useDependencyGraph: jest.fn(),
}));

// Mock the useDependencyGraph hook
const mockUseDependencyGraph = jest.spyOn(dependencyGraph, 'useDependencyGraph') as jest.Mock;

// Create mock data with proper types
const createMockCrate = (name: string, version: string, isRoot = false, isDirect = false): CrateDependencyNode => ({
  id: `${name}@${version}`,
  name,
  version,
  type: 'crate',
  isRoot,
  isDirect,
  features: [],
});

const createMockFeature = (name: string, version: string, isRoot = false, isDirect = false): FeatureDependencyNode => ({
  id: `${name}@${version}`,
  name,
  version,
  type: 'feature',
  isRoot,
  isDirect,
  parentId: 'parent',
});

const createMockWorkspace = (name: string, version: string, isRoot = false, isDirect = false): WorkspaceDependencyNode => ({
  id: `${name}@${version}`,
  name,
  version,
  type: 'workspace',
  isRoot,
  isDirect,
  members: [],
});

// Create mock links with proper typing
const createMockLink = (source: string, target: string, type: 'depends' | 'feature' | 'optional' | 'default' = 'depends'): DependencyLink => ({
  source,
  target,
  type,
});

// Type alias for test data
type TestNode = CrateDependencyNode | FeatureDependencyNode | WorkspaceDependencyNode;

const mockNodes: TestNode[] = [
  createMockCrate('test-crate', '1.0.0', true, true),
  createMockCrate('dep-crate', '1.0.0', false, true),
];

// Create a mock Cargo manifest for testing
const mockManifest = {
  package: {
    name: 'test-crate',
    version: '1.0.0',
  },
  dependencies: {
    'dep-crate': '1.0.0',
  },
};

const mockLinks: DependencyLink[] = [
  createMockLink('test-crate@1.0.0', 'dep-crate@1.0.0', 'depends'),
];

// Default mock implementation
const defaultMockData: DependencyGraphData = {
  nodes: [...mockNodes],
  links: [...mockLinks],
  crates: mockNodes.filter((node): node is CrateDependencyNode => node.type === 'crate'),
  features: mockNodes.filter((node): node is FeatureDependencyNode => node.type === 'feature'),
  loading: false,
  error: null,
  refresh: jest.fn(),
  manifest: mockManifest,
};

beforeEach(() => {
  // Reset all mocks before each test
  jest.clearAllMocks();
  // Set default mock implementation
  mockUseDependencyGraph.mockImplementation(() => defaultMockData);
});

describe('DependencyGraphVisualization', () => {
  it('should render the graph container with correct dimensions', () => {
    const { container } = render(
      <DependencyGraphVisualization 
        projectPath="/test/path" 
        width={800} 
        height={600} 
      />,
    );
    
    // Use screen.getByRole for better testing-library practices
    const svg = screen.getByRole('graphics-document');
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('width', '800');
    expect(svg).toHaveAttribute('height', '600');
  });

  it('displays loading state when loading', async () => {
    // Override the mock for this test
    (dependencyGraph.useDependencyGraph as jest.Mock).mockImplementation(() => ({
      ...defaultMockData,
      loading: true,
    }));

    await act(async () => {
      render(
        <DependencyGraphVisualization 
          projectPath="/test/path"
          width={800}
          height={600}
        />,
      );
    });
    
    // Check for loading indicator using testing-library queries
    const loadingIndicator = screen.getByRole('status');
    expect(loadingIndicator).toBeInTheDocument();
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('displays error message when there is an error', async () => {
    // Override the mock for this test
    const errorMessage = 'Failed to load dependencies';
    (dependencyGraph.useDependencyGraph as jest.Mock).mockImplementation(() => ({
      nodes: [],
      links: [],
      crates: [],
      features: [],
      workspaces: [],
      loading: false,
      error: errorMessage,
      refresh: jest.fn(),
      manifest: mockManifest,
    }));

    await act(async () => {
      render(
        <DependencyGraphVisualization 
          projectPath="/test/path"
          width={800}
          height={600}
        />,
      );
    });
    
    // Check for error message in the component's error display
    const errorElement = screen.getByText(errorMessage);
    expect(errorElement).toBeInTheDocument();
  });

  it('renders with controls when showControls is true', async () => {
    await act(async () => {
      render(
        <DependencyGraphVisualization 
          projectPath="/test/path"
          width={800}
          height={600}
          showControls={true}
        />,
      );
    });
    
    // Check if controls are rendered using testing-library queries
    const controls = screen.getByRole('button', { name: /reload/i });
    expect(controls).toBeInTheDocument();
  });
});
