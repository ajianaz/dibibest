<script lang="ts">
	import { FORMATS, ws, type Layout } from '../workspace.svelte';
	import { convert } from '../core';
	import { copy } from '../clipboard';
	import FidelityPanel from './FidelityPanel.svelte';

	const LAYOUTS: { id: Layout; label: string }[] = [
		{ id: 'text', label: '▤ Text-first' },
		{ id: 'canvas', label: '▦ Canvas-first' },
		{ id: 'explorer', label: '☰ Explorer' }
	];
	const EXPORTS = [...FORMATS, { id: 'erdjson', label: 'ERD-JSON' }];

	let showFidelity = $state(false);
	let showExport = $state(false);
	let copied = $state<string | null>(null);

	// Chip numbers come straight from the core Fidelity struct.
	const chip = $derived.by(() => {
		const f = ws.fidelity;
		if (!f) return null;
		const parts = [f.tables, f.columns, f.refs, f.indexes];
		const kept = parts.reduce((a, o) => a + o.kept, 0);
		const total = parts.reduce((a, o) => a + o.total, 0);
		const lossless = f.idempotent && kept === total && !f.dropped?.length;
		const pct = total ? Math.floor((kept / total) * 100) : 100;
		return { lossless, text: lossless ? 'round-trip 100% lossless' : `round-trip ${pct}% · ${f.dropped?.length ?? 0} dropped` };
	});

	async function exportAs(to: string) {
		if (!ws.erd) return;
		const r = to === ws.format ? { ok: true as const, output: ws.text } : await convert(JSON.stringify(ws.erd), 'erdjson', to);
		if (!r.ok) return (ws.error = r.error);
		copied = (await copy(r.output)) ? to : null;
		showExport = false;
		setTimeout(() => (copied = null), 2000);
	}
</script>

<header class="topbar">
	<div class="brand"><span class="mark">db</span> dibibest <span class="project-name">/ shop-demo</span></div>
	{#if ws.layout === 'text'}
		<div class="seg" role="tablist" aria-label="Source format">
			{#each FORMATS as f (f.id)}
				<button role="tab" aria-selected={ws.format === f.id} onclick={() => ws.setFormat(f.id)}>{f.label}</button>
			{/each}
		</div>
	{/if}
	<div class="seg layout" role="tablist" aria-label="Workspace layout">
		{#each LAYOUTS as l (l.id)}
			<button role="tab" aria-selected={ws.layout === l.id} onclick={() => (ws.layout = l.id)}>{l.label}</button>
		{/each}
	</div>
	<div class="spacer"></div>
	{#if chip}
		<button
			class="chip"
			class:lossy={!chip.lossless}
			aria-expanded={showFidelity}
			onclick={() => (showFidelity = !showFidelity)}>◈ {chip.text}</button
		>
	{/if}
	<div class="export">
		<button class="btn-primary" aria-expanded={showExport} onclick={() => (showExport = !showExport)}>
			{copied ? `Copied ${copied} ✓` : 'Export ▾'}
		</button>
		{#if showExport}
			<div class="menu" role="menu">
				{#each EXPORTS as f (f.id)}
					<button role="menuitem" onclick={() => exportAs(f.id)}>Copy as {f.label}</button>
				{/each}
			</div>
		{/if}
	</div>
	{#if showFidelity}<FidelityPanel />{/if}
</header>

<style>
	.topbar {
		position: relative;
		height: 48px;
		flex: 0 0 48px;
		background: var(--bg-1);
		border-bottom: 1px solid var(--line);
		display: flex;
		align-items: center;
		gap: var(--sp-3);
		padding: 0 var(--sp-4);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: var(--sp-2);
		font-weight: 650;
	}
	.mark {
		width: 22px;
		height: 22px;
		border-radius: var(--r-sm);
		background: linear-gradient(135deg, var(--accent), #8a5cff);
		display: grid;
		place-items: center;
		font-size: 12px;
		font-weight: 800;
		color: #0b0d10;
	}
	.project-name {
		color: var(--text-2);
		font-weight: 500;
	}
	.seg {
		display: flex;
		gap: 2px;
		background: var(--bg-2);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		padding: 2px;
	}
	.seg button {
		padding: 5px 14px;
		border-radius: 8px;
		color: var(--text-2);
		font-weight: 550;
		font-size: 13px;
	}
	.seg button[aria-selected='true'] {
		background: var(--bg-3);
		color: var(--text-1);
		box-shadow: inset 0 0 0 1px var(--line);
	}
	.seg.layout button[aria-selected='true'] {
		background: var(--accent-strong);
		color: #fff;
		box-shadow: none;
	}
	.spacer {
		flex: 1;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 12.5px;
		font-weight: 600;
		color: var(--ok);
		background: rgba(62, 207, 142, 0.09);
		border: 1px solid rgba(62, 207, 142, 0.25);
		padding: 4px 10px;
		border-radius: 999px;
		font-variant-numeric: tabular-nums;
	}
	.chip.lossy {
		color: var(--warn);
		background: rgba(245, 181, 68, 0.09);
		border-color: rgba(245, 181, 68, 0.3);
	}
	.btn-primary {
		background: var(--accent-strong);
		color: #fff;
		font-weight: 600;
		font-size: 13.5px;
		padding: 7px 16px;
		border-radius: var(--r-md);
	}
	.export {
		position: relative;
	}
	.menu {
		position: absolute;
		right: 0;
		top: 40px;
		z-index: 30;
		min-width: 180px;
		display: flex;
		flex-direction: column;
		background: var(--bg-2);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		padding: var(--sp-1);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
	}
	.menu button {
		text-align: left;
		padding: 7px var(--sp-3);
		border-radius: var(--r-sm);
		color: var(--text-2);
	}
	.menu button:hover {
		background: var(--bg-3);
		color: var(--text-1);
	}
</style>
