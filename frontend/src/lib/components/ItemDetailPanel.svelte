<script lang="ts">
	import type { Item, Category, Tag, ItemUsageCount } from '$lib/types';

	let { item, categories, tags, usageCount = 0, onEdit, onDelete }: {
		item: Item;
		categories: Category[];
		tags: Tag[];
		usageCount?: number;
		onEdit: () => void;
		onDelete: () => void;
	} = $props();

	const category = $derived(categories.find(c => c.id === item.category_id));
	const tag = $derived(item.tag_id ? tags.find(t => t.id === item.tag_id) : null);

	function starRating(value: number, max: number = 5): string {
		return '★'.repeat(value) + '☆'.repeat(max - value);
	}

	function barPercent(value: number, max: number): number {
		return Math.min(100, Math.round((value / max) * 100));
	}

	const seasons: Record<string, string> = { '春': '🌱', '夏': '☀️', '秋': '🍂', '冬': '❄️' };
	const bodyPartLabels: Record<string, string> = { '头': '🧢', '躯干': '👕', '腿': '👖', '脚': '👟', '手': '🧤' };

	const seasonList = $derived(item.season ? item.season.split(',').filter(Boolean) : []);
	const bodyPartList = $derived(item.body_parts ? item.body_parts.split(',').filter(Boolean) : []);
	const materialList = $derived(item.material ? item.material.split(',').filter(Boolean) : []);
</script>

