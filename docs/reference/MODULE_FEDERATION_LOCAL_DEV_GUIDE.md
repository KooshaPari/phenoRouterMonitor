# Module Federation: Local Development Setup Guide

## Quick Start (3-Terminal Setup)

### Prerequisites
- Node.js 18+ installed
- npm or yarn package manager
- AgilePlus, heliosApp, and agent-wave repos available
- All modules on same network/machine

### Start Local Dev Environment

**Terminal 1: Start AgilePlus Host**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
npm install  # if not already done
npm run dev
# Watch for output:
# ➜ Local: http://localhost:3000/
```

**Terminal 2: Start heliosApp Remote**
```bash
cd /path/to/heliosApp  # or repos if monorepo
npm install
npm run dev:remote  # or FEDERATION_MODE=federated npm run dev
# Watch for output:
# ➜ Local: http://localhost:3001/
```

**Terminal 3: Start agent-wave Remote**
```bash
cd /path/to/agent-wave  # or repos if monorepo
npm install
npm run dev:remote
# Watch for output:
# ➜ Local: http://localhost:3002/
```

### Verify Setup
1. Open browser: http://localhost:3000
2. You should see AgilePlus layout with navigation menu
3. Click "Helios Dashboard" → waits for 1-2 seconds → renders heliosApp module
4. Click "Agent Wave Forecast" → renders agent-wave module
5. All modules use same @phenotype/docs theme

## Troubleshooting Local Dev

### Issue: "heliosApp Remote Failed to Load"

**Symptoms**: Click "Helios Dashboard" → error message shows

**Solutions**:
1. **Check port 3001 is running**:
   ```bash
   curl http://localhost:3001/assets/remoteEntry.js
   # Should return JavaScript code, not 404
   ```

2. **Check browser Network tab**:
   - Open DevTools (F12)
   - Go to Network tab
   - Click "Helios Dashboard"
   - Look for request to `localhost:3001/assets/remoteEntry.js`
   - If 404 or Connection refused → Terminal 2 not running

3. **Kill Terminal 2, restart**:
   ```bash
   # Terminal 2
   npm run dev:remote
   ```

4. **Check console for version mismatch**:
   ```
   Uncaught TypeError: Cannot find module 'react'
   ```
   → All modules MUST use same React version (see Appendix A)

### Issue: Shared Library Version Mismatch

**Symptoms**:
```
Uncaught TypeError: Cannot find module '@phenotype/docs'
or
Multiple versions of React loaded
```

**Solutions**:
1. **Check package.json in all modules**:
   ```bash
   # In each module root
   npm ls react
   npm ls @phenotype/docs
   npm ls phenotype-shared
   ```

2. **All must show SAME version**:
   ```
   agileplus@1.0.0
   └── react@18.2.0

   heliosApp@1.0.0
   └── react@18.2.0

   agent-wave@1.0.0
   └── react@18.2.0
   ```

3. **If versions differ**:
   ```bash
   # Update to same version in all modules
   npm install react@18.2.0 react-dom@18.2.0 --save
   npm install @phenotype/docs@1.5.0 --save
   npm install phenotype-shared@2.1.0 --save
   ```

4. **Clear and reinstall**:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   npm run dev:remote
   ```

### Issue: Port Already in Use

**Symptoms**:
```
Error: listen EADDRINUSE: address already in use :::3000
```

**Solutions**:
1. **Find process using port**:
   ```bash
   lsof -i :3000  # macOS/Linux
   netstat -ano | findstr :3000  # Windows
   ```

2. **Kill process**:
   ```bash
   kill -9 <PID>  # macOS/Linux
   taskkill /PID <PID> /F  # Windows
   ```

3. **Or use different port**:
   ```bash
   # In Terminal 1
   PORT=3010 npm run dev

   # Then update Terminal 2 & 3 to point to 3010
   # Edit vite-federation.config.ts in AgilePlus
   # Change remotes URL from localhost:3000 to localhost:3010
   ```

### Issue: Module Timeout on Load

**Symptoms**:
```
Timeout waiting for module heliosApp/Dashboard (30s)
```

