<script lang="ts">
	import type { Item, Category, Tag, AttributeDefinition } from '$lib/types';
	import { getAttrConfig } from '$lib/utils/attrs';
	import { attrMatchesScope } from '$lib/utils/columns';
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

	// Scoped attribute definitions
	const scopedAttrDefs = $derived(
		attrDefs.filter(ad => attrMatchesScope(ad, item.category_id, item.tag_id))
	);

	// Ad-hoc keys: keys in attrs that are NOT in any attrDef
	const allDefKeys = $derived(new Set(attrDefs.map(ad => ad.key)));
	const adHocKeys = $derived(
		Object.keys(item.attrs ?? {}).filter(k => !allDefKeys.has(k))
	);

	function getAttrValue(key: string): unknown {
		return item.attrs?.[key] ?? 0;
	}

	function updateAttr(key: string, value: unknown) {
		const newAttrs = { ...item.attrs, [key]: value };
		onUpdate('attrs', newAttrs);
	}

	function updateAdHocKey(oldKey: string, newKey: string) {
		if (!newKey.trim() || oldKey === newKey) return;
		const newAttrs: Record<string, unknown> = {};
		for (const [k, v] of Object.entries(item.attrs ?? {})) {
			if (k === oldKey) {
				newAttrs[newKey.trim()] = v;
			} else {
				newAttrs[k] = v;
			}
		}
		onUpdate('attrs', newAttrs);
	}

	function removeAdHocKey(key: string) {
		const newAttrs = { ...item.attrs };
		delete newAttrs[key];
		onUpdate('attrs', newAttrs);
	}

	function addAdHocKey() {
		let candidate = 'new_key';
		let idx = 1;
		while (candidate in (item.attrs ?? {})) {
			candidate = `new_key_${idx}`;
			idx++;
		}
		const newAttrs = { ...item.attrs, [candidate]: '' };
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

	<!-- Known attributes (scoped) -->
	{#if scopedAttrDefs.length > 0}
		<div class="detail-section">
			<h3 class="section-title">已知属性</h3>
			{#each scopedAttrDefs as ad (ad.id)}
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

	<!-- Ad-hoc attributes (not registered) -->
	{#if adHocKeys.length > 0}
		<div class="detail-section">
			<h3 class="section-title">其他属性</h3>
			{#each adHocKeys as key (key)}
				<div class="field-row">
					<span class="field-label adhoc-key">
						<InlineEdit value={key} oncommit={(v) => updateAdHocKey(key, String(v))} placeholder="键" />
					</span>
					<span class="field-value">
						<InlineEdit value={String(item.attrs?.[key] ?? '')} oncommit={(v) => updateAttr(key, v)} placeholder="值" />
					</span>
					<button class="remove-attr-btn" onclick={() => removeAdHocKey(key)} title="删除">&times;</button>
				</div>
			{/each}
		</div>
	{/if}

	<div class="adhoc-add">
		<button class="small" onclick={addAdHocKey}>+ 添加属性</button>
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
	.field-label.adhoc-key {
		width: auto;
		min-width: 60px;
		max-width: 100px;
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
	.remove-attr-btn {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: transparent;
		color: var(--text-secondary);
		border: 1px solid var(--border);
		cursor: pointer;
		font-size: 12px;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.remove-attr-btn:hover {
		background: var(--danger);
		color: white;
		border-color: var(--danger);
	}
	.adhoc-add {
		padding: 6px 12px;
	}
</style>
