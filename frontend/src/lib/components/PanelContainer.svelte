<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: {
		children: Snippet;
	} = $props();
</script>

<div class="panel-container">
	{@render children()}
</div>

<style>
	.panel-container {
		position: sticky;
		top: 16px;
		align-self: flex-start;
		overflow-y: auto;
		max-height: calc(100vh - 80px);
		width: 100%;
	}
</style>
