import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { useDependencyGraph, type CrateInfo } from './services/DependencyGraphProcessor';
import {
  type DependencyNode,
  type DependencyLink,
  type FeatureInfo,
} from './services/FeatureResolver';
import type { CargoManifest } from '../../types/cargo';

/**
 * Props for the DependencyGraphRenderer component
 */
interface DependencyGraphRendererProps {
  manifest: CargoManifest;
  width?: number;
  height?: number;
}

/**
 * Component that renders a dependency graph visualization using D3.js
 *
 * @param manifest - The Cargo manifest containing dependency information
 * @param width - Width of the SVG container (default: 800)
 * @param height - Height of the SVG container (default: 600)
 */
export function DependencyGraphRenderer({
  manifest,
  width = 800,
  height = 600
}: DependencyGraphRendererProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const { nodes, links } = useDependencyGraph(manifest);

  useEffect(() => {
    if (!svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const simulation = d3.forceSimulation()
      .force('link', d3.forceLink().id((d: any) => d.id).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2));

    const link = svg.append('g')
      .selectAll('line')
      .data(links)
      .enter().append('line')
      .attr('stroke', (d: any) => (d.type === 'feature' ? '#ff7f0e' : '#999'))
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 1.5);

    const node = svg.append('g')
      .selectAll('circle')
      .data(nodes)
      .enter().append('circle')
      .attr('r', 5)
      .attr('fill', (d: any) => (d.type === 'feature' ? '#ff7f0e' : '#1f77b4'))
      .call(d3.drag()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended) as any,
      );

    const label = svg.append('g')
      .selectAll('text')
      .data(nodes)
      .enter().append('text')
      .attr('dx', 12)
      .attr('dy', '.35em')
      .text((d: any) => d.id);

    simulation
      .nodes(nodes as any)
      .on('tick', ticked);

    (simulation.force('link') as any).links(links);

    function ticked() {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node
        .attr('cx', (d: any) => d.x = Math.max(5, Math.min(width - 5, d.x)))
        .attr('cy', (d: any) => d.y = Math.max(5, Math.min(height - 5, d.y)));

      label
        .attr('x', (d: any) => d.x + 10)
        .attr('y', (d: any) => d.y);
    }

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
      d.fx = null;
      d.fy = null;
    }

    return () => {
      simulation.stop();
    };
  }, [nodes, links, width, height]);

  return (
    <div style={{ width: '100%', height: '100%', overflow: 'hidden' }}>
      <svg ref={svgRef} width={width} height={height} />
    </div>
  );
}
