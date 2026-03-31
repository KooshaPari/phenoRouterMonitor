import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Input } from './Input';

/**
 * Unit tests for Input component
 * Tests rendering, validation, accessibility, and state management
 */

describe('Input Component', () => {
  // ============================================================================
  // Rendering Tests
  // ============================================================================

  it('renders input element', () => {
    render(<Input />);
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('renders with placeholder', () => {
    render(<Input placeholder="Enter text" />);
    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
  });

  it('renders with label', () => {
    render(<Input label="Email" />);
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
  });

  it('renders required indicator when required', () => {
    render(<Input label="Name" required />);
    expect(screen.getByText('*')).toBeInTheDocument();
  });

  // ============================================================================
  // Type Tests
  // ============================================================================

  it('renders email input type', () => {
    render(<Input type="email" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('type', 'email');
  });

  it('renders password input type', () => {
    render(<Input type="password" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('type', 'password');
  });

  it('renders number input type', () => {
    render(<Input type="number" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('type', 'number');
  });

  // ============================================================================
  // State & Interaction Tests
  // ============================================================================

  it('handles onChange event', async () => {
    const onChange = jest.fn();
    render(<Input onChange={onChange} />);
    const input = screen.getByRole('textbox');

    await userEvent.type(input, 'test');
    expect(onChange).toHaveBeenCalled();
  });

  it('can be controlled', async () => {
    const { rerender } = render(<Input value="initial" onChange={() => {}} />);
    expect(screen.getByRole('textbox')).toHaveValue('initial');

    rerender(<Input value="updated" onChange={() => {}} />);
    expect(screen.getByRole('textbox')).toHaveValue('updated');
  });

  it('renders as disabled', () => {
    render(<Input disabled />);
    expect(screen.getByRole('textbox')).toBeDisabled();
  });

  // ============================================================================
  // Error Handling Tests
  // ============================================================================

  it('displays error message', () => {
    render(<Input error="This field is required" />);
    expect(screen.getByText('This field is required')).toBeInTheDocument();
  });

  it('applies error styling when error is present', () => {
    const { container } = render(<Input error="Invalid input" />);
    const input = container.querySelector('input');
    expect(input).toHaveClass('border-red-500');
  });

  it('has aria-invalid when error is present', () => {
    render(<Input error="Error message" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
  });

  // ============================================================================
  // Accessibility Tests
  // ============================================================================

  it('associates label with input', () => {
    render(<Input label="Username" />);
    const input = screen.getByRole('textbox');
    const label = screen.getByText('Username');
    expect(label).toHaveAttribute('for', input.id);
  });

  it('has aria-label when provided', () => {
    render(<Input ariaLabel="Search input" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('aria-label', 'Search input');
  });

  it('supports aria-describedby', () => {
    render(<Input ariaDescribedBy="helper-text" />);
    expect(screen.getByRole('textbox')).toHaveAttribute('aria-describedby', 'helper-text');
  });

  it('is focusable', async () => {
    render(<Input />);
    const input = screen.getByRole('textbox');
    input.focus();
    expect(input).toHaveFocus();
  });
});
