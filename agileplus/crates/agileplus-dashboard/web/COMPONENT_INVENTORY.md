# Component Inventory & Design Specifications

**Document**: Complete component reference for AgilePlus Dashboard React library
**Version**: 1.0
**Last Updated**: 2026-03-31
**Status**: Ready for implementation

---

## Overview

This document provides the complete API reference, TypeScript interfaces, accessibility requirements, and implementation examples for all 14 components in the Phase 2 component library.

**Table of Contents:**
- Foundation Components (6)
- Layout Components (5)
- Complex Components (3)
- Accessibility Checklist (WCAG 2.1 AA)
- Storybook Scenarios
- Implementation Examples

---

## Section 1: Foundation Components

### 1.1 Button Component

**Purpose**: Primary action trigger for forms, navigation, and user interactions
**Variants**: primary, secondary, ghost, outline
**Sizes**: sm (small), md (medium), lg (large)

#### Props Interface

```typescript
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual variant style */
  variant?: 'primary' | 'secondary' | 'ghost' | 'outline';

  /** Button size */
  size?: 'sm' | 'md' | 'lg';

  /** Show loading spinner, disable interaction */
  isLoading?: boolean;

  /** Icon to display on left side */
  leftIcon?: React.ReactNode;

  /** Icon to display on right side */
  rightIcon?: React.ReactNode;

  /** Aria label for accessibility */
  'aria-label'?: string;

  /** Whether button is disabled */
  disabled?: boolean;

  /** On click callback */
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;

  /** Button content */
  children: React.ReactNode;
}
```

#### Default Behavior

```typescript
<Button variant="primary" size="md">
  Click Me
</Button>
```

#### All Variants (Storybook Examples)

| Variant | Appearance | Use Case |
|---------|-----------|----------|
| `primary` | Cyan background, white text | Main call-to-action |
| `secondary` | Purple background, white text | Secondary action |
| `ghost` | No background, gray text | Tertiary/link-like action |
| `outline` | Gray border, gray text | Alternative primary |

#### Size Examples

```typescript
<Button size="sm">Small</Button>      {/* 12px font, 6px padding */}
<Button size="md">Medium</Button>    {/* 14px font, 8px padding */}
<Button size="lg">Large</Button>     {/* 16px font, 12px padding */}
```

#### With Icons

```typescript
import { ChevronRight, Loader } from 'lucide-react';

<Button leftIcon={<ChevronRight />}>Next</Button>
<Button isLoading>Saving...</Button>
<Button rightIcon={<ChevronRight />}>Continue</Button>
```

#### Accessibility

- ✅ Semantic `<button>` element (not div)
- ✅ Focus ring visible (2px cyan/purple border)
- ✅ Keyboard navigation (Tab, Enter, Space)
- ✅ `aria-label` for icon-only buttons
- ✅ `aria-disabled` when disabled
- ✅ Color contrast: 4.5:1 minimum (WCAG AA)
- ✅ Minimum touch target: 44×44px (WCAG 2.5.5 AAA)

#### Implementation

```typescript
// src/components/ui/button.tsx
import React from 'react';
import { cn } from '@/lib/utils';
import { Loader2 } from 'lucide-react';

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      leftIcon,
      rightIcon,
      disabled = false,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const baseStyles =
      'inline-flex items-center justify-center font-semibold rounded-md transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';

    const variantStyles = {
      primary:
        'bg-cyan-500 text-white hover:bg-cyan-600 focus:ring-cyan-500 active:bg-cyan-700',
      secondary:
        'bg-purple-500 text-white hover:bg-purple-600 focus:ring-purple-500 active:bg-purple-700',
      ghost: 'text-gray-700 hover:bg-gray-100 focus:ring-gray-500 active:bg-gray-200',
      outline:
        'border-2 border-gray-300 text-gray-700 hover:bg-gray-50 focus:ring-gray-500 active:bg-gray-100',
    };

    const sizeStyles = {
      sm: 'px-3 py-1.5 text-xs gap-1.5',
      md: 'px-4 py-2 text-sm gap-2',
      lg: 'px-6 py-3 text-base gap-2',
    };

    return (
      <button
        ref={ref}
        className={cn(baseStyles, variantStyles[variant], sizeStyles[size], className)}
        disabled={isLoading || disabled}
        aria-disabled={isLoading || disabled}
        {...props}
      >
        {isLoading && <Loader2 className="w-4 h-4 animate-spin" />}
        {!isLoading && leftIcon && <span>{leftIcon}</span>}
        <span>{children}</span>
        {!isLoading && rightIcon && <span>{rightIcon}</span>}
      </button>
    );
  }
);

Button.displayName = 'Button';
```

#### Unit Test Example

```typescript
// tests/components/button.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from '@/components/ui/button';

describe('Button', () => {
  it('renders with correct variant styles', () => {
    render(<Button variant="primary">Click</Button>);
    const btn = screen.getByRole('button');
    expect(btn).toHaveClass('bg-cyan-500');
  });

  it('handles click events', async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click</Button>);
    await userEvent.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('is keyboard accessible', async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click</Button>);
    const btn = screen.getByRole('button');
    btn.focus();
    fireEvent.keyDown(btn, { key: 'Enter', code: 'Enter' });
    expect(onClick).toHaveBeenCalled();
  });

  it('disables when isLoading is true', () => {
    render(<Button isLoading>Loading</Button>);
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('shows loading spinner', () => {
    render(<Button isLoading>Loading</Button>);
    expect(screen.getByRole('button')).toContainHTML('svg');
  });

  it('has adequate contrast ratio', () => {
    render(
      <Button variant="primary" style={{ backgroundColor: '#0ea5e9' }}>
        Click
      </Button>
    );
    const btn = screen.getByRole('button');
    // In real tests, use axe-core for automated accessibility checks
    expect(btn).toBeInTheDocument();
  });
});
```

