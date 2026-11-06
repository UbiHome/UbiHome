# Documentation (Astro Starlight)

```bash
cd documentation
npm install
npm run dev         # local preview on http://localhost:8000
npm run check       # biome check with --write (formats/lints and updates files)
npm run check-only  # non-mutating checks for CI/verification
npm run build       # production build in documentation/site
npm run preview    # preview built site
```

Dagger commands:

```bash
dagger call docs-preview up
dagger check
dagger call docs-build-dir export --path ./documentation/site
```
