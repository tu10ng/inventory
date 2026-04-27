<script lang="ts">
	let { checked, total }: { checked: number; total: number } = $props();
	const progress = $derived(total > 0 ? Math.round((checked / total) * 100) : 0);
</script>

{#if total > 0}
	<div class="progress-wrapper">
		<div class="progress-info">
			<span>进度</span>
			<span>{checked}/{total} ({progress}%)</span>
		</div>
		<div class="progress-track">
			<div class="progress-fill" style="width: {progress}%;"></div>
		</div>
	</div>
{/if}

<style>
	.progress-wrapper {
		margin-bottom: 16px;
	}
	.progress-info {
		display: flex;
		justify-content: space-between;
		font-size: 14px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}
	.progress-track {
		height: 8px;
		background: var(--border);
		border-radius: 4px;
		overflow: hidden;
	}
	.progress-fill {
		height: 100%;
		background: var(--success);
		transition: width 0.3s;
	}
</style>
