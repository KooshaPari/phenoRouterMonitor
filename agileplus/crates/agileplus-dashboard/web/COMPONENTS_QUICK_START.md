# AgilePlus Dashboard Components — Quick Start Guide

Welcome to the AgilePlus Dashboard React component library. This guide helps you get started with the 11 production-ready foundation and layout components.

---

## Installation & Setup

### 1. Install Dependencies

```bash
cd web/
npm install
```

### 2. Run Development Server

```bash
npm run dev
# Open http://localhost:5173
```

### 3. Run Tests

```bash
npm run test              # Run all tests
npm run test:ui          # Open Vitest UI
npm run test:coverage    # Generate coverage report
```

### 4. Build for Production

```bash
npm run build            # Build & optimize
npm run preview          # Preview production build
```

---

## Component Usage

### Foundation Components

#### Button

```typescript
import { Button } from '@/components';

export function MyComponent() {
  return (
    <>
      <Button onClick={() => alert('Clicked!')}>
        Primary Button
      </Button>

      <Button variant="secondary" size="lg">
        Large Secondary
      </Button>

      <Button variant="destructive" disabled>
        Delete (Disabled)
      </Button>

      <Button variant="ghost" size="sm">
        Ghost Small
      </Button>
    </>
  );
}
```

**Props:**
- `variant`: 'primary' | 'secondary' | 'ghost' | 'destructive'
- `size`: 'sm' | 'md' | 'lg'
- `disabled`: boolean
- `onClick`: (e) => void
- `type`: 'button' | 'submit' | 'reset'
- `ariaLabel`: string (for icon buttons)

---

#### Input

```typescript
import { Input } from '@/components';
import { useState } from 'react';

export function LoginForm() {
  const [email, setEmail] = useState('');
  const [error, setError] = useState('');

  const handleSubmit = () => {
    if (!email.includes('@')) {
      setError('Invalid email');
      return;
    }
    setError('');
    // Submit logic
  };

  return (
    <>
      <Input
        type="email"
        label="Email Address"
        placeholder="user@example.com"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        error={error}
        required
      />
      <Button type="submit" onClick={handleSubmit}>
        Sign In
      </Button>
    </>
  );
}
```

**Props:**
- `type`: 'text' | 'email' | 'password' | 'number' | 'date' | 'time'
- `label`: string
- `placeholder`: string
- `value`: string
- `onChange`: (e) => void
- `error`: string (displays error message)
- `required`: boolean

---

#### Select

```typescript
import { Select } from '@/components';
import { useState } from 'react';

export function StatusFilter() {
  const [status, setStatus] = useState<string>('');

  return (
    <Select
      label="Status"
      placeholder="Select status..."
      value={status}
      onChange={(value) => setStatus(value as string)}
      options={[
        { value: 'open', label: 'Open' },
        { value: 'in_progress', label: 'In Progress' },
        { value: 'closed', label: 'Closed' },
        { value: 'blocked', label: 'Blocked', disabled: true },
      ]}
    />
  );
}
```

**Props:**
- `options`: { value, label, disabled? }[]
- `value`: string | number
- `onChange`: (value) => void
- `placeholder`: string
- `label`: string

---

#### Checkbox

```typescript
import { Checkbox } from '@/components';
import { useState } from 'react';

export function TermsCheckbox() {
  const [agreed, setAgreed] = useState(false);

  return (
    <Checkbox
      label="I agree to the terms and conditions"
      checked={agreed}
      onChange={setAgreed}
      required
    />
  );
}
```

**Props:**
- `checked`: boolean
- `onChange`: (checked) => void
- `label`: string
- `required`: boolean
- `disabled`: boolean

---

#### Radio

```typescript
import { Radio } from '@/components';
import { useState } from 'react';

export function ThemeSelector() {
  const [theme, setTheme] = useState('light');

  return (
    <div role="radiogroup" aria-label="Theme">
      <Radio
        value="light"
        label="Light"
        checked={theme === 'light'}
        onChange={setTheme}
      />
      <Radio
        value="dark"
        label="Dark"
        checked={theme === 'dark'}
        onChange={setTheme}
      />
      <Radio
        value="auto"
        label="Auto"
        checked={theme === 'auto'}
        onChange={setTheme}
      />
    </div>
  );
}
```

**Props:**
- `value`: string
- `checked`: boolean
- `onChange`: (value) => void
- `label`: string

