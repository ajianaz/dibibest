<script lang="ts">
	import { untrack } from 'svelte';
	import { EditorState } from '@codemirror/state';
	import { EditorView, keymap, lineNumbers } from '@codemirror/view';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { HighlightStyle, StreamLanguage, syntaxHighlighting } from '@codemirror/language';
	import { tags as t } from '@lezer/highlight';

	let { value, readonly = false, label, onchange }: {
		value: string;
		readonly?: boolean;
		label: string;
		onchange?: (v: string) => void;
	} = $props();

	// Deliberately tiny shared tokenizer for DBML / SQL / Prisma — keywords, strings, comments, [settings].
	const KEYWORDS =
		/^(Table|Ref|Enum|Project|Indexes|Note|TableGroup|create|table|primary|key|foreign|references|not|null|unique|default|index|on|constraint|model|datasource|generator|enum|alter|add|if|exists)\b/i;
	const lang = StreamLanguage.define<{ inSettings: boolean }>({
		startState: () => ({ inSettings: false }),
		token(stream, state) {
			if (stream.eatSpace()) return null;
			if (stream.match(/^(\/\/|--).*/)) return 'comment';
			if (stream.match(/^('[^']*'|"[^"]*"|`[^`]*`)/)) return 'string';
			if (stream.eat('[')) return ((state.inSettings = true), 'bracket');
			if (stream.eat(']')) return ((state.inSettings = false), 'bracket');
			if (stream.match(KEYWORDS)) return 'keyword';
			if (stream.match(/^[0-9][0-9.]*/)) return 'number';
			if (state.inSettings && stream.match(/^[a-z_]+/i)) return 'atom';
			stream.next();
			return null;
		}
	});
	const highlight = HighlightStyle.define([
		{ tag: t.keyword, color: '#7da2ff' },
		{ tag: t.comment, color: '#8b95a5', fontStyle: 'italic' },
		{ tag: t.string, color: '#e8c07d' },
		{ tag: t.number, color: '#e8c07d' },
		{ tag: t.atom, color: '#b48cff' }
	]);
	const theme = EditorView.theme(
		{
			'&': { height: '100%', color: 'var(--text-1)', backgroundColor: 'transparent', fontSize: '13px' },
			'.cm-scroller': { fontFamily: 'var(--mono)', lineHeight: '1.7' },
			'.cm-gutters': { backgroundColor: 'transparent', color: 'var(--text-3)', border: 'none' },
			'.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'var(--accent-soft)' },
			'.cm-cursor': { borderLeftColor: 'var(--text-1)' },
			'&.cm-focused': { outline: 'none' }
		},
		{ dark: true }
	);

	let host: HTMLDivElement;
	let view: EditorView | undefined;

	// Create once; props are read untracked so a value change never rebuilds the view.
	$effect(() => {
		view = untrack(() => new EditorView({
			parent: host,
			state: EditorState.create({
				doc: value,
				extensions: [
					lineNumbers(),
					history(),
					keymap.of([...defaultKeymap, ...historyKeymap]),
					lang,
					syntaxHighlighting(highlight),
					theme,
					EditorState.readOnly.of(readonly),
					EditorView.editable.of(!readonly),
					EditorView.contentAttributes.of({ 'aria-label': label }),
					EditorView.updateListener.of((u) => u.docChanged && u.state.doc.toString() !== value && onchange?.(u.state.doc.toString()))
				]
			})
		}));
		return () => view?.destroy();
	});

	// Store → editor (canvas edits, format switches). Skip when the editor already shows it.
	$effect(() => {
		if (view && value !== view.state.doc.toString()) {
			view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
		}
	});
</script>

<div class="editor" bind:this={host}></div>

<style>
	.editor {
		flex: 1;
		min-height: 0;
		padding: var(--sp-3) 0;
		overflow: hidden;
	}
</style>
