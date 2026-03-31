import React, { useEffect, useRef, useCallback } from 'react';
import { cn } from '../../lib/utils';
import type { ModalProps } from '../../types';

// ============================================================================
// Modal Component
// Accessible dialog with focus trap and backdrop
// ============================================================================

/**
 * Modal Component
 * Dialog overlay with focus management and keyboard support
 *
 * @example
 * <Modal isOpen={open} onClose={() => setOpen(false)} title="Confirm">
 *   <p>Are you sure?</p>
 * </Modal>
 */
export const Modal = React.forwardRef<HTMLDivElement, ModalProps>(
  (
    {
      isOpen,
      onClose,
      title,
      children,
      footer,
      size = 'md',
      className,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const modalRef = useRef<HTMLDivElement>(null);
    const contentRef = ref || modalRef;

    // Focus trap on mount
    useEffect(() => {
      if (!isOpen) return;

      const handleEscape = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          onClose();
        }
      };

      document.addEventListener('keydown', handleEscape);

      // Focus modal on open
      if (contentRef && 'current' in contentRef && contentRef.current) {
        contentRef.current.focus();
      }

      return () => {
        document.removeEventListener('keydown', handleEscape);
      };
    }, [isOpen, onClose, contentRef]);

    if (!isOpen) return null;

    const sizeClass = {
      sm: 'max-w-sm',
      md: 'max-w-md',
      lg: 'max-w-lg',
    };

    return (
      <>
        {/* Backdrop */}
        <div
          className="fixed inset-0 bg-black/30 z-40 transition-opacity"
          onClick={onClose}
          role="presentation"
        />

        {/* Modal */}
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          role="presentation"
        >
          <div
            ref={contentRef}
            role="dialog"
            aria-modal="true"
            aria-label={ariaLabel || title}
            className={cn(
              'bg-white rounded-lg shadow-lg w-full overflow-hidden',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
              sizeClass[size],
              className
            )}
            tabIndex={-1}
            {...props}
          >
            {/* Header */}
            {title && (
              <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-semibold text-gray-900">{title}</h2>
                <button
                  onClick={onClose}
                  className="text-gray-500 hover:text-gray-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                  aria-label="Close dialog"
                >
                  <span className="text-2xl">×</span>
                </button>
              </div>
            )}

            {/* Content */}
            <div className="px-6 py-4 max-h-[60vh] overflow-y-auto">{children}</div>

            {/* Footer */}
            {footer && (
              <div className="px-6 py-4 bg-gray-50 border-t border-gray-200">
                {footer}
              </div>
            )}
          </div>
        </div>
      </>
    );
  }
);

Modal.displayName = 'Modal';

export default Modal;