<div class="detail-panel">
	<div class="detail-header">
		<div class="header-top">
			<span class="cat-icon">{category?.icon ?? '📦'}</span>
			<div class="header-info">
				<h2 class="item-name">{item.name}</h2>
				<div class="item-meta">
					{#if tag}
						<span class="tag-pill">{tag.name}</span>
					{/if}
					{#if item.brand || item.model}
						<span class="brand-model">{item.brand} {item.model}</span>
					{/if}
				</div>
			</div>
		</div>
		<div class="header-actions">
			<button class="small" onclick={onEdit}>编辑</button>
			<button class="small danger" onclick={onDelete}>删除</button>
		</div>
	</div>

	<div class="detail-section">
		<div class="info-row">
			<span class="info-label">默认数量</span>
			<span class="info-value">{item.default_qty}</span>
		</div>
		{#if item.notes}
			<div class="info-row">
				<span class="info-label">备注</span>
				<span class="info-value">{item.notes}</span>
			</div>
		{/if}
	</div>

	<div class="detail-section">
		<h3 class="section-title">物理属性</h3>

		<div class="stat-row">
			<span class="stat-label">保暖</span>
			{#if item.warmth_rating > 0}
				<div class="stat-bar-wrap">
					<div class="stat-bar" style="width: {barPercent(item.warmth_rating, 50)}%"></div>
				</div>
				<span class="stat-value">{item.warmth_rating}</span>
			{:else}
				<span class="stat-empty">-</span>
			{/if}
		</div>

		<div class="stat-row">
			<span class="stat-label">累赘</span>
			{#if item.encumbrance > 0}
				<div class="stat-bar-wrap">
					<div class="stat-bar encumbrance" style="width: {barPercent(item.encumbrance, 10)}%"></div>
				</div>
				<span class="stat-value">{item.encumbrance}</span>
			{:else}
				<span class="stat-empty">-</span>
			{/if}
		</div>

		<div class="stat-row">
			<span class="stat-label">环境防护</span>
			{#if item.env_protection > 0}
				<span class="stat-stars">{starRating(item.env_protection)}</span>
			{:else}
				<span class="stat-empty">-</span>
			{/if}
		</div>

		<div class="stat-row">
			<span class="stat-label">耐久</span>
			{#if item.durability > 0}
				<span class="stat-stars">{starRating(item.durability)}</span>
			{:else}
				<span class="stat-empty">-</span>
			{/if}
		</div>

		<div class="stat-row">
			<span class="stat-label">重量</span>
			{#if item.weight_grams > 0}
				<span class="stat-value">{item.weight_grams}g</span>
			{:else}
				<span class="stat-empty">-</span>
			{/if}
		</div>

		{#if item.storage_ml > 0}
			<div class="stat-row">
				<span class="stat-label">容量</span>
				<span class="stat-value">{item.storage_ml}ml</span>
			</div>
		{/if}

		<div class="stat-row">
			<span class="stat-label">防水</span>
			<span class="stat-bool" class:active={item.waterproof > 0}>
				{item.waterproof > 0 ? '✓' : '✗'}
			</span>
		</div>

		<div class="stat-row">
			<span class="stat-label">透气</span>
			<span class="stat-bool" class:active={item.breathable > 0}>
				{item.breathable > 0 ? '✓' : '✗'}
			</span>
		</div>
	</div>

	{#if materialList.length > 0 || seasonList.length > 0 || bodyPartList.length > 0}
		<div class="detail-section">
			<h3 class="section-title">标签</h3>

			{#if materialList.length > 0}
				<div class="pill-row">
					<span class="pill-label">材质</span>
					{#each materialList as m}
						<span class="pill">{m}</span>
					{/each}
				</div>
			{/if}

			{#if seasonList.length > 0}
				<div class="pill-row">
					<span class="pill-label">季节</span>
					{#each seasonList as s}
						<span class="pill">{seasons[s] ?? ''} {s}</span>
					{/each}
				</div>
			{/if}

			{#if bodyPartList.length > 0}
				<div class="pill-row">
					<span class="pill-label">部位</span>
					{#each bodyPartList as bp}
						<span class="pill">{bodyPartLabels[bp] ?? ''} {bp}</span>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	{#if usageCount > 0}
		<div class="detail-section usage-section">
			被 {usageCount} 个行程使用
		</div>
	{/if}
</div>

<style>
	.detail-panel {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
	}
	.detail-header {
		padding: 16px;
		border-bottom: 1px solid var(--border);
	}
	.header-top {
		display: flex;
		gap: 12px;
		align-items: flex-start;
		margin-bottom: 10px;
	}
	.cat-icon {
		font-size: 32px;
		line-height: 1;
	}
	.header-info {
		flex: 1;
		min-width: 0;
	}
	.item-name {
		font-size: 18px;
		font-weight: 600;
		line-height: 1.3;
		word-break: break-word;
	}
	.item-meta {
		display: flex;
		gap: 6px;
		align-items: center;
		flex-wrap: wrap;
		margin-top: 4px;
	}
	.tag-pill {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 1px 8px;
		border-radius: 10px;
		border: 1px solid #c7d2fe;
	}
	.brand-model {
		font-size: 13px;
		color: var(--text-secondary);
	}
	.header-actions {
		display: flex;
		gap: 6px;
	}
	.detail-section {
		padding: 12px 16px;
		border-bottom: 1px solid var(--border);
	}
	.detail-section:last-child {
		border-bottom: none;
	}
	.section-title {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
	}
	.info-row {
		display: flex;
		justify-content: space-between;
		padding: 4px 0;
		font-size: 14px;
	}
	.info-label {
		color: var(--text-secondary);
	}
	.info-value {
		color: var(--text);
	}
	.stat-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 0;
		font-size: 13px;
	}
	.stat-label {
		width: 70px;
		flex-shrink: 0;
		color: var(--text-secondary);
	}
	.stat-bar-wrap {
		flex: 1;
		height: 8px;
		background: var(--bg);
		border-radius: 4px;
		overflow: hidden;
	}
	.stat-bar {
		height: 100%;
		background: var(--primary);
		border-radius: 4px;
		transition: width 0.3s;
	}
	.stat-bar.encumbrance {
		background: var(--warning);
	}
	.stat-value {
		width: 40px;
		text-align: right;
		font-weight: 500;
		font-size: 12px;
	}
	.stat-stars {
		color: #f0ad4e;
		letter-spacing: 1px;
	}
	.stat-empty {
		color: var(--border);
	}
	.stat-bool {
		color: var(--border);
		font-size: 16px;
	}
	.stat-bool.active {
		color: var(--success);
	}
	.pill-row {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 6px;
	}
	.pill-label {
		font-size: 12px;
		color: var(--text-secondary);
		width: 36px;
		flex-shrink: 0;
	}
	.pill {
		font-size: 12px;
		background: var(--bg);
		border: 1px solid var(--border);
		padding: 2px 8px;
		border-radius: 12px;
	}
	.usage-section {
		font-size: 13px;
		color: var(--text-secondary);
		text-align: center;
		padding: 10px 16px;
	}
</style>
