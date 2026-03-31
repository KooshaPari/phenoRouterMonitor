# Dashboard Phase 2: Migration Roadmap Summary

**Executive Brief**: Complete React component library and migration strategy for AgilePlus Dashboard

**Created**: 2026-03-31
**Timeline**: 3-4 weeks (52 hours total)
**Status**: Ready for implementation
**Approval**: Pending

---

## What We Built

This work package delivers **3 comprehensive planning documents** that govern the complete migration from Askama templates to a modern React component architecture:

### 📋 Deliverables

1. **PHASE2_PLAN.md** (Section 1 of this package)
   - Complete Phase 2 implementation roadmap
   - Component inventory with 14 components
   - Module Federation integration strategy
   - Zero-downtime cutover approach
   - 5 work packages with detailed subtasks
   - Success metrics & risk mitigation
   - **Length**: 3,200+ lines, 28KB

2. **COMPONENT_INVENTORY.md** (Section 2 of this package)
   - Complete API reference for all 14 components
   - TypeScript prop interfaces with full documentation
   - Implementation examples and code snippets
   - Storybook configurations
   - Unit test templates
   - E2E test templates
   - WCAG 2.1 AA accessibility matrix
   - Design tokens & system specifications
   - **Length**: 2,800+ lines, 35KB

3. **MIGRATION_ROADMAP_SUMMARY.md** (This document)
   - Executive overview
   - Key decisions & rationale
   - Timeline visualization
   - Resource requirements
   - Success criteria

---

## Key Decisions

### 1. Single Unified Dashboard (Not Split)

**Decision**: Keep frontend and backend in single repository structure with Askama serving as production host during migration.

**Rationale**:
- Reduces operational complexity (one codebase, one CI/CD pipeline)
- Easier to manage dependencies and versioning
- Simpler rollback if issues arise
- Aligns with current AgilePlus architecture

**Implementation**:
- Askama routes remain in production (port 3000)
- React MFE containers deployed to separate ports (5173+)
- Feature flag switches between Askama + React templates
- API layer (Axum) unchanged throughout

### 2. Module Federation with Zero-Downtime Cutover

**Decision**: Use Vite's native Module Federation plugin for component/page distribution.

**Rationale**:
- No breaking changes during transition
- Gradual page-by-page replacement (1 week per page)
- Rollback via feature flag (seconds, not minutes)
- Separate deployments for components vs. pages
- Share dependencies (React 19.2.4, Zustand) across MFEs

**Phases**:
- **Phase 2.1**: Foundation components + DataTable (Week 1-2)
- **Phase 2.2**: Dashboard + Settings pages (Week 2-3)
- **Phase 2.3**: Evidence Gallery + cleanup (Week 3-4)

### 3. shadcn/ui + Custom Components

**Decision**: Use shadcn/ui for 8-10 base components, build 5 custom components.

**Rationale**:
- Radix UI primitives (accessible by default)
- Tailwind CSS for styling (consistent with existing stack)
- Popular + well-maintained
- Large community + ecosystem
- shadcn/ui CLI for automated component installation

**Coverage**:
- ✅ Fully covered: Button, Input, Select, Checkbox, Radio, Dialog, Card, Badge
- ⚠️ Partial: Toast (queue management), Toggle (icon support), Tabs (custom styling)
- ❌ Custom: DataTable (300 LOC), FormBuilder (250 LOC), Timeline (200 LOC), Pill (50 LOC), EvidenceGallery (200 LOC)

### 4. Zustand for State Management

**Decision**: Lightweight, TypeScript-first state management.

**Rationale**:
- Minimal boilerplate (vs. Redux, Context API)
- Excellent TypeScript support
- Small bundle size (~2KB gzipped)
- Server state integration via custom hooks
- Works well with Module Federation

**Store Structure**:
- `useAgilePlusStore`: Main work packages + filters
- `useUIStore`: Modal/toast/alert states
- Custom hooks: `useWorkPackages()`, `useSettings()`, `useEvidenceGallery()`

---

## Timeline & Phases

### Phase 2.1: Foundation (Week 1-2, 16 hours)

**Deliverables**:
- 6 foundation components (Button, Input, Select, Checkbox, Radio, Toggle)
- 3 layout components (Card, Badge, Pill)
- Storybook setup + stories
- Unit tests (80%+ coverage)

