# Dashboard Phase 2: React Component Library & Migration

**Status**: Ready for Implementation
**Created**: 2026-03-31
**Timeline**: 3-4 weeks (52 hours total)
**Backend Status**: 70% complete
**Frontend Status**: 0% (React scaffold ready)

---

## Quick Start

### What is Phase 2?

Phase 2 is the complete migration from Askama templates to a modern React component architecture using:
- **shadcn/ui** for base components (Radix UI + Tailwind CSS)
- **14 custom components** (Foundation, Layout, Complex)
- **Module Federation** for zero-downtime deployment
- **Zustand** for lightweight state management
- **Vite** for ultra-fast builds

### Key Decision: Single Unified Dashboard

We're building a **single dashboard**, not splitting frontend and backend. The Askama server remains in production throughout migration, with React MFE containers deployed to separate ports (5173+). This approach:
- ✅ Zero breaking changes
- ✅ Gradual page-by-page replacement (1 week per page)
- ✅ Rollback via feature flag (seconds)
- ✅ Simpler operations (one codebase, one deployment)

---

## The 3 Planning Documents

All Phase 2 work is governed by **3 comprehensive documents**:

### 1. PHASE2_PLAN.md (28KB, 791 lines)

**Purpose**: Complete implementation roadmap and planning document

**Contents**:
- Executive summary & key decisions
- Component inventory (14 components with specs)
- UI library audit (shadcn/ui coverage analysis)
- Design system decisions (Tailwind tokens, spacing, color palette)
- Module Federation integration strategy
- Implementation timeline (5 work packages)
- Work package breakdown (subtasks, effort, AC)
- Component design specifications (40+ LOC examples)
- File structure (web/ directory layout)
- Storybook setup guide
- Testing strategy (unit, E2E, A11y)
- Success metrics & risk mitigation
- Approval & sign-off checklist

**Read this document if**:
- You're the project lead or PM
- You need to understand the overall strategy
- You want to see work package details
- You're planning resource allocation

### 2. COMPONENT_INVENTORY.md (49KB, 2006 lines)

**Purpose**: Complete API reference and implementation guide for all components

**Contents**:
- Section 1: Foundation Components (6 components)
  - Button, Input, Select, Checkbox, Radio, Toggle
  - Props interfaces, examples, tests, Storybook stories
  - 120+ lines per component (spec + code + tests)
- Section 2: Layout Components (5 components)
  - Card, Modal, Toast, Badge, Pill
  - Props interfaces, variants, accessibility
- Section 3: Complex Components (3 components)
  - DataTable (300 LOC) — sorting, filtering, pagination
  - FormBuilder (250 LOC) — schema-driven form generation
  - Timeline (200 LOC) — event display with interactions
  - Full implementation outlines with examples
- Section 4: Accessibility Compliance Matrix
  - WCAG 2.1 AA checklist for all 14 components
  - Keyboard navigation, focus management, ARIA attributes
- Section 5: Testing & Validation
  - Unit test template (Jest + React Testing Library)
  - E2E test template (Playwright)
  - A11y test template (axe-core)
- Section 6: Storybook Setup
  - Installation instructions
  - Story templates
  - Preview configuration
- Section 7: Appendices
  - Design tokens (colors, typography, spacing, shadows, radius)
  - Browser support matrix
  - Document control & versioning

**Read this document if**:
- You're implementing components
- You need TypeScript prop interfaces
- You want code examples
- You're writing tests
- You need to understand accessibility requirements

### 3. MIGRATION_ROADMAP_SUMMARY.md (19KB, 633 lines)

**Purpose**: Executive overview and governance document

**Contents**:
- Executive summary (what was built)
- Key decisions & rationale (why we chose each approach)
- Timeline & phases (detailed week-by-week)
- Resource requirements (skills, effort, tech stack)
- Work packages (WP3.1 → WP3.5 with checklists)
- Success metrics (quality, performance, a11y, testing)
- Risk mitigation (7 identified risks + mitigations)
- Deployment strategy (feature flag rollout 0% → 100%)
- Approval & sign-off checklist
- Next steps (immediate, week 1-4)
- Document references

**Read this document if**:
- You're the project lead or stakeholder
- You need executive-level overview
- You want resource/timeline estimates
- You're approving the plan
- You need deployment strategy

---

## Document Relationships

