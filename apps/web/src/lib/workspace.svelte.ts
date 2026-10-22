// The one workspace store. Every layout reads `erd` (ERD-JSON, $state.raw) — there
// is no second model. Text and canvas edits both end in `analyze`, so what you see
// is always what the core parsed.
import { convert, toErd, type ErdDoc, type ErdNode, type Fidelity } from './core';

export type Layout = 'text' | 'canvas' | 'explorer';
export const FORMATS = [
	{ id: 'dbml', label: 'DBML' },
	{ id: 'sql', label: 'SQL' },
	{ id: 'prisma', label: 'Prisma' }
] as const;

export const SAMPLE = `Table users {
  id integer [pk]
  email varchar [not null, unique]
  created_at datetime
}

Table orders {
  id integer [pk]
  user_id integer [not null]
  status varchar
  total_cents integer [default: 0]
}

Ref: orders.user_id > users.id
`;

const DEBOUNCE_MS = 150;

class Workspace {
	layout = $state<Layout>('text');
	format = $state('dbml');
	text = $state(SAMPLE);
	erd = $state.raw<ErdDoc | null>(null);
	fidelity = $state.raw<Fidelity | null>(null);
	error = $state<string | null>(null);
	parseMs = $state(0);
	/** Explorer: ids of tables hidden by the sidebar checkboxes. */
	hidden = $state.raw<ReadonlySet<string>>(new Set());
	/** Explorer focus mode: id of the focused table. */
	focus = $state<string | null>(null);

	#timer: ReturnType<typeof setTimeout> | undefined;
	#seq = 0;

	/** Text-pane edit: debounced re-parse. */
	setText(text: string) {
		this.text = text;
		clearTimeout(this.#timer);
		this.#timer = setTimeout(() => this.analyze(), DEBOUNCE_MS);
	}

	/** Parse `text` → erd. Positions of nodes already on the canvas survive. */
	async analyze() {
		const seq = ++this.#seq;
		const r = await toErd(this.text, this.format);
		if (seq !== this.#seq) return; // a newer edit superseded this one
		if (!r.ok) {
			this.error = r.error;
			return;
		}
		const prev = new Map(this.erd?.nodes.map((n) => [n.id, n.position]));
		const doc: ErdDoc = JSON.parse(r.output);
		for (const n of doc.nodes) n.position = prev.get(n.id) ?? n.position;
		this.erd = doc;
		this.fidelity = r.fidelity;
		this.parseMs = r.ms;
		this.error = null;
	}

	/** Switch the text pane's format by re-emitting the current erd. */
	async setFormat(format: string) {
		if (!this.erd || format === this.format) return;
		const r = await convert(JSON.stringify(this.erd), 'erdjson', format);
		if (!r.ok) {
			this.error = r.error;
			return;
		}
		this.format = format;
		this.text = r.output;
		this.fidelity = r.fidelity;
		this.error = null;
	}

	/** Canvas edit: regenerate text from the edited doc, then re-parse it back. */
	async commit(doc: ErdDoc) {
		this.erd = doc;
		const r = await convert(JSON.stringify(doc), 'erdjson', this.format);
		if (!r.ok) {
			this.error = r.error;
			return;
		}
		this.text = r.output;
		await this.analyze();
		this.fidelity = r.fidelity;
	}

	/** Dragging only moves pixels — no text change, no reparse. */
	move(positions: Map<string, { x: number; y: number }>) {
		if (!this.erd) return;
		this.erd = {
			...this.erd,
			meta: { ...this.erd.meta, layout: 'manual' },
			nodes: this.erd.nodes.map((n) => ({ ...n, position: positions.get(n.id) ?? n.position }))
		};
	}

	addTable() {
		if (!this.erd) return;
		const used = new Set(this.erd.nodes.map((n) => n.id));
		let i = used.size + 1;
		while (used.has(`table_${i}`)) i++;
		const node: ErdNode = {
			id: `table_${i}`,
			name: `table_${i}`,
			position: { x: 40 + (i % 4) * 40, y: 40 + (i % 4) * 40 },
			columns: [{ name: 'id', type: 'integer', isPk: true, isFk: false, isUnique: false, isNullable: false }]
		};
		return this.commit({ ...this.erd, nodes: [...this.erd.nodes, node] });
	}

	/** Delete tables (and every edge touching them) and/or individual edges. */
	remove(tableIds: string[], edgeIds: string[]) {
		if (!this.erd || !(tableIds.length || edgeIds.length)) return;
		const gone = new Set(tableIds);
		const goneEdges = new Set(edgeIds);
		return this.commit({
			...this.erd,
			nodes: this.erd.nodes.filter((n) => !gone.has(n.id)),
			edges: this.erd.edges.filter(
				(e) => !goneEdges.has(e.id) && !gone.has(e.from.table) && !gone.has(e.to.table)
			)
		});
	}

	addRelation(from: { table: string; column: string }, to: { table: string; column: string }) {
		if (!this.erd) return;
		const id = `${from.table}.${from.column}->${to.table}.${to.column}`;
		if (this.erd.edges.some((e) => e.id === id)) return;
		return this.commit({
			...this.erd,
			edges: [
				...this.erd.edges,
				{
					id,
					from: { table: from.table, columns: [from.column] },
					to: { table: to.table, columns: [to.column] },
					kind: 'many-to-one'
				}
			]
		});
	}

	renameColumn(table: string, from: string, to: string) {
		if (!this.erd || !to || to === from) return;
		const rn = (t: string, c: string) => (t === table && c === from ? to : c);
		return this.commit({
			...this.erd,
			nodes: this.erd.nodes.map((n) =>
				n.id === table ? { ...n, columns: n.columns.map((c) => (c.name === from ? { ...c, name: to } : c)) } : n
			),
			edges: this.erd.edges.map((e) => ({
				...e,
				from: { ...e.from, columns: e.from.columns.map((c) => rn(e.from.table, c)) },
				to: { ...e.to, columns: e.to.columns.map((c) => rn(e.to.table, c)) }
			}))
		});
	}

	toggleHidden(id: string) {
		const next = new Set(this.hidden);
		if (!next.delete(id)) next.add(id);
		this.hidden = next;
	}
}

export const ws = new Workspace();
