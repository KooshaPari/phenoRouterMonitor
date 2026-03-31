import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from './Button';

/**
 * Unit tests for Button component
 * Tests rendering, variants, interactions, and accessibility
 */

describe('Button Component', () => {
  // ============================================================================
  // Rendering Tests
  // ============================================================================

  it('renders button with children', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument();
  });

  it('renders with default variant (primary)', () => {
    const { container } = render(<Button>Default</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('bg-cyan-500');
  });

  // ============================================================================
  // Variant Tests
  // ============================================================================

  it('applies primary variant', () => {
    const { container } = render(<Button variant="primary">Primary</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('bg-cyan-500');
  });

  it('applies secondary variant', () => {
    const { container } = render(<Button variant="secondary">Secondary</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('bg-purple-500');
  });

  it('applies ghost variant', () => {
    const { container } = render(<Button variant="ghost">Ghost</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('hover:bg-gray-100');
  });

  it('applies destructive variant', () => {
    const { container } = render(<Button variant="destructive">Delete</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('bg-red-500');
  });

  // ============================================================================
  // Size Tests
  // ============================================================================

  it('applies small size', () => {
    const { container } = render(<Button size="sm">Small</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('h-8');
  });

  it('applies medium size', () => {
    const { container } = render(<Button size="md">Medium</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('h-10');
  });

  it('applies large size', () => {
    const { container } = render(<Button size="lg">Large</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('h-12');
  });

  // ============================================================================
  // State Tests
  // ============================================================================

  it('renders as disabled', () => {
    render(<Button disabled>Disabled</Button>);
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-disabled', 'true');
  });

  it('does not trigger onClick when disabled', async () => {
    const onClick = jest.fn();
    render(
      <Button disabled onClick={onClick}>
        Disabled
      </Button>
    );
    await userEvent.click(screen.getByRole('button'));
    expect(onClick).not.toHaveBeenCalled();
  });

  // ============================================================================
  // Interaction Tests
  // ============================================================================

  it('calls onClick handler on click', async () => {
    const onClick = jest.fn();
    render(<Button onClick={onClick}>Click</Button>);
    await userEvent.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('triggers on keyboard enter', async () => {
    const onClick = jest.fn();
    render(<Button onClick={onClick}>Click</Button>);
    const button = screen.getByRole('button');
    button.focus();
    await userEvent.keyboard('{Enter}');
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  // ============================================================================
  // Accessibility Tests
  // ============================================================================

  it('renders with aria-label when provided', () => {
    render(<Button ariaLabel="Close dialog">×</Button>);
    expect(screen.getByRole('button', { name: 'Close dialog' })).toBeInTheDocument();
  });

  it('is focusable with keyboard', async () => {
    render(<Button>Focusable</Button>);
    const button = screen.getByRole('button');
    button.focus();
    expect(button).toHaveFocus();
  });

  it('supports different button types', () => {
    const { container: submitContainer } = render(<Button type="submit">Submit</Button>);
    expect(submitContainer.querySelector('button')).toHaveAttribute('type', 'submit');

    const { container: resetContainer } = render(<Button type="reset">Reset</Button>);
    expect(resetContainer.querySelector('button')).toHaveAttribute('type', 'reset');
  });

  // ============================================================================
  // Custom Class Tests
  // ============================================================================

  it('merges custom className', () => {
    const { container } = render(<Button className="custom-class">Custom</Button>);
    const button = container.querySelector('button');
    expect(button).toHaveClass('custom-class');
    // Should still have base classes
    expect(button).toHaveClass('bg-cyan-500');
  });
});
