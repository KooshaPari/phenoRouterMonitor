import React from 'react';
import { cn } from '../../lib/utils';
import type { CheckboxProps } from '../../types';

// ============================================================================
// Checkbox Component
// Accessible checkbox with label and keyboard support
// ============================================================================

/**
 * Checkbox Component
 * Boolean input with optional label
 *
 * @example
 * <Checkbox label="Agree to terms" checked={agreed} onChange={setAgreed} />
 */
export const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  (
    {
      checked = false,
      onChange,
      label,
      disabled = false,
      required,
      className,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const checkboxId = React.useId();

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (onChange) {
        onChange(e.target.checked);
      }
    };

    return (
      <div className={cn('flex items-center gap-2', className)}>
        <input
          ref={ref}
          id={checkboxId}
          type="checkbox"
          checked={checked}
          onChange={handleChange}
          disabled={disabled}
          required={required}
          className={cn(
            'w-4 h-4 rounded border transition-colors',
            'accent-cyan-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            'focus-visible:ring-cyan-400 cursor-pointer',
            checked && 'bg-cyan-500 border-cyan-600',
            !checked && 'border-gray-300 hover:border-gray-400',
            disabled && 'opacity-50 cursor-not-allowed',
            className
          )}
          aria-label={ariaLabel}
          aria-checked={checked}
          role="checkbox"
          {...props}
        />
        {label && (
          <label
            htmlFor={checkboxId}
            className={cn(
              'text-sm font-normal text-gray-700',
              disabled && 'opacity-50 cursor-not-allowed'
            )}
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
      </div>
    );
  }
);

Checkbox.displayName = 'Checkbox';

export default Checkbox;
