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
		height: 100%;
		overflow-y: auto;
		width: 100%;
	}
</style>