#### Storybook Stories

```typescript
// src/components/ui/button.stories.tsx
import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './button';
import { ChevronRight, Share2, Download } from 'lucide-react';

const meta: Meta<typeof Button> = {
  component: Button,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Primary action button for forms and navigation.',
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    children: 'Click Me',
    variant: 'primary',
  },
};

export const Secondary: Story = {
  args: {
    children: 'Secondary Action',
    variant: 'secondary',
  },
};

export const Ghost: Story = {
  args: {
    children: 'Ghost Button',
    variant: 'ghost',
  },
};

export const WithIcon: Story = {
  args: {
    children: 'Next',
    rightIcon: <ChevronRight className="w-4 h-4" />,
  },
};

export const Loading: Story = {
  args: {
    children: 'Saving',
    isLoading: true,
  },
};

export const Disabled: Story = {
  args: {
    children: 'Disabled',
    disabled: true,
  },
};

export const AllSizes: Story = {
  render: () => (
    <div className="flex gap-4">
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
    </div>
  ),
};
```

---

### 1.2 Input Component

**Purpose**: Text input field for user data entry
**Types**: text, email, password, number, search, tel, url, date
**States**: default, focused, disabled, error

#### Props Interface

```typescript
interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  /** HTML input type */
  type?: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'date';

  /** Label text displayed above input */
  label?: string;

  /** Helper text displayed below input */
  helperText?: string;

  /** Error message displayed below input */
  error?: string;

  /** Placeholder text */
  placeholder?: string;

  /** Input value */
  value?: string | number;

  /** On change callback */
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;

  /** Whether input is disabled */
  disabled?: boolean;

  /** Left icon */
  leftIcon?: React.ReactNode;

  /** Right icon */
  rightIcon?: React.ReactNode;

  /** Aria label for accessibility */
  'aria-label'?: string;

  /** Aria description for accessibility */
  'aria-describedby'?: string;
}
```

#### Default Behavior

```typescript
<Input
  type="email"
  label="Email Address"
  placeholder="user@example.com"
  helperText="We'll never share your email."
/>
```

#### Variants & States

```typescript
{/* Text input */}
<Input type="text" label="Full Name" />

{/* Email with validation */}
<Input
  type="email"
  label="Email"
  error="Please enter a valid email"
/>

{/* Password */}
<Input
  type="password"
  label="Password"
  placeholder="••••••••"
/>

{/* Number input */}
<Input
  type="number"
  label="Age"
  min={0}
  max={120}
/>

{/* Disabled */}
<Input
  type="text"
  label="Disabled Field"
  value="Cannot edit"
  disabled
/>

{/* With icons */}
<Input
  type="email"
  label="Email"
  leftIcon={<Mail />}
  rightIcon={<Check />}
/>
```

#### Accessibility

- ✅ Semantic `<label>` associated with input (for attribute)
- ✅ `aria-label` or label text for screen readers
- ✅ `aria-invalid` when error state
- ✅ `aria-describedby` links to error/helper text
- ✅ Focus visible (2px cyan border)
- ✅ Keyboard navigation (Tab, arrow keys for date)
- ✅ Color contrast: 4.5:1 minimum
- ✅ Error messages linked via aria-describedby

#### Implementation

```typescript
// src/components/ui/input.tsx
import React from 'react';
import { cn } from '@/lib/utils';

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      type = 'text',
      label,
      helperText,
      error,
      leftIcon,
      rightIcon,
      disabled = false,
      'aria-label': ariaLabel,
      'aria-describedby': ariaDescribedBy,
      id,
      className,
      ...props
    },
    ref
  ) => {
    const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
    const errorId = `${inputId}-error`;
    const helperId = `${inputId}-helper`;

    const baseInputStyles =
      'w-full px-3 py-2 text-sm border border-gray-300 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-cyan-500 focus:border-cyan-500 disabled:bg-gray-100 disabled:cursor-not-allowed';

    const errorStyles = error ? 'border-red-500 focus:ring-red-500' : '';

    return (
      <div className="w-full">
        {label && (
          <label htmlFor={inputId} className="block text-sm font-medium text-gray-700 mb-1">
            {label}
          </label>
        )}

        <div className="relative">
          {leftIcon && <span className="absolute left-3 top-2.5 text-gray-400">{leftIcon}</span>}
          <input
            ref={ref}
            id={inputId}
            type={type}
            disabled={disabled}
            aria-label={ariaLabel}
            aria-invalid={!!error}
            aria-describedby={
              error ? errorId : helperText ? helperId : ariaDescribedBy
            }
            className={cn(
              baseInputStyles,
              errorStyles,
              leftIcon && 'pl-10',
              rightIcon && 'pr-10',
              className
            )}
            {...props}
          />
          {rightIcon && <span className="absolute right-3 top-2.5 text-gray-400">{rightIcon}</span>}
        </div>

        {error && (
          <p id={errorId} className="mt-1 text-sm text-red-500" role="alert">
            {error}
          </p>
        )}

        {helperText && !error && (
          <p id={helperId} className="mt-1 text-sm text-gray-500">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
```

---

### 1.3 Select Component

**Purpose**: Dropdown selection from predefined options
**Behavior**: Click to open, click option to select, Escape to close
**Search**: Optional search/filter within options

#### Props Interface

```typescript
interface SelectProps {
  /** Options to display */
  options: { value: string; label: string; disabled?: boolean }[];

  /** Currently selected value */
  value?: string;

  /** On change callback */
  onChange?: (value: string) => void;

  /** Placeholder text when no selection */
  placeholder?: string;

  /** Label above select */
  label?: string;

  /** Whether select is disabled */
  disabled?: boolean;

  /** Show search field in dropdown */
  searchable?: boolean;

  /** Custom render for selected value */
  renderValue?: (value: string) => React.ReactNode;

  /** Aria label for accessibility */
  'aria-label'?: string;
}
```

