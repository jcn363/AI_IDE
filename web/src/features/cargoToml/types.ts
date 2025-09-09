export type DependencyType = 'crate' | 'feature' | 'workspace';
export type LinkType = 'depends' | 'feature' | 'optional' | 'default';

export interface BaseDependencyNode {
  id: string;
  name: string;
  version: string;
  type: DependencyType;
  isRoot: boolean;
  isDirect: boolean;
  features?: Array<{ name: string; requires?: string[] }>;
}

export interface CrateDependencyNode extends BaseDependencyNode {
  type: 'crate';
  features: Array<{ name: string; requires?: string[] }>;
}

export interface FeatureDependencyNode extends BaseDependencyNode {
  type: 'feature';
  parentId: string;
}

export interface WorkspaceDependencyNode extends BaseDependencyNode {
  type: 'workspace';
  members: string[];
}

export type DependencyNode = CrateDependencyNode | FeatureDependencyNode | WorkspaceDependencyNode;

export interface DependencyLink {
  source: string;
  target: string;
  type: LinkType;
  color?: string;
  width?: number;
  label?: string;
}

export interface DependencyGraphData {
  nodes: DependencyNode[];
  links: DependencyLink[];
  crates: CrateDependencyNode[];
  features: FeatureDependencyNode[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
  manifest?: any; // TODO: Define proper Cargo.toml manifest type
}

export interface DependencyGraphVisualizationProps {
  projectPath: string;
  width?: number | string;
  height?: number | string;
  className?: string;
  onNodeClick?: (node: DependencyNode) => void;
  onLinkClick?: (link: DependencyLink) => void;
  onGraphLoad?: (data: DependencyGraphData) => void;
  showControls?: boolean;
  showLegend?: boolean;
  showToolbar?: boolean;
  showNodeLabels?: boolean;
  showLinkLabels?: boolean;
  showFeatureNodes?: boolean;
  showWorkspaceNodes?: boolean;
  showOptionalDependencies?: boolean;
  showDevDependencies?: boolean;
  showBuildDependencies?: boolean;
  showTransitiveDependencies?: boolean;
  theme?: 'light' | 'dark' | 'system';
  nodeSize?: number;
  linkWidth?: number;
  nodeColor?: (node: DependencyNode) => string;
  linkColor?: (link: DependencyLink) => string;
  nodeLabel?: (node: DependencyNode) => string;
  linkLabel?: (link: DependencyLink) => string;
  nodeTooltip?: (node: DependencyNode) => React.ReactNode;
  linkTooltip?: (link: DependencyLink) => React.ReactNode;
  onNodeMouseEnter?: (node: DependencyNode) => void;
  onNodeMouseLeave?: (node: DependencyNode) => void;
  onLinkMouseEnter?: (link: DependencyLink) => void;
  onLinkMouseLeave?: (link: DependencyLink) => void;
  onZoom?: (transform: d3.ZoomTransform) => void;
  onZoomStart?: () => void;
  onZoomEnd?: () => void;
  onDragStart?: (node: DependencyNode) => void;
  onDrag?: (node: DependencyNode) => void;
  onDragEnd?: (node: DependencyNode) => void;
  onSimulationTick?: () => void;
  onSimulationEnd?: () => void;
  simulationAlphaMin?: number;
  simulationAlphaDecay?: number;
  simulationAlphaTarget?: number;
  simulationVelocityDecay?: number;
  simulationForces?: {
    centerX?: number;
    centerY?: number;
    charge?: number;
    collide?: number;
    linkDistance?: number;
    linkStrength?: number;
  };
}
