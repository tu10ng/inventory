<script lang="ts">
	export interface BulkAction {
		label: string;
		action: () => Promise<void> | void;
		variant?: 'default' | 'danger';
	}

	let {
		selectedCount,
		actions = [],
	}: {
		selectedCount: number;
		actions: BulkAction[];
	} = $props();
</script>

{#if selectedCount > 0}
	<div class="bulk-bar card">
		<span class="bulk-count">已选 {selectedCount} 项</span>
		{#each actions as act}
			<button
				class="small {act.variant === 'danger' ? 'danger' : ''}"
				onclick={() => act.action()}
			>
				{act.label}
			</button>
		{/each}
	</div>
{/if}

<style>
	.bulk-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #e8f0fe;
		margin-bottom: 12px;
	}
	.bulk-count {
		font-size: 13px;
		font-weight: 500;
		color: var(--primary);
	}
</style>
