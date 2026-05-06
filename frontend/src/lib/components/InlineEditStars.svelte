<script lang="ts">
	let { value, max = 5, oncommit }: {
		value: number;
		max?: number;
		oncommit: (val: number) => void;
	} = $props();

	function handleClick(star: number) {
		// Click on current value to clear (set to 0)
		const newVal = star === value ? 0 : star;
		oncommit(newVal);
	}
</script>

<span class="inline-stars">
	{#each Array.from({ length: max }, (_, i) => i + 1) as star (star)}
		<span
			class="star"
			class:filled={star <= value}
			onclick={() => handleClick(star)}
			role="button"
			tabindex="0"
			onkeydown={(e) => e.key === 'Enter' && handleClick(star)}
		>
			{star <= value ? '★' : '☆'}
		</span>
	{/each}
</span>

<style>
	.inline-stars {
		display: inline-flex;
		gap: 1px;
		cursor: pointer;
	}
	.star {
		color: var(--border);
		font-size: 16px;
		transition: color 0.1s;
		user-select: none;
		line-height: 1;
	}
	.star:hover {
		color: #f0ad4e;
	}
	.star.filled {
		color: #f0ad4e;
	}
</style>
