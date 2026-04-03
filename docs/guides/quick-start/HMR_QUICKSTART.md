# HMR Quick Start

Get your development environment running with Hot Module Reload in 5 minutes.

---

## 5-Minute Setup

### Step 1: Start Backend Stack (Terminal 1)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
task up
```

**Wait for green checkmarks** — all services healthy (takes ~30-60s).

### Step 2: Start AgilePlus Dashboard (Terminal 2)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/crates/agileplus-dashboard/web
npm install  # First time only
npm run dev
```

**Output should say**: `VITE v8.0.1 ready in XXXms`

### Step 3: Open Browser

Go to `http://localhost:5173` — AgilePlus Dashboard loads.

### Step 4: Edit and Watch

Open `src/components/Button.tsx` and change a color or text. **Save the file.**

✅ **Watch the browser update instantly** — no reload needed!

---

## How It Works

| Task | Terminal | Command |
|------|----------|---------|
| Backend services | 1 | `task up` (process-compose) |
| Dashboard HMR | 2 | `npm run dev` (Vite) |
| API proxy | — | Automatic: `/api` → `http://localhost:3000` |
| Logs | 3 | `task logs` |

---

## File Changes That Trigger HMR

✅ **Auto-updates** (no reload):
- `src/**/*.tsx` — React component changes
- `src/**/*.ts` — TypeScript logic
- `src/**/*.css` — Tailwind/CSS changes
- Images/assets in `public/`

⚠️ **Requires restart** (`npm run dev`):
- `vite.config.ts`
- `tsconfig.json`
- `package.json`
- Dependencies added/removed

---

## Common Issues

### "Port 5173 already in use"

```bash
lsof -i :5173
kill -9 <PID>
npm run dev  # Try again
```

### "Cannot GET /"

1. Verify backend is running: `task status`
2. Check API proxy in `vite.config.ts` targets `http://localhost:3000`

### "HMR not updating"

Press `Ctrl+Shift+R` (hard refresh) or:
```bash
npm run dev  # Restart Vite
```

---

## Other Services with HMR

### heliosApp (Module Federation)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp
npm run dev  # Port 3001
```

### AgilePlus Docs

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs
npm run docs:dev  # Port 5175
```

---

## Useful Commands

```bash
# Status of all services
task status

# Tail logs (all services)
task logs

# Restart a single service (e.g., backend API)
process-compose restart agileplus-api

# Stop everything
task down

# Quick restart backend
task down && task up
```

---

## Next: Full Documentation

See `/docs/guides/HMR_SETUP.md` for:
- Detailed configuration options
- Debugging tips
- API proxy setup
- Port assignments
- Performance tuning
