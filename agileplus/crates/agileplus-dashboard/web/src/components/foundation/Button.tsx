import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';
import type { ButtonProps } from '../../types';

// ============================================================================
// Button Component
// Accessible, type-safe button with multiple variants and sizes
// ============================================================================

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 rounded font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      variant: {
        primary: 'bg-cyan-500 text-white hover:bg-cyan-600 focus-visible:ring-cyan-400',
        secondary: 'bg-purple-500 text-white hover:bg-purple-600 focus-visible:ring-purple-400',
        ghost: 'hover:bg-gray-100 text-gray-900 focus-visible:ring-gray-400',
        destructive: 'bg-red-500 text-white hover:bg-red-600 focus-visible:ring-red-400',
      },
      size: {
        sm: 'h-8 px-3 text-xs',
        md: 'h-10 px-4 text-sm',
        lg: 'h-12 px-6 text-base',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
);

/**
 * Button Component
 * Primary interaction element for actions and CTAs
 *
 * @example
 * <Button onClick={() => alert('Clicked!')}>Submit</Button>
 * <Button variant="secondary" size="lg" disabled>Disabled</Button>
 */
export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      disabled = false,
      onClick,
      className,
      children,
      type = 'button',
      ariaLabel,
      ...props
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        type={type}
        disabled={disabled}
        onClick={onClick}
        className={cn(buttonVariants({ variant, size }), className)}
        aria-label={ariaLabel}
        aria-disabled={disabled}
        {...props}
      >
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';

export default Button;