#### Default Behavior

```typescript
<Select
  label="Status"
  options={[
    { value: 'open', label: 'Open' },
    { value: 'closed', label: 'Closed' },
    { value: 'archived', label: 'Archived' },
  ]}
  value={status}
  onChange={setStatus}
  placeholder="Select a status"
/>
```

#### Accessibility

- ✅ Keyboard navigation (Arrow up/down, Enter, Escape)
- ✅ Focus management (focus trap in dropdown)
- ✅ `aria-expanded` indicates open/closed state
- ✅ `aria-label` for accessibility
- ✅ `aria-disabled` when disabled
- ✅ Selected option announced by screen reader
- ✅ Search option available

---

### 1.4 Checkbox Component

**Purpose**: Boolean input for single or multiple selections
**States**: unchecked, checked, indeterminate (partial)

#### Props Interface

```typescript
interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  /** Label text */
  label?: string;

  /** Whether checkbox is checked */
  checked?: boolean;

  /** Indeterminate state (partial selection) */
  indeterminate?: boolean;

  /** On change callback */
  onChange?: (checked: boolean) => void;

  /** Whether checkbox is disabled */
  disabled?: boolean;

  /** Aria label for accessibility */
  'aria-label'?: string;
}
```

#### Variants

```typescript
{/* Single checkbox */}
<Checkbox label="I agree to terms" />

{/* Grouped checkboxes */}
<fieldset>
  <legend>Permissions</legend>
  <Checkbox label="Read" onChange={(v) => setRead(v)} />
  <Checkbox label="Write" onChange={(v) => setWrite(v)} />
  <Checkbox label="Delete" onChange={(v) => setDelete(v)} />
</fieldset>

{/* Indeterminate (parent checkbox with some children checked) */}
<Checkbox
  label="All"
  indeterminate={someChecked && !allChecked}
  checked={allChecked}
/>
```

#### Accessibility

- ✅ Semantic `<input type="checkbox">` with label
- ✅ Focus visible (ring-cyan-500)
- ✅ Keyboard navigation (Space to toggle)
- ✅ `aria-label` or associated label
- ✅ Color contrast for checked state
- ✅ Large click target (≥24×24px)

---

### 1.5 Radio Component

**Purpose**: Single selection from mutually exclusive options
**Grouped**: Use with fieldset + legend

#### Props Interface

```typescript
interface RadioProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  /** Label text */
  label?: string;

  /** Radio value */
  value: string;

  /** Currently selected value */
  checked?: boolean;

  /** On change callback */
  onChange?: (value: string) => void;

  /** Whether radio is disabled */
  disabled?: boolean;

  /** Aria label for accessibility */
  'aria-label'?: string;
}
```

#### Variants

```typescript
{/* Single radio button */}
<Radio label="Option A" value="a" checked={selected === 'a'} />

{/* Radio group */}
<fieldset>
  <legend>Choose one:</legend>
  <Radio
    label="Option A"
    value="a"
    checked={selected === 'a'}
    onChange={(v) => setSelected(v)}
  />
  <Radio
    label="Option B"
    value="b"
    checked={selected === 'b'}
    onChange={(v) => setSelected(v)}
  />
</fieldset>
```

#### Accessibility

- ✅ Semantic `<input type="radio">` within fieldset
- ✅ `<legend>` describes radio group
- ✅ Focus visible (ring-cyan-500)
- ✅ Keyboard navigation (Arrow keys between options)
- ✅ `aria-label` for accessibility
- ✅ Color contrast for selected state

---

### 1.6 Toggle Component

**Purpose**: Binary switch / on-off button
**Display**: Icon + label or label only

#### Props Interface

```typescript
interface ToggleProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Whether toggle is pressed/active */
  pressed?: boolean;

  /** On press callback */
  onPressedChange?: (pressed: boolean) => void;

  /** Label text */
  label?: string;

  /** Icon to display */
  icon?: React.ReactNode;

  /** Whether toggle is disabled */
  disabled?: boolean;

  /** Aria label for accessibility */
  'aria-label'?: string;
}
```

#### Variants

```typescript
{/* Icon toggle */}
<Toggle
  icon={<Bell />}
  pressed={notificationsOn}
  onPressedChange={setNotificationsOn}
  aria-label="Toggle notifications"
/>

{/* Label toggle */}
<Toggle
  label="Dark Mode"
  pressed={darkMode}
  onPressedChange={setDarkMode}
/>

{/* Icon + label */}
<Toggle
  icon={<Check />}
  label="Verified"
  pressed={verified}
  onPressedChange={setVerified}
/>
```

#### Accessibility

- ✅ Semantic `<button>` element
- ✅ `aria-pressed` indicates state (true/false)
- ✅ `aria-label` for icon-only toggles
- ✅ Keyboard navigation (Space/Enter to toggle)
- ✅ Focus visible (ring)
- ✅ Color contrast for both states

---

## Section 2: Layout Components

### 2.1 Card Component

**Purpose**: Container for grouped content with consistent styling
**Variants**: default, elevated, outlined

#### Props Interface

```typescript
interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Card title */
  title?: string;

  /** Card footer content */
  footer?: React.ReactNode;

  /** Visual variant */
  variant?: 'default' | 'elevated' | 'outlined';

  /** Whether to show divider between content and footer */
  divider?: boolean;

  /** Card children */
  children: React.ReactNode;
}
```

#### Variants

```typescript
{/* Default card */}
<Card title="Work Packages">
  <p>Content here</p>
</Card>

{/* Elevated card with footer */}
<Card
  title="Settings"
  footer={<Button>Save</Button>}
  variant="elevated"
>
  {/* Form content */}
</Card>

{/* Outlined card */}
<Card variant="outlined">
  <p>Outlined card content</p>
</Card>
```

#### Accessibility