```
PHASE2_PLAN.md (Roadmap)
├─ Executive summary + key decisions
├─ Component inventory (summary)
├─ 5 work packages (WP3.1 - WP3.5)
├─ Design system decisions
├─ Module Federation strategy
├─ Testing strategy
└─ Success metrics

        ↓ (detailed specs)

COMPONENT_INVENTORY.md (API Reference)
├─ Foundation: Button, Input, Select, Checkbox, Radio, Toggle
├─ Layout: Card, Modal, Toast, Badge, Pill
├─ Complex: DataTable, FormBuilder, Timeline
├─ Props interfaces (TS)
├─ Implementation examples (code)
├─ Unit test templates
├─ E2E test templates
├─ Storybook stories
├─ A11y matrix (WCAG 2.1 AA)
└─ Design tokens

        ↓ (synthesized)

MIGRATION_ROADMAP_SUMMARY.md (Governance)
├─ Key decisions + rationale
├─ Phase breakdown (Week 1-4)
├─ Work packages (WP3.1-3.5)
├─ Resource requirements
├─ Success metrics
├─ Risk mitigation
└─ Approval checklist
```

---

## How to Use These Documents

### For Project Leads & PMs

1. **Read**: MIGRATION_ROADMAP_SUMMARY.md (15 min)
   - Understand timeline, phases, decisions
   - Review resource requirements
   - Identify blockers/dependencies

2. **Skim**: PHASE2_PLAN.md sections 1, 4, 8 (20 min)
   - Component inventory
   - Module Federation strategy
   - Success metrics

3. **Share**: MIGRATION_ROADMAP_SUMMARY.md with stakeholders
   - Get approvals from PM, QA, DevOps, Security
   - Create GitHub issues for each WP
   - Create AgilePlus specs (eco-YYY series)

### For Frontend Developers

1. **Read**: PHASE2_PLAN.md section 5 (30 min)
   - Understand your assigned work package
   - Know the subtasks and effort estimates
   - Review acceptance criteria

2. **Deep Dive**: COMPONENT_INVENTORY.md (1-2 hours)
   - Find your component(s)
   - Study props interfaces
   - Review implementation examples
   - Read test templates
   - Check accessibility requirements

3. **Reference**: PHASE2_PLAN.md sections 3, 6, 7 (ongoing)
   - Design system decisions
   - File structure
   - Storybook setup

### For QA Engineers

1. **Read**: PHASE2_PLAN.md section 8 (20 min)
   - Testing strategy (unit, E2E, A11y)
   - Test templates

2. **Review**: COMPONENT_INVENTORY.md sections 4, 5, 6 (1 hour)
   - Accessibility matrix (WCAG 2.1 AA)
   - Test templates (Jest, Playwright, axe-core)
   - Storybook setup

3. **Plan**: MIGRATION_ROADMAP_SUMMARY.md section "Success Metrics" (15 min)
   - Define test coverage targets
   - Plan E2E test scenarios
   - Setup performance monitoring

### For Design/Security Review

1. **Read**: PHASE2_PLAN.md sections 3, 4 (15 min)
   - Design system decisions
   - Tailwind token system
   - Color palette, typography, spacing

2. **Review**: COMPONENT_INVENTORY.md section 4 (15 min)
   - Accessibility compliance matrix
   - Focus management
   - Color contrast verification

3. **Approve**: Design tokens in tailwind.config.ts
   - Confirm color palette
   - Confirm typography scale
   - Confirm spacing/sizing

---

## Component Inventory at a Glance

### Foundation Components (60 LOC each)

| Component | Props | A11y | Test Template |
|-----------|-------|------|---------------|
| **Button** | variant, size, isLoading, leftIcon, rightIcon | aria-disabled, focus ring | Button.test.tsx |
| **Input** | type, label, error, helperText, leftIcon | aria-invalid, aria-describedby | Input.test.tsx |
| **Select** | options, value, onChange, placeholder, searchable | aria-expanded, aria-label | Select.test.tsx |
| **Checkbox** | label, checked, indeterminate, disabled | aria-label, role=checkbox | Checkbox.test.tsx |
| **Radio** | label, value, checked, group | aria-label, role=radio | Radio.test.tsx |
| **Toggle** | pressed, onPressedChange, icon, label | aria-pressed, aria-label | Toggle.test.tsx |

### Layout Components (40-80 LOC each)

