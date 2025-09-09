import {
  DownloadOutlined,
  EyeInvisibleOutlined,
  EyeOutlined,
  FilterOutlined,
  ReloadOutlined,
  SearchOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { Button, Empty, Space, Spin, Switch, Tooltip } from 'antd';
import * as d3 from 'd3';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { CargoManifest } from '../../types/cargo';
import './EnhancedDependencyGraph.css';
import { useDependencyGraph } from './services/DependencyGraphProcessor';
import { type DependencyNode } from './services/FeatureResolver';

// Extend Window interface to include toml
declare global {
  interface Window {
    toml?: {
      parse: (toml: string) => any;
    };
  }
}

// Type definitions for the dependency graph
type DependencyType = 'all' | 'normal' | 'dev' | 'build' | 'workspace' | 'transitive' | 'feature';

// Extended node type for visualization
export type NodeType = 'crate' | 'feature' | 'workspace' | 'dev' | 'build' | 'transitive';

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

export interface GraphNode extends d3.SimulationNodeDatum, Omit<DependencyNode, 'type' | 'id'> {
  id: string;
  type: NodeType;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  size?: number;
  color?: string;
  label?: string;
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
  layout: string;
  nodeSize: {
    min: number;
    max: number;
    scale: number;
  };
  linkDistance: {
    min: number;
    max: number;
  };
  charge: number;
  collision: number;
}

interface DependencyGraphVisualizationProps {
  projectPath: string;
  width?: string | number;
  height?: string | number;
  showControls?: boolean;
  className?: string;
  manifest?: CargoManifest; // Allow passing manifest directly
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

const DependencyGraphVisualization: React.FC<DependencyGraphVisualizationProps> = ({
  projectPath,
  width = '100%',
  height = '600px',
  showControls = true,
  className = '',
  manifest,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const graphContainerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [graphData, setGraphData] = useState<DependencyGraphData | null>(null);
  const [filters, setFilters] = useState<DependencyGraphFilters>(DEFAULT_FILTERS);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [simulation, setSimulation] = useState<d3.Simulation<GraphNode, GraphLink> | null>(null);
  const [isFilterModalOpen, setIsFilterModalOpen] = useState(false);

  // Helper function to safely access simulation
  const withSimulation = (callback: (sim: d3.Simulation<GraphNode, GraphLink>) => void) => {
    if (simulation) {
      callback(simulation);
    }
  };

  // Handle downloading the SVG
  const handleDownloadSVG = useCallback(() => {
    if (!svgRef.current) return;
    
    // Get the SVG element and its outer HTML
    const svgElement = svgRef.current;
    const serializer = new XMLSerializer();
    // Use type assertion to handle the SVG element
    const svgString = serializer.serializeToString(svgElement as unknown as Node);
    
    // Create a proper SVG document with XML declaration and DOCTYPE
    const svgData = `<?xml version="1.0" standalone="no"?>
      <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" 
      "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
      ${svgString}`;
    
    // Create and trigger download
    const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
    const svgUrl = URL.createObjectURL(svgBlob);
    
    const downloadLink = document.createElement('a') as HTMLAnchorElement;
    downloadLink.href = svgUrl;
    downloadLink.download = 'dependency-graph.svg';
    document.body.appendChild(downloadLink);
    downloadLink.click();
    document.body.removeChild(downloadLink);
    URL.revokeObjectURL(svgUrl);
  }, []);
  
  // Helper to safely access window.location
  const reloadPage = useCallback(() => {
    if (typeof window !== 'undefined' && 'location' in globalThis) {
      const win = window as unknown as { location: { reload: () => void } };
      win.location.reload();
    }
  }, []);

  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [manifestState, setManifestState] = useState<CargoManifest | null>(manifest || null);
  const [isLoadingManifest, setIsLoadingManifest] = useState(!manifest);
  const [manifestError, setManifestError] = useState<string | null>(null);

  // Fetch and parse Cargo.toml if manifest is not provided
  useEffect(() => {
    if (manifest) {
      setManifestState(manifest);
      setIsLoadingManifest(false);
      return;
    }
    
    const loadManifest = async () => {
      try {
        setIsLoadingManifest(true);
        setManifestError(null);
        
        const manifestPath = `${projectPath}/Cargo.toml`;
        const tomlContent = await invoke<string>('read_file', { path: manifestPath });
        
        // Use the TOML parser if available, otherwise try to parse as JSON
        let parsedManifest: CargoManifest;
        if (typeof window.toml !== 'undefined') {
          // @ts-ignore - Using TOML parser if available
          parsedManifest = window.toml.parse(tomlContent);
        } else {
          // Fallback to JSON parsing if TOML parser is not available
          parsedManifest = JSON.parse(tomlContent);
        }
        
        setManifestState(parsedManifest);
      } catch (err) {
        console.error('Failed to load Cargo.toml:', err);
        setManifestError('Failed to load Cargo.toml. Please check the file path and try again.');
      } finally {
        setIsLoadingManifest(false);
      }
    };
    
    loadManifest();
  }, [projectPath, manifest]);

  // Load and process the Cargo.toml manifest
  const { nodes, links, crates, features } = useDependencyGraph(manifestState || { package: { name: 'unknown' } }, {
    includeDevDeps: filters.showDevDeps,
    includeBuildDeps: filters.showBuildDeps,
    includeFeatures: filters.showFeatures,
    resolveFeatures: filters.showFeatureDeps,
  });

  // Process graph data when dependencies change
  useEffect(() => {
    if (!nodes.length || isLoadingManifest) return;
    
    try {
      setLoading(true);
      
      // Process nodes for visualization
      const processedNodes: GraphNode[] = nodes.map((node: DependencyNode) => {
        // Safely determine node type and properties
        let nodeType: NodeType = 'crate';
        let size = 8;
        let color = '#1890ff';
        let label = node.name;
        
        // Determine node type and set properties based on type
        if (node.type === 'feature') {
          nodeType = 'feature';
          size = 4;
          color = '#8884d8';
          label = node.name.split('/').pop() || node.name;
        } else if (node.type === 'workspace') {
          nodeType = 'workspace';
          size = 10;
          color = '#82ca9d';
        } else if ('isDev' in node && node.isDev) {
          nodeType = 'dev';
          size = 6;
          color = '#ffc658';
        } else if ('isBuild' in node && node.isBuild) {
          nodeType = 'build';
          size = 8;
          color = '#ff8042';
        } else if ('isTransitive' in node && node.isTransitive) {
          nodeType = 'transitive';
          size = 5;
          color = '#888888';
        }
        
        const isFeature = nodeType === 'feature';
        const isWorkspace = nodeType === 'workspace';
        
        return {
          ...node,
          id: node.id,
          type: nodeType,
          size: size,
          color: color,
          label: label,
        };
      });
      
      // Process links for visualization
      const processedLinks: GraphLink[] = links.map(link => ({
        ...link,
        source: link.source,
        target: link.target,
        type: (link.type as any) || 'depends',
        color: (link as any).color || '#999',
        width: (link as any).width || 1,
        label: (link as any).label || link.type,
        // Set default values for optional GraphLink properties
        optional: (link as any).optional,
        required: (link as any).required,
        feature: (link as any).feature,
        // Add d3 simulation properties
        index: undefined,
      }));
      
      setGraphData({
        nodes: processedNodes,
        links: processedLinks,
        crates: nodes.filter(n => n.type === 'crate'),
        features: nodes.filter(n => n.type === 'feature'),
        rootNode: projectPath,
      });
      
      setLoading(false);
    } catch (err) {
      console.error('Error processing graph data:', err);
      setError('Failed to process dependency graph');
      setLoading(false);
    }
  }, [nodes, links, projectPath]);

  // Initialize and update the D3 force simulation
  useEffect(() => {
    if (!graphData || !svgRef.current) return;
    
    const { width, height } = dimensions;
    if (width === 0 || height === 0) return;
    
    // Clear previous simulation
    if (simulation) {
      simulation.stop();
    }
    
    // Filter nodes and links based on current filters
    const filteredNodes = graphData.nodes.filter(node => {
      if (filters.dependencyType !== 'all' && node.type !== filters.dependencyType) {
        return false;
      }
      if (filters.searchTerm && !node.name.toLowerCase().includes(filters.searchTerm.toLowerCase())) {
        return false;
      }
      if (!filters.showFeatures && node.type === 'feature') return false;
      if (!filters.showDevDeps && node.type === 'dev') return false;
      if (!filters.showBuildDeps && node.type === 'build') return false;
      return !(!filters.showTransitiveDeps && node.type === 'transitive');
    });
    
    const nodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredLinks = graphData.links.filter(
      link => nodeIds.has(link.source as string) && nodeIds.has(link.target as string),
    );
    
    // Create a new simulation
    const newSimulation = d3.forceSimulation<GraphNode>(filteredNodes as any[])
      .force('link', d3.forceLink<GraphNode, GraphLink>(filteredLinks as any[])
        .id((d: any) => d.id)
        .distance(100)
        .strength(0.1),
      )
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(40));
    
    setSimulation(newSimulation);
    
    return () => {
      newSimulation.stop();
    };
  }, [graphData, filters, dimensions]);

  // Render the D3 visualization
  useEffect(() => {
    if (!svgRef.current || !graphData) return;

    const svg = d3.select(svgRef.current);
    const { width, height } = dimensions;

    // Clear previous SVG content
    svg.selectAll('*').remove();

    // Create simulation if it doesn't exist
    if (!simulation) {
      const newSimulation = d3.forceSimulation<GraphNode>(graphData.nodes as any[])
        .force('link', d3.forceLink<GraphNode, GraphLink>(graphData.links as any[])
          .id((d: any) => d.id)
          .distance(100)
          .strength(0.1))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(40));
      
      setSimulation(newSimulation);
      return;
    }

    // Make sure we have a valid simulation before proceeding
    if (!simulation) return;

    // Create a group for zoom/pan
    const g = svg.append('g');
    
    // Add zoom/pan behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });
    
    svg.call(zoom as any);
    
    // Create arrow markers for links
    svg.append('defs').selectAll('marker')
      .data(['end'])
      .enter().append('marker')
      .attr('id', d => d)
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 15)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#999');
    
    // Create links
    const link = g.append('g')
      .selectAll('line')
      .data(graphData.links, (d: any) => `${d.source.id}-${d.target.id}`)
      .enter().append('line')
      .attr('stroke', (d: any) => d.color || '#999')
      .attr('stroke-width', (d: any) => d.width || 1)
      .attr('stroke-opacity', 0.6)
      .attr('marker-end', 'url(#end)');
    
    // Create nodes
    const node = g.append('g')
      .selectAll('circle')
      .data(graphData.nodes, (d: any) => d.id)
      .enter().append('circle')
      .attr('r', (d: any) => d.size || 5)
      .attr('fill', (d: any) => d.color || '#1890ff')
      .attr('stroke', '#fff')
      .attr('stroke-width', 1.5)
      .call(
        d3.drag<SVGCircleElement, GraphNode>()
          .on('start', dragstarted)
          .on('drag', dragged)
          .on('end', dragended) as any,
      )
      .on('mouseover', (event: any, d: any) => {
        setHoveredNode(d);
        d3.select(event.currentTarget).attr('stroke', '#ff4d4f').attr('stroke-width', 3);
      })
      .on('mouseout', (event: any) => {
        setHoveredNode(null);
        d3.select(event.currentTarget).attr('stroke', '#fff').attr('stroke-width', 1.5);
      })
      .on('click', (event: any, d: any) => {
        setSelectedNode(d);
        event.stopPropagation();
      });
    
    // Add labels
    const labels = g.append('g')
      .selectAll('text')
      .data(graphData.nodes, (d: any) => d.id)
      .enter().append('text')
      .text((d: any) => d.label || d.name)
      .attr('font-size', '10px')
      .attr('dx', 12)
      .attr('dy', '.35em')
      .attr('fill', '#333');
    
    // Update positions on each tick
    withSimulation((sim) => {
      sim.on('tick', () => {
        link
          .attr('x1', (d: any) => d.source.x)
          .attr('y1', (d: any) => d.source.y)
          .attr('x2', (d: any) => d.target.x)
          .attr('y2', (d: any) => d.target.y);
        
        node
          .attr('cx', (d: any) => d.x)
          .attr('cy', (d: any) => d.y);
        
        labels
          .attr('x', (d: any) => d.x + 10)
          .attr('y', (d: any) => d.y + 5);
      });
    });
    
    // Drag functions
    function dragstarted(event: any, d: any) {
      withSimulation(sim => {
        if (!event.active) sim.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
      });
    }
    
    function dragged(event: any, d: any) {
      d.fx = event.x;
      d.fy = event.y;
    }
    
    function dragended(event: any, d: any) {
      withSimulation(sim => {
        if (!event.active) sim.alphaTarget(0);
        d.fx = event.x;
        d.fy = event.y;
      });
    }
  }, [simulation, graphData, dimensions]);

  const handleZoomToFit = useCallback(() => {
    if (!svgRef.current || !graphData || !graphContainerRef.current) return;
    
    const svg = d3.select<SVGSVGElement, unknown>(svgRef.current);
    const container = graphContainerRef.current;
    const containerRect = container.getBoundingClientRect();
    
    // Get bounds of all nodes
    const xExtent = d3.extent(graphData.nodes, d => d.x) as [number, number];
    const yExtent = d3.extent(graphData.nodes, d => d.y) as [number, number];
    
    const graphWidth = xExtent[1] - xExtent[0];
    const graphHeight = yExtent[1] - yExtent[0];
    
    if (graphWidth === 0 || graphHeight === 0) return;
    
    // Calculate scale and translation
    const scale = Math.min(
      0.9 * containerRect.width / graphWidth,
      0.9 * containerRect.height / graphHeight,
    );
    
    const dx = (containerRect.width - graphWidth * scale) / 2 - xExtent[0] * scale;
    const dy = (containerRect.height - graphHeight * scale) / 2 - yExtent[0] * scale;
    
    // Apply zoom transform
    const zoom = d3.zoom<SVGSVGElement, unknown>();
    svg.transition()
      .duration(750)
      .call(
        zoom.transform as any,
        d3.zoomIdentity.translate(dx, dy).scale(scale),
      );
  }, [graphData]);

  if (isLoadingManifest) {
    return (
      <div className="loading-container">
        <Spin tip="Loading Cargo.toml..." size="large" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="error-container" style={{ padding: '20px', textAlign: 'center' }}>
        <p style={{ color: 'red', marginBottom: '16px' }}>Error: {error}</p>
        <Button 
          type="primary" 
          icon={<ReloadOutlined />} 
          onClick={reloadPage}
        >
          Retry
        </Button>
      </div>
    );
  }
  
  if (!projectPath) {
    return (
      <div className="error-container" style={{ padding: '20px', textAlign: 'center' }}>
        <p>No project path provided</p>
      </div>
    );
  }

  return (
    <div className={`dependency-graph-container ${className}`}>
      {showControls && (
        <div className="graph-controls">
          <Space>
            <Tooltip title="Reload graph">
              <Button 
                icon={<ReloadOutlined />} 
                onClick={reloadPage}
              />
            </Tooltip>
            
            <Tooltip title="Download as SVG">
              <Button 
                icon={<DownloadOutlined />} 
                onClick={handleDownloadSVG}
              />
            </Tooltip>
            
            <Tooltip title="Zoom to fit">
              <Button 
                icon={<SearchOutlined />} 
                onClick={handleZoomToFit}
              />
            </Tooltip>
            
            <Tooltip title="Toggle features">
              <Switch 
                checked={filters.showFeatures}
                onChange={checked => setFilters(prev => ({ ...prev, showFeatures: checked }))}
                checkedChildren={<EyeOutlined />}
                unCheckedChildren={<EyeInvisibleOutlined />}
              />
            </Tooltip>
            
            <Tooltip title="Filter dependencies">
              <Button
                icon={<FilterOutlined />}
                onClick={() => setIsFilterModalOpen(true)}
              />
            </Tooltip>
          </Space>
        </div>
      )}
      
      <div 
        ref={graphContainerRef}
        className="graph-container" 
        style={{ 
          position: 'relative', 
          width: '100%', 
          height: '100%',
          border: '1px solid #f0f0f0',
          borderRadius: '4px',
          backgroundColor: '#fff',
        }}
      >
        {loading ? (
          <div 
            className="graph-loading" 
            style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center', 
              height: '100%',
              backgroundColor: '#fff',
            }}
          >
            <Spin tip="Loading dependency graph..." />
          </div>
        ) : error ? (
          <div 
            className="graph-error" 
            style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center', 
              height: '100%',
              backgroundColor: '#fff',
            }}
          >
            <Empty description={error} />
          </div>
        ) : (
          <svg
            ref={svgRef}
            width="100%"
            height="100%"
            className="dependency-graph"
          />
        )}
      </div>
      
      {(selectedNode || hoveredNode) && (
        <div className="graph-sidebar" style={{
          position: 'absolute',
          right: '20px',
          top: '60px',
          width: '300px',
          backgroundColor: '#fff',
          boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
          padding: '16px',
          borderRadius: '4px',
          maxHeight: 'calc(100% - 80px)',
          overflow: 'auto',
          zIndex: 10,
        }}>
          <h3 style={{ marginTop: 0 }}>{(hoveredNode || selectedNode)?.name}</h3>
          <pre style={{
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            fontSize: '12px',
            margin: 0,
          }}>
            {JSON.stringify(hoveredNode || selectedNode, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
};

export default DependencyGraphVisualization;
