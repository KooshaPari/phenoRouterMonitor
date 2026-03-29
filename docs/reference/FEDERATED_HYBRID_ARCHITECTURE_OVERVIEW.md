# Federated Hybrid Architecture - Complete Overview

## Project Vision

Create a **unified, federated dashboard** where:
- **AgilePlus** serves as the host application (single entry point, main routing, shared layout)
- **heliosApp** and **agent-wave** load dynamically as federated remote modules
- Users see a seamless, integrated experience with shared theming and services
- Each module remains independently deployable and maintainable

## What is Module Federation?

Module Federation is a JavaScript architecture pattern that allows:
1. **Dynamic loading**: One application loads code from another at runtime
2. **Shared dependencies**: Multiple applications share the same React, libraries, etc. (loaded once)
3. **Independent deployment**: Each module can be deployed separately without redeploying the host
4. **Lazy loading**: Modules only load when accessed (not at startup)

### Example
```
User opens https://agileplus.example.com
↓
Browser loads AgilePlus host (main layout, nav, router)
↓
User clicks "Helios Dashboard"
↓
Browser dynamically fetches heliosApp module from https://remote-helios.example.com/remoteEntry.js
↓
Module loads into AgilePlus layout
↓
User sees integrated dashboard with shared theme
```

## Problem We're Solving

### Before Module Federation (Monolithic Approach)
```
Problem: All apps bundled together
agileplus.js (500KB)
  ├── AgilePlus code (200KB)
  ├── heliosApp code (150KB)
  ├── agent-wave code (100KB)
  └── shared libraries (50KB)

Result:
- Slow initial load (download 500KB)
- Can't deploy modules independently
- Any change requires full rebuild
- Code is tightly coupled
```

### After Module Federation (Federated Approach)
```
Solution: Separate bundles, loaded on demand
agileplus.js (100KB) - Host loads first
├── Layout, nav, routing
├── Shared libraries (React, @phenotype/docs)
└── Error boundaries

remote-helios.js (100KB) - Loads when needed
├── Helios dashboard code
├── Uses shared React from host

remote-agent-wave.js (80KB) - Loads when needed
├── Agent wave forecast code
├── Uses shared React from host

Result:
- Fast initial load (download 100KB)
- Deploy heliosApp independently without touching AgilePlus
- Change agent-wave, deploy instantly
- Modules are decoupled but share dependencies
```

## Architecture Components

### 1. AgilePlus Host (Port 3000)
**Responsibilities:**
- Main entry point (index.html, router)
- Layout shell (header, nav, footer)
- Global error boundaries
- Module loading orchestration
- Shared context (theme, auth, etc.)

