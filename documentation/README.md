# Documentation (Astro Starlight)

```bash
cd documentation
npm install
npm run dev      # local preview on http://localhost:8000
npm run check    # content/type checks
npm run build    # production build in documentation/site
npm run preview  # preview built site
```

Dagger commands:

```bash
dagger call docs-preview up
dagger check
dagger call docs-build-dir export --path ./documentation/site
```