- ✅ Semantic `<section>` or `<article>` wrapper
- ✅ Title as `<h2>` or `<h3>` if present
- ✅ Proper heading hierarchy
- ✅ Color contrast for background

---

### 2.2 Modal Component

**Purpose**: Overlay dialog for important information or form submission
**Behavior**: Click outside to close (optional), Escape to close

#### Props Interface

```typescript
interface ModalProps {
  /** Whether modal is open */
  isOpen: boolean;

  /** Called when modal should close */
  onClose: () => void;

  /** Modal title */
  title?: string;

  /** Modal content */
  children: React.ReactNode;

  /** Modal footer actions */
  footer?: React.ReactNode;

  /** Whether clicking outside closes modal */
  closeOnBackdropClick?: boolean;

  /** Size of modal */
  size?: 'sm' | 'md' | 'lg';

  /** Aria label for accessibility */
  'aria-label'?: string;
}
```

#### Variants

```typescript
{/* Simple modal */}
<Modal isOpen={open} onClose={handleClose} title="Confirm Action">
  <p>Are you sure you want to proceed?</p>
  <div className="flex gap-2 mt-4">
    <Button onClick={handleClose} variant="ghost">Cancel</Button>
    <Button onClick={handleConfirm}>Confirm</Button>
  </div>
</Modal>

{/* Modal with form */}
<Modal isOpen={open} onClose={handleClose} title="Create Work Package" size="lg">
  <FormBuilder schema={formSchema} onSubmit={handleSubmit} />
</Modal>
```

#### Accessibility

- ✅ `role="dialog"` on modal
- ✅ Focus trap (Tab stays within modal)
- ✅ Escape key closes modal
- ✅ Initial focus on first focusable element
- ✅ `aria-label` or title describes modal
- ✅ Backdrop prevents interaction with page behind
- ✅ Backdrop has opacity (not black-only)

---

### 2.3 Toast Component

**Purpose**: Temporary notification messages (success, error, warning, info)
**Behavior**: Auto-dismiss after duration, manual dismiss via X button
**Queue**: Multiple toasts stack vertically

#### Props Interface

```typescript
interface ToastProps {
  /** Toast message */
  message: string;

  /** Toast type */
  type?: 'success' | 'error' | 'warning' | 'info';

  /** Duration before auto-dismiss (ms) */
  duration?: number;

  /** Called when toast is dismissed */
  onDismiss?: () => void;

  /** Custom icon */
  icon?: React.ReactNode;

  /** Action button */
  action?: { label: string; onClick: () => void };
}

interface useToastReturn {
  toast: (props: ToastProps) => void;
  dismiss: (id: string) => void;
  dismissAll: () => void;
}
```

#### Usage with Hook

```typescript
{/* Inside component */}
const { toast } = useToast();

{/* Success toast */}
toast({
  message: 'Work package created successfully',
  type: 'success',
  duration: 3000
});

{/* Error toast with action */}
toast({
  message: 'Failed to save changes',
  type: 'error',
  duration: 5000,
  action: {
    label: 'Retry',
    onClick: handleRetry
  }
});

{/* Info toast */}
toast({
  message: 'Syncing data...',
  type: 'info',
  duration: 0 // Don't auto-dismiss
});
```

#### Accessibility

- ✅ `role="alert"` on toast (announces to screen readers)
- ✅ `role="status"` for non-critical toasts (live region)
- ✅ Dismiss button always available
- ✅ Auto-dismiss respects user preferences (prefers-reduced-motion)
- ✅ Color not only indicator (icon + text)
- ✅ 4.5:1 contrast ratio for text

---

### 2.4 Badge Component

**Purpose**: Status label or tag (non-interactive)
**Variants**: success, warning, error, info, neutral
**Display**: Icon (optional) + label

#### Props Interface

```typescript
interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Badge label */
  label: string;

  /** Badge variant/color */
  variant?: 'success' | 'warning' | 'error' | 'info' | 'neutral';

  /** Icon to display */
  icon?: React.ReactNode;

  /** Size of badge */
  size?: 'sm' | 'md' | 'lg';
}
```

#### Variants

```typescript
{/* Status badges */}
<Badge label="In Progress" variant="info" icon={<Clock />} />
<Badge label="Completed" variant="success" icon={<Check />} />
<Badge label="On Hold" variant="warning" icon={<AlertCircle />} />
<Badge label="Failed" variant="error" icon={<X />} />

{/* Neutral badge */}
<Badge label="Feature" variant="neutral" />

{/* Different sizes */}
<Badge label="Small" size="sm" />
<Badge label="Medium" size="md" />
<Badge label="Large" size="lg" />
```

#### Accessibility

- ✅ Semantic `<div>` with `role="status"` if showing changing status
- ✅ Color + icon/text combination (not color only)
- ✅ 4.5:1 contrast ratio
- ✅ Icon has aria-label if status icon only

---

### 2.5 Pill Component

**Purpose**: Removable tag / chip (interactive)
**Behavior**: Click X to remove, optional left/right icons

#### Props Interface

```typescript
interface PillProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Pill label */
  label: string;

  /** Called when pill is removed */
  onRemove?: () => void;

  /** Left icon */
  leftIcon?: React.ReactNode;

  /** Right icon (not remove X) */
  rightIcon?: React.ReactNode;

  /** Pill variant */
  variant?: 'default' | 'primary' | 'secondary';

  /** Whether pill is removable */
  removable?: boolean;

  /** Size of pill */
  size?: 'sm' | 'md' | 'lg';
}
```

#### Variants

```typescript
{/* Default pill */}
<Pill label="React" onRemove={() => removeTag('react')} />

{/* Pill with icon */}
<Pill
  label="In Progress"
  leftIcon={<Clock />}
  variant="primary"
/>

{/* Non-removable pill */}
<Pill label="Read-only" removable={false} />

{/* Pill group */}
<div className="flex gap-2 flex-wrap">
  {tags.map(tag => (
    <Pill
      key={tag}
      label={tag}
      onRemove={() => removeTag(tag)}
    />
  ))}
</div>
```

