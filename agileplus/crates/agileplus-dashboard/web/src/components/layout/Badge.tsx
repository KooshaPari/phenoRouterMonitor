import React from 'react';
import { cn } from '../../lib/utils';
import type { BadgeProps } from '../../types';

// ============================================================================
// Badge Component
// Display-only status label with optional icon
// ============================================================================

/**
 * Badge Component
 * Status indicator with color variants
 *
 * @example
 * <Badge label="Active" variant="success" icon={<CheckIcon />} />
 */
export const Badge: React.FC<BadgeProps> = ({
  label,
  variant = 'default',
  icon,
  className,
}) => {
  const variantClass = {
    default: 'bg-gray-100 text-gray-900',
    success: 'bg-green-100 text-green-900',
    warning: 'bg-amber-100 text-amber-900',
    error: 'bg-red-100 text-red-900',
    info: 'bg-blue-100 text-blue-900',
  };

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-semibold',
        variantClass[variant],
        className
      )}
    >
      {icon && <span className="flex-shrink-0">{icon}</span>}
      <span>{label}</span>
    </span>
  );
};

Badge.displayName = 'Badge';

export default Badge;