---

#### Toggle

```typescript
import { Toggle } from '@/components';
import { useState } from 'react';
import { Moon } from 'lucide-react';

export function DarkModeToggle() {
  const [darkMode, setDarkMode] = useState(false);

  return (
    <Toggle
      checked={darkMode}
      onChange={setDarkMode}
      label="Dark mode"
      icon={<Moon size={16} />}
    />
  );
}
```

**Props:**
- `checked`: boolean
- `onChange`: (checked) => void
- `label`: string
- `icon`: ReactNode
- `disabled`: boolean

---

### Layout Components

#### Card

```typescript
import { Card, Button } from '@/components';

export function UserProfile() {
  return (
    <Card
      title="User Profile"
      footer={<Button>Edit Profile</Button>}
      variant="default"
    >
      <p>Name: John Doe</p>
      <p>Email: john@example.com</p>
    </Card>
  );
}
```

**Props:**
- `title`: string
- `children`: ReactNode
- `footer`: ReactNode
- `variant`: 'default' | 'elevated' | 'outlined'

---

#### Modal

```typescript
import { Modal, Button } from '@/components';
import { useState } from 'react';

export function ConfirmDialog() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button onClick={() => setOpen(true)}>Delete</Button>

      <Modal
        isOpen={open}
        onClose={() => setOpen(false)}
        title="Confirm Delete"
        footer={
          <>
            <Button variant="ghost" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => {
              // Handle delete
              setOpen(false);
            }}>
              Delete
            </Button>
          </>
        }
      >
        <p>Are you sure? This action cannot be undone.</p>
      </Modal>
    </>
  );
}
```

**Props:**
- `isOpen`: boolean
- `onClose`: () => void
- `title`: string
- `children`: ReactNode
- `footer`: ReactNode
- `size`: 'sm' | 'md' | 'lg'

---

#### Toast

```typescript
import { Toast } from '@/components';
import { useState } from 'react';

export function NotificationExample() {
  const [visible, setVisible] = useState(false);

  return (
    <>
      <Button onClick={() => setVisible(true)}>Show Toast</Button>

      {visible && (
        <Toast
          type="success"
          message="Changes saved successfully!"
          duration={3000}
          onClose={() => setVisible(false)}
        />
      )}
    </>
  );
}
```

**Props:**
- `type`: 'success' | 'error' | 'warning' | 'info'
- `message`: string
- `duration`: number (ms, 0 = no auto-dismiss)
- `onClose`: () => void

---

#### Badge

```typescript
import { Badge } from '@/components';
import { Check } from 'lucide-react';

export function StatusBadge() {
  return (
    <>
      <Badge label="Active" variant="success" icon={<Check size={12} />} />
      <Badge label="Pending" variant="warning" />
      <Badge label="Error" variant="error" />
    </>
  );
}
```

**Props:**
- `label`: string
- `variant`: 'default' | 'success' | 'warning' | 'error' | 'info'
- `icon`: ReactNode

---

#### Pill

```typescript
import { Pill } from '@/components';
import { useState } from 'react';

export function TagManager() {
  const [tags, setTags] = useState(['bug', 'feature', 'high-priority']);

  const removeTag = (tag: string) => {
    setTags(tags.filter((t) => t !== tag));
  };

  return (
    <div className="flex flex-wrap gap-2">
      {tags.map((tag) => (
        <Pill
          key={tag}
          label={tag}
          variant="primary"
          onRemove={() => removeTag(tag)}
        />
      ))}
    </div>
  );
}
```

**Props:**
- `label`: string
- `onRemove`: () => void
- `variant`: 'default' | 'primary' | 'secondary'

---

## State Management

### Using Zustand Store

```typescript
import { useAgilePlusStore } from '@/stores/agileplus';
import { useEffect } from 'react';

export function WorkPackageList() {
  const workPackages = useAgilePlusStore((state) => state.workPackages);
  const selectWP = useAgilePlusStore((state) => state.selectWorkPackage);

  return (
    <div>
      {workPackages.map((wp) => (
        <button
          key={wp.id}
          onClick={() => selectWP(wp.id)}
          className="block w-full text-left p-2 hover:bg-gray-100"
        >
          {wp.title} — {wp.status}
        </button>
      ))}
    </div>
  );
}
```

### Using Custom Hooks

