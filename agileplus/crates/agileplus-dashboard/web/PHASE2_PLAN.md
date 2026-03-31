# Dashboard Phase 2: React Component Library & Migration Roadmap

**Document**: Phase 2 implementation plan for AgilePlus Dashboard frontend migration
**Status**: Ready for implementation
**Timeline**: 3-4 weeks (40 hours core + 12 hours testing = 52 hours total)
**Owner**: Dashboard team
**Approval**: Pending

---

## Executive Summary

This document outlines the Phase 2 migration strategy from Askama templates to a modern React component library using shadcn/ui, TypeScript, and Module Federation for zero-downtime cutover. The backend (70% complete) remains production-ready throughout migration.

**Key Decisions:**
- Single unified dashboard (not split frontend/backend repos)
- Gradual 1-page-per-week replacement using Module Federation
- shadcn/ui for base components + custom extensions for AgilePlus-specific UI
- Zustand for client state management (lightweight, TypeScript-first)
- Askama templates continue in production until React pages fully tested

---

## 1. Component Inventory

### 1.1 Foundation Components (60 LOC each)

| Component | Purpose | Props | State | A11y |
|-----------|---------|-------|-------|------|
| **Button** | Primary action trigger | variant, size, disabled, onClick | None | aria-disabled, keyboard nav |
| **Input** | Text field | type, placeholder, value, onChange, error | None | aria-label, aria-invalid |
| **Select** | Dropdown selection | options, value, onChange, placeholder | open/closed | aria-expanded, aria-label |
| **Checkbox** | Boolean toggle | checked, onChange, label, disabled | None | aria-label, role=checkbox |
| **Radio** | Single selection | value, checked, onChange, group | None | aria-label, role=radio |
| **Toggle** | Binary switch | checked, onChange, label, icon | None | aria-pressed, aria-label |

**Implementation approach**: Wrap Radix UI primitives with shadcn/ui styling + Tailwind token system.

### 1.2 Layout Components (80 LOC each)

| Component | Purpose | Props | State | A11y |
|-----------|---------|-------|-------|------|
| **Card** | Content container | title, children, footer, variant | None | semantic HTML |
| **Modal** | Dialog overlay | isOpen, onClose, title, children | open/close | role=dialog, focus trap |
| **Toast** | Notification | type, message, duration, onClose | display state | role=alert |
| **Badge** | Status label | label, variant, icon | None | color contrast |
| **Pill** | Removable tag | label, onRemove, variant | None | keyboard dismissible |

**Design decisions**: Use Radix Dialog for modals (accessible by default), custom Toast wrapper with queue management, Badge as utility component.

### 1.3 Complex Components (200-300 LOC)

| Component | Purpose | Props | State | A11y |
|-----------|---------|-------|-------|------|
| **DataTable** | Sortable table with pagination | columns, data, onSort, pageSize, currentPage | sort/page state | aria-sort, aria-label |
| **FormBuilder** | Dynamic form generation | schema, onSubmit, defaultValues, validation | form state | fieldset, legend, aria-invalid |
| **Timeline** | Event sequence display | events, onEventClick, variant | selected event | role=list, keyboard nav |

**Complexity drivers**:
- DataTable: sorting, filtering, pagination with API integration
- FormBuilder: schema-driven field generation (text, email, select, checkbox, date)
- Timeline: click handlers to referenced items + git log link generation

### 1.4 Page-Level Components (150-200 LOC)

| Component | Purpose | Layout | Data Dependencies | A11y |
|-----------|---------|--------|-------------------|------|
| **Dashboard** | Main overview grid | 3-column grid + 2 sidebars | events, stats, agents | semantic landmarks |
| **Settings** | Configuration interface | sidebar nav + content panel | config schema | form landmarks |
| **EvidenceGallery** | Lightbox + grid view | masonry grid + modal | test results, CI logs | image alt text, modal focus |

---

## 2. UI Library Audit: shadcn/ui Coverage

