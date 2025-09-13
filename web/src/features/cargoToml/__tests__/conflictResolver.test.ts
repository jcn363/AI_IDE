import { detectAndResolveConflicts } from '../conflictResolver';

describe('DependencyConflictResolver', () => {
  it('should detect version conflicts', () => {
    const manifest = {
      package: { name: 'test-package', version: '0.1.0' },
      dependencies: {
        'dep-a': '^1.0.0',
        'dep-b': '^2.0.0',
      },
      'dev-dependencies': {
        'dep-a': '^1.5.0', // Different version of dep-a
      },
    };

    const lockfile = {
      package: [
        { name: 'dep-a', version: '1.0.0' },
        { name: 'dep-a', version: '1.5.0' },
        { name: 'dep-b', version: '2.0.0' },
      ],
    };

    const conflicts = detectAndResolveConflicts(manifest as any, lockfile);

    expect(conflicts).toHaveLength(1);
    expect(conflicts[0].package).toBe('dep-a');
    expect(conflicts[0].requestedVersions).toHaveLength(2);
    expect(conflicts[0].resolution).toBeDefined();
  });

  it('should resolve conflicts using highest version by default', () => {
    const manifest = {
      package: { name: 'test-package', version: '0.1.0' },
      dependencies: {
        'dep-a': '^1.0.0',
      },
      'dev-dependencies': {
        'dep-a': '^1.5.0',
      },
    };

    const lockfile = {
      package: [
        { name: 'dep-a', version: '1.0.0' },
        { name: 'dep-a', version: '1.5.0' },
        { name: 'dep-a', version: '1.6.0' },
      ],
    };

    const conflicts = detectAndResolveConflicts(manifest as any, lockfile);
    expect(conflicts[0].resolution?.version).toBe('1.6.0');
  });

  it('should prefer stable versions when configured', () => {
    const manifest = {
      package: { name: 'test-package', version: '0.1.0' },
      dependencies: {
        'dep-a': '^1.0.0',
      },
      'dev-dependencies': {
        'dep-a': '^1.5.0',
      },
    };

    const lockfile = {
      package: [
        { name: 'dep-a', version: '1.0.0' },
        { name: 'dep-a', version: '1.5.0' },
        { name: 'dep-a', version: '2.0.0-alpha.1' },
      ],
    };

    const conflicts = detectAndResolveConflicts(manifest as any, lockfile, {
      preferStable: true,
    });

    // Should pick the highest stable version (1.5.0) over the pre-release (2.0.0-alpha.1)
    expect(conflicts[0].resolution?.version).toBe('1.5.0');
  });
});
