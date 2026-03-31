import React, { useEffect } from 'react';
import { cn } from '../../lib/utils';
import type { ToastProps } from '../../types';

// ============================================================================
// Toast Component
// Temporary notification with auto-dismiss and type indicators
// ============================================================================

/**
 * Toast Component
 * Short-lived notification for feedback messages
 *
 * @example
 * <Toast type="success" message="Saved successfully" duration={3000} onClose={handleClose} />
 */
export const Toast: React.FC<ToastProps> = ({
  type = 'info',
  message,
  duration = 3000,
  onClose,
  className,
}) => {
  useEffect(() => {
    if (!duration) return;

    const timer = setTimeout(() => {
      if (onClose) onClose();
    }, duration);

    return () => clearTimeout(timer);
  }, [duration, onClose]);

  const typeStyles = {
    success: 'bg-green-50 border-green-200 text-green-900',
    error: 'bg-red-50 border-red-200 text-red-900',
    warning: 'bg-amber-50 border-amber-200 text-amber-900',
    info: 'bg-blue-50 border-blue-200 text-blue-900',
  };

  const typeIcons = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ',
  };

  return (
    <div
      role="alert"
      className={cn(
        'flex items-center gap-3 px-4 py-3 rounded border',
        'animate-slide-in-top',
        typeStyles[type],
        className
      )}
    >
      <span className="flex-shrink-0 text-lg font-bold">
        {typeIcons[type]}
      </span>
      <p className="flex-1 text-sm font-medium">{message}</p>
      {onClose && (
        <button
          onClick={onClose}
          className="flex-shrink-0 text-lg hover:opacity-70 transition-opacity"
          aria-label="Close notification"
        >
          ×
        </button>
      )}
    </div>
  );
};

Toast.displayName = 'Toast';

export default Toast;
