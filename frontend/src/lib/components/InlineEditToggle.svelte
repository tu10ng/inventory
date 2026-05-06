<script lang="ts">
	let { value, oncommit }: {
		value: boolean;
		oncommit: (val: boolean) => void;
	} = $props();
</script>

<span
	class="inline-toggle"
	class:active={value}
	onclick={() => oncommit(!value)}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && oncommit(!value)}
>
	{value ? '✓' : '✗'}
</span>

<style>
	.inline-toggle {
		cursor: pointer;
		font-size: 16px;
		color: var(--border);
		padding: 0 2px;
		user-select: none;
	}
	.inline-toggle:hover {
		opacity: 0.7;
	}
	.inline-toggle.active {
		color: var(--success);
	}
</style>
