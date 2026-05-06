<script lang="ts">
	import type { Item, Category, Tag } from '$lib/types';
	import InlineEdit from './InlineEdit.svelte';
	import InlineEditSelect from './InlineEditSelect.svelte';
	import InlineEditToggle from './InlineEditToggle.svelte';
	import InlineEditPills from './InlineEditPills.svelte';
	import InlineEditStars from './InlineEditStars.svelte';
	import InlineEditBar from './InlineEditBar.svelte';

	let { item, categories, tags, usageCount = 0, onUpdate, onDelete }: {
		item: Item;
		categories: Category[];
		tags: Tag[];
		usageCount?: number;
		onUpdate: (field: string, value: unknown) => void;
		onDelete: () => void;
	} = $props();

	const category = $derived(categories.find(c => c.id === item.category_id));
	const tag = $derived(item.tag_id ? tags.find(t => t.id === item.tag_id) : null);

	const categoryOptions = $derived(categories.map(c => ({ value: c.id, label: `${c.icon} ${c.name}` })));
	const tagOptionsList = $derived.by(() => {
		const filtered = tags.filter(t => t.category_id === item.category_id);
		return [
			{ value: null as number | null, label: '-' },
			...filtered.map(t => ({ value: t.id as number | null, label: t.name }))
		];
	});

	const seasonOptions = ['春', '夏', '秋', '冬'];
	const bodyPartOptions = ['头', '躯干', '腿', '脚', '手'];
</script>

<div class="detail-panel">
	<div class="detail-header">
		<div class="header-top">
			<span class="cat-icon">{category?.icon ?? '📦'}</span>
			<div class="header-info">
				<h2 class="item-name">
					<InlineEdit value={item.name} oncommit={(v) => onUpdate('name', v)} placeholder="物品名称" />
				</h2>
				<div class="item-meta">
					{#if tag}
						<span class="tag-pill">{tag.name}</span>
					{/if}
				</div>
			</div>
		</div>
		<div class="header-actions">
			<button class="small danger" onclick={onDelete}>删除</button>
		</div>
	</div>

	<!-- 基本信息 -->
	<div class="detail-section">
		<div class="field-row">
			<span class="field-label">分类</span>
			<span class="field-value">
				<InlineEditSelect value={item.category_id} options={categoryOptions} oncommit={(v) => onUpdate('category_id', v)} />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">标签</span>
			<span class="field-value">
				<InlineEditSelect value={item.tag_id} options={tagOptionsList} oncommit={(v) => onUpdate('tag_id', v)} />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">品牌</span>
			<span class="field-value">
				<InlineEdit value={item.brand} oncommit={(v) => onUpdate('brand', v)} placeholder="-" />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">型号</span>
			<span class="field-value">
				<InlineEdit value={item.model} oncommit={(v) => onUpdate('model', v)} placeholder="-" />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">默认数量</span>
			<span class="field-value">
				<InlineEdit value={item.default_qty} type="number" min={1} oncommit={(v) => onUpdate('default_qty', v)} />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">备注</span>
			<span class="field-value">
				<InlineEdit value={item.notes} oncommit={(v) => onUpdate('notes', v)} placeholder="-" />
			</span>
		</div>
	</div>

	<!-- 物理属性 -->
	<div class="detail-section">
		<h3 class="section-title">物理属性</h3>

		<div class="field-row">
			<span class="field-label">重量</span>
			<span class="field-value">
				<InlineEdit value={item.weight_grams} type="number" min={0} suffix="g" oncommit={(v) => onUpdate('weight_grams', v)} placeholder="-" />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">容量</span>
			<span class="field-value">
				<InlineEdit value={item.storage_ml} type="number" min={0} suffix="ml" oncommit={(v) => onUpdate('storage_ml', v)} placeholder="-" />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">保暖</span>
			<span class="field-value">
				<InlineEditBar value={item.warmth_rating} max={50} oncommit={(v) => onUpdate('warmth_rating', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">累赘</span>
			<span class="field-value">
				<InlineEditBar value={item.encumbrance} max={10} oncommit={(v) => onUpdate('encumbrance', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">环境防护</span>
			<span class="field-value">
				<InlineEditStars value={item.env_protection} oncommit={(v) => onUpdate('env_protection', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">耐久</span>
			<span class="field-value">
				<InlineEditStars value={item.durability} oncommit={(v) => onUpdate('durability', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">防水</span>
			<span class="field-value">
				<InlineEditToggle value={item.waterproof > 0} oncommit={(v) => onUpdate('waterproof', v ? 1 : 0)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">透气</span>
			<span class="field-value">
				<InlineEditToggle value={item.breathable > 0} oncommit={(v) => onUpdate('breathable', v ? 1 : 0)} />
			</span>
		</div>
	</div>

	<!-- 标签属性 -->
	<div class="detail-section">
		<h3 class="section-title">标签</h3>

		<div class="field-row">
			<span class="field-label">材质</span>
			<span class="field-value">
				<InlineEditPills value={item.material} options={[]} freeform={true} oncommit={(v) => onUpdate('material', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">季节</span>
			<span class="field-value">
				<InlineEditPills value={item.season} options={seasonOptions} oncommit={(v) => onUpdate('season', v)} />
			</span>
		</div>

		<div class="field-row">
			<span class="field-label">部位</span>
			<span class="field-value">
				<InlineEditPills value={item.body_parts} options={bodyPartOptions} oncommit={(v) => onUpdate('body_parts', v)} />
			</span>
		</div>
	</div>

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
	.field-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 0;
		font-size: 13px;
	}
	.field-label {
		width: 70px;
		flex-shrink: 0;
		color: var(--text-secondary);
	}
	.field-value {
		flex: 1;
		min-width: 0;
	}
	.usage-section {
		font-size: 13px;
		color: var(--text-secondary);
		text-align: center;
		padding: 10px 16px;
	}
</style>
