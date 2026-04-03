# Hot Module Reload (HMR) Setup Guide

This guide explains how to set up and use Hot Module Reload (HMR) for the full Phenotype development stack. HMR enables instant code updates during development without page reloads or losing application state.

---

## Overview

### Services with HMR Support

The Phenotype monorepo contains multiple frontend/web services with HMR capabilities:

| Service | Framework | Port | Status | HMR Enabled |
|---------|-----------|------|--------|-------------|
| **AgilePlus Dashboard** | React + Vite | 5173 | Production | ✅ Yes |
| **heliosApp** | React + Vite (Federation) | 3001 | Standalone/Remote | ✅ Yes |
| **portage/viewer** | React Router + Tailwind | 5174 | Development | ⚠️ Minimal |
| **AgilePlus Docs** | VitePress | 5175 | Development | ✅ Yes |

### Backend Services (No HMR)

Backend services use **file watching and auto-restart** patterns instead of HMR:

- **agileplus-api** (Rust): Managed via `process-compose` with file watchers
- **plane-api** (Python Django): File watcher-based reload
- **plane-web** (Next.js): Built-in HMR via `next dev` (when not in production mode)

---

## Quick Start

### Option 1: Start Full Stack with All HMR Services

```bash
# From repo root
task up           # Start backend services (agileplus-api, NATS, Neo4j, etc.)

# In separate terminal(s), start frontend services
cd AgilePlus/crates/agileplus-dashboard/web && npm run dev

# In another terminal
cd heliosApp && npm run dev

# In another terminal
cd docs && npm run docs:dev
```

### Option 2: Start Individual Service with HMR

```bash
# AgilePlus Dashboard
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-dashboard/web
npm run dev
# Listens on http://localhost:5173
# API proxy: http://localhost:3000

# heliosApp (standalone)
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp
npm run dev
# Listens on http://localhost:3001
# As module federation remote: http://localhost:3001/remoteEntry.js

# heliosApp (federated/remote mode)
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp
npm run dev:remote
# Serves only the remote entry
# Use from host app configured with remotes.heliosApp='heliosApp@http://localhost:3001/remoteEntry.js'

# AgilePlus Docs
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs
npm run docs:dev
# Listens on http://localhost:5175
```

---

## Detailed Service Configuration

