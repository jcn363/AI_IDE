import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import * as d3 from 'd3';
import { invoke } from '@tauri-apps/api/core';
import { Button, Empty, Space, Spin, message } from 'antd';
import { 
  DownloadOutlined, 
  ReloadOutlined, 
} from '@ant-design/icons';
import { type DependencyNode } from './dependencyGraph';
import './EnhancedDependencyGraph.css';

// Type definitions for the dependency graph
type DependencyType = 'all' | 'normal' | 'dev' | 'build' | 'workspace' | 'transitive' | 'feature';

// Extended node type for visualization
export interface GraphNode extends d3.SimulationNodeDatum, Omit<DependencyNode, 'type' | 'id'> {
  id: string;
  type: 'crate' | 'feature' | 'workspace' | 'dev' | 'build' | 'transitive';
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  size?: number;
  color?: string;
  label?: string;
}

// Extended link type for visualization
export interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
  type: 'depends' | 'feature' | 'workspace' | 'optional' | 'default' | 'requires';
  label?: string;
  optional?: boolean;
  required?: boolean;
  feature?: string;
  color?: string;
  width?: number;
}

export interface DependencyGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
  crates: DependencyNode[];
  features: DependencyNode[];
  rootNode?: string;
  timestamp?: string;
  cargoVersion?: string;
}

interface DependencyGraphFilters {
  searchTerm: string;
  dependencyType: DependencyType;
  showFeatures: boolean;
  showDevDeps: boolean;
  showBuildDeps: boolean;
  showOptionalDeps: boolean;
  showTransitiveDeps: boolean;
  showFeatureDeps: boolean;
  showTransitive: boolean;
  showOptional?: boolean;
  showDev?: boolean;
  showWorkspace?: boolean;
  showBuild?: boolean;
  layout?: string;
  nodeSize?: {
    min: number;
    max: number;
    scale: number;
  };
  linkDistance?: {
    min: number;
    max: number;
  };
  charge?: number;
  collision?: number;
}

interface EnhancedDependencyGraphProps {
  projectPath: string;
  width?: string | number;
  height?: string | number;
  showControls?: boolean;
  className?: string;
}

const DEFAULT_FILTERS: DependencyGraphFilters = {
  searchTerm: '',
  dependencyType: 'all',
  showFeatures: true,
  showDevDeps: true,
  showBuildDeps: true,
  showOptionalDeps: true,
  showTransitiveDeps: true,
  showFeatureDeps: true,
  showTransitive: true,
  showOptional: true,
  showDev: true,
  showWorkspace: true,
  showBuild: true,
  layout: 'force',
  nodeSize: {
    min: 5,
    max: 30,
    scale: 1,
  },
  linkDistance: {
    min: 50,
    max: 200,
  },
  charge: -300,
  collision: 50,
};