```typescript
import { useWorkPackages } from '@/hooks/useWorkPackages';

export function Dashboard() {
  const { workPackages, loading } = useWorkPackages();

  if (loading) return <p>Loading...</p>;

  return (
    <ul>
      {workPackages.map((wp) => (
        <li key={wp.id}>{wp.title}</li>
      ))}
    </ul>
  );
}
```

---

## Styling & Customization

### Using Tailwind Classes

All components accept a `className` prop for additional styling:

```typescript
<Button className="w-full">Full Width Button</Button>

<Input className="bg-blue-50" label="Custom background" />

<Card className="border-2 border-purple-500">
  Custom card styling
</Card>
```

### Design Tokens

CSS variables are available for consistent theming:

```css
/* In your styles or components */
color: var(--color-primary);
padding: var(--spacing-md);
border-radius: var(--border-radius);
box-shadow: var(--shadow-md);
```

---

## Testing Components

### Unit Tests

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from '@/components';

test('Button calls onClick handler', async () => {
  const onClick = jest.fn();
  render(<Button onClick={onClick}>Click</Button>);

  await userEvent.click(screen.getByRole('button'));
  expect(onClick).toHaveBeenCalledOnce();
});
```

### Running Tests

```bash
npm run test              # All tests
npm run test:coverage    # Coverage report
npm run test:ui          # Interactive UI
```

---

## Accessibility Best Practices

### Keyboard Navigation
- All buttons are Tab-focusable
- Modals can be closed with Escape
- Checkboxes/Radios activate with Space
- Form inputs support standard browser behavior

### ARIA Attributes
- Use `ariaLabel` for icon-only buttons
- Errors use `aria-invalid` and `aria-describedby`
- Modals have `role="dialog"` with focus trap

### Example:

```typescript
<Button ariaLabel="Close dialog">×</Button>

<Input
  label="Email"
  error={error}
  ariaLabel="Email address input"
/>

<Modal
  isOpen={open}
  onClose={handleClose}
  ariaLabel="Confirm action"
>
  Content
</Modal>
```

---

## Common Patterns

### Form with Validation

```typescript
import { Button, Input, Checkbox } from '@/components';
import { useState } from 'react';

export function SignupForm() {
  const [form, setForm] = useState({ email: '', agreed: false });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = () => {
    const newErrors: Record<string, string> = {};

    if (!form.email.includes('@')) {
      newErrors.email = 'Invalid email address';
    }

    if (!form.agreed) {
      newErrors.agreed = 'You must agree to continue';
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    // Submit
    console.log('Submitting:', form);
  };

  return (
    <form onSubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <Input
        type="email"
        label="Email"
        value={form.email}
        onChange={(e) => setForm({ ...form, email: e.target.value })}
        error={errors.email}
        required
      />

      <Checkbox
        label="I agree to the terms"
        checked={form.agreed}
        onChange={(agreed) => setForm({ ...form, agreed })}
        required
      />

      {errors.agreed && <p className="text-red-500">{errors.agreed}</p>}

      <Button type="submit">Sign Up</Button>
    </form>
  );
}
```

---

## Troubleshooting

### Issue: Styles not applying

**Solution**: Ensure Tailwind CSS is properly configured and `globals.css` is imported in your main entry point.

### Issue: TypeScript errors on props

**Solution**: Import types from `@/types`:

```typescript
import type { ButtonProps, InputProps } from '@/types';
```

### Issue: Focus ring not visible

**Solution**: Check that `globals.css` is loaded and `:focus-visible` styling is not overridden.

### Issue: Tests failing

**Solution**: Ensure `vitest` is running and test setup file is initialized properly.

```bash
npm run test -- --reporter=verbose  # Debug output
```

---

## Next Steps

### Week 2 Components Coming
- DataTable (sort, filter, paginate)
- FormBuilder (schema-driven forms)
- Timeline (event sequences)

### Module Federation
- Pages will be served as micro-frontends
- Zero-downtime cutover via feature flag
- Shared component library via Module Federation

---

## Resources

- **Types**: `src/types/index.ts`
- **Components**: `src/components/`
- **Styles**: `src/styles/globals.css`
- **State**: `src/stores/agileplus.ts`
- **Tests**: `src/components/**/*.test.tsx`

---

## Support

For issues or questions:
1. Check component props in `src/types/index.ts`
2. Review component JSDoc comments
3. Run tests to verify functionality
4. Check test files for usage examples

Happy building! 🚀
