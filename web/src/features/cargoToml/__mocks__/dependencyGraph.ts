import { DependencyNode, DependencyLink } from '../dependencyGraph';

export const useDependencyGraph = () => ({
  nodes: [
    {
      id: 'test-crate@1.0.0',
      name: 'test-crate',
      version: '1.0.0',
      type: 'crate',
      isRoot: true,
    },
    {
      id: 'dep-crate@1.0.0',
      name: 'dep-crate',
      version: '1.0.0',
      type: 'crate',
      isDirect: true,
    },
  ],
  links: [
    {
      source: 'test-crate@1.0.0',
      target: 'dep-crate@1.0.0',
      type: 'depends',
    },
  ],
  loading: false,
  error: null,
  refresh: jest.fn(),
});

export type { DependencyNode, DependencyLink };
