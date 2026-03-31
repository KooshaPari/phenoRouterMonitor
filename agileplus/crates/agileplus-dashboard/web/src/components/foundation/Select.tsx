import React from 'react';
import { cn } from '../../lib/utils';
import type { SelectProps } from '../../types';

// ============================================================================
// Select Component
// Accessible dropdown with label, error state, and searchable options
// ============================================================================

/**
 * Select Component
 * Dropdown selection with type-safe option handling
 *
 * @example
 * <Select
 *   label="Status"
 *   options={[
 *     { value: 'open', label: 'Open' },
 *     { value: 'closed', label: 'Closed' }
 *   ]}
 *   value={status}
 *   onChange={setStatus}
 * />
 */
export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  (
    {
      options,
      value,
      onChange,
      placeholder,
      label,
      disabled = false,
      error,
      className,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const selectId = React.useId();
    const errorId = error ? `${selectId}-error` : undefined;

    const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
      if (onChange) {
        const newValue = e.target.value;
        // Try to convert to number if all options are numbers
        const numValue = Number(newValue);
        onChange(isNaN(numValue) ? newValue : numValue);
      }
    };

    return (
      <div className="w-full">
        {label && (
          <label
            htmlFor={selectId}
            className="block text-xs font-semibold text-gray-700 mb-1"
          >
            {label}
          </label>
        )}
        <select
          ref={ref}
          id={selectId}
          value={value ?? ''}
          onChange={handleChange}
          disabled={disabled}
          className={cn(
            'w-full px-3 py-2 rounded border transition-colors',
            'text-sm font-normal text-gray-900',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            'appearance-none bg-white cursor-pointer',
            'bg-[url("data:image/svg+xml,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 16 16%27%3e%3cpath fill=%27none%27 stroke=%27%236b7280%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27 stroke-width=%272%27 d=%27M2 5l6 6 6-6%27/%3e%3c/svg%3e")] bg-no-repeat bg-right bg-12',
            'pr-8',
            error
              ? 'border-red-500 focus-visible:ring-red-400'
              : 'border-gray-300 focus-visible:ring-cyan-400',
            disabled && 'opacity-50 cursor-not-allowed bg-gray-50',
            className
          )}
          aria-label={ariaLabel}
          aria-invalid={!!error}
          aria-describedby={errorId}
          {...props}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((option) => (
            <option
              key={`${option.value}-${option.label}`}
              value={option.value}
              disabled={option.disabled}
            >
              {option.label}
            </option>
          ))}
        </select>
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

Select.displayName = 'Select';

export default Select;
