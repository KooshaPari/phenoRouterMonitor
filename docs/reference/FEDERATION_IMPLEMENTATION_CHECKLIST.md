# Module Federation Implementation Checklist

## Overview
This checklist tracks the implementation of Module Federation for the Federated Hybrid Architecture Phase 2.

**Project**: Federated Hybrid Architecture - Phase 2
**Status**: Design Complete, Implementation Ready
**Timeline**: 10 working days estimated

---

## Phase 2.1: AgilePlus Host Setup (Days 1-2)

### Research & Planning
- [ ] Verify AgilePlus is React 18+ with Vite
- [ ] Identify existing router structure (React Router v6+)
- [ ] Map current pages and routes
- [ ] Identify layout shell components
- [ ] List shared libraries (@phenotype/docs, phenotype-shared, etc.)
- [ ] Confirm target ports (3000 for host, 3001-3002 for remotes)

### Dependencies & Tools
- [ ] Install @module-federation/enhanced
  ```bash
  npm install --save-dev @module-federation/enhanced
  ```
- [ ] Install @vitejs/plugin-react if not present
- [ ] Verify Vite version >= 4.0.0
- [ ] Check Node.js version >= 18.0.0

### Module Federation Configuration
- [ ] Create `/vite-federation.config.ts` in AgilePlus root
  - [ ] Define host name: "agileplus-host"
  - [ ] Configure remotes: heliosApp@localhost:3001, agent-wave@localhost:3002
  - [ ] Define shared libraries with correct versions
  - [ ] Set singleton: true for React, @phenotype/docs, phenotype-shared
  - [ ] Set eager: true for @phenotype/docs (design system must load immediately)
  - [ ] Configure port: 3000

### Router Updates
- [ ] Create `/src/routes/index.tsx` with remote module routes
  - [ ] Import remoteEntry points for heliosApp and agent-wave
  - [ ] Use React.lazy() for dynamic imports
  - [ ] Wrap with Suspense + error boundaries
  - [ ] Add route handlers for `/dashboard/helios/*` and `/forecast/agent-wave/*`
  - [ ] Test local routes still work

### Layout & Error Handling
- [ ] Create `/src/components/LayoutShell.tsx`
  - [ ] Header with nav menu
  - [ ] Links to host pages + remote modules
  - [ ] Footer
  - [ ] Use @phenotype/docs theme provider

- [ ] Create error boundary component for modules
  - [ ] Fallback UI for module load failure
  - [ ] Helpful error messages
  - [ ] Retry button

- [ ] Create loading/spinner component
  - [ ] Show while module is loading
  - [ ] Skeleton screens (optional)

### Testing (Local)
- [ ] Run AgilePlus locally: `npm run dev`
- [ ] Verify host loads on http://localhost:3000
- [ ] Verify routing works for local pages
- [ ] Check console for federation setup logs
- [ ] Verify remoteEntry.js generated in dist/assets/

### Documentation
- [ ] Document vite-federation.config.ts structure
- [ ] Document LayoutShell component API
- [ ] Document error boundary usage

---

## Phase 2.2: heliosApp Remote Module Setup (Days 3-4)

### Analysis
- [ ] Identify heliosApp dashboard page/components
- [ ] List mobile-specific UI components to expose
- [ ] Identify services/hooks that could be shared
- [ ] Check current build process (Vite, Webpack, etc.)
- [ ] Map heliosApp dependencies

### Module Federation Configuration
- [ ] Create `vite-federation.config.ts` in heliosApp root
  - [ ] Define module name: "heliosApp"
  - [ ] Expose: ./Dashboard (main page), ./MobileComponents, ./Services
  - [ ] List shared libraries (same as AgilePlus host)
  - [ ] Configure port: 3001

### Dual-Mode Support
- [ ] Create `/src/bootstrap.ts` or update `/src/main.tsx`
  - [ ] Detect FEDERATION_MODE environment variable
  - [ ] If "standalone": render full app in div#root
  - [ ] If "federated": don't render (wait for host to load modules)
  - [ ] Add console log for mode detection

### Package.json Scripts
- [ ] Add "dev" script (standalone mode)
  ```json
  "dev": "FEDERATION_MODE=standalone vite"
  ```
- [ ] Add "dev:remote" script (federated mode)
  ```json
  "dev:remote": "FEDERATION_MODE=federated vite --config vite-federation.config.ts"
  ```
