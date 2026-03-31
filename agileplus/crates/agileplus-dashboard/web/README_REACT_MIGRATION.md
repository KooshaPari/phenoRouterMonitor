# AgilePlus Dashboard Web (React + Vite + shadcn/ui)

This is the React frontend for the AgilePlus Dashboard, replacing the previous Askama server-side templates.

## Project Structure

```
web/
├── src/
│   ├── components/      # Reusable shadcn/ui components
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── dialog.tsx
│   │   └── EvidenceLightbox.tsx
│   ├── pages/          # Full page components
│   │   ├── Dashboard.tsx
│   │   └── Settings.tsx
│   ├── lib/
│   │   ├── api.ts      # Axios-based API client
│   │   └── utils.ts    # Utility functions (cn, etc)
│   ├── App.tsx         # Main app with routing
│   ├── main.tsx        # Entry point
│   └── index.css       # Global styles + Tailwind
├── vite.config.ts      # Vite config with path aliases and API proxy
├── tailwind.config.js  # Tailwind CSS config
├── postcss.config.js   # PostCSS config
└── package.json
```

## Technologies

- **Vite**: Fast build tool for React/TypeScript
- **React 18**: UI library
- **TypeScript**: Type-safe JavaScript
- **Tailwind CSS**: Utility-first CSS framework
- **shadcn/ui**: High-quality, accessible component library
- **Radix UI**: Headless component primitives
- **Axios**: HTTP client for API calls
- **Lucide React**: Beautiful SVG icons

## Development

### Install

```bash
npm install
```

### Dev Server

```bash
# Start Vite (port 5173)
npm run dev

# API proxy to localhost:3000
# Open http://localhost:5173
```

### Build

```bash
npm run build
npm run preview  # Test production build
```

## Pages Implemented

### Dashboard (`src/pages/Dashboard.tsx`)

- Service health status cards with live data
- Evidence gallery with hover-to-expand preview
- Rich lightbox modal with artifact details
- Error handling and loading states

### Settings (`src/pages/Settings.tsx`)

- Health check interval configuration
- Notification/debug log toggles
- Retry configuration
- Save feedback

## Components

- **Button**: shadcn variant (default, outline, ghost, etc)
- **Card**: Container with header, title, content, footer
- **Dialog**: Modal overlay with Radix UI
- **EvidenceLightbox**: Custom gallery component

## Features (Phase 1)

✅ React + Vite + TypeScript
✅ Tailwind CSS + shadcn/ui
✅ API client (Axios)
✅ Dashboard page with rich components
✅ Evidence lightbox with preview
✅ Settings page with forms
✅ Dark mode CSS variables
✅ Dev server with API proxy
✅ Zero TypeScript errors
✅ No console warnings

## Not Yet Implemented

- Clickable navigation links (Phase 2)
- Agent activity detection (Phase 3)
- WebSocket real-time updates (Phase 3)
- Rust static file integration (Phase 2)

## API Integration

All requests to `/api` are proxied to `http://localhost:3000`:

```typescript
dashboardAPI.getHealth()
dashboardAPI.getEvidenceGallery(featureId)
dashboardAPI.getSettings()
```

See `src/lib/api.ts` for full API definitions.

## Dark Mode

CSS variables automatically switch between light/dark:

```css
:root { --background: 0 0% 100%; }  /* Light */
.dark { --background: 0 0% 3.6%; }  /* Dark */
```

All components use HSL color variables.
