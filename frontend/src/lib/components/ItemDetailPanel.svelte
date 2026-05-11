<script lang="ts">
	import type { Item, Category, Tag, AttributeDefinition } from '$lib/types';
	import InlineEdit from './InlineEdit.svelte';
	import InlineEditSelect from './InlineEditSelect.svelte';
	import InlineEditToggle from './InlineEditToggle.svelte';
	import InlineEditPills from './InlineEditPills.svelte';
	import InlineEditStars from './InlineEditStars.svelte';
	import InlineEditBar from './InlineEditBar.svelte';

	let { item, categories, tags, attrDefs = [], usageCount = 0, onUpdate, onDelete }: {
		item: Item;
		categories: Category[];
		tags: Tag[];
		attrDefs?: AttributeDefinition[];
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

	function getAttrConfig(ad: AttributeDefinition): { max?: number; suffix?: string; options?: string[] } {
		try { return JSON.parse(ad.config || '{}'); } catch { return {}; }
	}

	function getAttrValue(key: string): unknown {
		return item.attrs?.[key] ?? 0;
	}

	function updateAttr(key: string, value: unknown) {
		const newAttrs = { ...item.attrs, [key]: value };
		onUpdate('attrs', newAttrs);
	}
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

	<!-- 动态属性 -->
	{#if attrDefs.length > 0}
		<div class="detail-section">
			<h3 class="section-title">物理属性</h3>

			{#each attrDefs as ad (ad.id)}
				{@const config = getAttrConfig(ad)}
				{@const val = getAttrValue(ad.key)}
				<div class="field-row">
					<span class="field-label">{ad.label}</span>
					<span class="field-value">
						{#if ad.attr_type === 'weight' || ad.attr_type === 'number'}
							<InlineEdit value={val as number} type="number" min={0} suffix={config.suffix ?? ''} oncommit={(v) => updateAttr(ad.key, v)} placeholder="-" />
						{:else if ad.attr_type === 'bar'}
							<InlineEditBar value={val as number} max={config.max ?? 10} oncommit={(v) => updateAttr(ad.key, v)} />
						{:else if ad.attr_type === 'stars'}
							<InlineEditStars value={val as number} oncommit={(v) => updateAttr(ad.key, v)} />
						{:else if ad.attr_type === 'bool'}
							<InlineEditToggle value={(val as number) > 0} oncommit={(v) => updateAttr(ad.key, v ? 1 : 0)} />
						{:else if ad.attr_type === 'text' && config.options}
							<InlineEditPills value={val as string} options={config.options} oncommit={(v) => updateAttr(ad.key, v)} />
						{:else}
							<InlineEditPills value={val as string} options={[]} freeform={true} oncommit={(v) => updateAttr(ad.key, v)} />
						{/if}
					</span>
				</div>
			{/each}
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
		padding: 10px 12px;
		border-bottom: 1px solid var(--border);
	}
	.header-top {
		display: flex;
		gap: 8px;
		align-items: flex-start;
	}
	.cat-icon {
		font-size: 24px;
		line-height: 1;
	}
	.header-info {
		flex: 1;
		min-width: 0;
	}
	.item-name {
		font-size: 15px;
		font-weight: 600;
		line-height: 1.3;
		word-break: break-word;
	}
	.item-meta {
		display: flex;
		gap: 6px;
		align-items: center;
		flex-wrap: wrap;
		margin-top: 2px;
	}
	.tag-pill {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 1px 8px;
		border-radius: 10px;
		border: 1px solid #c7d2fe;
	}
	.detail-section {
		padding: 6px 12px;
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
		margin-bottom: 4px;
	}
	.field-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 0;
		font-size: 13px;
	}
	.field-label {
		width: 60px;
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
		padding: 6px 12px;
	}
</style>
