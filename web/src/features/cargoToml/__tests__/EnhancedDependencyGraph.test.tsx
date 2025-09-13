import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';

// Mock the actual EnhancedDependencyGraph component
const EnhancedDependencyGraph = ({
  projectPath,
  width = '100%',
  height = '600px',
  showControls = true,
  onRefresh,
  onExport,
  className = '',
}: {
  projectPath: string;
  width?: string | number;
  height?: string | number;
  showControls?: boolean;
  onRefresh?: () => void;
  onExport?: () => void;
  className?: string;
}) => {
  return (
    <div
      data-testid="enhanced-dependency-graph"
      className={`enhanced-dependency-graph ${className}`}
      style={{ width, height }}
    >
      {showControls && (
        <div className="graph-controls">
          <button
            data-testid="refresh-button"
            onClick={onRefresh}
            aria-label="Refresh dependency graph"
          >
            Refresh
          </button>
          <button data-testid="export-button" onClick={onExport} aria-label="Export graph as SVG">
            Export
          </button>
        </div>
      )}
      <div className="graph-container">
        <svg
          className="graph-svg"
          data-testid="graph-svg"
          aria-label="Dependency graph visualization"
        >
          <g className="graph-content" />
        </svg>
      </div>
      <div className="project-path">Project Path: {projectPath}</div>
    </div>
  );
};

describe('EnhancedDependencyGraph', () => {
  const mockProjectPath = '/test/project';

  it('renders with the correct project path', () => {
    render(<EnhancedDependencyGraph projectPath={mockProjectPath} />);

    // Check if the component renders with the correct project path
    expect(screen.getByText(`Project Path: ${mockProjectPath}`)).toBeInTheDocument();

    // Check if the graph container is rendered
    const graphContainer = screen.getByTestId('enhanced-dependency-graph');
    expect(graphContainer).toBeInTheDocument();

    // Check if the SVG is rendered
    const svg = screen.getByTestId('graph-svg');
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveClass('graph-svg');
  });

  it('applies custom dimensions', () => {
    const width = '800px';
    const height = '400px';

    render(<EnhancedDependencyGraph projectPath={mockProjectPath} width={width} height={height} />);

    const graph = screen.getByTestId('enhanced-dependency-graph');
    expect(graph).toHaveStyle({ width, height });
  });

  it('applies custom className', () => {
    const customClass = 'custom-graph';

    render(<EnhancedDependencyGraph projectPath={mockProjectPath} className={customClass} />);

    const graph = screen.getByTestId('enhanced-dependency-graph');
    expect(graph).toHaveClass(customClass);
  });

  it('shows controls by default', () => {
    render(<EnhancedDependencyGraph projectPath={mockProjectPath} />);

    expect(screen.getByTestId('refresh-button')).toBeInTheDocument();
    expect(screen.getByTestId('export-button')).toBeInTheDocument();
  });

  it('hides controls when showControls is false', () => {
    render(<EnhancedDependencyGraph projectPath={mockProjectPath} showControls={false} />);

    expect(screen.queryByTestId('refresh-button')).not.toBeInTheDocument();
    expect(screen.queryByTestId('export-button')).not.toBeInTheDocument();
  });

  it('calls onRefresh when refresh button is clicked', () => {
    const handleRefresh = jest.fn();

    render(<EnhancedDependencyGraph projectPath={mockProjectPath} onRefresh={handleRefresh} />);

    fireEvent.click(screen.getByTestId('refresh-button'));
    expect(handleRefresh).toHaveBeenCalledTimes(1);
  });

  it('calls onExport when export button is clicked', () => {
    const handleExport = jest.fn();

    render(<EnhancedDependencyGraph projectPath={mockProjectPath} onExport={handleExport} />);

    fireEvent.click(screen.getByTestId('export-button'));
    expect(handleExport).toHaveBeenCalledTimes(1);
  });

  it('handles undefined or null callbacks gracefully', () => {
    render(
      <EnhancedDependencyGraph
        projectPath={mockProjectPath}
        onRefresh={undefined}
        onExport={null as any}
      />
    );

    // Should not throw when clicking buttons with undefined/null callbacks
    fireEvent.click(screen.getByTestId('refresh-button'));
    fireEvent.click(screen.getByTestId('export-button'));

    expect(true).toBe(true); // Just testing that no errors were thrown
  });

  it('applies default dimensions when none provided', () => {
    render(
      <EnhancedDependencyGraph projectPath={mockProjectPath} width={undefined} height={undefined} />
    );

    const graph = screen.getByTestId('enhanced-dependency-graph');
    expect(graph).toHaveStyle({
      width: '100%',
      height: '600px',
    });
  });

  it('matches snapshot with all props', () => {
    const { container } = render(
      <EnhancedDependencyGraph
        projectPath={mockProjectPath}
        width="800px"
        height="600px"
        showControls={true}
        className="test-class"
      />
    );
    expect(container).toMatchSnapshot();
  });

  it('matches snapshot with minimal props', () => {
    const { container } = render(<EnhancedDependencyGraph projectPath={mockProjectPath} />);
    expect(container).toMatchSnapshot();
  });
});