| Component | Props | Features | A11y |
|-----------|-------|----------|------|
| **Card** | title, footer, variant, children | default, elevated, outlined | semantic HTML, heading |
| **Modal** | isOpen, onClose, title, children | focus trap, Escape key | role=dialog, focus mgmt |
| **Toast** | message, type, duration, action | queue, auto-dismiss | role=alert, live region |
| **Badge** | label, variant, icon, size | status indicator | role=status, contrast |
| **Pill** | label, onRemove, removable | dismissible tag | keyboard accessible |

### Complex Components (200-300 LOC each)

| Component | Features | Data | A11y |
|-----------|----------|------|------|
| **DataTable** | Sort, Filter, Paginate, Responsive | Column defs, data array | aria-sort, role=table |
| **FormBuilder** | Schema-driven, Validation, Error messages | Form schema, defaults | fieldset, legend, aria-invalid |
| **Timeline** | Event display, Click handlers, Links | Events array, linked items | role=list, keyboard nav |

---

## Technology Stack

```
Frontend:
- React 19.2.4 (latest, Suspense + hooks)
- TypeScript 5.9.3 (strict mode)
- Vite 8.0.1 (Module Federation plugin)
- Tailwind CSS 4.2.2 (PostCSS)
- Zustand 5.0.12 (state management)
- @radix-ui/* (headless primitives)
- @shadcn/ui 0.0.4 (styled components)
- lucide-react 1.7.0 (icons)
- axios 1.14.1 (HTTP)

Testing:
- Vitest (unit tests)
- @testing-library/react (component testing)
- Playwright (E2E)
- axe-playwright (a11y)

Development:
- Storybook 8+ (component docs)
- ESLint 9+, Prettier
- TypeScript strict
```

---

## Quick Facts

| Aspect | Details |
|--------|---------|
| **Total Lines** | 3,430 lines (3 documents) |
| **Total Size** | 96KB |
| **Components** | 14 (Foundation 6, Layout 5, Complex 3) |
| **Work Packages** | 5 (WP3.1 - WP3.5) |
| **Timeline** | 3-4 weeks |
| **Effort** | 52 hours total |
| **Resources** | 3-4 FTE (2 FE + 1 QA + 0.5 DevOps) |
| **Success Metrics** | 8 (quality, perf, a11y, testing) |
| **Risk Items** | 7 identified + mitigations |
| **Browser Support** | Chrome, Firefox, Safari, Edge (latest 2) |
| **A11y Target** | WCAG 2.1 AA (100%) |
| **Test Coverage** | 80%+ unit, 90%+ E2E critical paths |
| **Bundle Size** | <500KB gzipped |
| **Lighthouse** | ≥80 on all pages |

---

## Phase Timeline

```
PHASE 2.1: Foundation (Mon 4/1 - Fri 4/5)
├─ Button, Input, Select, Checkbox, Radio, Toggle
├─ Card, Badge, Pill
├─ Storybook setup
├─ Unit tests (80%+)
└─ Deliverable: 9 components in src/components/ui/

PHASE 2.2: Complex (Mon 4/8 - Fri 4/12)
├─ DataTable (sorting, filtering, pagination)
├─ FormBuilder (schema-driven generation)
├─ Timeline (event display + interactions)
├─ Modal, Toast providers
└─ Deliverable: 3 complex + 2 provider components

PHASE 2.3: Pages & MFE (Mon 4/15 - Fri 4/19)
├─ Dashboard page (grid layout, widgets)
├─ Settings page (config form)
├─ Evidence Gallery (lightbox, masonry)
├─ Vite Module Federation setup
├─ Feature flag router
└─ Deliverable: 3 pages + MFE containers (ports 5173+)

PHASE 2.4: Testing & Deployment (Mon 4/21 - Fri 4/26)
├─ E2E tests (Playwright)
├─ Integration tests (store + API)
├─ Performance audit (Lighthouse, bundle)
├─ A11y audit (axe-core, keyboard nav)
├─ Staging deployment
├─ Production rollout (feature flag 10% → 100%)
└─ Deliverable: Fully tested, zero-downtime migration
```

---

## Success Criteria (Final)

### Quality
- [ ] 100% TypeScript coverage (no `any`)
- [ ] 80%+ unit test coverage
- [ ] 0 ESLint errors
- [ ] 0 console errors (prod)

### Performance
- [ ] Bundle size <500KB (gzipped)
- [ ] Lighthouse ≥80 (all pages)
- [ ] LCP <2.5s
- [ ] FID <100ms
- [ ] CLS <0.1