const EnhancedDependencyGraph: React.FC<EnhancedDependencyGraphProps> = ({
  projectPath,
  width = '100%',
  height = '600px',
  showControls = true,
  className = '',
}) => {
  const [graphData, setGraphData] = useState<DependencyGraphData | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<DependencyGraphFilters>(DEFAULT_FILTERS);
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const simulationRef = useRef<d3.Simulation<GraphNode, GraphLink> | null>(null);

  // Fetch graph data from backend
  const fetchGraphData = useCallback(async () => {
    if (!projectPath) return;
    
    setIsLoading(true);
    setError(null);
    
    try {
      const data = await invoke<DependencyGraphData>('get_dependency_graph', {
        projectPath,
      });
      
      setGraphData(data);
    } catch (err) {
      console.error('Failed to fetch dependency graph:', err);
      setError('Failed to load dependency graph. Please try again.');
      message.error('Failed to load dependency graph');
    } finally {
      setIsLoading(false);
    }
  }, [projectPath]);

  // Initial data fetch
  useEffect(() => {
    fetchGraphData();
  }, [fetchGraphData]);

  // Handle refresh
  const handleRefresh = useCallback(() => {
    fetchGraphData();
  }, [fetchGraphData]);

  // Handle download
  const handleDownload = useCallback(() => {
    if (!svgRef.current) return;
    
    // Use type assertion for XMLSerializer
    const serializer = new (window as any).XMLSerializer();
    const svgData = serializer.serializeToString(svgRef.current);
    const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
    const svgUrl = (window.URL || window.webkitURL).createObjectURL(svgBlob);
    
    const downloadLink = document.createElement('a');
    // Use setAttribute for better cross-browser compatibility
    downloadLink.setAttribute('href', svgUrl);
    downloadLink.setAttribute('download', 'dependency-graph.svg');
    
    // Make the link invisible
    downloadLink.style.display = 'none';
    document.body.appendChild(downloadLink);
    
    // Trigger the download
    downloadLink.click();
    
    // Clean up
    setTimeout(() => {
      document.body.removeChild(downloadLink);
      (window.URL || window.webkitURL).revokeObjectURL(svgUrl);
    }, 100);
  }, []);

  // Render the graph using D3
  useEffect(() => {
    if (!graphData || !svgRef.current || !containerRef.current) return;
    
    const container = containerRef.current;
    const rect = container.getBoundingClientRect();
    const width = rect.width || 800;
    const height = rect.height || 600;
    
    // Clear previous graph
    d3.select(svgRef.current).selectAll('*').remove();
    
    // Set up the SVG
    const svg = d3.select(svgRef.current)
      .attr('width', '100%')
      .attr('height', '100%')
      .attr('viewBox', `0 0 ${width} ${height}`);
    
    // Add zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 8])
      .on('zoom', (event) => {
        svg.select('.graph-content').attr('transform', event.transform);
      });
    
    svg.call(zoom as any);
    
    // Create a group for the graph content
    const g = svg.append('g').attr('class', 'graph-content');
    
    // Create the force simulation
    const simulation = d3.forceSimulation<GraphNode>()
      .force('link', d3.forceLink<GraphNode, GraphLink>(graphData.links).id(d => d.id).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2));
    
    // Store simulation reference for cleanup
    simulationRef.current = simulation;
    
    // Create links
    const link = g.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(graphData.links)
      .enter().append('line')
      .attr('stroke', '#999')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 1.5);
    
    // Create nodes
    const node = g.append('g')
      .attr('class', 'nodes')
      .selectAll('.node')
      .data(graphData.nodes)
      .enter().append('g')
      .attr('class', 'node')
      .call(d3.drag<SVGGElement, GraphNode>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended) as any);
    
    // Add circles to nodes
    node.append('circle')
      .attr('r', 10)
      .attr('fill', '#69b3a2');
    
    // Add labels to nodes
    node.append('text')
      .attr('dx', 12)
      .attr('dy', '.35em')
      .text(d => d.name);
    
    // Update positions on simulation tick
    simulation.nodes(graphData.nodes).on('tick', () => {
      link
        .attr('x1', d => (d.source as any).x || 0)
        .attr('y1', d => (d.source as any).y || 0)
        .attr('x2', d => (d.target as any).x || 0)
        .attr('y2', d => (d.target as any).y || 0);
      
      node.attr('transform', d => `translate(${d.x},${d.y})`);
    });
    
    // Drag functions
    function dragstarted(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }
    
    function dragged(event: any, d: any) {
      d.fx = event.x;
      d.fy = event.y;
    }
    
    function dragended(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = event.x;
      d.fy = event.y;
    }
    
    // Clean up simulation on unmount
    return () => {
      simulation.stop();
    };
  }, [graphData]);

  return (
    <div className={`enhanced-dependency-graph ${className}`}>
      {showControls && (
        <div className="graph-controls">
          <Space>
            <Button
              type="primary"
              icon={<ReloadOutlined />}
              onClick={handleRefresh}
              loading={isLoading}
            >
              Refresh
            </Button>
            <Button
              icon={<DownloadOutlined />}
              onClick={handleDownload}
              disabled={!graphData}
            >
              Export
            </Button>
          </Space>
        </div>
      )}
      <div
        ref={containerRef}
        className="graph-container"
        data-testid="dependency-graph"
        style={{ width, height }}
      >
        {isLoading ? (
          <div className="graph-loading">
            <Spin size="large" />
            <p>Loading dependency graph...</p>
          </div>
        ) : error ? (
          <div className="graph-error">
            <Empty
              description={
                <>
                  <div>{error}</div>
                  <Button 
                    type="link" 
                    onClick={handleRefresh}
                    icon={<ReloadOutlined />}
                  >
                    Retry
                  </Button>
                </>
              }
            />
          </div>
        ) : graphData ? (
          <svg ref={svgRef} className="graph-svg">
            <defs>
              <marker
                id="arrowhead"
                markerWidth="10"
                markerHeight="7"
                refX="9"
                refY="3.5"
                orient="auto"
              >
                <polygon points="0 0, 10 3.5, 0 7" />
              </marker>
            </defs>
          </svg>
        ) : (
          <Empty description="No data available" />
        )}
      </div>
    </div>
  );
};

export default EnhancedDependencyGraph;
