<script lang="ts">
	import { SvelteFlowProvider } from '@xyflow/svelte';
	import CanvasInner from './CanvasInner.svelte';

	let { mode }: { mode: 'preview' | 'edit' | 'explorer' } = $props();
</script>

<SvelteFlowProvider>
	<CanvasInner {mode} />
</SvelteFlowProvider>
