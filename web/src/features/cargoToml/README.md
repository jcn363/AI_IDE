# Cargo.toml Editor

An advanced editor for managing Rust project dependencies with rich features for
dependency management, security scanning, and license compliance.

## Features

### 1. Feature Flag Optimization

- Analyze feature flag usage across dependencies
- Identify and remove unused features
- Optimize default features
- Get suggestions for feature flag improvements

### 2. Dependency Graph Visualization

- Interactive visualization of project dependencies
- Color-coded nodes for different dependency types
- Zoom and pan functionality
- Click to inspect dependency details

### 3. Security Vulnerability Scanning

- Scan for known security vulnerabilities in dependencies
- Severity-based vulnerability classification
- Detailed vulnerability information and remediation advice
- Integration with RustSec advisory database

### 4. License Compliance

- Automatic license detection for all dependencies
- License compatibility checking
- Banned license detection
- Summary of license usage across the project

### 5. Dependency Management

- Add, remove, and update dependencies
- Toggle between development and production dependencies
- View and manage dependency features
- Version requirement validation

## Components

- `CargoTomlEditor`: Main editor component with tabbed interface
- `featureFlags.ts`: Feature flag analysis and optimization
- `dependencyGraph.tsx`: Interactive dependency graph visualization
- `securityScanner.ts`: Security vulnerability scanning
- `licenseChecker.ts`: License compliance checking
- `useCargoTomlEditor.ts`: Custom hook for managing editor state

## Usage

```tsx
import { CargoTomlEditor } from './features/cargoToml/CargoTomlEditor';

function App() {
  const [toml, setToml] = useState('');

  // Load your Cargo.toml content
  useEffect(() => {
    fetch('/path/to/Cargo.toml')
      .then((res) => res.text())
      .then(setToml);
  }, []);

  const handleSave = async (newToml: string) => {
    // Save the updated Cargo.toml
    await fetch('/path/to/Cargo.toml', {
      method: 'POST',
      body: newToml,
      headers: {
        'Content-Type': 'text/x-toml',
      },
    });
  };

  return (
    <div className="container mx-auto p-4">
      <CargoTomlEditor initialToml={toml} onSave={handleSave} />
    </div>
  );
}
```

## Dependencies

- `react`: ^17.0.0
- `react-dom`: ^17.0.0
- `@types/react`: ^17.0.0
- `@types/react-dom`: ^17.0.0
- `d3`: ^7.0.0
- `@types/d3`: ^7.0.0
- `toml`: ^3.0.0

## Development

1. Install dependencies:

   ```bash
   npm install
   ```

2. Start the development server:

   ```bash
   npm start
   ```

## Testing

Run the test suite:

```bash
npm test
```

## License

MIT
