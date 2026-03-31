import React from 'react';
import { cn } from '../../lib/utils';
import type { ToggleProps } from '../../types';

// ============================================================================
// Toggle Component
// Accessible on/off switch with optional icon and label
// ============================================================================

/**
 * Toggle Component
 * Binary switch for boolean states
 *
 * @example
 * <Toggle
 *   checked={darkMode}
 *   onChange={setDarkMode}
 *   label="Dark mode"
 *   icon={<MoonIcon />}
 * />
 */
export const Toggle = React.forwardRef<HTMLButtonElement, ToggleProps>(
  (
    {
      checked = false,
      onChange,
      label,
      icon,
      disabled = false,
      className,
      ariaLabel,
      ariaPressed = checked,
      ...props
    },
    ref
  ) => {
    const handleClick = () => {
      if (!disabled && onChange) {
        onChange(!checked);
      }
    };

    return (
      <div className={cn('flex items-center gap-2', className)}>
        <button
          ref={ref}
          type="button"
          onClick={handleClick}
          disabled={disabled}
          className={cn(
            'relative inline-flex h-6 w-11 rounded-full transition-colors',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            'focus-visible:ring-cyan-400',
            checked ? 'bg-cyan-500' : 'bg-gray-300',
            disabled && 'opacity-50 cursor-not-allowed',
            !disabled && 'cursor-pointer'
          )}
          aria-label={ariaLabel}
          aria-pressed={ariaPressed}
          {...props}
        >
          <span
            className={cn(
              'inline-block h-5 w-5 transform rounded-full bg-white transition-transform',
              'absolute top-0.5',
              checked ? 'translate-x-5 right-0.5' : 'left-0.5'
            )}
          />
        </button>
        {(label || icon) && (
          <div className="flex items-center gap-1">
            {icon && <span className="text-gray-700">{icon}</span>}
            {label && (
              <label className={cn('text-sm font-normal text-gray-700', disabled && 'opacity-50')}>
                {label}
              </label>
            )}
          </div>
        )}
      </div>
    );
  }
);

Toggle.displayName = 'Toggle';

export default Toggle;
