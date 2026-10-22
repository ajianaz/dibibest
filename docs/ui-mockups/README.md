# UI Mockups — approved visual reference (owner, 22 Oct 2026)

Three workspace layouts share ONE IR; users switch with ⌘/ or the header toggle.
These mockups are the **binding visual reference** for issue #10 (canvas, SvelteKit).

| File | Layout | For | Screenshot |
|---|---|---|---|
| `text-first.html` | ▤ Text-first (editor 42% left, live preview right) | dbdiagram-style typing | `text-first.png` |
| `canvas-first.html` | ▦ Canvas-first (canvas editor 60% left, generated DBML right) | Liam-style visual editing | `canvas-first.png` |
| `explorer-large-schema.html` | ☰ Explorer (sidebar + focus mode + minimap, 28-table scenario) | large schemas (Azimutt-inspired) | `explorer-large-schema.png` |
| `PRODUCT.md` | product context (impeccable Operate mode, owner brief) | design decisions | — |

## QA performed
- impeccable 61-rule detector: v3 30→1 findings; v4a/v4b ≤2 (advisory only)
- Playwright bounds/luminance: fonts ≥11px, edges anchored, contrast ≥4.5:1 (qa_v3.py pattern)
- Vision review rounds with fixes (edge stroke visibility, spacing variety, count consistency)

## Implementation notes for #10
- Layout switcher = 3 modes over the same ERD-JSON state (never separate state stores)
- Fidelity chip/report binds to the REAL `Fidelity` struct from WASM — no fake numbers
- DBML text pane: CodeMirror or CodeJar; autosync debounced ≤150ms
- Canvas: @xyflow/svelte 1.7+ (peer svelte ^5.25), `LayoutStrategy::Grid` from core first, ELK in R4
- Dark theme tokens from the mockups (:root vars) → design tokens file