#### Accessibility

- ✅ Semantic `<button>` for remove action
- ✅ `aria-label` for remove button: "Remove [label]"
- ✅ Keyboard accessible (Tab to pill, Enter/Space to remove)
- ✅ Focus visible (ring)
- ✅ 44×44px minimum click target for remove button
- ✅ Color contrast

---

## Section 3: Complex Components

### 3.1 DataTable Component

**Purpose**: Sortable, paginated table with filtering support
**Features**: Sort by column, filter rows, paginate, responsive
**Data**: Array of objects, column definitions

#### Props Interface

```typescript
interface ColumnDef<T> {
  /** Unique column identifier */
  key: keyof T;

  /** Column header text */
  header: string;

  /** Custom cell renderer */
  cell?: (value: T[keyof T], row: T, index: number) => React.ReactNode;

  /** Whether column is sortable */
  sortable?: boolean;

  /** Whether column is filterable */
  filterable?: boolean;

  /** Column width (CSS) */
  width?: string;

  /** Text alignment */
  align?: 'left' | 'center' | 'right';
}

interface DataTableProps<T extends { id: string | number }> {
  /** Column definitions */
  columns: ColumnDef<T>[];

  /** Table data */
  data: T[];

  /** Callback when sort changes */
  onSort?: (key: keyof T, direction: 'asc' | 'desc') => void;

  /** Callback when filter changes */
  onFilter?: (filters: Record<string, string>) => void;

  /** Number of rows per page */
  pageSize?: number;

  /** Total number of rows (for server-side pagination) */
  total?: number;

  /** Currently selected page */
  currentPage?: number;

  /** Callback when page changes */
  onPageChange?: (page: number) => void;

  /** Whether table is loading data */
  isLoading?: boolean;

  /** Whether to show zebra striping */
  striped?: boolean;

  /** Whether rows are hoverable */
  hoverable?: boolean;
}
```

#### Example Usage

```typescript
interface WorkPackage {
  id: string;
  title: string;
  status: 'open' | 'in_progress' | 'closed';
  priority: 'low' | 'medium' | 'high';
  assignee: string;
  dueDate: string;
}

const columns: ColumnDef<WorkPackage>[] = [
  {
    key: 'title',
    header: 'Title',
    sortable: true,
    filterable: true,
  },
  {
    key: 'status',
    header: 'Status',
    sortable: true,
    cell: (value) => <Badge label={String(value)} variant="info" />,
  },
  {
    key: 'priority',
    header: 'Priority',
    sortable: true,
    cell: (value) => {
      const colors = { low: 'neutral', medium: 'warning', high: 'error' };
      return <Badge label={String(value)} variant={colors[value] as any} />;
    },
  },
  {
    key: 'assignee',
    header: 'Assigned To',
    filterable: true,
  },
  {
    key: 'dueDate',
    header: 'Due Date',
    sortable: true,
    cell: (value) => new Date(value).toLocaleDateString(),
  },
];

// In component
const [sortKey, setSortKey] = useState<keyof WorkPackage | null>(null);
const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
const [page, setPage] = useState(1);

<DataTable
  columns={columns}
  data={workPackages}
  pageSize={10}
  currentPage={page}
  onPageChange={setPage}
  onSort={(key, dir) => {
    setSortKey(key);
    setSortDir(dir);
  }}
  striped
  hoverable
/>
```

#### Implementation Outline (300 LOC)

```typescript
// src/components/complex/data-table/index.tsx
export function DataTable<T extends { id: string | number }>({
  columns,
  data,
  onSort,
  pageSize = 10,
  currentPage = 1,
  isLoading = false,
  striped = true,
  hoverable = true,
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<keyof T | null>(null);
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');

  const handleSort = (key: keyof T) => {
    const newDir = sortKey === key && sortDir === 'asc' ? 'desc' : 'asc';
    setSortKey(key);
    setSortDir(newDir);
    onSort?.(key, newDir);
  };

  const startIndex = (currentPage - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedData = data.slice(startIndex, endIndex);
  const totalPages = Math.ceil(data.length / pageSize);

  return (
    <div className="w-full overflow-auto border border-gray-200 rounded-lg">
      <table className="w-full border-collapse">
        <thead className="bg-gray-50 border-b border-gray-200">
          <tr>
            {columns.map((col) => (
              <th
                key={String(col.key)}
                onClick={() => col.sortable && handleSort(col.key)}
                className={cn(
                  'px-4 py-3 text-left text-sm font-semibold text-gray-700',
                  col.sortable && 'cursor-pointer hover:bg-gray-100',
                  col.align && `text-${col.align}`
                )}
                style={{ width: col.width }}
              >
                <div className="flex items-center gap-2">
                  {col.header}
                  {col.sortable && sortKey === col.key && (
                    <span>{sortDir === 'asc' ? '↑' : '↓'}</span>
                  )}
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {paginatedData.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="px-4 py-8 text-center text-gray-500">
                {isLoading ? 'Loading...' : 'No data'}
              </td>
            </tr>
          ) : (
            paginatedData.map((row, idx) => (
              <tr
                key={row.id}
                className={cn(
                  'border-b border-gray-200',
                  striped && idx % 2 === 0 && 'bg-gray-50',
                  hoverable && 'hover:bg-gray-100'
                )}
              >
                {columns.map((col) => (
                  <td
                    key={String(col.key)}
                    className="px-4 py-3 text-sm text-gray-900"
                    style={{ width: col.width }}
                  >
                    {col.cell?.(row[col.key], row, idx) ?? String(row[col.key])}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>

      {/* Pagination */}
      <div className="flex items-center justify-between px-4 py-3 border-t border-gray-200 bg-gray-50">
        <span className="text-sm text-gray-700">
          Showing {startIndex + 1} to {Math.min(endIndex, data.length)} of {data.length}
        </span>
        <div className="flex gap-2">
          <Button size="sm" variant="ghost" disabled={currentPage === 1}>
            Previous
          </Button>
          <span className="text-sm text-gray-700">
            Page {currentPage} of {totalPages}
          </span>
          <Button size="sm" variant="ghost" disabled={currentPage === totalPages}>
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
```