### Accessibility
- [ ] WCAG 2.1 AA (0 violations)
- [ ] 100% keyboard navigation
- [ ] 4.5:1 contrast minimum
- [ ] 44×44px touch targets
- [ ] Screen reader tested

### Testing
- [ ] 80%+ unit test coverage
- [ ] 90%+ E2E critical path coverage
- [ ] Cross-browser (Chrome, Firefox, Safari, Edge)
- [ ] Mobile tested (iOS 12+, Android 8+)
- [ ] 0 flaky tests

### Deployment
- [ ] Zero-downtime cutover (feature flag)
- [ ] Rollback in <60 seconds
- [ ] Monitoring in place
- [ ] Staged rollout (10% → 50% → 100%)

---

## File Locations

```
/repos/AgilePlus/crates/agileplus-dashboard/
├── web/
│   ├── PHASE2_PLAN.md                    ← Implementation roadmap
│   ├── COMPONENT_INVENTORY.md             ← API reference (14 components)
│   ├── MIGRATION_ROADMAP_SUMMARY.md       ← Executive overview
│   ├── README_PHASE2.md                   ← This file
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/                        ← Foundation components
│   │   │   ├── complex/                   ← DataTable, FormBuilder, Timeline
│   │   │   ├── dialog/
│   │   │   ├── toast/
│   │   │   └── icons/
│   │   ├── pages/                         ← Dashboard, Settings, Gallery
│   │   ├── stores/                        ← Zustand stores
│   │   ├── hooks/                         ← Custom hooks (API integration)
│   │   ├── lib/                           ← Utilities (cn, api config)
│   │   ├── styles/                        ← Tailwind + design tokens
│   │   ├── types/                         ← TypeScript definitions
│   │   ├── app.tsx
│   │   └── main.tsx
│   ├── tests/
│   │   ├── components/                    ← Unit tests (Jest)
│   │   └── e2e/                           ← E2E tests (Playwright)
│   ├── vite.config.ts                     ← Module Federation config
│   ├── tailwind.config.ts                 ← Design tokens
│   ├── tsconfig.json
│   ├── package.json
│   └── README.md
```

---

## Getting Started

### Step 1: Review Documents (30 min)
```bash
# Product managers & leads
cat MIGRATION_ROADMAP_SUMMARY.md

# Frontend developers
cat PHASE2_PLAN.md
cat COMPONENT_INVENTORY.md

# QA engineers
grep -A 20 "Testing Strategy" PHASE2_PLAN.md
grep -A 10 "Accessibility" COMPONENT_INVENTORY.md
```

### Step 2: Get Approvals (By 4/1)
- [ ] PM approval (MIGRATION_ROADMAP_SUMMARY.md)
- [ ] QA approval (Testing strategy section)
- [ ] DevOps approval (Deployment strategy section)
- [ ] Security approval (Dependencies audit)

### Step 3: Create Work Items (By 4/1)
- [ ] Create GitHub issues for WP3.1 - WP3.5
- [ ] Create AgilePlus specs (if applicable)
- [ ] Assign to developers + QA

### Step 4: Start Phase 2.1 (Mon 4/1)
- [ ] Clone/checkout feature branch
- [ ] Read PHASE2_PLAN.md section 5 (WP3.1)
- [ ] Read COMPONENT_INVENTORY.md sections 1-2
- [ ] Begin implementing foundation components

---

## Questions?

Refer to the appropriate document:

| Question | Document | Section |
|----------|----------|---------|
| "How long will Phase 2 take?" | MIGRATION_ROADMAP_SUMMARY.md | Timeline & Phases |
| "What components do we need?" | PHASE2_PLAN.md | Section 1: Component Inventory |
| "How do I implement Button?" | COMPONENT_INVENTORY.md | Section 1.1: Button Component |
| "What's the testing strategy?" | PHASE2_PLAN.md | Section 8: Testing Strategy |
| "What's the deployment plan?" | MIGRATION_ROADMAP_SUMMARY.md | Deployment Strategy |
| "How is accessibility handled?" | COMPONENT_INVENTORY.md | Section 4: A11y Matrix |
| "What are the risks?" | MIGRATION_ROADMAP_SUMMARY.md | Risk Mitigation |
| "Who should review my code?" | PHASE2_PLAN.md | Section 6: Work Packages |

---

**Document Created**: 2026-03-31
**Status**: Ready for Implementation
**Approval Deadline**: 2026-04-01
**Phase 2 Start**: Monday, 2026-04-01
**Phase 2 End**: Friday, 2026-04-26
