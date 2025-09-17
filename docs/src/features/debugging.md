# Debugging Tools

Rust AI IDE includes powerful debugging capabilities to help you identify and fix issues in your code.

## Features

- **Breakpoints**: Pause execution at specific points
- **Step Through Code**: Execute code line by line
- **Watch Variables**: Monitor variable values in real-time
- **Call Stack**: View the execution call stack
- **Debug Console**: Execute code in the current debug context

## Getting Started

### Setting Up a Debug Configuration

1. Open the Run and Debug view (`Ctrl+Shift+D` or `Cmd+Shift+D`)
2. Click "create a launch.json file"
3. Select your environment (e.g., Rust, Node.js, Python)
4. Configure the launch settings as needed

### Basic Debugging

1. Set breakpoints by clicking in the gutter next to line numbers
2. Start debugging (`F5` or the green play button)
3. Use the debug toolbar to step through code
   - Step Over (`F10`)
   - Step Into (`F11`)
   - Step Out (`Shift+F11`)
   - Continue (`F5`)

## Advanced Features

### Conditional Breakpoints

1. Right-click on a breakpoint
2. Select "Edit Breakpoint"
3. Enter a condition (e.g., `x > 5`)

### Watch Window

1. Open the Watch view
2. Click the + icon
3. Enter an expression to watch

### Debug Console

1. Open the Debug Console
2. Type expressions to evaluate in the current context

## Debugging Rust

### Prerequisites

- Install the Rust extension
- Install the required debugger (LLDB or GDB)

### Configuration

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug",
            "program": "${workspaceFolder}/target/debug/your_binary",
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

## Troubleshooting

- **Debugger Not Attaching**: Ensure your program was built with debug symbols
- **Missing Variables**: Some optimizations may prevent variables from being available
- **Performance Issues**: Consider using sampling profilers for performance debugging

## Learn More

- [Rust Debugging](https://rust-lang.github.io/rustup-components-history/)
- [LLDB Documentation](https://lldb.llvm.org/)
- [GDB Documentation](https://www.gnu.org/software/gdb/documentation/)
