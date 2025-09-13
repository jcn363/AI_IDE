import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { WorkspaceAnalysis } from './workspaceAnalyzer';

interface Link {
  source: string;
  target: string;
  type: 'inheritance' | 'dependency';
}

interface WorkspaceInheritanceGraphProps {
  analysis: WorkspaceAnalysis;
  width?: number;
  height?: number;
}

const WorkspaceInheritanceGraph: React.FC<WorkspaceInheritanceGraphProps> = ({
  analysis,
  width = 800,
  height = 600,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !analysis) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    // Set up the simulation
    const simulation = d3
      .forceSimulation()
      .force(
        'link',
        d3.forceLink().id((d: any) => d.id)
      )
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(60));

    // Prepare nodes and links
    const nodes = [
      { id: 'workspace', type: 'workspace', name: 'Workspace' },
      ...analysis.members.map((member) => ({
        id: member.name,
        type: 'member',
        name: member.name,
      })),
    ];

    const links: Link[] = [];

    // Add workspace inheritance links
    analysis.members.forEach((member) => {
      if (member.manifest.workspace) {
        links.push({
          source: 'workspace',
          target: member.name,
          type: 'inheritance',
        });
      }
    });

    // Add member-to-member dependency links
    analysis.members.forEach((member) => {
      Object.keys(member.directDependencies).forEach((depName) => {
        if (analysis.workspaceDependencies[depName]) {
          // Skip workspace dependencies already shown
          return;
        }
        const depMember = analysis.members.find((m) => m.name === depName);
        if (depMember) {
          links.push({
            source: member.name,
            target: depName,
            type: 'dependency',
          });
        }
      });
    });

    // Create the links
    const link = svg
      .append('g')
      .selectAll('line')
      .data(links)
      .enter()
      .append('line')
      .attr('stroke', (d: any) => (d.type === 'inheritance' ? '#4CAF50' : '#2196F3'))
      .attr('stroke-width', (d: any) => (d.type === 'inheritance' ? 2 : 1))
      .attr('stroke-opacity', 0.6);

    // Create the nodes
    const node = svg
      .append('g')
      .selectAll('g')
      .data(nodes)
      .enter()
      .append('g')
      .call(d3.drag().on('start', dragstarted).on('drag', dragged).on('end', dragended) as any);

    // Add circles for nodes
    node
      .append('circle')
      .attr('r', 20)
      .attr('fill', (d: any) => (d.type === 'workspace' ? '#9C27B0' : '#3F51B5'));

    // Add text labels
    node
      .append('text')
      .text((d: any) => d.name)
      .attr('text-anchor', 'middle')
      .attr('dy', 40)
      .attr('fill', '#333')
      .style('font-size', '12px')
      .style('pointer-events', 'none');

    // Add workspace icon
    node
      .filter((d: any) => d.type === 'workspace')
      .append('text')
      .text('⚙️')
      .attr('text-anchor', 'middle')
      .attr('dy', 5)
      .style('font-size', '16px')
      .style('pointer-events', 'none');

    // Add member icons
    node
      .filter((d: any) => d.type === 'member')
      .append('text')
      .text('📦')
      .attr('text-anchor', 'middle')
      .attr('dy', 5)
      .style('font-size', '16px')
      .style('pointer-events', 'none');

    // Add tooltips
    const tooltip = d3
      .select('body')
      .append('div')
      .style('position', 'absolute')
      .style('padding', '8px')
      .style('background', 'rgba(0, 0, 0, 0.8)')
      .style('color', 'white')
      .style('border-radius', '4px')
      .style('pointer-events', 'none')
      .style('opacity', 0);

    node
      .on('mouseover', (event: any, d: any) => {
        let html = `<strong>${d.name}</strong>`;

        if (d.type === 'member') {
          const member = analysis.members.find((m) => m.name === d.name);
          if (member) {
            html += '<br/><br/><strong>Direct Dependencies:</strong>';
            Object.entries(member.directDependencies).forEach(([name, version]) => {
              html += `<br/>${name} = "${version}"`;
            });

            if (Object.keys(member.inheritedDependencies).length > 0) {
              html += '<br/><br/><strong>Inherited Dependencies:</strong>';
              Object.entries(member.inheritedDependencies).forEach(([name, version]) => {
                html += `<br/>${name} = "${version}"`;
              });
            }
          }
        } else if (d.type === 'workspace') {
          html += '<br/><br/><strong>Workspace Dependencies:</strong>';
          Object.entries(analysis.workspaceDependencies).forEach(([name, version]) => {
            html += `<br/>${name} = "${version}"`;
          });
        }

        tooltip
          .html(html)
          .style('opacity', 1)
          .style('left', `${event.pageX || event.clientX + 10}px`)
          .style('top', `${event.pageY || event.clientY + 10}px`);
      })
      .on('mousemove', (event: any) => {
        const [x, y] = d3.pointer(event);
        tooltip.style('left', `${x + 10}px`).style('top', `${y + 10}px`);
      })
      .on('mouseout', () => {
        tooltip.style('opacity', 0);
      });

    // Update simulation with nodes and links
    simulation.nodes(nodes as any).on('tick', ticked);
    (simulation.force('link') as any).links(links);

    function ticked() {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
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
      tooltip.remove();
    };
  }, [analysis, width, height]);

  return (
    <div className="relative">
      <div className="absolute top-2 left-2 bg-white bg-opacity-80 p-2 rounded text-xs">
        <div className="flex items-center mb-1">
          <div className="w-3 h-3 bg-purple-600 rounded-full mr-1"></div>
          <span>Workspace</span>
        </div>
        <div className="flex items-center mb-1">
          <div className="w-3 h-3 bg-blue-600 rounded-full mr-1"></div>
          <span>Member</span>
        </div>
        <div className="flex items-center">
          <div className="w-4 h-0.5 bg-green-500 mr-1"></div>
          <span>Inheritance</span>
        </div>
        <div className="flex items-center">
          <div className="w-4 h-0.5 bg-blue-500 mr-1"></div>
          <span>Dependency</span>
        </div>
      </div>
      <svg ref={svgRef} width={width} height={height} className="border rounded-lg bg-white"></svg>
    </div>
  );
};

export default WorkspaceInheritanceGraph;