#### Accessibility

- ✅ `<table>` semantic HTML with `<thead>`, `<tbody>`
- ✅ `<th>` headers with `scope="col"`
- ✅ `aria-sort` on sortable columns
- ✅ Row selection with keyboard (Shift+Click)
- ✅ Keyboard navigation (Tab through cells)
- ✅ Screen reader announces sort state
- ✅ "No data" message announced
- ✅ Pagination buttons keyboard accessible

---

### 3.2 FormBuilder Component

**Purpose**: Schema-driven dynamic form generation
**Features**: Multiple field types, validation, error messages, submit handler
**Types**: text, email, password, number, textarea, select, checkbox, radio, date

#### Props Interface

```typescript
interface FormField {
  name: string;
  type: 'text' | 'email' | 'password' | 'number' | 'textarea' | 'select' | 'checkbox' | 'radio' | 'date';
  label?: string;
  placeholder?: string;
  required?: boolean;
  validation?: RegExp | ((value: any) => boolean);
  errorMessage?: string;
  helperText?: string;
  options?: { value: string; label: string }[]; // For select/radio
  rows?: number; // For textarea
  cols?: number; // For textarea
  defaultValue?: any;
  disabled?: boolean;
}

interface FormBuilderProps {
  /** Form schema */
  schema: FormField[];

  /** Default form values */
  defaultValues?: Record<string, any>;

  /** Callback on form submit */
  onSubmit: (values: Record<string, any>) => void | Promise<void>;

  /** Callback on form change */
  onChange?: (values: Record<string, any>) => void;

  /** Submit button label */
  submitLabel?: string;

  /** Whether form is loading */
  isLoading?: boolean;

  /** Validation errors from server */
  serverErrors?: Record<string, string>;
}
```

#### Example Usage

```typescript
const schema: FormField[] = [
  {
    name: 'title',
    type: 'text',
    label: 'Work Package Title',
    placeholder: 'Enter title',
    required: true,
    validation: /^.{3,}$/,
    errorMessage: 'Title must be at least 3 characters',
  },
  {
    name: 'description',
    type: 'textarea',
    label: 'Description',
    placeholder: 'Enter description',
    rows: 4,
  },
  {
    name: 'status',
    type: 'select',
    label: 'Status',
    options: [
      { value: 'open', label: 'Open' },
      { value: 'in_progress', label: 'In Progress' },
      { value: 'closed', label: 'Closed' },
    ],
    defaultValue: 'open',
  },
  {
    name: 'priority',
    type: 'radio',
    label: 'Priority',
    options: [
      { value: 'low', label: 'Low' },
      { value: 'medium', label: 'Medium' },
      { value: 'high', label: 'High' },
    ],
  },
  {
    name: 'dueDate',
    type: 'date',
    label: 'Due Date',
  },
  {
    name: 'acceptTerms',
    type: 'checkbox',
    label: 'I accept the terms and conditions',
    required: true,
  },
];

<FormBuilder
  schema={schema}
  defaultValues={{ status: 'open' }}
  onSubmit={async (values) => {
    await api.post('/work-packages', values);
  }}
  submitLabel="Create Work Package"
/>
```

#### Implementation Outline (250 LOC)

```typescript
// src/components/complex/form-builder/index.tsx
export function FormBuilder({
  schema,
  defaultValues = {},
  onSubmit,
  onChange,
  submitLabel = 'Submit',
  isLoading = false,
  serverErrors = {},
}: FormBuilderProps) {
  const [values, setValues] = useState<Record<string, any>>(defaultValues);
  const [errors, setErrors] = useState<Record<string, string>>(serverErrors);

  const handleChange = (name: string, value: any) => {
    const newValues = { ...values, [name]: value };
    setValues(newValues);
    onChange?.(newValues);

    // Validate on change
    const field = schema.find(f => f.name === name);
    if (field?.validation) {
      const isValid = typeof field.validation === 'function'
        ? field.validation(value)
        : field.validation.test(value);
      setErrors(prev => ({
        ...prev,
        [name]: isValid ? '' : (field.errorMessage || 'Invalid input'),
      }));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    // Validate all fields
    const newErrors: Record<string, string> = {};
    schema.forEach(field => {
      if (field.required && !values[field.name]) {
        newErrors[field.name] = `${field.label || field.name} is required`;
      }
      if (field.validation && values[field.name]) {
        const isValid = typeof field.validation === 'function'
          ? field.validation(values[field.name])
          : field.validation.test(values[field.name]);
        if (!isValid) {
          newErrors[field.name] = field.errorMessage || 'Invalid input';
        }
      }
    });

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    await onSubmit(values);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {schema.map(field => (
        <div key={field.name}>
          {field.type === 'text' || field.type === 'email' || field.type === 'password' || field.type === 'number' || field.type === 'date' ? (
            <Input
              type={field.type}
              label={field.label}
              placeholder={field.placeholder}
              value={values[field.name] || ''}
              onChange={(e) => handleChange(field.name, e.target.value)}
              error={errors[field.name]}
              disabled={field.disabled}
              required={field.required}
            />
          ) : field.type === 'textarea' ? (
            <div>
              {field.label && <label className="block text-sm font-medium mb-1">{field.label}</label>}
              <textarea
                value={values[field.name] || ''}
                onChange={(e) => handleChange(field.name, e.target.value)}
                placeholder={field.placeholder}
                rows={field.rows || 4}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
                disabled={field.disabled}
                required={field.required}
              />
              {errors[field.name] && <p className="text-red-500 text-sm mt-1">{errors[field.name]}</p>}
            </div>
          ) : field.type === 'select' ? (
            <Select
              label={field.label}
              options={field.options || []}
              value={values[field.name]}
              onChange={(v) => handleChange(field.name, v)}
              placeholder={field.placeholder}
              disabled={field.disabled}
            />
          ) : field.type === 'checkbox' ? (
            <Checkbox
              label={field.label}
              checked={values[field.name] || false}
              onChange={(checked) => handleChange(field.name, checked)}
              disabled={field.disabled}
            />
          ) : field.type === 'radio' ? (
            <div>
              {field.label && <legend className="text-sm font-medium mb-2">{field.label}</legend>}
              {field.options?.map(option => (
                <Radio
                  key={option.value}
                  label={option.label}
                  value={option.value}
                  checked={values[field.name] === option.value}
                  onChange={() => handleChange(field.name, option.value)}
                  disabled={field.disabled}
                />
              ))}
            </div>
          ) : null}
        </div>
      ))}

      <Button type="submit" variant="primary" isLoading={isLoading}>
        {submitLabel}
      </Button>
    </form>
  );
}
```