### 2.1 Current Stack
- **UI Foundation**: Radix UI (headless components)
- **Styling**: Tailwind CSS 4.2.2 (PostCSS)
- **Icons**: lucide-react 1.7.0
- **Component Library**: @shadcn/ui 0.0.4 (meta-installer)
- **Utilities**: clsx + tailwind-merge for className composition
- **State**: Zustand 5.0.12

### 2.2 Coverage Analysis

**Fully covered by shadcn/ui:**
- ✅ Button (primitive + variants)
- ✅ Input (text, email, number, etc.)
- ✅ Select (dropdown with search)
- ✅ Checkbox
- ✅ Radio
- ✅ Dialog (modal)
- ✅ Card (semantic div wrapper)
- ✅ Badge (display-only label)

**Partially covered (minor extensions needed):**
- ⚠️ Toast (shadcn/ui provides component; need queue management)
- ⚠️ Toggle (Radix primitive; shadcn wrapper exists; add icon support)
- ⚠️ Tabs (exists in @radix-ui/react-tabs; need custom styling)

**Gaps (custom implementation required):**
- ❌ DataTable (shadcn provides template; need full implementation)
- ❌ FormBuilder (no out-of-box; build schema-driven pattern)
- ❌ Timeline (not in shadcn; custom component)
- ❌ Pill (custom dismissible tag)
- ❌ EvidenceGallery (custom lightbox + masonry)
- ❌ Dashboard page layout (custom grid system)
- ❌ Settings page (custom config form)

**Actions:**
1. Install shadcn/ui components via CLI: `npx shadcn-ui@latest add <component>`
2. Create 5 custom components (DataTable, FormBuilder, Timeline, Pill, EvidenceGallery)
3. Create 3 page layouts (Dashboard, Settings, Gallery)

---

## 3. Design System Decisions

### 3.1 Tailwind Configuration

**Token System** (`tailwind.config.ts`):
```typescript
{
  theme: {
    extend: {
      colors: {
        primary: { 50: '#f0f9ff', 500: '#0ea5e9', 900: '#082f49' },
        secondary: { 50: '#faf5ff', 500: '#a855f7', 900: '#44063c' },
        status: {
          success: '#10b981',
          warning: '#f59e0b',
          error: '#ef4444',
          info: '#3b82f6'
        },
        neutral: { 50: '#f9fafb', 500: '#6b7280', 900: '#111827' }
      },
      spacing: {
        xs: '0.25rem',
        sm: '0.5rem',
        md: '1rem',
        lg: '1.5rem',
        xl: '2rem',
        '2xl': '3rem'
      },
      typography: {
        body: { fontSize: '0.875rem', lineHeight: '1.5rem' },
        label: { fontSize: '0.75rem', fontWeight: '600' },
        heading: { fontSize: '1.25rem', fontWeight: '700' }
      }
    }
  }
}
```

**CSS Variables** (`src/styles/globals.css`):
```css
:root {
  --color-primary: #0ea5e9;
  --color-secondary: #a855f7;
  --color-success: #10b981;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --spacing-unit: 0.5rem;
  --border-radius: 0.5rem;
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow-md: 0 4px 6px rgba(0,0,0,0.1);
}
```

### 3.2 Component Spacing & Layout

- **Padding**: Card (16px), Dialog (20px), Forms (12px between fields)
- **Gap**: Grid (24px), Flex rows (8px)
- **Border radius**: 6px (buttons, cards), 8px (modals)
- **Typography scale**: 12px (label) → 14px (body) → 16px (heading) → 24px (title)

### 3.3 Color Palette Usage

