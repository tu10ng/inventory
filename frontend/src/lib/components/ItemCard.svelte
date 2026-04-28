<script lang="ts">
	let { name, brand, model, categoryIcon, qty, alreadyAdded = false, onclick }: {
		name: string;
		brand: string;
		model: string;
		categoryIcon: string;
		qty: number;
		alreadyAdded?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<div
	class="item-card"
	class:already-added={alreadyAdded}
	onclick={onclick}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && onclick?.()}
>
	<div class="card-icon">{categoryIcon}</div>
	<div class="card-name">{name}</div>
	{#if brand || model}
		<div class="card-detail">{brand} {model}</div>
	{/if}
	{#if qty > 1}
		<div class="card-qty">x{qty}</div>
	{/if}
	{#if alreadyAdded}
		<div class="added-tag">已添加</div>
	{/if}
</div>

<style>
	.item-card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 10px 8px;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
		position: relative;
		min-height: 90px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
	}
	.item-card:hover {
		border-color: var(--primary);
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}
	.item-card.already-added {
		opacity: 0.5;
	}
	.card-icon {
		font-size: 22px;
	}
	.card-name {
		font-size: 13px;
		color: var(--text);
		font-weight: 500;
		line-height: 1.2;
	}
	.card-detail {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.card-qty {
		position: absolute;
		top: 4px;
		right: 6px;
		background: var(--primary);
		color: white;
		font-size: 10px;
		padding: 0 5px;
		border-radius: 8px;
		font-weight: 600;
	}
	.added-tag {
		font-size: 10px;
		color: var(--text-secondary);
		background: var(--bg);
		padding: 0 6px;
		border-radius: 4px;
		margin-top: 2px;
	}
</style>