- [ ] Add "build" script
- [ ] Add "build:remote" script (for federated build)

### Code Organization
- [ ] Ensure Dashboard page is at `/src/pages/Dashboard.tsx`
- [ ] Ensure Mobile components in `/src/components/Mobile/index.ts`
- [ ] Ensure services exported from `/src/services/index.ts`
- [ ] Remove any hardcoded routing (use relative paths)

### Testing (Standalone)
- [ ] Run heliosApp standalone: `npm run dev`
- [ ] Verify loads on http://localhost:3001 as full app
- [ ] Verify all features work without AgilePlus host
- [ ] Check @phenotype/docs theme applied correctly

### Testing (Federated - with Host)
- [ ] Run both host (Terminal 1) and heliosApp (Terminal 2)
- [ ] Host on http://localhost:3000
- [ ] heliosApp on http://localhost:3001
- [ ] Click "Helios Dashboard" link in host
- [ ] Verify module loads from 3001 into host layout
- [ ] Verify dashboard works inside host shell
- [ ] Check network requests go to correct domain

### Documentation
- [ ] Document module federation config
- [ ] Document standalone vs federated mode
- [ ] Document exposed exports

---

## Phase 2.3: agent-wave Remote Module Setup (Days 5-6)

Same checklist as Phase 2.2, but for agent-wave:

### Analysis
- [ ] Identify agent-wave forecasting/prediction pages
- [ ] List agent UI components
- [ ] Identify services/prediction logic
- [ ] Check current build process

### Module Federation Configuration
- [ ] Create `vite-federation.config.ts`
  - [ ] Module name: "agent-wave"
  - [ ] Expose: ./Forecast, ./AgentComponents, ./Services
  - [ ] Port: 3002

### Dual-Mode Support
- [ ] Create bootstrap with FEDERATION_MODE detection
- [ ] Add npm scripts (dev, dev:remote, build, build:remote)

### Code Organization
- [ ] Forecast page at `/src/pages/Forecast.tsx`
- [ ] Agent components in `/src/components/Agent/index.ts`
- [ ] Services in `/src/services/index.ts`

### Testing (Standalone)
- [ ] Run standalone: `npm run dev` on port 3002
- [ ] Verify all features work

### Testing (Federated - with Host)
- [ ] Run host + agent-wave remote
- [ ] Click "Agent Wave Forecast" in host
- [ ] Verify module loads correctly
- [ ] Check features work inside host layout

---

## Phase 2.4: Integration & Local Testing (Days 7-8)

### Three-Terminal Setup
- [ ] Terminal 1: AgilePlus host running on 3000
- [ ] Terminal 2: heliosApp remote running on 3001
- [ ] Terminal 3: agent-wave remote running on 3002
- [ ] All three services started simultaneously
- [ ] No port conflicts

### Navigation Testing
- [ ] [ ] Click "Helios Dashboard" → loads heliosApp module
- [ ] [ ] Click "Agent Wave Forecast" → loads agent-wave module
- [ ] [ ] Return to host home page
- [ ] [ ] Navigate between all pages/modules without errors

### Theme & Styling
- [ ] [ ] All modules use same @phenotype/docs theme
- [ ] [ ] Color scheme consistent across modules
- [ ] [ ] Typography consistent
- [ ] [ ] Dark mode (if applicable) consistent

### Shared Dependencies
- [ ] [ ] Verify React not duplicated: check DevTools
- [ ] [ ] Verify @phenotype/docs loaded once
- [ ] [ ] Verify phenotype-shared loaded once
- [ ] [ ] Run `npm ls` in each module - no conflicting versions

### Error Scenarios

#### Scenario 1: Remote Module Down
- [ ] Kill Terminal 2 (heliosApp)
- [ ] Click "Helios Dashboard"
- [ ] Verify error fallback UI shown
- [ ] Restart Terminal 2
- [ ] Refresh page
- [ ] Verify module loads successfully

#### Scenario 2: Network Failure
- [ ] Simulate network error (DevTools throttling)
- [ ] Click module link
- [ ] Verify appropriate error message
- [ ] Restore network
- [ ] Refresh and verify reload works

#### Scenario 3: Version Mismatch
- [ ] Change React version in heliosApp package.json
- [ ] npm install
- [ ] npm run dev:remote
- [ ] Click module link in host
- [ ] Verify error about shared dependency
- [ ] Fix version, test resolution