**Solutions**:
1. **Check remote is responding**:
   ```bash
   curl -v http://localhost:3001/assets/remoteEntry.js
   # Should return 200 OK with JavaScript code
   ```

2. **Check network tab latency**:
   - Look for slow network in DevTools Network tab
   - If >5 seconds → possible performance issue
   - Check if Terminal 2/3 is frozen or processing

3. **Increase timeout** (temporary debug only):
   ```typescript
   // In src/routes/index.tsx
   const HeliosRemote = React.lazy(() =>
     Promise.race([
       import('heliosApp/Dashboard'),
       new Promise((_, reject) =>
         setTimeout(() => reject(new Error('Timeout')), 10000) // 10s
       ),
     ])
   );
   ```

### Issue: HMR (Hot Module Reload) Not Working

**Symptoms**:
```
Make a change to heliosApp code → page doesn't update
```

**Solutions**:
1. **Check file was saved** with correct format
2. **HMR may be disabled for federated modules**:
   ```bash
   # Add to vite.config.ts in heliosApp
   hmr: {
     host: 'localhost',
     port: 3001,
   }
   ```

3. **Hard refresh browser** (Cmd+Shift+R / Ctrl+Shift+R) if HMR doesn't work

## Testing Scenarios

### Scenario 1: All Modules Running
```
✓ AgilePlus host loads (port 3000)
✓ heliosApp remote responds (port 3001)
✓ agent-wave remote responds (port 3002)
✓ Click "Helios Dashboard" → loads module
✓ Click "Agent Wave Forecast" → loads module
```

### Scenario 2: One Remote Down (heliosApp Offline)
```
1. Kill Terminal 2 (heliosApp)
2. In browser, click "Helios Dashboard"
3. Expected: Show fallback error message
   "heliosApp failed to load. Please try:
    - Refresh the page
    - Check that heliosApp service is running"
4. Restart Terminal 2
5. Refresh browser
6. Expected: Module loads successfully
```

### Scenario 3: Shared Library Version Mismatch
```
1. In heliosApp package.json, change react to 18.1.0
2. npm install
3. npm run dev:remote
4. In browser, click "Helios Dashboard"
5. Expected: Error in console about shared dependency
   "Cannot find module 'react@18.2.0', found '@18.1.0'"
6. Fix by updating heliosApp react back to 18.2.0
```

### Scenario 4: Standalone Mode (No Host)
```
1. Kill Terminal 1 (AgilePlus host)
2. In Terminal 2, run: npm run dev  (not dev:remote)
3. Open browser: http://localhost:3001
4. Expected: heliosApp loads as full standalone app
5. All features work without AgilePlus host
```

## Configuration Reference

### AgilePlus vite-federation.config.ts

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@module-federation/enhanced/dist/plugins/vite';

const federationConfig = {
  name: 'agileplus-host',
  filename: 'remoteEntry.js',
  remotes: {
    heliosApp: 'http://localhost:3001/assets/remoteEntry.js',
    'agent-wave': 'http://localhost:3002/assets/remoteEntry.js',
  },
  shared: {
    react: { singleton: true, requiredVersion: '18.2.0' },
    'react-dom': { singleton: true, requiredVersion: '18.2.0' },
    '@phenotype/docs': { singleton: true, eager: true },
    'phenotype-shared': { singleton: true, eager: true },
    'react-router-dom': { singleton: true, requiredVersion: '6.20.0' },
  },
};

export default defineConfig({
  plugins: [
    react(),
    federation(federationConfig),
  ],
  server: {
    port: 3000,
    strictPort: false,
    fs: {
      allow: ['..'],
    },
  },
});
```

### heliosApp vite-federation.config.ts

```typescript
const federationConfig = {
  name: 'heliosApp',
  filename: 'remoteEntry.js',
  exposes: {
    './Dashboard': './src/pages/Dashboard.tsx',
    './MobileComponents': './src/components/Mobile/index.ts',
  },
  shared: {
    react: { singleton: true, requiredVersion: '18.2.0' },
    'react-dom': { singleton: true, requiredVersion: '18.2.0' },
    '@phenotype/docs': { singleton: true, eager: true },
    'phenotype-shared': { singleton: true, eager: true },
    'react-router-dom': { singleton: true, requiredVersion: '6.20.0' },
  },
};
```

### Bootstrap Mode Detection

Both heliosApp and agent-wave should detect if running in standalone vs. federated mode:

```typescript
// src/index.tsx or src/main.tsx
const FEDERATION_MODE = process.env.FEDERATION_MODE || 'standalone';