**Does NOT contain:**
- Dashboard logic (that's heliosApp)
- Forecast logic (that's agent-wave)

**Exposes:**
- Layout components (optional)
- Theme context (optional)

### 2. heliosApp Remote (Port 3001)
**Responsibilities:**
- Dashboard page and components
- Mobile UI components
- Dashboard-specific services

**Can run two ways:**
- Standalone: `npm run dev` → full app on port 3001
- Federated: `npm run dev:remote` → module loaded by host at port 3001

**Exposes:**
- ./Dashboard (main page)
- ./MobileComponents (UI library)
- ./Services (business logic)

### 3. agent-wave Remote (Port 3002)
**Responsibilities:**
- Forecasting/prediction pages
- Agent UI components
- Prediction services

**Can run two ways:**
- Standalone: `npm run dev` → full app on port 3002
- Federated: `npm run dev:remote` → module loaded by host at port 3002

**Exposes:**
- ./Forecast (main page)
- ./AgentComponents (UI library)
- ./Services (prediction logic)

### 4. Shared Libraries (Synchronized)
**Must be identical across all modules:**
- `react@18.2.0`
- `react-dom@18.2.0`
- `@phenotype/docs@1.5.0` (design system)
- `phenotype-shared@2.1.0` (services)
- `react-router-dom@6.20.0`

**Benefit**: Loaded once in host, reused by all remotes (saves 200KB+ of duplicated code)

## Development Workflow

### Local Development (3-Terminal Setup)

```bash
# Terminal 1: Start AgilePlus host
cd repos
npm install
npm run dev
# Open http://localhost:3000

# Terminal 2: Start heliosApp remote
cd heliosApp
npm install
npm run dev:remote
# Server on http://localhost:3001

# Terminal 3: Start agent-wave remote
cd agent-wave
npm install
npm run dev:remote
# Server on http://localhost:3002
```

### User Journey (Local Dev)
1. Open http://localhost:3000 (AgilePlus loads)
2. See navigation: "Home", "Helios Dashboard", "Agent Wave Forecast"
3. Click "Helios Dashboard"
4. Module fetches from localhost:3001 and loads
5. User sees dashboard inside AgilePlus layout
6. Click "Agent Wave Forecast"
7. Module fetches from localhost:3002 and loads
8. User sees forecast inside AgilePlus layout
9. All modules share @phenotype/docs theme

### Standalone Development (If Needed)
```bash
# Develop heliosApp without the host
npm run dev  # not dev:remote
# Opens http://localhost:3001 as full standalone app
# Still uses @phenotype/docs theme
# Can test standalone features
```

## Production Deployment

### Architecture
```
User → https://agileplus.example.com (CloudFront CDN)
         ↓
    AgilePlus Host (S3 bucket)
         ↓
         └─ Click module link
            ↓
            → https://remote-helios.example.com (CloudFront CDN)
            → S3 bucket for heliosApp remote
```

### Deployment Process
1. **Build all modules** separately:
   ```bash
   npm run build      # Each module builds independently
   ```

2. **Upload to CDN**:
   ```bash
   # AgilePlus → S3 agileplus.example.com
   # heliosApp → S3 remote-helios.example.com
   # agent-wave → S3 remote-agent-wave.example.com
   ```

3. **Cloudfront distributions** cache and serve

4. **Independent deployments**:
   ```bash
   # Update heliosApp dashboard
   npm run build:remote
   aws s3 sync dist/ s3://remote-helios.example.com/
   # Done! No host rebuild needed.
   ```

## Configuration Files

### AgilePlus: vite-federation.config.ts
```typescript
{
  name: 'agileplus-host',
  remotes: {
    heliosApp: 'http://localhost:3001/assets/remoteEntry.js',
    'agent-wave': 'http://localhost:3002/assets/remoteEntry.js',
  },
  shared: {
    react: { singleton: true },
    '@phenotype/docs': { singleton: true, eager: true },
  },
}
```

### heliosApp: vite-federation.config.ts
```typescript
{
  name: 'heliosApp',
  exposes: {
    './Dashboard': './src/pages/Dashboard.tsx',
    './MobileComponents': './src/components/Mobile/index.ts',
  },
  shared: {
    react: { singleton: true },
    '@phenotype/docs': { singleton: true, eager: true },
  },
}
```

## Success Metrics

### Functional
- [x] Design document complete
- [ ] All 3 modules runnable (local dev)
- [ ] Navigation between modules works
- [ ] Each module standalone-capable
- [ ] Error fallbacks shown when module unavailable

### Performance
- [ ] Host loads in < 2 seconds
- [ ] Modules load in < 3 seconds each
- [ ] No JavaScript duplication (React loaded once)
- [ ] Network requests optimized (gzip, minification)

### Quality
- [ ] All tests passing
- [ ] No console errors/warnings
- [ ] Linting clean
- [ ] TypeScript strict mode
- [ ] Security audit clean

### Operations
- [ ] Production URLs working
- [ ] Health checks implemented
- [ ] Monitoring/alerting configured
- [ ] Rollback procedure documented
- [ ] CI/CD automated

## Documentation Deliverables

1. **FEDERATED_HYBRID_ARCHITECTURE_PHASE2.md** ✓
   - Full design specification
   - Architecture diagrams
   - Implementation guide
   - Part A-F breakdown

2. **MODULE_FEDERATION_LOCAL_DEV_GUIDE.md** ✓
   - 3-terminal quick start
   - Troubleshooting
   - Testing scenarios
   - Configuration reference

3. **FEDERATION_PRODUCTION_DEPLOYMENT.md** ✓
   - Deployment architecture
   - AWS S3 + CloudFront setup
   - Cloudflare Pages setup
   - Monitoring & alerting
   - Rollback procedures

4. **FEDERATION_IMPLEMENTATION_CHECKLIST.md** ✓
   - Phase-by-phase checklist
   - Success criteria
   - Risk mitigation
   - Timeline

## Timeline & Effort

### Phases
| Phase | Duration | Focus |
|-------|----------|-------|
| 2.1 | Days 1-2 | AgilePlus host setup (vite-federation config, router, error boundaries) |
| 2.2 | Days 3-4 | heliosApp remote module (dual-mode support, npm scripts) |
| 2.3 | Days 5-6 | agent-wave remote module (dual-mode support, npm scripts) |
| 2.4 | Days 7-8 | Integration testing (3-terminal setup, all scenarios) |
| 2.5 | Days 9-10 | Documentation, deployment, PR review |

**Total**: ~10 working days
**Start**: After this design is approved
**Completion**: ~2 weeks

## Key Decisions

### 1. Framework & Tool
- **React 18+** (matches existing AgilePlus stack)
- **Vite** (fast bundling, native ES modules support)
- **@module-federation/enhanced** (latest, best performance)

### 2. Shared Dependencies
- Enforced via `singleton: true` in Module Federation config
- Verified in CI before merge
- Version locked in package.json

### 3. Error Handling
- Graceful fallback UI when module fails to load
- Clear error messages to user
- Retry button to re-attempt load

### 4. Dual-Mode Support
- All remotes can run as standalone OR federated
- Environment variable controls mode: `FEDERATION_MODE=standalone|federated`
- npm scripts: `npm run dev` (standalone), `npm run dev:remote` (federated)

### 5. Port Assignment
- Host: **3000** (main entry point)
- heliosApp: **3001** (remote module)
- agent-wave: **3002** (remote module)
- All configurable via environment/config

## Risk Assessment

| Risk | Probability | Mitigation |
|------|-------------|-----------|
| Module load failure | Medium | Error boundaries, fallback UI, health checks |
| Version conflicts | Medium | Lock versions, pre-merge checks, automated tests |
| CORS issues | Low | Proper headers, test before deploy |
| Performance issues | Low | Code splitting, CDN caching, monitoring |

## Next Steps

1. **Approve this design** ✓ (you're reading it)
2. **Create AgilePlus spec** via `agileplus specify`
3. **Create feature branch** in repos/worktrees/
4. **Implement Phase 2.1** (AgilePlus host)
5. **Implement Phase 2.2** (heliosApp)
6. **Implement Phase 2.3** (agent-wave)
7. **Test locally** (3-terminal setup)
8. **Deploy to production**
9. **Monitor for issues**
10. **Document lessons learned**

## References

- **Module Federation Docs**: https://module-federation.io
- **Vite + MF**: https://module-federation.io/docs/en/guide/start/vite
- **React Integration**: https://module-federation.io/docs/en/guide/start/react
- **Shared Dependencies**: https://module-federation.io/docs/en/guide/advanced/shared-api

---

**Document Version**: 1.0
**Status**: Design Complete, Ready for Implementation
**Last Updated**: 2026-03-29
**Next Phase**: AgilePlus Spec Creation & Phase 2.1 Implementation