### Performance Testing
- [ ] Measure initial host load time (<2s)
- [ ] Measure module load time (<3s)
- [ ] Check bundle sizes:
  - [ ] Host: < 100KB
  - [ ] heliosApp: < 150KB
  - [ ] agent-wave: < 150KB
- [ ] Check for console warnings/errors

### Browser Compatibility
- [ ] Test in Chrome/Edge (latest)
- [ ] Test in Firefox (latest)
- [ ] Test in Safari (latest)
- [ ] Test on mobile (iOS Safari, Chrome Android)

---

## Phase 2.5: Documentation & Deployment (Days 9-10)

### Documentation Deliverables

#### MODULE_FEDERATION_SETUP.md
- [ ] Architecture diagram (Host/Remotes)
- [ ] Design decisions documented
- [ ] Module boundary definitions clear
- [ ] Shared dependency strategy explained
- [ ] Port assignments documented
- [ ] Implementation plan steps
- [ ] Error handling patterns
- [ ] Production deployment overview
- [ ] Success criteria listed

#### MODULE_FEDERATION_LOCAL_DEV_GUIDE.md
- [ ] Quick start (3-terminal setup)
- [ ] Prerequisites listed
- [ ] Troubleshooting section:
  - [ ] "Module failed to load"
  - [ ] "Version mismatch"
  - [ ] "Port in use"
  - [ ] "Module timeout"
  - [ ] "HMR not working"
- [ ] Testing scenarios
- [ ] Configuration reference
- [ ] Environment variables documented
- [ ] Debugging tips

#### FEDERATION_PRODUCTION_DEPLOYMENT.md
- [ ] Deployment architecture diagram
- [ ] Pre-deployment checklist
- [ ] Build process (per module)
- [ ] AWS S3 + CloudFront setup
- [ ] Or Cloudflare Pages setup
- [ ] Environment configuration
- [ ] CORS & security headers
- [ ] Version management
- [ ] Health checks
- [ ] Monitoring & alerting
- [ ] Rollback procedure
- [ ] CI/CD pipeline example
- [ ] Cost optimization
- [ ] Post-deployment verification

#### FEDERATION_IMPLEMENTATION_CHECKLIST.md
- [ ] This file! ✓
- [ ] Links to other docs
- [ ] Phase breakdown
- [ ] Success criteria

### Update Main README.md
- [ ] Add "Federated Architecture" section
- [ ] Explain benefits (unified dashboard, independent deployment)
- [ ] Link to MODULE_FEDERATION_SETUP.md
- [ ] Link to local dev guide
- [ ] Link to deployment guide
- [ ] Quick start example (3 terminals)

### Production Deployment

#### Pre-Deployment
- [ ] All code changes merged to main
- [ ] All tests passing
- [ ] No linting errors
- [ ] Security audit clean (gitleaks, npm audit)
- [ ] Shared dependencies versions aligned
- [ ] Build artifacts generated

#### Choose Deployment Platform
- [ ] Option A: AWS S3 + CloudFront
  - [ ] Create S3 buckets
  - [ ] Configure static hosting
  - [ ] Create CloudFront distributions
  - [ ] Setup DNS records

- [ ] Option B: Cloudflare Pages
  - [ ] Create projects
  - [ ] Connect repos
  - [ ] Configure custom domains

#### Environment Setup
- [ ] Production domain registered
- [ ] SSL/TLS certificates (auto with Cloudflare)
- [ ] Environment variables configured
- [ ] CORS policies set
- [ ] Security headers configured
- [ ] Health check endpoints created

#### Deploy & Verify
- [ ] Build all modules
- [ ] Upload to CDN
- [ ] Test in production environment
- [ ] Verify all routes work
- [ ] Verify modules load
- [ ] Check performance
- [ ] Monitor logs for errors

#### Post-Deployment
- [ ] Monitor for issues (24 hours)
- [ ] Check error logs daily (1 week)
- [ ] Set up alerts
- [ ] Document any issues found
- [ ] Schedule follow-up review

### Code Review & PR

#### PR Setup
- [ ] Feature branch: `repos/worktrees/AgilePlus/phase2-federation`
- [ ] All changes in worktree (not canonical main)
- [ ] PR title: "feat: implement module federation (Phase 2)"
- [ ] PR description includes:
  - [ ] Summary of changes
  - [ ] Links to design docs
  - [ ] Links to local dev guide
  - [ ] Testing instructions (3-terminal setup)
  - [ ] Deployment notes
  - [ ] Screenshots/demos (optional)