if (FEDERATION_MODE === 'standalone') {
  // Render full application
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
} else {
  // Federated mode: modules exported, no root render
  console.log('Running in federated mode, awaiting host...');
}
```

### package.json Scripts

**AgilePlus**:
```json
{
  "scripts": {
    "dev": "vite --config vite-federation.config.ts",
    "build": "vite build --config vite-federation.config.ts"
  }
}
```

**heliosApp**:
```json
{
  "scripts": {
    "dev": "FEDERATION_MODE=standalone vite",
    "dev:remote": "FEDERATION_MODE=federated vite --config vite-federation.config.ts",
    "build": "vite build",
    "build:remote": "vite build --config vite-federation.config.ts --mode federated"
  }
}
```

## Environment Variables

### For Local Development

**In Terminal 1 (AgilePlus)**:
```bash
# Default - uses localhost:3001 and localhost:3002
npm run dev
```

**Custom Remote Ports**:
```bash
# If using different ports
VITE_HELIOS_REMOTE=http://192.168.1.100:3001/assets/remoteEntry.js \
VITE_AGENT_WAVE_REMOTE=http://192.168.1.100:3002/assets/remoteEntry.js \
npm run dev
```

### For Production

**In CI/CD**:
```bash
VITE_HELIOS_REMOTE=https://remote-helios.example.com/assets/remoteEntry.js \
VITE_AGENT_WAVE_REMOTE=https://remote-agent-wave.example.com/assets/remoteEntry.js \
npm run build
```

## Debugging Tips

### Enable Verbose Logging
```typescript
// In src/routes/index.tsx
const HeliosRemote = React.lazy(() => {
  console.log('Attempting to load heliosApp...');
  return import('heliosApp/Dashboard')
    .then(mod => {
      console.log('✓ heliosApp loaded successfully');
      return mod;
    })
    .catch(err => {
      console.error('✗ heliosApp failed to load:', err);
      return { default: () => <ErrorFallback error={err} /> };
    });
});
```

### Check Module Exports
```bash
# In Terminal 2 (heliosApp)
curl http://localhost:3001/assets/remoteEntry.js | head -50
# Should show exports like:
# __webpack_require__.d(exports, {
#   "./Dashboard": () => (Dashboard)
# });
```

### Monitor Network Requests
1. Open DevTools (F12)
2. Go to Network tab
3. Filter by XHR
4. Click navigation link to trigger module load
5. Look for remoteEntry.js request
6. Check if 200 OK or error

## Performance Monitoring

### Measure Module Load Time
```typescript
const startTime = performance.now();

const HeliosRemote = React.lazy(() =>
  import('heliosApp/Dashboard').then(mod => {
    const loadTime = performance.now() - startTime;
    console.log(`heliosApp loaded in ${loadTime.toFixed(2)}ms`);
    return mod;
  })
);
```

### Check Bundle Size
```bash
# In AgilePlus
npm run build
ls -lh dist/assets/remoteEntry.js
# Should be <50KB

# In heliosApp
npm run build:remote
ls -lh dist/assets/remoteEntry.js
# Should be <100KB
```

## Next Steps

1. Follow "Quick Start (3-Terminal Setup)" above
2. Run through "Testing Scenarios" to verify setup
3. Use "Troubleshooting" section if issues arise
4. Refer to "Debugging Tips" for deep dives
5. See FEDERATED_HYBRID_ARCHITECTURE_PHASE2.md for full design

---

**Document Version**: 1.0
**Last Updated**: 2026-03-29
**Status**: Implementation Guide (Ready for Use)
