import React from 'react';
import { cn } from '../../lib/utils';
import type { InputProps } from '../../types';

// ============================================================================
// Input Component
// Accessible text input with label, error state, and variants
// ============================================================================

/**
 * Input Component
 * Standard text input field with optional label and error handling
 *
 * @example
 * <Input type="email" placeholder="user@example.com" label="Email" />
 * <Input error="Email is required" />
 */
export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      type = 'text',
      placeholder,
      value,
      onChange,
      disabled = false,
      error,
      label,
      required,
      className,
      ariaLabel,
      ariaDescribedBy,
      ...props
    },
    ref
  ) => {
    const inputId = React.useId();
    const errorId = error ? `${inputId}-error` : undefined;
    const describedBy = ariaDescribedBy || errorId;

    return (
      <div className="w-full">
        {label && (
          <label
            htmlFor={inputId}
            className="block text-xs font-semibold text-gray-700 mb-1"
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
        <input
          ref={ref}
          id={inputId}
          type={type}
          placeholder={placeholder}
          value={value}
          onChange={onChange}
          disabled={disabled}
          required={required}
          className={cn(
            'w-full px-3 py-2 rounded border transition-colors',
            'text-sm font-normal text-gray-900 placeholder-gray-500',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            error
              ? 'border-red-500 focus-visible:ring-red-400'
              : 'border-gray-300 focus-visible:ring-cyan-400',
            disabled && 'opacity-50 cursor-not-allowed bg-gray-50',
            className
          )}
          aria-label={ariaLabel}
          aria-invalid={!!error}
          aria-describedby={describedBy}
          {...props}
        />
        {error && (
          <p
            id={errorId}
            className="mt-1 text-xs text-red-500 font-medium"
            role="alert"
          >
            {error}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

export default Input;
