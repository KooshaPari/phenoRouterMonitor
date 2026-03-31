import React from 'react';
import { cn } from '../../lib/utils';
import type { RadioProps } from '../../types';

// ============================================================================
// Radio Component
// Accessible radio button for single selection within groups
// ============================================================================

/**
 * Radio Component
 * Single-select from a group of options
 *
 * @example
 * <div role="radiogroup">
 *   <Radio value="opt1" label="Option 1" onChange={setSelected} />
 *   <Radio value="opt2" label="Option 2" onChange={setSelected} />
 * </div>
 */
export const Radio = React.forwardRef<HTMLInputElement, RadioProps>(
  (
    {
      value,
      checked = false,
      onChange,
      label,
      disabled = false,
      className,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const radioId = React.useId();

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (onChange && e.target.checked) {
        onChange(value);
      }
    };

    return (
      <div className={cn('flex items-center gap-2', className)}>
        <input
          ref={ref}
          id={radioId}
          type="radio"
          value={value}
          checked={checked}
          onChange={handleChange}
          disabled={disabled}
          className={cn(
            'w-4 h-4 rounded-full border-2 transition-colors',
            'accent-cyan-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            'focus-visible:ring-cyan-400 cursor-pointer',
            checked && 'border-cyan-600',
            !checked && 'border-gray-300 hover:border-gray-400',
            disabled && 'opacity-50 cursor-not-allowed',
            className
          )}
          aria-label={ariaLabel}
          aria-checked={checked}
          role="radio"
          {...props}
        />
        {label && (
          <label
            htmlFor={radioId}
            className={cn(
              'text-sm font-normal text-gray-700',
              disabled && 'opacity-50 cursor-not-allowed'
            )}
          >
            {label}
          </label>
        )}
      </div>
    );
  }
);

Radio.displayName = 'Radio';

export default Radio;
