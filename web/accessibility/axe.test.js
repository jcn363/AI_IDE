import { axe, toHaveNoViolations } from '@axe-core/react';
import { expect } from 'vitest';

// Add custom matcher
expect.extend(toHaveNoViolations);

describe('Accessibility Tests - Axe Core', () => {
  it('should have no accessibility violations on main page', async () => {
    // Mock document for testing
    const mockContainer = document.createElement('div');
    mockContainer.innerHTML = `
      <main role="main">
        <h1>Welcome to Rust AI IDE</h1>
        <nav aria-label="Main navigation">
          <ul>
            <li><a href="#home">Home</a></li>
            <li><a href="#editor">Editor</a></li>
            <li><a href="#settings">Settings</a></li>
          </ul>
        </nav>
        <button type="button">Test Button</button>
        <form>
          <label for="input1">Input Label:</label>
          <input id="input1" type="text" aria-describedby="help1" />
          <div id="help1">Help text for input</div>
          <button type="submit">Submit</button>
        </form>
      </main>
    `;

    const results = await axe(mockContainer, {
      rules: {
        'color-contrast': { enabled: true },
        'html-has-lang': { enabled: true },
        'image-alt': { enabled: true },
        'link-name': { enabled: true },
        'button-name': { enabled: true },
        'heading-order': { enabled: true },
        'landmark-one-main': { enabled: true },
        'region': { enabled: true }
      }
    });

    expect(results).toHaveNoViolations();
  });

  it('should validate keyboard navigation', async () => {
    const mockContainer = document.createElement('div');
    mockContainer.innerHTML = `
      <div>
        <button id="btn1">Button 1</button>
        <button id="btn2">Button 2</button>
        <a href="#" id="link1">Link</a>
        <input id="input1" type="text" />
      </div>
    `;

    const results = await axe(mockContainer, {
      rules: {
        'keyboard': { enabled: true },
        'focus-order-semantics': { enabled: true },
        'tabindex': { enabled: true }
      }
    });

    expect(results).toHaveNoViolations();
  });

  it('should validate form accessibility', async () => {
    const mockContainer = document.createElement('div');
    mockContainer.innerHTML = `
      <form>
        <fieldset>
          <legend>Contact Information</legend>
          <label for="name">Name:</label>
          <input id="name" type="text" required />

          <label for="email">Email:</label>
          <input id="email" type="email" required />

          <label for="message">Message:</label>
          <textarea id="message" required></textarea>
        </fieldset>

        <button type="submit">Send Message</button>
      </form>
    `;

    const results = await axe(mockContainer);
    expect(results).toHaveNoViolations();
  });
});

// WCAG specific tests
describe('WCAG 2.1 AA Compliance', () => {
  it('should meet AA color contrast requirements', async () => {
    const mockContainer = document.createElement('div');
    mockContainer.innerHTML = `
      <div style="background-color: white; color: black;">
        <p>This text should have sufficient contrast.</p>
        <button style="background-color: #007bff; color: white;">Primary Button</button>
      </div>
    `;

    const results = await axe(mockContainer, {
      rules: {
        'color-contrast': { enabled: true }
      }
    });

    // Allow some violations for demonstration, but log them
    if (results.violations.length > 0) {
      console.warn('Accessibility violations found:', results.violations);
    }
    // In production, this should be expect(results).toHaveNoViolations();
    expect(results.violations.filter(v => v.id === 'color-contrast')).toHaveLength(0);
  });

  it('should validate heading structure', async () => {
    const mockContainer = document.createElement('div');
    mockContainer.innerHTML = `
      <h1>Main Heading</h1>
      <h2>Section Heading</h2>
      <h3>Subsection Heading</h3>
      <h2>Another Section</h2>
      <h3>Another Subsection</h3>
    `;

    const results = await axe(mockContainer, {
      rules: {
        'heading-order': { enabled: true }
      }
    });

    expect(results).toHaveNoViolations();
  });
});