- **Primary (Cyan #0ea5e9)**: CTAs, active states, focus rings
- **Secondary (Purple #a855f7)**: Accents, hover states, icons
- **Status colors**: Success (green), Warning (amber), Error (red), Info (blue)
- **Neutral (Gray)**: Text, borders, backgrounds

### 3.4 Accessibility Defaults

All components implement:
- **Keyboard navigation** (Tab, Enter, Escape, Arrow keys)
- **Focus management** (visible focus rings, focus traps in modals)
- **ARIA attributes** (labels, roles, live regions for alerts)
- **Color contrast** (WCAG AA minimum 4.5:1 for text)
- **Semantic HTML** (button vs div, label for form fields)

---

## 4. Module Federation Integration Strategy

### 4.1 Architecture Overview

```
┌─────────────────────────────────────────┐
│        Askama Host (Port 3000)          │
│   - Serves HTML shell + router          │
│   - Loads React MFE containers at       │
│     runtime from ports 5173+            │
└─────────────────────────────────────────┘
            ↓ (script tags)
    ┌──────────────────────────────────────┐
    │  React MFE Container (Port 517x)     │
    │  - Shared components library         │
    │  - Individual page micro-frontends   │
    └──────────────────────────────────────┘
            ↓ (script tags)
    ┌──────────────────────────────────────┐
    │      Zustand Store (Shared)          │
    │  - Client state management           │
    │  - API integration layer             │
    └──────────────────────────────────────┘
```

### 4.2 Deployment Strategy

**Phase 2.1 (Week 1-2): Foundation + DataTable**
- Build components + Foundation library in separate MFE (port 5173)
- Askama host unchanged; no breaking changes
- Test in dev/staging before cutover

**Phase 2.2 (Week 2-3): Dashboard + Settings Pages**
- Deploy Dashboard page as MFE (port 5174)
- Router redirects `/dashboard` to MFE or Askama template (configurable)
- Settings page follows same pattern (port 5175)

**Phase 2.3 (Week 3-4): Evidence Gallery + Cleanup**
- Deploy Evidence Gallery (port 5176)
- Migrate all routes from Askama to React equivalents
- Disable Askama endpoints; Axum serves static assets + API only
- Zero-downtime cutover via feature flag in host

### 4.3 Zero-Downtime Cutover Plan

**Feature Flag Approach:**
```rust
// In Axum routes: routes.rs
let use_react_dashboard = env::var("USE_REACT_DASHBOARD")
  .unwrap_or_else(|_| "false".to_string())
  .parse::<bool>()
  .unwrap_or(false);

if use_react_dashboard {
  // Serve React MFE container
  response::Html::from_static_file("mfe-loader.html")
} else {
  // Serve Askama template
  DashboardTemplate { ... }.render()
}
```

**Rollback Plan:**
- Feature flag disabled by default
- Enable in staging for testing
- Gradual rollout: 10% → 50% → 100% of traffic
- If issues detected: flip flag to disable React, revert to Askama
- No database migrations required (same backend API)

### 4.4 Shared State Management (Zustand)

```typescript
// src/stores/agileplus.ts
export const useAgilePlusStore = create<AgilePlusState>((set) => ({
  workPackages: [],
  selectedWP: null,
  filters: { status: 'all', assignee: '', priority: '' },

  setWorkPackages: (wps) => set({ workPackages: wps }),
  selectWorkPackage: (id) => set({ selectedWP: id }),
  updateFilters: (filters) => set((state) => ({
    filters: { ...state.filters, ...filters }
  }))
}));
```

**API Integration Pattern:**
```typescript
// src/hooks/useWorkPackages.ts
export function useWorkPackages() {
  const { workPackages, setWorkPackages } = useAgilePlusStore();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    axios.get('/api/work-packages')
      .then(({ data }) => setWorkPackages(data))
      .finally(() => setLoading(false));
  }, []);

  return { workPackages, loading };
}
```

---

## 5. Implementation Timeline & Work Packages

### 5.1 Phases & Milestones

| Phase | Duration | Deliverable | Acceptance Criteria |
|-------|----------|------------|-------------------|
| **WP3.1: Foundation** | 16h (Mon-Tue) | 6 base components | All components in Storybook, unit tests pass |
| **WP3.2: Complex** | 16h (Wed-Thu) | 3 complex components | DataTable sorts/filters, FormBuilder generates forms, Timeline clickable |
| **WP3.3: Pages** | 8h (Fri) | 3 page layouts | Dashboard loads data, Settings updates config, Gallery shows images |
| **WP3.4: MFE Setup** | 8h (Mon-Tue W2) | Module Federation config | MFE containers deploy, share dependencies, hot reload works |
| **WP3.5: Testing** | 12h (Wed-Thu W2) | E2E + integration tests | All pages render, API calls work, no React console errors |

### 5.2 Work Package Breakdown

#### WP3.1: Foundation Components (16 hours)

**Subtasks:**
- [ ] Button component (2h) - wrapper + 4 variants (primary, secondary, ghost, outline)
- [ ] Input component (2h) - text + password + email + number variants
- [ ] Select component (2h) - dropdown with search + icon support
- [ ] Checkbox + Radio (2h) - grouped + standalone
- [ ] Toggle component (1h) - icon toggle + label
- [ ] Card component (1h) - simple wrapper with padding/border
- [ ] Setup Storybook (2h) - stories for all components
- [ ] Setup unit tests (2h) - Jest + React Testing Library

**Acceptance Criteria:**
- [ ] All 6 components render without errors
- [ ] Props interface complete (TypeScript)
- [ ] Keyboard navigation works (Tab, Enter, Escape)
- [ ] A11y audit passes (axe-core)
- [ ] Storybook stories document all variants
- [ ] 80%+ unit test coverage

**Estimated LOC:** 6 × 60 = 360 LOC (components + tests)

#### WP3.2: Complex Components (16 hours)

**Subtasks:**
- [ ] DataTable component (8h)
  - [ ] Column definitions (1h)
  - [ ] Sorting logic (2h)
  - [ ] Pagination (2h)
  - [ ] Filtering (2h)
  - [ ] Tests (1h)
- [ ] FormBuilder component (5h)
  - [ ] Schema parser (1h)
  - [ ] Field rendering (2h)
  - [ ] Validation integration (1h)
  - [ ] Tests (1h)
- [ ] Timeline component (3h)
  - [ ] Event rendering (1h)
  - [ ] Click handlers (1h)
  - [ ] Tests (1h)

**Acceptance Criteria:**
- [ ] DataTable sorts by any column, filters show/hide rows, pagination changes pages
- [ ] FormBuilder reads schema, renders all field types, validates on submit
- [ ] Timeline displays events chronologically, click event emits callback
- [ ] All components handle loading + error states
- [ ] 70%+ test coverage for complex logic

**Estimated LOC:** DataTable (300) + FormBuilder (250) + Timeline (200) = 750 LOC

#### WP3.3: Page Layouts (8 hours)

**Subtasks:**
- [ ] Dashboard page (3h)
  - [ ] Grid layout (1h)
  - [ ] Statistics widgets (1h)
  - [ ] Event feed (1h)
- [ ] Settings page (2h)
  - [ ] Config form (1h)
  - [ ] Save/cancel actions (0.5h)
  - [ ] Confirmation dialog (0.5h)
- [ ] Evidence Gallery (3h)
  - [ ] Masonry grid layout (1h)
  - [ ] Lightbox modal (1h)
  - [ ] Image filtering (1h)

**Acceptance Criteria:**
- [ ] Dashboard renders without data, loads data on mount
- [ ] Settings form submits updates via API
- [ ] Gallery displays thumbnails, opens lightbox on click
- [ ] All pages responsive (mobile + desktop)

**Estimated LOC:** Dashboard (180) + Settings (150) + Gallery (200) = 530 LOC

#### WP3.4: Module Federation Setup (8 hours)

**Subtasks:**
- [ ] Vite MFE plugin config (2h)
  - [ ] Expose shared components
  - [ ] Expose page containers
  - [ ] Configure shared dependencies (React, Zustand)
- [ ] Askama host HTML template (2h)
  - [ ] Script loaders for MFE containers
  - [ ] Feature flag router
  - [ ] Error boundary wrapper
- [ ] Build & deployment scripts (2h)
  - [ ] Docker images for MFE containers
  - [ ] Port configuration
- [ ] Hot reload setup (2h)
  - [ ] Dev environment auto-reload
  - [ ] Production fallback to Askama

**Acceptance Criteria:**
- [ ] MFE containers load from ports 5173+
- [ ] Shared components tree-shake correctly
- [ ] Feature flag switches between Askama + React
- [ ] No console errors on page load

**Estimated LOC:** Vite config (150) + Router (100) + Docker (50) = 300 LOC

#### WP3.5: Testing & Integration (12 hours)

**Subtasks:**
- [ ] E2E tests with Playwright (6h)
  - [ ] Dashboard page flow (2h)
  - [ ] Settings form flow (2h)
  - [ ] Evidence gallery interaction (2h)
- [ ] Integration tests (3h)
  - [ ] API mocking with MSW
  - [ ] Store state transitions
  - [ ] Component interactions
- [ ] Performance audit (2h)
  - [ ] Lighthouse score
  - [ ] Bundle size analysis
  - [ ] Network waterfall
- [ ] Accessibility audit (1h)
  - [ ] axe-core automated scan
  - [ ] Keyboard navigation verification
  - [ ] Screen reader testing

**Acceptance Criteria:**
- [ ] E2E tests pass on Chrome, Firefox, Safari
- [ ] All critical user flows covered
- [ ] Lighthouse score >= 80 on all pages
- [ ] Bundle size < 500KB (gzipped)
- [ ] A11y audit passes (0 violations)

---

## 6. Component Design Specifications

### 6.1 Button Component Example

```typescript
// src/components/ui/button.tsx
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'outline';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = 'primary', size = 'md', isLoading, ...props }, ref) => {
    const baseStyles = 'font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2';
    const variantStyles = {
      primary: 'bg-cyan-500 text-white hover:bg-cyan-600 focus:ring-cyan-500',
      secondary: 'bg-purple-500 text-white hover:bg-purple-600 focus:ring-purple-500',
      ghost: 'text-gray-700 hover:bg-gray-100 focus:ring-gray-500',
      outline: 'border border-gray-300 text-gray-700 hover:bg-gray-50 focus:ring-gray-500'
    };
    const sizeStyles = {
      sm: 'px-3 py-1.5 text-sm',
      md: 'px-4 py-2 text-base',
      lg: 'px-6 py-3 text-lg'
    };

    return (
      <button
        ref={ref}
        className={cn(baseStyles, variantStyles[variant], sizeStyles[size])}
        disabled={isLoading || props.disabled}
        {...props}
      >
        {isLoading ? <Spinner /> : (
          <>
            {props.leftIcon && <span className="mr-2">{props.leftIcon}</span>}
            {props.children}
            {props.rightIcon && <span className="ml-2">{props.rightIcon}</span>}
          </>
        )}
      </button>
    );
  }
);
```

### 6.2 DataTable Component Example

```typescript
// src/components/data-table/index.tsx
export interface ColumnDef<T> {
  key: keyof T;
  header: string;
  cell?: (value: T[keyof T], row: T) => React.ReactNode;
  sortable?: boolean;
  filterable?: boolean;
  width?: string;
}

interface DataTableProps<T> {
  columns: ColumnDef<T>[];
  data: T[];
  onSort?: (key: keyof T, direction: 'asc' | 'desc') => void;
  onFilter?: (filters: Record<string, string>) => void;
  pageSize?: number;
  total?: number;
  isLoading?: boolean;
}

export function DataTable<T extends { id: string | number }>({
  columns,
  data,
  onSort,
  pageSize = 10,
  isLoading
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<keyof T | null>(null);
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
  const [page, setPage] = useState(1);

  const handleSort = (key: keyof T) => {
    const newDir = sortKey === key && sortDir === 'asc' ? 'desc' : 'asc';
    setSortKey(key);
    setSortDir(newDir);
    onSort?.(key, newDir);
  };

  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse border border-gray-200">
        <thead className="bg-gray-50">
          <tr>
            {columns.map((col) => (
              <th
                key={String(col.key)}
                onClick={() => col.sortable && handleSort(col.key)}
                className="px-4 py-2 text-left text-sm font-semibold cursor-pointer hover:bg-gray-100"
                aria-sort={sortKey === col.key ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
              >
                {col.header} {sortKey === col.key && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.length === 0 ? (
            <tr><td colSpan={columns.length} className="px-4 py-8 text-center text-gray-500">No data</td></tr>
          ) : (
            data.slice((page - 1) * pageSize, page * pageSize).map((row) => (
              <tr key={row.id} className="border-t border-gray-200 hover:bg-gray-50">
                {columns.map((col) => (
                  <td key={String(col.key)} className="px-4 py-2 text-sm">
                    {col.cell?.(row[col.key], row) ?? String(row[col.key])}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
```

---

## 7. File Structure

```
web/
├── src/
│   ├── components/
│   │   ├── ui/                          # Base components from shadcn/ui + custom
│   │   │   ├── button.tsx               # Wrapped Button (60 LOC)
│   │   │   ├── input.tsx                # Wrapped Input (60 LOC)
│   │   │   ├── select.tsx               # Wrapped Select (80 LOC)
│   │   │   ├── checkbox.tsx             # Wrapped Checkbox (50 LOC)
│   │   │   ├── radio.tsx                # Wrapped Radio (50 LOC)
│   │   │   ├── toggle.tsx               # Custom Toggle (60 LOC)
│   │   │   ├── card.tsx                 # Wrapper Card (40 LOC)
│   │   │   ├── badge.tsx                # Custom Badge (40 LOC)
│   │   │   └── pill.tsx                 # Custom Pill (50 LOC)
│   │   ├── complex/
│   │   │   ├── data-table/
│   │   │   │   ├── index.tsx             # DataTable component (300 LOC)
│   │   │   │   ├── columns.tsx           # Column definitions
│   │   │   │   └── hooks.ts              # useSort, useFilter hooks
│   │   │   ├── form-builder/
│   │   │   │   ├── index.tsx             # FormBuilder component (250 LOC)
│   │   │   │   ├── fields.tsx            # Field type registry
│   │   │   │   └── validation.ts         # Schema validation
│   │   │   └── timeline/
│   │   │       ├── index.tsx             # Timeline component (200 LOC)
│   │   │       └── types.ts              # Event type definitions
│   │   ├── dialog/
│   │   │   └── modal.tsx                 # Modal wrapper (50 LOC)
│   │   ├── toast/
│   │   │   ├── provider.tsx              # Toast context + provider
│   │   │   └── useToast.ts               # useToast hook
│   │   └── icons/
│   │       └── index.ts                  # Lucide icon exports
│   ├── pages/
│   │   ├── dashboard.tsx                 # Dashboard page (180 LOC)
│   │   ├── settings.tsx                  # Settings page (150 LOC)
│   │   └── gallery.tsx                   # Evidence gallery (200 LOC)
│   ├── stores/
│   │   ├── agileplus.ts                  # Zustand main store
│   │   ├── ui.ts                         # UI state (modals, toasts)
│   │   └── types.ts                      # Store type definitions
│   ├── hooks/
│   │   ├── useWorkPackages.ts            # Fetch WPs
│   │   ├── useSettings.ts                # Fetch/update settings
│   │   ├── useEvidenceGallery.ts         # Fetch test results
│   │   └── useApi.ts                     # Generic API hook
│   ├── lib/
│   │   ├── utils.ts                      # cn() + helpers
│   │   ├── api.ts                        # Axios config
│   │   └── constants.ts                  # Magic strings
│   ├── styles/
│   │   ├── globals.css                   # Tailwind + impeccable baseline
│   │   ├── tokens.css                    # CSS variables
│   │   └── typography.css                # Font definitions
│   ├── types/
│   │   ├── api.ts                        # API response types
│   │   ├── domain.ts                     # Business domain types
│   │   └── ui.ts                         # UI component types
│   ├── app.tsx                           # Root component
│   ├── main.tsx                          # Vite entry point
│   └── vite-env.d.ts                     # Vite type definitions
├── tests/
│   ├── components/
│   │   ├── button.test.tsx               # Button unit tests
│   │   └── data-table.test.tsx           # DataTable unit tests
│   ├── e2e/
│   │   ├── dashboard.spec.ts             # Playwright dashboard tests
│   │   └── settings.spec.ts              # Playwright settings tests
│   └── setup.ts                          # Test environment setup
├── vite.config.ts                        # MFE config + plugins
├── tsconfig.json                         # TypeScript config
├── tailwind.config.ts                    # Tailwind design tokens
├── postcss.config.js                     # Tailwind processor
├── package.json                          # Dependencies
└── COMPONENT_INVENTORY.md                # (Generated in section 5)
```

---

## 8. Storybook Setup

Create `storybook/` directory with preview configuration:

```typescript
// storybook/preview.ts
import '../src/styles/globals.css';

export const parameters = {
  docs: {
    theme: require('@storybook/theming').themes.dark,
  },
};

export const decorators = [
  (Story) => (
    <div className="p-8 bg-white">
      <Story />
    </div>
  ),
];
```

**Component stories** (`src/components/ui/button.stories.tsx`):
```typescript
export default { component: Button };

export const Primary = { args: { children: 'Click me', variant: 'primary' } };
export const Secondary = { args: { children: 'Click me', variant: 'secondary' } };
export const Loading = { args: { children: 'Loading...', isLoading: true } };
export const Disabled = { args: { children: 'Disabled', disabled: true } };
```

---

## 9. Testing Strategy

### 9.1 Unit Tests (Jest + React Testing Library)

```typescript
// tests/components/button.test.tsx
describe('Button', () => {
  it('renders primary variant', () => {
    render(<Button variant="primary">Click me</Button>);
    expect(screen.getByRole('button')).toHaveClass('bg-cyan-500');
  });

  it('is keyboard accessible', () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click me</Button>);
    const btn = screen.getByRole('button');
    btn.focus();
    fireEvent.keyDown(btn, { key: 'Enter' });
    expect(onClick).toHaveBeenCalled();
  });
});
```

### 9.2 E2E Tests (Playwright)

```typescript
// tests/e2e/dashboard.spec.ts
test('Dashboard loads and displays work packages', async ({ page }) => {
  await page.goto('http://localhost:5173/dashboard');
  await page.waitForLoadState('networkidle');

  const title = page.locator('h1');
  await expect(title).toContainText('Dashboard');

  const wpTable = page.locator('table');
  await expect(wpTable).toBeVisible();
});
```

### 9.3 A11y Tests (axe-core)

```typescript
// tests/a11y/button.test.tsx
import { injectAxe, checkA11y } from 'axe-playwright';

test('Button component passes a11y checks', async ({ page }) => {
  await page.goto('http://localhost:6006/?path=/story/ui-button--primary');
  await injectAxe(page);
  await checkA11y(page);
});
```

---

## 10. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Component Coverage** | 14/14 components | 100% implemented by WP3.3 |
| **Test Coverage** | 80%+ | Jest + Playwright coverage reports |
| **A11y Score** | 0 violations | axe-core automated scan |
| **Performance** | Lighthouse 80+ | PageSpeed Insights per page |
| **Bundle Size** | <500KB gzipped | webpack-bundle-analyzer |
| **Browser Support** | Chrome, Firefox, Safari, Edge | Browserstack testing |
| **Zero-Downtime Cutover** | 0 production incidents | Feature flag rollout monitoring |

---

## 11. Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| React version incompatibility | Pin React 19.2.4, test with exact deps |
| Module Federation complexity | Use Vite official plugin, test locally first |
| Askama template regression | Keep Askama routes in parallel, gradual cutover |
| Performance regression | Bundle analysis per PR, Lighthouse gates in CI |
| A11y regression | axe-core in CI, keyboard nav testing |
| API contract mismatch | Mock API with MSW, test against current backend |

---

## 12. Approval & Sign-Off

- [ ] Product Manager: Approve timeline + scope
- [ ] Backend Lead: Confirm API contracts + coverage
- [ ] QA Lead: Review test strategy + coverage targets
- [ ] DevOps: Approve MFE deployment strategy + monitoring
- [ ] Security: Review dependency audit (npm audit)

**Target Start Date**: 2026-04-01 (Monday)
**Target Completion**: 2026-04-21 (Monday, Week 3)

---

## Appendix: Design System Reference

See **COMPONENT_INVENTORY.md** (next section) for:
- Complete component API reference
- TypeScript prop interfaces
- Accessibility checklist (WCAG 2.1 AA)
- Storybook entry points
- Code examples for each component
