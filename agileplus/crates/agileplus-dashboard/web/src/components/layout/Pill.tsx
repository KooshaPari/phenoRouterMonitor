import React from 'react';
import { cn } from '../../lib/utils';
import type { PillProps } from '../../types';

// ============================================================================
// Pill Component
// Removable tag with optional delete action
// ============================================================================

/**
 * Pill Component
 * Tag with optional dismissible action
 *
 * @example
 * <Pill label="bug" variant="error" onRemove={() => removeTag('bug')} />
 */
export const Pill = React.forwardRef<HTMLDivElement, PillProps>(
  (
    {
      label,
      onRemove,
      variant = 'default',
      className,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const variantClass = {
      default: 'bg-gray-100 text-gray-900 hover:bg-gray-200',
      primary: 'bg-cyan-100 text-cyan-900 hover:bg-cyan-200',
      secondary: 'bg-purple-100 text-purple-900 hover:bg-purple-200',
    };

    return (
      <div
        ref={ref}
        className={cn(
          'inline-flex items-center gap-2 px-3 py-1 rounded-full text-sm font-medium',
          'transition-colors',
          variantClass[variant],
          className
        )}
        {...props}
      >
        <span>{label}</span>
        {onRemove && (
          <button
            onClick={onRemove}
            className={cn(
              'flex-shrink-0 text-lg leading-none hover:opacity-70',
              'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-offset-1 rounded',
              'transition-opacity'
            )}
            aria-label={ariaLabel || `Remove ${label}`}
            type="button"
          >
            ×
          </button>
        )}
      </div>
    );
  }
);

Pill.displayName = 'Pill';

export default Pill;
