// Test file for generated types integration
// This demonstrates that the shared types migration is working correctly

import { User, Theme, UserPreferences, AppConfig, Status } from './generated';

describe('Generated Types Integration', () => {
  it('should create and validate User instance', () => {
    const user: User = {
      id: 123,
      name: 'Test User',
      email: 'test@example.com',
    };

    expect(user.id).toBe(123);
    expect(user.name).toBe('Test User');
    expect(user.email).toBe('test@example.com'); // Note: in generated types, optional fields are marked with `?`
  });

  it('should create User with optional email', () => {
    const user: User = {
      id: 456,
      name: 'Test User No Email',
      // email is optional in generated types (marked with `?`)
    };

    expect(user.id).toBe(456);
    expect(user.name).toBe('Test User No Email');
    expect(user.email).toBeUndefined();
  });

  it('should work with enum values', () => {
    const theme: Theme = Theme.Dark;
    expect(theme).toBe('Dark');
  });

  it('should create nested UserPreferences', () => {
    const preferences: UserPreferences = {
      theme: Theme.Light,
      settings: {
        notifications: true,
        auto_save: false,
      },
    };

    expect(preferences.theme).toBe(Theme.Light);
    expect(preferences.settings.notifications).toBe(true);
    expect(preferences.settings.auto_save).toBe(false);
  });

  it('should validate AppConfig structure', () => {
    const config: AppConfig = {
      core: {
        app_name: 'Test IDE',
        app_version: '1.0.0',
        theme: 'dark',
        fonts: {
          editor_font_family: 'JetBrains Mono',
          editor_font_size: 14,
          ui_font_family: 'System',
          ui_font_size: 13,
        },
        editor: {
          tab_size: 4,
          insert_spaces: true,
          word_wrap: true,
          minimap: true,
          line_numbers: true,
          auto_save_delay: 60,
          bracket_matching: true,
          highlight_current_line: true,
        },
      },
      ai: {
        default_provider: 'OpenAI',
        endpoints: {
          OpenAI: 'https://api.openai.com/v1',
        },
      },
      performance: {
        max_analysis_threads: 4,
        max_memory_mb: 4096,
        ai_concurrency_limit: 3,
        io_thread_pool_size: 2,
      },
    };

    expect(config.core.app_name).toBe('Test IDE');
    expect(config.ai.default_provider).toBe('OpenAI');
    expect(config.performance.max_analysis_threads).toBe(4);
  });

  it('should work with Status enum', () => {
    const status: Status = Status.Ok;
    expect(status).toBe('ok');

    const errorStatus: Status = Status.Error;
    expect(errorStatus).toBe('error');
  });

  it('should validate readme example compatibility', () => {
    // This test ensures the types generated match the examples in documentation

    // Create a user as shown in the migration guide example
    const user: User = {
      id: 1,
      name: 'John Doe',
      email: 'john@example.com',
    };

    expect(user.id).toBe(1);
    expect(user.name).toBe('John Doe');
    expect(user.email).toBe('john@example.com');

    // Test Theme enum usage
    const lightTheme: Theme = Theme.Light;

    // Test nested structure access
    const preferences: UserPreferences = {
      theme: lightTheme,
      settings: {
        notifications: true,
        auto_save: true,
      },
    };

    expect(preferences.theme).toBe(Theme.Light);
    expect(preferences.settings.notifications).toBe(true);
  });
});

// Integration test for Cargo types (demonstrating combined usage)
import { User as ImportedUser } from '../shared/types/index'; // From consolidated location

describe('Combined Type Usage', () => {
  it('should support both generated and consolidated type imports', () => {
    // This shows that imports from generated and consolidated locations both work
    const generatedUser: User = {
      id: 100,
      name: 'Generated User',
      email: 'generated@example.com',
    };

    // The consolidated location might have additional types or different structures
    const consolidatedUser: ImportedUser = {
      id: 200,
      name: 'Consolidated User',
    };

    expect(generatedUser.email).toBeDefined();
    expect((consolidatedUser as any).email).toBeUndefined(); // Different structure potentially
  });
});
