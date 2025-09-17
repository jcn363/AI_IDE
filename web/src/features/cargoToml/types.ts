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
  manifest?: {
    package?: {
      name: string;
      version: string;
      edition?: string;
      authors?: string[];
      description?: string;
      license?: string;
      repository?: string;
      documentation?: string;
      readme?: string;
      homepage?: string;
      keywords?: string[];
      categories?: string[];
      publish?: boolean | string[];
      default_run?: string;
      autobins?: boolean;
      autoexamples?: boolean;
      autotests?: boolean;
      autobenches?: boolean;
      resolver?: string;
      rust_version?: string;
      exclude?: string[];
      include?: string[];
      workspace?: string;
      build?: string;
      links?: string;
    };
    dependencies?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
    'dev-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
    'build-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
    features?: Record<string, string[]>;
    target?: Record<string, {
      dependencies?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      'dev-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      'build-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      features?: Record<string, string[]>;
    }>;
    workspace?: {
      members?: string[];
      exclude?: string[];
      default_members?: string[];
      package?: {
        version?: string;
        authors?: string[];
        description?: string;
        homepage?: string;
        documentation?: string;
        readme?: string;
        keywords?: string[];
        categories?: string[];
        license?: string;
        repository?: string;
        edition?: string;
        rust_version?: string;
      };
      metadata?: Record<string, unknown>;
      dependencies?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      'dev-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      'build-dependencies'?: Record<string, string | { version?: string; path?: string; features?: string[]; optional?: boolean; default_features?: boolean }>;
      features?: Record<string, string[]>;
    };
    lib?: {
      name?: string;
      path?: string;
      crate_type?: string | string[];
      test?: boolean;
      doctest?: boolean;
      bench?: boolean;
      doc?: boolean;
      plugin?: boolean;
      proc_macro?: boolean;
      harness?: boolean;
      required_features?: string[];
    };
    bin?: Array<{
      name: string;
      path?: string;
      test?: boolean;
      doctest?: boolean;
      bench?: boolean;
      doc?: boolean;
      plugin?: boolean;
      proc_macro?: boolean;
      harness?: boolean;
      required_features?: string[];
    }>;
    test?: Array<{
      name: string;
      path?: string;
      test?: boolean;
      doctest?: boolean;
      bench?: boolean;
      doc?: boolean;
      plugin?: boolean;
      proc_macro?: boolean;
      harness?: boolean;
      required_features?: string[];
    }>;
    bench?: Array<{
      name: string;
      path?: string;
      test?: boolean;
      doctest?: boolean;
      bench?: boolean;
      doc?: boolean;
      plugin?: boolean;
      proc_macro?: boolean;
      harness?: boolean;
      required_features?: string[];
    }>;
    example?: Array<{
      name: string;
      path?: string;
      test?: boolean;
      doctest?: boolean;
      bench?: boolean;
      doc?: boolean;
      plugin?: boolean;
      proc_macro?: boolean;
      harness?: boolean;
      required_features?: string[];
    }>;
    [key: string]: unknown;
  };
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