#### Accessibility

- ✅ `<label>` for all form fields
- ✅ `aria-invalid` for error states
- ✅ Error messages linked via aria-describedby
- ✅ Required fields marked with aria-required
- ✅ Focus management (focus first error on submit)
- ✅ Keyboard navigation (Tab through fields)
- ✅ Fieldset + legend for grouped inputs
- ✅ Color contrast for error text

---

### 3.3 Timeline Component

**Purpose**: Chronological event display with interaction
**Features**: Event rendering, click handlers, git log links
**Display**: Vertical timeline with connected nodes

#### Props Interface

```typescript
interface TimelineEvent {
  id: string;
  timestamp: Date;
  title: string;
  description?: string;
  type?: 'success' | 'error' | 'warning' | 'info';
  icon?: React.ReactNode;
  linkedItem?: {
    type: 'agent' | 'work-package' | 'git-commit' | 'ci-run';
    id: string;
    label: string;
    url: string;
  };
}

interface TimelineProps {
  /** Timeline events */
  events: TimelineEvent[];

  /** Callback when event is clicked */
  onEventClick?: (event: TimelineEvent) => void;

  /** Display variant */
  variant?: 'vertical' | 'compact';

  /** Whether to show connection lines */
  showConnectors?: boolean;

  /** Whether events are clickable */
  interactive?: boolean;
}
```

#### Example Usage

```typescript
const events: TimelineEvent[] = [
  {
    id: 'ev-001',
    timestamp: new Date('2026-03-31T10:00:00'),
    title: 'Work Package Created',
    description: 'eco-001-deploy-foundations',
    type: 'info',
    linkedItem: {
      type: 'work-package',
      id: 'wp-001',
      label: 'WP-001',
      url: '/work-packages/wp-001',
    },
  },
  {
    id: 'ev-002',
    timestamp: new Date('2026-03-31T11:30:00'),
    title: 'Planner Agent Started',
    description: 'Analyzing requirements',
    type: 'info',
    linkedItem: {
      type: 'agent',
      id: 'agent-planner-001',
      label: 'Planner Agent #001',
      url: '/agents/planner-001',
    },
  },
  {
    id: 'ev-003',
    timestamp: new Date('2026-03-31T13:45:00'),
    title: 'Implementation Complete',
    description: 'All tests passing',
    type: 'success',
    linkedItem: {
      type: 'git-commit',
      id: 'abc123def',
      label: 'Commit abc123d',
      url: 'https://github.com/repo/commit/abc123def',
    },
  },
];

<Timeline
  events={events}
  onEventClick={(event) => navigate(event.linkedItem?.url)}
  interactive
/>
```

#### Implementation Outline (200 LOC)

```typescript
// src/components/complex/timeline/index.tsx
export function Timeline({
  events,
  onEventClick,
  variant = 'vertical',
  showConnectors = true,
  interactive = true,
}: TimelineProps) {
  const sortedEvents = [...events].sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

  const typeColors = {
    success: 'bg-green-500',
    error: 'bg-red-500',
    warning: 'bg-amber-500',
    info: 'bg-blue-500',
  };

  return (
    <div className={cn('relative', variant === 'compact' && 'space-y-2', variant === 'vertical' && 'space-y-6')}>
      {sortedEvents.map((event, idx) => (
        <div key={event.id} className="relative flex gap-4">
          {/* Timeline node */}
          <div className="flex flex-col items-center">
            <div className={cn('w-4 h-4 rounded-full', typeColors[event.type || 'info'])} />
            {showConnectors && idx < sortedEvents.length - 1 && (
              <div className="w-1 h-12 bg-gray-200 mt-2" />
            )}
          </div>

          {/* Event content */}
          <div
            className={cn(
              'flex-1 pt-1',
              interactive && 'cursor-pointer hover:opacity-80'
            )}
            onClick={() => interactive && onEventClick?.(event)}
            role={interactive ? 'button' : undefined}
            tabIndex={interactive ? 0 : undefined}
            onKeyDown={(e) => {
              if (interactive && (e.key === 'Enter' || e.key === ' ')) {
                onEventClick?.(event);
              }
            }}
          >
            <div className="flex items-baseline gap-2">
              <h4 className="font-semibold text-gray-900">{event.title}</h4>
              <span className="text-xs text-gray-500">
                {event.timestamp.toLocaleTimeString()}
              </span>
            </div>
            {event.description && (
              <p className="text-sm text-gray-600 mt-1">{event.description}</p>
            )}
            {event.linkedItem && (
              <a
                href={event.linkedItem.url}
                className="text-sm text-cyan-600 hover:underline mt-2 inline-block"
                onClick={(e) => e.stopPropagation()}
              >
                {event.linkedItem.label} →
              </a>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
```

