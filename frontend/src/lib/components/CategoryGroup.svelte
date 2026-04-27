<script lang="ts">
	import type { Snippet } from 'svelte';

	let { icon, name, checked, total, collapsed = false, onToggle, children }: {
		icon: string;
		name: string;
		checked: number;
		total: number;
		collapsed?: boolean;
		onToggle: () => void;
		children: Snippet;
	} = $props();
</script>

<div class="card category-group">
	<div
		class="category-header"
		class:collapsed
		onclick={onToggle}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && onToggle()}
	>
		<strong>{icon} {name} ({checked}/{total})</strong>
		<span>{collapsed ? '▶' : '▼'}</span>
	</div>

	{#if !collapsed}
		{@render children()}
	{/if}
</div>

<style>
	.category-group {
		padding: 0;
		overflow: hidden;
	}
	.category-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 12px 16px;
		cursor: pointer;
		transition: background 0.15s;
	}
	.category-header:hover {
		background: var(--bg);
	}
	.category-header.collapsed {
		background: var(--bg);
	}
</style>