#### Code Quality
- [ ] All tests passing
- [ ] Linting clean (ruff, eslint, etc.)
- [ ] TypeScript strict mode passes
- [ ] No console errors/warnings
- [ ] Code coverage adequate
- [ ] No security issues (gitleaks)

#### Review & Merge
- [ ] Code review approved
- [ ] CI/CD checks passing (except Actions billing)
- [ ] All conversations resolved
- [ ] Rebase on main to avoid conflicts
- [ ] Merge via PR (not force push)

---

## Success Criteria

### Functional
- [x] Design document complete (FEDERATED_HYBRID_ARCHITECTURE_PHASE2.md)
- [ ] AgilePlus host loads correctly
- [ ] heliosApp module loads dynamically
- [ ] agent-wave module loads dynamically
- [ ] All modules work standalone
- [ ] Navigation between modules works
- [ ] Error fallbacks display correctly
- [ ] Shared libraries loaded once (no duplication)

### Performance
- [ ] Host loads in < 2 seconds
- [ ] Modules load in < 3 seconds
- [ ] Host bundle < 100KB
- [ ] Module bundles < 150KB each
- [ ] No memory leaks
- [ ] No console errors

### Quality
- [ ] All tests passing
- [ ] 100% TypeScript coverage (strict mode)
- [ ] Linting clean
- [ ] Security audit clean
- [ ] No code duplication
- [ ] Code review approved

### Documentation
- [ ] Design document (FEDERATED_HYBRID_ARCHITECTURE_PHASE2.md)
- [ ] Local dev guide (MODULE_FEDERATION_LOCAL_DEV_GUIDE.md)
- [ ] Deployment guide (FEDERATION_PRODUCTION_DEPLOYMENT.md)
- [ ] Implementation checklist (this file)
- [ ] README.md updated
- [ ] All code documented
- [ ] Troubleshooting guide complete

### Deployment
- [ ] Production URLs configured
- [ ] Health checks implemented
- [ ] Monitoring/alerting set up
- [ ] Rollback procedure documented
- [ ] CI/CD pipeline updated
- [ ] Team trained on deployment

---

## Timeline Summary

| Phase | Days | Deliverables |
|-------|------|--------------|
| 2.1 - Host Setup | 1-2 | vite-federation.config.ts, LayoutShell, Router, Error Boundaries |
| 2.2 - heliosApp Remote | 3-4 | Module config, Dual-mode support, npm scripts, Testing |
| 2.3 - agent-wave Remote | 5-6 | Module config, Dual-mode support, npm scripts, Testing |
| 2.4 - Integration Testing | 7-8 | 3-terminal setup verified, All scenarios tested, Performance checked |
| 2.5 - Docs & Deploy | 9-10 | Design docs, Dev guide, Deploy guide, PR review, Production deploy |

**Total**: 10 working days
**Estimated Start**: After AgilePlus spec approval
**Estimated Completion**: ~2 weeks from start

---

## Risks & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| heliosApp doesn't exist as separate repo | High | Delay implementation | Create empty repo from template, populate gradually |
| agent-wave doesn't exist | High | Delay implementation | Create empty repo from template, populate gradually |
| Shared lib version conflicts | Medium | Module load failures | Pre-check all versions, automate in CI |
| React version mismatch | Medium | Runtime errors | Lock React version, test before merge |
| Module load latency | Medium | Poor UX | Add skeleton loading, code splitting, CDN caching |
| CORS configuration errors | Medium | Modules won't load | Thorough testing in prod environment |
| Security headers too restrictive | Low | Modules blocked | Whitelist remotes in CSP, test thoroughly |

---

## Notes & References

- **Design Doc**: FEDERATED_HYBRID_ARCHITECTURE_PHASE2.md
- **Local Dev Guide**: MODULE_FEDERATION_LOCAL_DEV_GUIDE.md
- **Deployment Guide**: FEDERATION_PRODUCTION_DEPLOYMENT.md
- **Module Federation Docs**: https://module-federation.io
- **Vite + MF**: https://module-federation.io/docs/en/guide/start/vite
- **React + MF**: https://module-federation.io/docs/en/guide/start/react

---

**Document Version**: 1.0
**Last Updated**: 2026-03-29
**Status**: Ready for Phase 2 Implementation
