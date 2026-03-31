import React from 'react';
import { cn } from '../../lib/utils';
import type { CardProps } from '../../types';

// ============================================================================
// Card Component
// Container with optional title, footer, and variant styles
// ============================================================================

/**
 * Card Component
 * Content container with semantic markup and variant styling
 *
 * @example
 * <Card title="User Profile" footer={<Button>Save</Button>}>
 *   <p>Card content here</p>
 * </Card>
 */
export const Card: React.FC<CardProps> = ({
  title,
  children,
  footer,
  variant = 'default',
  className,
}) => {
  const variantClass = {
    default: 'bg-white border border-gray-200 shadow-sm',
    elevated: 'bg-white border border-gray-100 shadow-md',
    outlined: 'bg-gray-50 border-2 border-gray-300',
  };

  return (
    <article
      className={cn(
        'rounded-lg overflow-hidden',
        variantClass[variant],
        className
      )}
    >
      {title && (
        <header className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">{title}</h3>
        </header>
      )}
      <div className="px-6 py-4">{children}</div>
      {footer && (
        <footer className="px-6 py-4 bg-gray-50 border-t border-gray-200">
          {footer}
        </footer>
      )}
    </article>
  );
};

Card.displayName = 'Card';

export default Card;
