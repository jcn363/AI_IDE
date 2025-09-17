# Code Refactoring

Rust AI IDE provides powerful refactoring tools to help you improve your code's structure, readability, and maintainability.

## Available Refactorings

### Extract to Function/Method

1. Select the code to extract
2. Right-click and select "Refactor" > "Extract to Function"
3. Enter a name for the new function
4. Choose the scope (module, impl, etc.)
5. Select parameters to pass

### Rename Symbol

1. Right-click on a symbol (variable, function, struct, etc.)
2. Select "Rename Symbol" or press `F2`
3. Enter the new name
4. Press Enter to apply

### Extract Variable

1. Select an expression
2. Right-click and select "Refactor" > "Extract to Variable"
3. Enter a name for the variable

### Inline Variable

1. Right-click on a variable
2. Select "Refactor" > "Inline Variable"
3. Choose to replace all occurrences or just the current one

### Move to Module

1. Right-click on an item (struct, function, etc.)
2. Select "Move to New File" or "Move to Module"
3. Choose the target module or create a new one

## Rust-Specific Refactorings

### Implement Trait

1. Place cursor on a type that should implement a trait
2. Click the lightbulb (💡) or press `Ctrl+.`
3. Select "Implement missing members"

### Add `derive` Attribute

1. Place cursor on a struct or enum
2. Click the lightbulb (💡) or press `Ctrl+.`
3. Select "Add `#[derive()]`"
4. Choose the traits to derive

### Convert to Tuple Struct

1. Place cursor on a struct with named fields
2. Click the lightbulb (💡) or press `Ctrl+.`
3. Select "Convert to tuple struct"

## Best Practices

### When to Refactor

- **Rule of Three**: After writing similar code three times, consider refactoring
- **Boy Scout Rule**: Always leave the code better than you found it
- **Before Adding Features**: Clean up the code before adding new functionality

### Code Smells to Watch For

- Long methods (over 20 lines)
- Large structs with many fields
- High cyclomatic complexity
- Duplicate code
- Too many parameters in functions
- Deeply nested conditionals

## Performance Considerations

- **Clone Minimization**: Avoid unnecessary cloning of data
- **Iterator Chaining**: Prefer iterator methods over manual loops
- **Lazy Evaluation**: Use `iter()` instead of `into_iter()` when possible
- **Avoid `unwrap()`**: Prefer proper error handling with `?` or `match`

## Testing After Refactoring

1. Run unit tests: `cargo test`
2. Run integration tests: `cargo test --test integration_test`
3. Check for performance regressions
4. Verify all functionality still works as expected

## Learn More

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Refactoring: Improving the Design of Existing Code](https://martinfowler.com/books/refactoring.html)