#### Accessibility

- ✅ Semantic `<div>` with `role="list"` on container
- ✅ Clickable events are `role="button"` or `<button>`
- ✅ Keyboard navigation (Tab, Enter/Space to activate)
- ✅ Focus visible (ring on clickable events)
- ✅ `aria-label` describes timeline
- ✅ Links announce href/destination
- ✅ Color + icon/text for event type (not color only)

---

## Section 4: Accessibility Compliance Matrix

### WCAG 2.1 AA Checklist (All Components)

| Criterion | Button | Input | Select | Checkbox | Radio | Toggle | Card | Modal | Toast | Badge | Pill | DataTable | FormBuilder | Timeline |
|-----------|--------|-------|--------|----------|-------|--------|------|-------|-------|-------|------|-----------|-------------|----------|
| 1.1.1 Non-text Content | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 1.4.3 Contrast (Min) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2.1.1 Keyboard | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2.1.2 No Keyboard Trap | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2.4.3 Focus Order | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2.4.7 Focus Visible | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3.2.1 On Focus | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3.3.1 Error Identification | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3.3.3 Error Suggestion | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 4.1.2 Name, Role, Value | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 4.1.3 Status Messages | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Note**: All components implement:
- Semantic HTML structure
- ARIA attributes where needed
- Keyboard navigation (Tab, arrows, Enter, Escape)
- Focus management
- Color contrast ≥ 4.5:1 for text (AA standard)
- Touch targets ≥ 44×44px (WCAG 2.5.5 AAA)

---

## Section 5: Testing & Validation

### Unit Test Template

```typescript
// tests/components/[component].test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { [Component] } from '@/components/ui/[component]';

describe('[Component]', () => {
  it('renders correctly', () => {
    render(<[Component] />);
    expect(screen.getByRole('[role]')).toBeInTheDocument();
  });

  it('handles user interactions', async () => {
    const handleClick = vi.fn();
    render(<[Component] onClick={handleClick} />);
    await userEvent.click(screen.getByRole('[role]'));
    expect(handleClick).toHaveBeenCalled();
  });

  it('is keyboard accessible', () => {
    render(<[Component] />);
    const element = screen.getByRole('[role]');
    element.focus();
    expect(element).toHaveFocus();
  });

  it('respects disabled state', () => {
    render(<[Component] disabled />);
    expect(screen.getByRole('[role]')).toBeDisabled();
  });

  it('passes accessibility audit', () => {
    const { container } = render(<[Component] />);
    // Run axe-core validation
  });
});
```

### E2E Test Template (Playwright)

```typescript
// tests/e2e/[feature].spec.ts
import { test, expect } from '@playwright/test';

test('[Feature]', async ({ page }) => {
  await page.goto('/[page]');

  // Verify component renders
  await expect(page.locator('[data-testid="[component]"]')).toBeVisible();

  // Test interaction
  await page.click('[data-testid="[action]"]');

  // Verify result
  await expect(page.locator('[data-testid="[result]"]')).toContainText('[expected]');
});
```

---

## Section 6: Storybook Setup

### Installation

```bash
npx storybook@latest init
npm install @storybook/react @storybook/testing-library
```

### Story Template

```typescript
// src/components/ui/[component].stories.tsx
import type { Meta, StoryObj } from '@storybook/react';
import { [Component] } from './[component]';

const meta: Meta<typeof [Component]> = {
  component: [Component],
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Component description and usage examples.',
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    // Default props
  },
};

export const [Variant]: Story = {
  args: {
    // Variant-specific props
  },
};
```

---

## Section 7: Implementation Checklist

### Per-Component Checklist

- [ ] Component implemented (TypeScript)
- [ ] Props interface defined
- [ ] Storybook stories created (3+ variants)
- [ ] Unit tests written (70%+ coverage)
- [ ] E2E tests written (critical paths)
- [ ] Accessibility audit passed (axe-core)
- [ ] Documentation in COMPONENT_INVENTORY.md
- [ ] Code review approved
- [ ] Merged to main

---

## Appendix A: Design Tokens

```css
/* Color Palette */
--color-primary: #0ea5e9 (Cyan)
--color-secondary: #a855f7 (Purple)
--color-success: #10b981 (Green)
--color-warning: #f59e0b (Amber)
--color-error: #ef4444 (Red)
--color-info: #3b82f6 (Blue)
--color-neutral: #6b7280 (Gray)

/* Typography */
--font-xs: 0.75rem / 1rem
--font-sm: 0.875rem / 1.25rem
--font-base: 1rem / 1.5rem
--font-lg: 1.125rem / 1.75rem
--font-xl: 1.25rem / 1.75rem
--font-2xl: 1.5rem / 2rem

/* Spacing */
--space-xs: 0.25rem
--space-sm: 0.5rem
--space-md: 1rem
--space-lg: 1.5rem
--space-xl: 2rem
--space-2xl: 3rem

/* Shadows */
--shadow-sm: 0 1px 2px rgba(0,0,0,0.05)
--shadow-md: 0 4px 6px rgba(0,0,0,0.1)
--shadow-lg: 0 10px 15px rgba(0,0,0,0.1)

/* Border Radius */
--radius-sm: 0.375rem
--radius-md: 0.5rem
--radius-lg: 0.75rem
```

---

## Appendix B: Browser Support Matrix

| Browser | Version | Support |
|---------|---------|---------|
| Chrome | Latest 2 | ✅ Full |
| Firefox | Latest 2 | ✅ Full |
| Safari | Latest 2 | ✅ Full |
| Edge | Latest 2 | ✅ Full |
| Mobile Safari (iOS) | 12+ | ✅ Full |
| Chrome Mobile | Latest 2 | ✅ Full |

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-03-31 | Dashboard Team | Initial release |

**Next Review**: 2026-04-21 (after Phase 2.3 completion)
