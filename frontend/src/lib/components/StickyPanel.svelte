<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		top = 16,
		offset = 150,
	}: {
		children: Snippet;
		top?: number;
		offset?: number;
	} = $props();
</script>

<div
	class="sticky-panel"
	style:--sticky-top="{top}px"
	style:--sticky-max-height="calc(100vh - {offset}px)"
>
	{@render children()}
</div>

<style>
	.sticky-panel {
		position: sticky;
		top: var(--sticky-top);
		max-height: var(--sticky-max-height);
		overflow-y: auto;
		overscroll-behavior: contain;
	}
</style>