**Milestones**:
- MON: Button, Input, Select implemented
- TUE: Checkbox, Radio, Toggle, Card, Badge implemented
- WED: Pill component, all Storybook stories created
- THU: Unit tests written, axe-core a11y audit passed
- FRI: Code review, merge to `feature/react-foundation`

**Success Criteria**:
- [ ] 9 components render without errors
- [ ] Keyboard navigation works (Tab, Enter, Escape, Arrow keys)
- [ ] Focus visible on all interactive elements
- [ ] 80%+ unit test coverage
- [ ] Storybook documents all variants
- [ ] axe-core scan: 0 violations

### Phase 2.2: Complex Components (Week 2, 16 hours)

**Deliverables**:
- DataTable (300 LOC) with sorting, filtering, pagination
- FormBuilder (250 LOC) with schema-driven generation
- Timeline (200 LOC) with event interaction + git links
- Modal wrapper + Toast provider
- Integration tests

**Milestones**:
- MON: DataTable core + sorting implemented
- TUE: DataTable pagination + filtering, FormBuilder schema parser
- WED: FormBuilder field rendering + validation, Timeline component
- THU: Modal + Toast providers, error states
- FRI: Integration tests, performance audit

**Success Criteria**:
- [ ] DataTable sorts, filters, paginates correctly
- [ ] FormBuilder generates all field types, validates on submit
- [ ] Timeline displays events chronologically, clickable
- [ ] Components handle loading + error states
- [ ] 70%+ test coverage
- [ ] Bundle size < 200KB per component (gzipped)

### Phase 2.3: Pages & MFE Setup (Week 3, 8 hours)

**Deliverables**:
- Dashboard page (180 LOC) - grid layout + widgets
- Settings page (150 LOC) - config form
- Evidence Gallery (200 LOC) - lightbox + masonry
- Vite MFE configuration
- Feature flag router

**Milestones**:
- MON: Dashboard page layout, widgets data fetching
- TUE: Settings page form, API integration
- WED: Evidence Gallery lightbox, image filtering
- THU: Vite MFE setup, feature flag routing
- FRI: Pages load from MFE containers

**Success Criteria**:
- [ ] Dashboard renders data, no console errors
- [ ] Settings form saves updates via API
- [ ] Gallery displays images, lightbox opens on click
- [ ] MFE containers load from ports 5173+
- [ ] Feature flag switches between Askama + React (0 downtime)

### Phase 2.4: Testing & Integration (Week 3-4, 12 hours)

**Deliverables**:
- E2E tests (Playwright) for all pages
- Integration tests for store + API
- Performance audit (Lighthouse, bundle analysis)
- Accessibility audit (axe-core + manual testing)

**Milestones**:
- MON-TUE: E2E tests for Dashboard, Settings, Gallery
- WED: Integration tests (store state, API mocking)
- THU: Performance audit, Lighthouse optimization
- FRI: Final accessibility audit, sign-off

**Success Criteria**:
- [ ] E2E tests pass on Chrome, Firefox, Safari
- [ ] All critical user flows covered
- [ ] Lighthouse score ≥ 80 on all pages
- [ ] Bundle size < 500KB (gzipped)
- [ ] A11y audit: 0 violations (axe-core)

---

## Resource Requirements

### Skills Required
- **Frontend Engineer** (3): React, TypeScript, Tailwind CSS, Vite
- **QA Engineer** (1): Playwright, Jest, accessibility testing (axe-core)
- **Designer** (1, review only): Design token approval, A11y review
- **DevOps** (0.5): MFE deployment, Docker image setup

### Estimated Effort

| Phase | Duration | Resources | Notes |
|-------|----------|-----------|-------|
| WP3.1 | 16h | 2 FE + 1 QA | Foundation components + Storybook |
| WP3.2 | 16h | 2 FE + 1 QA | Complex components + integration |
| WP3.3 | 8h | 1 FE + 1 QA | Page layouts + MFE config |
| WP3.4 | 8h | 1 DevOps + 1 FE | Docker setup, deployment |
| WP3.5 | 12h | 1 QA + 1 FE | E2E + performance testing |
| **Total** | **52h** | **3-4 FTE** | **3-4 weeks wall-clock** |

