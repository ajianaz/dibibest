<script lang="ts">
	import { ws } from '../workspace.svelte';

	let q = $state('');
	const tables = $derived(
		(ws.erd?.nodes ?? []).filter(
			(n) =>
				!q ||
				n.name.toLowerCase().includes(q.toLowerCase()) ||
				n.columns.some((c) => c.name.toLowerCase().includes(q.toLowerCase()))
		)
	);
	const shown = $derived((ws.erd?.nodes.length ?? 0) - ws.hidden.size);
</script>

<aside class="sidebar" aria-label="Schema explorer">
	<div class="search"><input type="search" placeholder="Search tables, columns…" aria-label="Search schema" bind:value={q} /></div>
	<div class="section-h">Visible <span>{shown} / {ws.erd?.nodes.length ?? 0} shown</span></div>
	<div class="list">
		{#each tables as n (n.id)}
			<label class="row" class:off={ws.hidden.has(n.id)} class:focus={ws.focus === n.id}>
				<input type="checkbox" checked={!ws.hidden.has(n.id)} onchange={() => ws.toggleHidden(n.id)} />
				<button class="nm" onclick={() => (ws.focus = ws.focus === n.id ? null : n.id)}>{n.name}</button>
				<span class="cnt">{n.columns.length}c</span>
			</label>
		{/each}
	</div>
</aside>

<style>
	.sidebar {
		width: 260px;
		flex: 0 0 260px;
		background: var(--bg-1);
		border-right: 1px solid var(--line);
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
	.search {
		padding: var(--sp-3);
	}
	input[type='search'] {
		width: 100%;
		background: var(--bg-2);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		color: var(--text-1);
		padding: 7px var(--sp-3);
		font: inherit;
	}
	.section-h {
		display: flex;
		justify-content: space-between;
		padding: var(--sp-2) var(--sp-4);
		font-size: 11.5px;
		font-weight: 650;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-3);
	}
	.section-h span {
		text-transform: none;
		letter-spacing: 0;
		font-weight: 500;
		font-variant-numeric: tabular-nums;
	}
	.list {
		overflow: auto;
		flex: 1;
	}
	.row {
		display: flex;
		align-items: center;
		gap: var(--sp-2);
		padding: 5px var(--sp-4);
		color: var(--text-1);
		cursor: pointer;
	}
	.row:hover {
		background: var(--bg-2);
	}
	.row.off .nm {
		color: var(--text-3);
	}
	.row.focus {
		background: var(--accent-soft);
	}
	.nm {
		flex: 1;
		text-align: left;
		font-family: var(--mono);
		font-size: 12.5px;
	}
	.cnt {
		color: var(--text-3);
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}
</style>
