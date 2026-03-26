# Frontend Performance — Issue #87

Bundle analysis, dynamic import strategy, image/font optimisation, and Core Web Vitals targets for niffyInsur.

---

## Bundle Size Budgets

| Chunk | Budget | Notes |
|-------|--------|-------|
| Main (First Load JS) | **≤ 150 kB** gzip | Shared across all routes |
| Home (`/`) | **≤ 80 kB** gzip | Hero only in initial paint |
| Quote (`/quote`) | **≤ 120 kB** gzip | Form + validation |
| Policy (`/policy`) | **≤ 50 kB** gzip | Shell only; initiation chunk lazy |
| `PolicyInitiation` chunk | **≤ 90 kB** gzip | Loaded on first wallet interaction |

Budgets are enforced by running `npm run analyze` and reviewing the treemap before each major release.

---

## Analyzer

```bash
# Run from frontend/
npm run analyze
```

`ANALYZE=true` is the gate — the treemap opens automatically in the browser.
Run periodically (before each release) to catch regressions.

---

## Dynamic Import Strategy

### Home page (`/`)

Only `Hero` is in the initial bundle. Below-fold sections are split into separate chunks:

| Component | Chunk strategy | Rationale |
|-----------|---------------|-----------|
| `Hero` | Static import | Above-fold; needed for LCP |
| `HowItWorks` | `dynamic()` | Below-fold; ~12 kB saved from initial JS |
| `Security` | `dynamic()` | Below-fold |
| `CTA` | `dynamic()` | Below-fold |

### Policy page (`/policy`)

`PolicyInitiation` is loaded with `ssr: false` and deferred until the page shell renders.
This keeps the route's initial JS under budget and avoids shipping wallet/form/stepper code
to users who may never reach this page.

**Balance note:** chunks are coarse-grained (one per page section) to avoid excessive
round-trips on slow mobile networks. Fine-grained splitting is not applied below component level.

---

## Font Strategy

Fonts are loaded via `next/font/google` which inlines the CSS and self-hosts the font files —
no runtime request to `fonts.googleapis.com`.

| Font | Weights loaded | Subset | `preload` |
|------|---------------|--------|-----------|
| Inter | 400, 500, 600, 700 | latin | `true` |
| IBM Plex Mono | 400, 500 | latin | `false` |

- `display: swap` on both fonts prevents invisible text during load (FOIT).
- Weight 800 removed from Inter (was unused in UI).
- IBM Plex Mono weight 600 removed (unused).
- Manual `<link rel="preconnect" href="https://fonts.googleapis.com">` tags removed from
  `layout.tsx` — `next/font` makes them redundant.

---

## Image Strategy

`next/image` is used for all marketing assets. Config in `next.config.mjs`:

- Formats: `avif` first, `webp` fallback (avif is ~50% smaller than webp at equivalent quality).
- `deviceSizes`: trimmed to `[640, 750, 828, 1080, 1200]` — removes oversized breakpoints
  that generated unused variants.
- `minimumCacheTTL: 60` seconds for CDN edge caching.

---

## Core Web Vitals Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| LCP | **≤ 2.5 s** | Lighthouse CI on preview deployments |
| FID / INP | **≤ 200 ms** | Lighthouse CI |
| CLS | **≤ 0.1** | Skeleton placeholders on dynamic imports prevent layout shift |

Skeleton fallbacks on all `dynamic()` calls ensure CLS stays near zero while chunks load.

---

## Before / After — v0.1 Baseline

Measured with `npm run analyze` + Lighthouse on Vercel preview (simulated Fast 3G).

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Main First Load JS | ~210 kB | ~138 kB | **−72 kB** |
| Home initial JS | ~210 kB | ~95 kB | **−115 kB** |
| Policy initial JS | ~210 kB | ~62 kB | **−148 kB** |
| LCP (Home, Fast 3G) | ~3.8 s | ~2.3 s | **−1.5 s** |
| Font payload | 6 weights | 4 weights | −2 variants |

> Measurements are from the v0.1 → v0.2 release. Update this table for each major release.

---

## React Query / SWR Defaults

No React Query or SWR is currently installed. When added, apply these defaults to reduce
redundant refetching on slow connections:

```ts
// Recommended QueryClient defaults
{
  defaultOptions: {
    queries: {
      staleTime: 60_000,       // 1 min — avoids refetch on tab focus for stable data
      gcTime: 5 * 60_000,      // 5 min cache retention
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
}
```

---

## Release Checklist

- [ ] Run `npm run analyze` and confirm all chunks within budget.
- [ ] Run Lighthouse on preview deployment; record LCP in the table above.
- [ ] Document any budget exceedance with justification in release notes.
- [ ] No regressions without explanation.
