# HTML Colors for Tezbar

Search standard HTML color palettes (basic 16 + extended 140) by name, hex or RGB with fuzzy matching in Tezbar on Windows or macOS.

## Command

- **Search HTML Colors** — list view with `Name, hex, or RGB…` search, dropdown for All Colors / Basic / Extended / Group by Shade, hex/RGB toggle, detail preview with SVG swatch, category icons, and shade-grouped sections sorted by relevance.

Supports Fuse.js fuzzy search (`threshold 0.5`) with deduplication by color name, shade categories (`pink`, `red`, `orange`, …), and copy-on-select (copies id/hex/rgb depending on match format).

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api`, `@raycast/utils`, and `fuse.js` (pure JS) and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `html-colors` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `Synacek` remains credited in `package.json`.