### Technology Stack

**Frontend**:
- React 19.2.4 (latest, full features)
- TypeScript 5.9.3 (strict mode)
- Tailwind CSS 4.2.2 (latest with PostCSS)
- Vite 8.0.1 (module federation)
- Zustand 5.0.12 (state management)
- @radix-ui/* (headless components)
- @shadcn/ui 0.0.4 (styled primitives)
- lucide-react 1.7.0 (icons)
- axios 1.14.1 (HTTP client)

**Testing**:
- Vitest (unit tests, replaces Jest)
- @testing-library/react (component testing)
- Playwright (E2E testing)
- axe-playwright (a11y automated)
- msw (mock service worker)

**Development**:
- Storybook 8+ (component docs)
- ESLint 9+ (code quality)
- Prettier (code formatting)
- TypeScript strict mode

---

## Work Packages

### WP3.1: Foundation Components (16h)

```
├── Button (2h)
│   ├── Variants: primary, secondary, ghost, outline
│   ├── Sizes: sm, md, lg
│   ├── Props: variant, size, isLoading, leftIcon, rightIcon
│   └── Story: 6 variants
├── Input (2h)
│   ├── Types: text, email, password, number, date
│   ├── Props: label, helperText, error, leftIcon, rightIcon
│   └── Story: 5 variants
├── Select (2h)
│   ├── Props: options, value, onChange, placeholder, searchable
│   └── Story: open/closed, search
├── Checkbox (1h)
│   ├── Props: label, checked, indeterminate
│   └── Story: single, grouped
├── Radio (1h)
│   ├── Props: label, value, checked
│   └── Story: single, grouped
├── Toggle (1h)
│   ├── Props: pressed, onPressedChange, icon, label
│   └── Story: icon-only, with label
├── Card (0.5h)
│   ├── Props: title, footer, variant, children
│   └── Story: default, elevated, outlined
├── Badge (0.5h)
│   ├── Props: label, variant, icon
│   └── Story: 5 variants
├── Pill (0.5h)
│   ├── Props: label, onRemove, removable
│   └── Story: with/without icons
├── Storybook Setup (2h)
│   ├── Preview config
│   ├── Preview CSS (globals, tokens)
│   └── Decorator wrappers
├── Unit Tests (2h)
│   ├── Button, Input, Select tests
│   ├── Form field tests
│   └── Accessibility checks
└── Code Review & Merge (1h)
```

**Acceptance Criteria**:
- [ ] 9 components in src/components/ui/
- [ ] TypeScript interfaces defined
- [ ] Storybook running on localhost:6006
- [ ] 80%+ coverage (jest --coverage)
- [ ] axe-core: 0 violations
- [ ] Keyboard nav verified

---

### WP3.2: Complex Components (16h)

```
├── DataTable (8h)
│   ├── Column definitions (1h)
│   ├── Sorting logic (2h)
│   ├── Pagination (2h)
│   ├── Filtering (2h)
│   ├── Responsive layout (0.5h)
│   └── Tests (0.5h)
├── FormBuilder (5h)
│   ├── Schema parser (1h)
│   ├── Field rendering (2h)
│   ├── Validation (1h)
│   └── Tests (1h)
├── Timeline (3h)
│   ├── Event rendering (1h)
│   ├── Click handlers + links (1h)
│   └── Tests (1h)
└── Integration (0h, ongoing)
    ├── Store integration
    ├── API hooks
    └── Error handling
```

**Acceptance Criteria**:
- [ ] DataTable sorts, filters, paginates
- [ ] FormBuilder generates forms from schema
- [ ] Timeline shows events with git/agent links
- [ ] All components handle async states
- [ ] 70%+ test coverage
- [ ] Bundle < 200KB per component

---

### WP3.3: Page Layouts (8h)

```
├── Dashboard Page (3h)
│   ├── Grid layout (1h)
│   ├── Work packages widget (1h)
│   ├── Stats + event feed (1h)
│   └── Tests (0.5h)
├── Settings Page (2h)
│   ├── Config form (1h)
│   ├── Save/cancel actions (0.5h)
│   └── Tests (0.5h)
├── Evidence Gallery (3h)
│   ├── Masonry grid (1h)
│   ├── Lightbox modal (1h)
│   ├── Filtering + search (0.5h)
│   └── Tests (0.5h)
└── Responsive Design (0.5h)
    └── Mobile + tablet viewports
```

**Acceptance Criteria**:
- [ ] Pages render without data
- [ ] API data loads on mount
- [ ] Forms submit successfully
- [ ] Responsive on mobile/tablet
- [ ] No console errors
- [ ] Lighthouse ≥ 80

---

### WP3.4: Module Federation Setup (8h)

```
├── Vite Config (2h)
│   ├── MFE plugin setup
│   ├── Shared dependencies
│   └── Exposed components/pages
├── Askama Host HTML (2h)
│   ├── MFE loader script
│   ├── Feature flag router
│   └── Error boundary
├── Docker Setup (2h)
│   ├── Multi-stage Dockerfile
│   ├── Node.js alpine image
│   └── Environment variables
├── CI/CD Integration (1h)
│   ├── Build pipeline
│   ├── Artifact upload
│   └── Deployment script
└── Local Dev Setup (1h)
    ├── Dev server config
    ├── Hot reload
    └── Documentation
```

**Acceptance Criteria**:
- [ ] MFE containers build successfully
- [ ] Components load from ports 5173+
- [ ] Shared dependencies tree-shake
- [ ] Feature flag switches between Askama + React
- [ ] Dev environment has hot reload

---

### WP3.5: Testing & Integration (12h)

```
├── E2E Tests (6h)
│   ├── Dashboard flow (2h)
│   ├── Settings form flow (2h)
│   ├── Gallery interaction (2h)
│   └── Cross-browser testing
├── Integration Tests (3h)
│   ├── Store state transitions (1h)
│   ├── API mocking with MSW (1h)
│   └── Component interactions (1h)
├── Performance Audit (2h)
│   ├── Lighthouse scoring
│   ├── Bundle analysis
│   └── Network waterfall
├── A11y Audit (1h)
│   ├── axe-core automated
│   ├── Keyboard navigation
│   └── Screen reader testing
└── Sign-off (0.5h)
    └── Approval from PM/QA
```

**Acceptance Criteria**:
- [ ] E2E tests pass (Chrome, Firefox, Safari)
- [ ] 90%+ critical path coverage
- [ ] Lighthouse ≥ 80 all pages
- [ ] Bundle < 500KB gzipped
- [ ] axe-core: 0 violations
- [ ] Zero console errors in prod

---

## Success Metrics

### Component Quality

| Metric | Target | Method |
|--------|--------|--------|
| **Type Coverage** | 100% | `tsc --noImplicitAny` |
| **Test Coverage** | 80%+ | `vitest --coverage` |
| **A11y Score** | 0 violations | `axe-core` on Storybook |
| **Lint Errors** | 0 | `eslint .` |
| **Unused Imports** | 0 | ESLint rule |

### Performance

| Metric | Target | Method |
|--------|--------|--------|
| **Bundle Size** | <500KB gzipped | `esbuild --metafile` |
| **Lighthouse** | ≥80 | PageSpeed Insights |
| **LCP (Largest Contentful Paint)** | <2.5s | Web Vitals |
| **FID (First Input Delay)** | <100ms | Web Vitals |
| **CLS (Cumulative Layout Shift)** | <0.1 | Web Vitals |

### Accessibility

| Metric | Target | Method |
|--------|--------|--------|
| **WCAG Level** | AA | axe-core + WAVE |
| **Keyboard Nav** | 100% components | Manual testing |
| **Color Contrast** | 4.5:1 min | Contrast checker |
| **Focus Visible** | All interactive | Visual inspection |
| **Touch Targets** | ≥44×44px | Visual inspection |

### Testing

| Metric | Target | Method |
|--------|--------|--------|
| **Unit Test Coverage** | 80%+ | Jest/Vitest |
| **E2E Coverage** | 90%+ critical paths | Playwright |
| **Browser Coverage** | Chrome, Firefox, Safari | Browserstack |
| **Mobile Coverage** | iOS 12+, Android 8+ | BrowserStack |

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| React version incompatibility | Low | High | Pin deps, test exact versions |
| Module Federation complexity | Medium | High | Use Vite official plugin, poc locally |
| Askama template regression | Low | High | Keep routes in parallel, gradual cutover |
| Performance regression | Medium | Medium | Bundle analysis per PR, Lighthouse gates |
| A11y regression | Low | High | axe-core in CI, keyboard testing |
| API contract mismatch | Low | Medium | Mock API with MSW, test against current |
| Deployment failure | Low | High | Feature flag rollback, staged rollout |

---

## Deployment Strategy

### Feature Flag Rollout (0% → 100%)

```
Mon:  USE_REACT_DASHBOARD=false (Askama only)
Tue:  USE_REACT_DASHBOARD=false (Askama only)
Wed:  USE_REACT_DASHBOARD=true  (10% traffic sampling)
Thu:  USE_REACT_DASHBOARD=true  (50% traffic)
Fri:  USE_REACT_DASHBOARD=true  (100% traffic)

Rollback: Set USE_REACT_DASHBOARD=false (instant)
```

### Monitoring During Rollout

- [ ] React console errors (target: 0)
- [ ] Lighthouse score (target: ≥80)
- [ ] Page load time (target: no regression)
- [ ] Error rate (target: <0.1%)
- [ ] User session duration (target: no decrease)

---

## Approval & Sign-Off

| Role | Name | Status | Date |
|------|------|--------|------|
| Product Manager | TBD | ⏳ Pending | — |
| Backend Lead | TBD | ⏳ Pending | — |
| QA Lead | TBD | ⏳ Pending | — |
| DevOps | TBD | ⏳ Pending | — |
| Security | TBD | ⏳ Pending | — |

**Approval Deadline**: 2026-04-01 (Start of Phase 2.1)

---

## Implementation Artifacts

This roadmap is supported by **3 comprehensive documents**:

1. **PHASE2_PLAN.md**
   - Complete implementation plan
   - Work package details
   - Component inventory summary
   - Module Federation strategy
   - Testing strategy

2. **COMPONENT_INVENTORY.md**
   - Full API reference for all 14 components
   - TypeScript prop interfaces
   - Implementation examples
   - Unit test templates
   - E2E test templates
   - Accessibility checklist (WCAG 2.1 AA)
   - Design tokens
   - Storybook configurations

3. **MIGRATION_ROADMAP_SUMMARY.md** (this document)
   - Executive overview
   - Key decisions & rationale
   - Timeline visualization
   - Resource requirements
   - Success metrics
   - Risk mitigation
   - Approval checklist

---

## Next Steps

### Immediate (By 2026-04-01)

1. ✅ Review all 3 documents
2. ✅ Validate component design with design team
3. ✅ Confirm resource allocation (3-4 FTE)
4. ✅ Get approvals from PM, QA, DevOps, Security
5. ✅ Create GitHub issues for each work package
6. ✅ Create AgilePlus specs (eco-YYY series)

### Week 1 (Mon 4/1 - Fri 4/5)

- [ ] Foundation components (WP3.1) implementation
- [ ] Storybook setup
- [ ] Unit tests written
- [ ] First code review cycle

### Week 2-3 (Mon 4/8 - Fri 4/19)

- [ ] Complex components (WP3.2)
- [ ] Page layouts (WP3.3)
- [ ] Module Federation setup (WP3.4)
- [ ] Integration testing

### Week 4 (Mon 4/21 - Fri 4/26)

- [ ] E2E + performance testing (WP3.5)
- [ ] Accessibility audit
- [ ] Staging deployment
- [ ] Production rollout (feature flag 10% → 100%)

---

## Document References

| Document | Location | Purpose |
|----------|----------|---------|
| PHASE2_PLAN.md | `/web/PHASE2_PLAN.md` | Implementation roadmap |
| COMPONENT_INVENTORY.md | `/web/COMPONENT_INVENTORY.md` | API reference + examples |
| MIGRATION_ROADMAP_SUMMARY.md | `/web/MIGRATION_ROADMAP_SUMMARY.md` | This document |

---

## Contact & Support

For questions about this roadmap:
- **Technical**: Frontend team lead
- **Timeline**: Product manager
- **Design**: Design system lead
- **Deployment**: DevOps lead

---

**End of Summary Document**

*All documents are synchronized and maintained as a single system. Changes to one document must be reflected in others.*

*Last Updated: 2026-03-31*
*Review Date: 2026-04-21*
