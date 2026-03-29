# Phase 2: Federated Hybrid Architecture - Module Federation Setup

## Executive Summary

Phase 2 implements a **Module Federation** architecture where:
- **AgilePlus** becomes the host application providing unified routing, layout, and navigation
- **heliosApp** and **agent-wave** become remote modules dynamically loaded at runtime
- Users see a seamless, unified dashboard while each app remains independently deployable and developable
- Shared libraries (@phenotype/docs design system, phenotype-shared services) are synchronized across all federated modules

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    AgilePlus Host (Port 3000)               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Layout Shell (Header, Nav, Footer)                    │  │
│  │ Routing Engine (React Router v6+)                     │  │
│  │ Error Boundaries & Module Loading States              │  │
│  └───────────────────────────────────────────────────────┘  │
│                          ↓                                    │
│  ┌──────────────────┬─────────────────┬──────────────────┐  │
│  │ /dashboard/...   │ /forecast/...   │ /settings/...    │  │
│  │ (local routes)   │ (local routes)  │ (local routes)   │  │
│  └──────────────────┴─────────────────┴──────────────────┘  │
│                          ↓                                    │
│  ┌──────────────────┬─────────────────────────────────────┐  │
│  │ /helios/* Route  │ /agent-wave/* Route                 │  │
│  │ Dynamic Import   │ Dynamic Import                      │  │
│  └─────────┬────────┴──────────────┬──────────────────────┘  │
└───────────┼──────────────────────────┼─────────────────────┘
            │                          │
         (HTTP)                     (HTTP)
            ↓                          ↓
┌─────────────────────────┐  ┌──────────────────────────┐
│ heliosApp Remote Module │  │ agent-wave Remote Module │
│   (Port 3001)           │  │   (Port 3002)            │
│                         │  │                          │
│ ┌─────────────────────┐ │  │ ┌────────────────────┐   │
│ │ Exposed Components: │ │  │ │ Exposed Components:│   │
│ │ - Dashboard Page    │ │  │ │ - Forecast Page    │   │
│ │ - Mobile UI Comps   │ │  │ │ - Agent UI Comps   │   │
│ │ - Services/Hooks    │ │  │ │ - Services/Hooks   │   │
│ └─────────────────────┘ │  │ └────────────────────┘   │
│                         │  │                          │
│ Shared:                 │  │ Shared:                  │
│ - @phenotype/docs       │  │ - @phenotype/docs        │
│ - phenotype-shared      │  │ - phenotype-shared       │
│ - React 18, React DOM   │  │ - React 18, React DOM    │
└─────────────────────────┘  └──────────────────────────┘
```

## Design Decisions

### 1. **Framework & Build Tool**
- **Framework**: React 18+ (assumes existing React codebase)
- **Build Tool**: Vite (blazing fast, native ES modules support)
- **Federation Library**: @module-federation/enhanced (latest, best performance)
- **Module Federation Version**: v2+ (latest stable)

### 2. **Module Boundary Design**
- **AgilePlus Host** owns:
  - Layout shell (header, nav, footer)
  - Main router configuration
  - Global error boundaries
  - Shared context providers (theming, auth state)
  - Fallback/loading UI

- **heliosApp Remote** exposes:
  - Dashboard page and components
  - Mobile-specific UI components
  - Service hooks and utilities
  - Can run standalone OR as federated module

- **agent-wave Remote** exposes:
  - Forecasting/prediction page
  - Agent UI components
  - Model/prediction services
  - Can run standalone OR as federated module

### 3. **Shared Dependencies Strategy**
**Shared libraries** (synchronized across host + remotes):
- `react` — host version enforced (18+)
- `react-dom` — host version enforced
- `@phenotype/docs` — design system, must be same version
- `phenotype-shared` — service layer, must be same version
- `react-router-dom` — host provides routing context
- `zustand` or state lib (if used)

**Non-shared** (each module owns):
- Form libraries (react-hook-form, formik)
- Data fetching (axios, fetch)
- Utility libraries (lodash, date-fns)

### 4. **Port Assignment**
- Host (AgilePlus): **3000** (main entry point)
- Remote (heliosApp): **3001** (local dev)
- Remote (agent-wave): **3002** (local dev)

### 5. **Module Loading Strategy**
- **Lazy loading** via React.lazy() + Suspense
- **Error recovery**: fallback UI if module fails to load
- **Health checks**: optional module readiness endpoints
- **Dual-mode support**: remotes can run standalone or federated

## Implementation Plan

### Part A: AgilePlus Host Setup

#### Step 1: Install Dependencies
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
npm install --save-dev @module-federation/enhanced
npm install --save-dev @vitejs/plugin-react
# Verify Vite is configured with Vite + React plugin
```

#### Step 2: Create Module Federation Config
File: `/Users/kooshapari/CodeProjects/Phenotype/repos/vite-federation.config.ts`

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { definePluginConfig } from '@module-federation/enhanced/dist/typings';

const federationConfig: definePluginConfig = {
  name: 'agileplus-host',
  filename: 'remoteEntry.js',
  remotes: {
    heliosApp: 'http://localhost:3001/assets/remoteEntry.js',
    'agent-wave': 'http://localhost:3002/assets/remoteEntry.js',
  },
  exposes: {
    // Host can expose shared layout/context if needed
    './LayoutShell': './src/components/LayoutShell.tsx',
  },
  shared: {
    react: { singleton: true, requiredVersion: '18.0.0' },
    'react-dom': { singleton: true, requiredVersion: '18.0.0' },
    '@phenotype/docs': { singleton: true, eager: true },
    'phenotype-shared': { singleton: true, eager: true },
    'react-router-dom': { singleton: true, requiredVersion: '6.0.0' },
  },
};

export default defineConfig({
  plugins: [
    react(),
    // @module-federation/enhanced plugin here
  ],
  server: {
    port: 3000,
    strictPort: false,
  },
});
```

#### Step 3: Update AgilePlus Router
File: `/Users/kooshapari/CodeProjects/Phenotype/repos/src/routes/index.tsx`

```typescript
import React, { Suspense } from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import LayoutShell from '../components/LayoutShell';
import LocalDashboard from '../pages/LocalDashboard';

// Lazy-load remote modules
const HeliosRemote = React.lazy(() =>
  import('heliosApp/Dashboard').catch(err => {
    console.error('Failed to load heliosApp:', err);
    return { default: () => <div>heliosApp failed to load</div> };
  })
);

const AgentWaveRemote = React.lazy(() =>
  import('agent-wave/Forecast').catch(err => {
    console.error('Failed to load agent-wave:', err);
    return { default: () => <div>agent-wave failed to load</div> };
  })
);

const ModuleErrorBoundary = ({ children, moduleName }) => {
  return (
    <ErrorBoundary fallback={<div>{moduleName} failed to load</div>}>
      {children}
    </ErrorBoundary>
  );
};

export const router = createBrowserRouter([
  {
    element: <LayoutShell />,
    children: [
      { path: '/', element: <LocalDashboard /> },
      {
        path: '/dashboard/helios/*',
        element: (
          <ModuleErrorBoundary moduleName="heliosApp">
            <Suspense fallback={<LoadingSpinner />}>
              <HeliosRemote />
            </Suspense>
          </ModuleErrorBoundary>
        ),
      },
      {
        path: '/forecast/agent-wave/*',
        element: (
          <ModuleErrorBoundary moduleName="agent-wave">
            <Suspense fallback={<LoadingSpinner />}>
              <AgentWaveRemote />
            </Suspense>
          </ModuleErrorBoundary>
        ),
      },
    ],
  },
]);

export function App() {
  return <RouterProvider router={router} />;
}
```

#### Step 4: Create Layout Shell
File: `/Users/kooshapari/CodeProjects/Phenotype/repos/src/components/LayoutShell.tsx`

```typescript
import React from 'react';
import { Outlet, Link } from 'react-router-dom';
import { useTheme } from '@phenotype/docs';

export default function LayoutShell() {
  const { theme } = useTheme();

  return (
    <div className={`layout-shell ${theme}`}>
      <header className="header">
        <nav className="nav">
          <Link to="/">AgilePlus</Link>
          <Link to="/dashboard/helios">Helios Dashboard</Link>
          <Link to="/forecast/agent-wave">Agent Wave Forecast</Link>
        </nav>
      </header>

      <main className="main-content">
        <Outlet />
      </main>

      <footer className="footer">
        <p>Federated Architecture &copy; 2026</p>
      </footer>
    </div>
  );
}
```

### Part B: heliosApp Remote Module Setup

#### Step 1: Create heliosApp Module Federation Config
File: `heliosApp/vite-federation.config.ts` (same structure as AgilePlus, but as remote)

```typescript
const federationConfig: definePluginConfig = {
  name: 'heliosApp',
  filename: 'remoteEntry.js',
  exposes: {
    './Dashboard': './src/pages/Dashboard.tsx',
    './MobileComponents': './src/components/Mobile/index.ts',
    './Services': './src/services/index.ts',
  },
  shared: {
    react: { singleton: true, requiredVersion: '18.0.0' },
    'react-dom': { singleton: true, requiredVersion: '18.0.0' },
    '@phenotype/docs': { singleton: true, eager: true },
    'phenotype-shared': { singleton: true, eager: true },
  },
};
```

#### Step 2: Support Dual-Mode (Standalone + Federated)
File: `heliosApp/src/bootstrap.ts`

```typescript
// Determine mode based on environment variable
const FEDERATION_MODE = process.env.FEDERATION_MODE || 'standalone';

if (FEDERATION_MODE === 'standalone') {
  // Standalone mode: render full app
  import('./app').then(module => module.renderApp());
} else {
  // Federated mode: do nothing, let host load modules
  // Modules are exported from vite-federation.config.ts exposes
  console.log('heliosApp running in federated mode');
}
```

#### Step 3: Add package.json Scripts
```json
{
  "scripts": {
    "dev": "FEDERATION_MODE=standalone vite",
    "dev:remote": "FEDERATION_MODE=federated vite",
    "build": "vite build",
    "build:remote": "vite build --mode federated"
  }
}
```

### Part C: agent-wave Remote Module Setup
Same as Part B, but for agent-wave.

### Part D: Local Development Testing

#### Test Setup
```bash
# Terminal 1: Start AgilePlus host
cd repos
npm run dev
# Output: Local: http://localhost:3000

# Terminal 2: Start heliosApp remote
cd repos  # or separate repo if exists
npm run dev:remote
# Output: Local: http://localhost:3001

# Terminal 3: Start agent-wave remote
cd repos  # or separate repo if exists
npm run dev:remote
# Output: Local: http://localhost:3002
```

#### Verification Checklist
- [ ] AgilePlus loads on http://localhost:3000
- [ ] Navigation links visible: "Helios Dashboard", "Agent Wave Forecast"
- [ ] Click "Helios Dashboard" → heliosApp module loads from 3001
- [ ] heliosApp renders correctly with @phenotype/docs theme
- [ ] Click "Agent Wave Forecast" → agent-wave module loads from 3002
- [ ] agent-wave renders correctly with @phenotype/docs theme
- [ ] Kill heliosApp (port 3001) → click Dashboard → fallback error UI shown
- [ ] Restart heliosApp → refresh → module loads successfully
- [ ] Kill AgilePlus host → start heliosApp standalone with `npm run dev` → full app loads

### Part E: Error Handling & Fallbacks

#### Module Loading Failure Scenario
```typescript
// In LayoutShell or dedicated component
export function ModuleFallback({ moduleName, error }) {
  return (
    <div className="module-error">
      <h2>Module Load Failed</h2>
      <p>{moduleName} failed to load. Please try:</p>
      <ul>
        <li>Refresh the page (Cmd+R / Ctrl+R)</li>
        <li>Clear browser cache</li>
        <li>Check that {moduleName} service is running</li>
      </ul>
      {error && <details><summary>Error details</summary>{error.message}</details>}
    </div>
  );
}
```

### Part F: Production Deployment

#### Deployment Strategy
1. **Separate CI/CD Pipelines**:
   - AgilePlus host: builds, tests, deploys to agileplus.example.com
   - heliosApp remote: builds, tests, deploys to remote-helios.example.com
   - agent-wave remote: builds, tests, deploys to remote-agent-wave.example.com

2. **Environment Variables**:
   ```
   # In production
   VITE_HELIOS_REMOTE_URL=https://remote-helios.example.com/remoteEntry.js
   VITE_AGENT_WAVE_REMOTE_URL=https://remote-agent-wave.example.com/remoteEntry.js
   ```

3. **Shared Library Versioning**:
   - All modules must use same versions of @phenotype/docs and phenotype-shared
   - Use semantic versioning + version mismatch detection
   - CI pipeline verifies all modules use compatible versions

4. **Health Checks**:
   ```typescript
   // Each remote exposes health check endpoint
   // GET /health → { status: 'ok', version: '1.0.0' }

   // Host periodically checks remote health
   setInterval(async () => {
     const health = await fetch('http://remote.example.com/health');
     // Update remote status in UI
   }, 30000);
   ```

## Files & Locations

### Configuration Files
| File | Location | Purpose |
|------|----------|---------|
| `vite-federation.config.ts` | AgilePlus root | Host federation config |
| `vite-federation.config.ts` | heliosApp root | Remote federation config |
| `vite-federation.config.ts` | agent-wave root | Remote federation config |

### Source Files
| File | Location | Purpose |
|------|----------|---------|
| `src/components/LayoutShell.tsx` | AgilePlus | Main layout + navigation |
| `src/routes/index.tsx` | AgilePlus | Router with remote routes |
| `src/bootstrap.ts` | heliosApp | Dual-mode bootstrap |
| `src/bootstrap.ts` | agent-wave | Dual-mode bootstrap |

### Documentation Files
| File | Location | Purpose |
|------|----------|---------|
| `docs/reference/MODULE_FEDERATION_SETUP.md` | repos | Architecture + dev guide |
| `docs/reference/FEDERATION_DEPLOYMENT.md` | repos | Production deployment guide |
| `README.md` | repos root | Updated with federation section |

## Phased Implementation

### Phase 2.1: Host Setup (Days 1-2)
- Install @module-federation/enhanced
- Create Module Federation config for AgilePlus
- Update routing to support dynamic remotes
- Create LayoutShell component
- Create error boundaries + loading UI

### Phase 2.2: heliosApp Remote (Days 3-4)
- Create Module Federation config
- Implement dual-mode (standalone/federated)
- Add npm scripts (dev, dev:remote, build, build:remote)
- Test standalone mode
- Test federated mode with host

### Phase 2.3: agent-wave Remote (Days 5-6)
- Same as Phase 2.2 for agent-wave

### Phase 2.4: Integration Testing (Days 7-8)
- All 3 modules running locally
- Navigation between modules works
- Error fallback scenarios tested
- Shared library versions aligned

### Phase 2.5: Documentation & Deployment (Days 9-10)
- Write MODULE_FEDERATION_SETUP.md
- Write FEDERATION_DEPLOYMENT.md
- Update main README.md
- Create production deployment guide
- Test production URLs

## Success Criteria

- [ ] AgilePlus host loads and renders correctly
- [ ] heliosApp module loads dynamically from AgilePlus
- [ ] agent-wave module loads dynamically from AgilePlus
- [ ] Each module works standalone (without host)
- [ ] All modules use same @phenotype/docs version
- [ ] Error fallbacks show when module unavailable
- [ ] Shared dependencies resolved correctly (no duplication)
- [ ] Local dev setup works with 3 terminals
- [ ] Documentation covers setup, troubleshooting, deployment
- [ ] PR review passes CI/CD checks

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Shared lib version mismatch | Modules fail to load | Lock versions in monorepo, pre-merge check |
| Module load latency | Poor UX | Add skeleton loading UI, code splitting |
| Browser cache conflicts | Module not updated | Add cache-busting via version hash |
| Network failure (remote down) | User sees error | Graceful fallback, offline mode (optional) |
| React version mismatch | Runtime errors | Enforce singleton: true for React in shared |

## Next Steps

1. **Create AgilePlus spec** in AgilePlus (`agileplus specify`)
2. **Implement Part A** (Host setup) in AgilePlus worktree
3. **Implement Part B** (heliosApp remote) in heliosApp (or create if needed)
4. **Implement Part C** (agent-wave remote) in agent-wave (or create if needed)
5. **Test locally** with 3 terminals
6. **Document deployment** for production
7. **Open PR** with all changes

---

## Appendices

### Appendix A: Shared Library Alignment

**Shared libraries MUST have identical versions across all modules:**

```json
{
  "dependencies": {
    "react": "18.2.0",
    "react-dom": "18.2.0",
    "@phenotype/docs": "1.5.0",
    "phenotype-shared": "2.1.0",
    "react-router-dom": "6.20.0"
  }
}
```

**Pre-merge check:**
```bash
# Script to verify all modules use same versions
agileplus verify-federation-versions
# Output: All modules aligned ✓
```

### Appendix B: Module Federation Config Reference

**Host Config** (AgilePlus):
- `name`: 'agileplus-host'
- `remotes`: { heliosApp, 'agent-wave' }
- `exposes`: optional (can expose LayoutShell if needed)
- `shared`: React, design system, services

**Remote Config** (heliosApp, agent-wave):
- `name`: module name
- `exposes`: Dashboard, Components, Services
- `shared`: same as host
- **No remotes** (remotes only in host)

### Appendix C: Troubleshooting

**Module fails to load:**
1. Check port is running: `curl http://localhost:3001/remoteEntry.js`
2. Check browser console for network errors
3. Verify shared dependencies versions match

**Shared library version mismatch:**
1. Check package.json in all modules
2. Run `npm ls <library>` in each
3. Update to same version, reinstall

**Local dev hangs:**
1. One of the 3 services crashed
2. Port conflict (use `lsof -i :3000` to find)
3. Clear node_modules and reinstall

---

**Document Version**: 1.0
**Last Updated**: 2026-03-29
**Status**: Phase 2 Design Document (Ready for Implementation)