### AgilePlus Dashboard (`agileplus-dashboard/web`)

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-dashboard/web/`

**Vite Configuration**:
```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/lib': path.resolve(__dirname, './src/lib'),
      '@/pages': path.resolve(__dirname, './src/pages'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
```

**Start Development**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-dashboard/web
npm install    # First time only
npm run dev    # Start dev server with HMR
```

**Access**:
- Frontend: `http://localhost:5173`
- API requests to `/api/*` automatically proxy to `http://localhost:3000`

**HMR Features**:
- Instant React component updates
- Preserved component state during edits
- Real-time stylesheet changes via Tailwind + PostCSS

**File Watchers**:
- `src/**/*.{tsx,ts,css}` — component and style updates
- `vite.config.ts` — requires manual restart

---

### heliosApp (Module Federation Remote)

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp/`

**Vite Configuration**:
```typescript
// vite.config.ts
export default defineConfig({
  plugins: [
    react(),
    ...federation({
      name: 'heliosApp',
      filename: 'remoteEntry.js',
      exposes: {
        './Dashboard': './src/pages/Dashboard.tsx',
        './Components': './src/components/index.tsx',
        './Hooks': './src/hooks/index.ts',
      },
      shared: {
        react: {
          singleton: true,
          requiredVersion: '^18.0.0',
          strictVersion: false,
        },
        'react-dom': {
          singleton: true,
          requiredVersion: '^18.0.0',
          strictVersion: false,
        },
      },
    }),
  ],
  server: {
    port: 3001,
    hmr: {
      host: 'localhost',
      port: 3001,
    },
  },
  build: {
    target: 'esnext',
    outDir: federationMode ? 'dist-remote' : 'dist',
  },
});
```

**Start Development**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp

# Standalone mode (full app with HMR)
npm install    # First time only
npm run dev    # Start on http://localhost:3001 with HMR

# Federation remote mode (host can load this)
npm run dev:remote # Serves remoteEntry.js on http://localhost:3001
```

**Access**:
- Standalone: `http://localhost:3001`
- Remote entry: `http://localhost:3001/remoteEntry.js`

**HMR Features**:
- React component HMR with state preservation
- Module Federation: updates propagate to host app if host app is also in dev mode

**Module Federation Shared Dependencies**:
- React and React DOM are singletons — host and remote use same instance
- Prevents version conflicts and reduces bundle size

---

### portage/viewer

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/portage/viewer/`

**Vite Configuration**:
```typescript
// vite.config.ts
export default defineConfig({
  plugins: [tailwindcss(), reactRouter(), tsconfigPaths()],
  // HMR implicitly enabled by Vite
});
```

**Start Development**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/portage/viewer
npm install
npm run dev    # Default Vite dev server with HMR
```

---

### AgilePlus Docs (VitePress)

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/`

**Start Development**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs
npm install
npm run docs:dev   # VitePress dev server with HMR
```

**Access**: `http://localhost:5175`

**HMR Features**:
- Markdown and Vue component hot reload
- Sidebar/nav updates without page reload
- Real-time preview of documentation changes

---

## Environment Configuration

### `.env.development` or `.env.local`

For frontend services that need environment variables, create a `.env.local` or `.env.development` file in the service directory:

**AgilePlus Dashboard** (`.env.local`):
```bash
# API Configuration
VITE_API_URL=http://localhost:3000

# Frontend
VITE_DEV_MODE=true
```

**heliosApp** (`.env.local`):
```bash
# Federation mode
FEDERATION_MODE=standalone

# Optional: host URL when used as remote
VITE_HOST_URL=http://localhost:5000
```

> Note: Vite env variables must be prefixed with `VITE_` to be accessible in browser code.

---

## Port Assignments

| Service | Port | Purpose |
|---------|------|---------|
| **AgilePlus Dashboard** | 5173 | Vite dev server (React) |
| **heliosApp** | 3001 | Vite dev server (Module Federation Remote) |
| **portage/viewer** | 5174 | Vite dev server (React Router) |
| **AgilePlus Docs** | 5175 | VitePress dev server |
| **agileplus-api** | 3000 | Backend REST API (managed by process-compose) |
| **plane-api** | 8000 | Django REST API (managed by process-compose) |
| **plane-web** | 3100 | Next.js web UI (managed by process-compose) |

### Changing Ports

To use different ports, edit the Vite config for each service:

```typescript
server: {
  port: 5200,  // Change to desired port
  hmr: {
    host: 'localhost',
    port: 5200,
  },
}
```

---

## File Watcher Configuration

### Vite Default Watching

By default, Vite watches:
- `src/**/*` — all source files
- `public/**/*` — static assets
- `vite.config.ts` — build config (requires restart)
- `tsconfig.json` — TypeScript config (requires restart)
- `package.json` — dependencies (requires restart)

### Customizing Watchers

To exclude directories or add custom patterns, modify `vite.config.ts`:

```typescript
export default defineConfig({
  server: {
    watch: {
      ignored: [
        '**/node_modules/**',
        '**/.git/**',
        '**/dist/**',
        '**/.env.local',
      ],
    },
  },
})
```

---

## API Proxy Configuration

### AgilePlus Dashboard

The dashboard proxies API requests to the backend:

```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3000',  // agileplus-api
      changeOrigin: true,
      // Optionally rewrite paths:
      // pathRewrite: { '^/api': '' }
    },
  },
}
```

**Usage in React**:
```typescript
// All requests to /api/* go to http://localhost:3000
const response = await fetch('/api/v1/specs');
```

---

## Troubleshooting HMR

### Port Already in Use

```bash
# Check what's using the port
lsof -i :5173

# Kill the process
kill -9 <PID>

# Or change the port in vite.config.ts
```

### HMR Not Working (Blank Page)

1. Ensure dev server is running: Check browser DevTools Network tab
2. Check browser console for errors
3. Verify proxy target is correct (`http://localhost:3000` for backend)
4. Clear browser cache: `Ctrl+Shift+Delete` (Chrome) or `Cmd+Shift+Delete` (Safari)

### Module Federation Remote Not Loading

```bash
# Verify remote is serving remoteEntry.js
curl -i http://localhost:3001/remoteEntry.js

# Check host federation config points to correct remote URL
# In host vite.config.ts:
remotes: {
  heliosApp: 'heliosApp@http://localhost:3001/remoteEntry.js'
}
```

### Vite Build Version Too Old

Check `package.json`:
```bash
npm list vite
```

Update if needed:
```bash
npm install vite@latest
```

---

## Development Workflow

### Recommended Terminal Layout

Use multiple terminals for simultaneous service development:

```
Terminal 1: Backend Stack
$ task up
# Outputs: agileplus-api, plane-api, NATS, Neo4j, MinIO, etc.

Terminal 2: AgilePlus Dashboard
$ cd AgilePlus/crates/agileplus-dashboard/web && npm run dev
# VITE v8.0.1 ready in 234 ms
# ➜  local:   http://localhost:5173/

Terminal 3: heliosApp (if doing Module Federation work)
$ cd heliosApp && npm run dev
# VITE v6.0.1 ready in 189 ms
# ➜  local:   http://localhost:3001/

Terminal 4: Logs
$ task logs
# Tails all service logs in real-time
```

### Debugging with Browser DevTools

1. Open `http://localhost:5173` in browser
2. Press `F12` to open DevTools
3. **Sources** tab → check file sources (should show `.tsx` not bundled `.js`)
4. Set breakpoints and reload with `F5`
5. Step through code normally

### Hot Reload Tips

- **Save and watch**: File changes trigger HMR automatically
- **Preserve state**: Component state persists during HMR (Vite Fast Refresh)
- **External resources**: Changes to files outside `src/` may require reload
- **Full page reload**: `Ctrl+Shift+R` (hard refresh) or `npm run dev` restart

---

## Performance Considerations

### HMR Performance Impact

- **Initial build**: ~500ms–1s (Vite's esbuild is fast)
- **HMR update**: ~50–200ms per file (depends on component size)
- **Memory usage**: ~200–300MB per dev server

### Optimizing HMR Speed

1. **Keep components small**: Smaller files = faster updates
2. **Reduce dependencies**: Fewer imports = fewer file watchers
3. **Exclude heavy files**: Add to `watch.ignored` in Vite config
4. **Use CSS modules**: Faster than global CSS
5. **Lazy load routes**: React Router's `lazy()` for code splitting

---

## Backend Service Management

### agileplus-api (Rust)

Managed by `process-compose`. Does NOT support HMR, but has file watchers via `cargo run --release` with inotify-tools:

```bash
task up   # Starts agileplus-api with auto-restart on source changes
```

### Plane Services (Python/Node)

- **plane-api** (Django): Uses Django's development server with file watchers
- **plane-web** (Next.js): In production mode; no HMR (use `npm run dev` for development)
- **plane-worker/plane-beat** (Celery): Auto-restart on code changes

---

## Integrating New Frontend Services

To add HMR to a new frontend service:

### Step 1: Add Vite Config

Create `vite.config.ts`:
```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5200,  // Unique port
    hmr: {
      host: 'localhost',
      port: 5200,
    },
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
```

### Step 2: Update package.json

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }
}
```

### Step 3: Install Dependencies

```bash
npm install
npm run dev
```

---

## CI/CD Considerations

HMR is **development-only**. For production builds, disable HMR:

```typescript
export default defineConfig({
  server: {
    hmr: process.env.NODE_ENV === 'development' ? { host: 'localhost', port: 5173 } : false,
  },
})
```

---

## References

- **Vite HMR Configuration**: https://vitejs.dev/config/server-options.html#server-hmr
- **Vite Fast Refresh**: https://vitejs.dev/guide/features.html#hot-module-replacement
- **Module Federation**: https://module-federation.io/
- **VitePress**: https://vitepress.dev/
- **process-compose**: https://github.com/F1bonacc1/process-compose

---

## Quick Reference Commands

```bash
# Start backend stack
task up

# Start AgilePlus Dashboard with HMR
cd AgilePlus/crates/agileplus-dashboard/web && npm run dev

# Start heliosApp with HMR (standalone)
cd heliosApp && npm run dev

# Start heliosApp as Module Federation remote
cd heliosApp && npm run dev:remote

# Start AgilePlus Docs with HMR
cd AgilePlus/docs && npm run docs:dev

# View all running services
task status

# Tail logs (all services)
task logs

# Restart a single service (e.g., agileplus-api)
process-compose restart agileplus-api

# Stop everything
task down
```

---

## Next Steps

1. **Start the full stack**: `task up` in one terminal
2. **Start dashboard HMR**: `cd AgilePlus/crates/agileplus-dashboard/web && npm run dev`
3. **Open browser**: `http://localhost:5173`
4. **Edit a component**: Save `src/components/Button.tsx` and watch the update instantly
5. **Check logs**: `task logs` to monitor backend activity
