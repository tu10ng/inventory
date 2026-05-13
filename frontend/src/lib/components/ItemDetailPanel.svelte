<script lang="ts">
	import type { Item, Category, Tag, AttributeDefinition, ItemRelationEnriched, RelationType, CreateItemRelation } from '$lib/types';
	import { getAttrConfig } from '$lib/utils/attrs';
	import { attrMatchesScope } from '$lib/utils/columns';
	import InlineEdit from './InlineEdit.svelte';
	import InlineEditSelect from './InlineEditSelect.svelte';
	import InlineEditToggle from './InlineEditToggle.svelte';
	import InlineEditPills from './InlineEditPills.svelte';
	import InlineEditStars from './InlineEditStars.svelte';
	import InlineEditBar from './InlineEditBar.svelte';

	let { item, categories, tags, attrDefs = [], usageCount = 0, relations = [], relationTypes = [], onUpdate, onDelete, onAddRelation, onRemoveRelation }: {
		item: Item;
		categories: Category[];
		tags: Tag[];
		attrDefs?: AttributeDefinition[];
		usageCount?: number;
		relations?: ItemRelationEnriched[];
		relationTypes?: RelationType[];
		onUpdate: (field: string, value: unknown) => void;
		onDelete: () => void;
		onAddRelation?: (relation: CreateItemRelation) => void;
		onRemoveRelation?: (id: number) => void;
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

	// Core fields now live in attrs, extract them for display
	const itemName = $derived(String(item.attrs?.name ?? ''));
	const itemBrand = $derived(String(item.attrs?.brand ?? ''));
	const itemModel = $derived(String(item.attrs?.model ?? ''));
	const itemDefaultQty = $derived(Number(item.attrs?.default_qty ?? 1));
	const itemNotes = $derived(String(item.attrs?.notes ?? ''));

	// Current item_type value
	const itemType = $derived(String(item.attrs?.item_type ?? '') || '实体');

	// Scoped attribute definitions — filter by category/tag scope, then by item_type visibility
	const scopedAttrDefs = $derived(
		attrDefs
			.filter(ad => attrMatchesScope(ad, item.category_id, item.tag_id))
			.filter(ad => {
				// Virtual-item-specific attributes only show when item_type is '虚拟'
				if (ad.key === 'expiry_date' || ad.key === 'file_url') {
					return itemType === '虚拟';
				}
				return true;
			})
	);

	// Ad-hoc keys: keys in attrs that are NOT in any attrDef and NOT core fields
	const allDefKeys = $derived(new Set(attrDefs.map(ad => ad.key)));
	const coreFieldKeys = new Set(['name', 'brand', 'model', 'default_qty', 'notes']);
	const adHocKeys = $derived(
		Object.keys(item.attrs ?? {}).filter(k => !allDefKeys.has(k) && !coreFieldKeys.has(k))
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
					<InlineEdit value={itemName} oncommit={(v) => updateAttr('name', v)} placeholder="物品名称" />
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
				<InlineEdit value={itemBrand} oncommit={(v) => updateAttr('brand', v)} placeholder="-" />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">型号</span>
			<span class="field-value">
				<InlineEdit value={itemModel} oncommit={(v) => updateAttr('model', v)} placeholder="-" />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">默认数量</span>
			<span class="field-value">
				<InlineEdit value={itemDefaultQty} type="number" min={1} oncommit={(v) => updateAttr('default_qty', v)} />
			</span>
		</div>
		<div class="field-row">
			<span class="field-label">备注</span>
			<span class="field-value">
				<InlineEdit value={itemNotes} oncommit={(v) => updateAttr('notes', v)} placeholder="-" />
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
						{:else if ad.attr_type === 'text'}
							<InlineEdit value={val as string} oncommit={(v) => updateAttr(ad.key, v)} placeholder="-" />
						{:else}
							<InlineEdit value={val as string} oncommit={(v) => updateAttr(ad.key, v)} placeholder="-" />
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

	<!-- Relations -->
	{#if relations.length > 0}
		<div class="detail-section">
			<h3 class="section-title">关联物品</h3>
			{#each relations as rel (rel.id)}
				<div class="relation-row">
					<span class="relation-icon" style="color:{rel.relation_color}">{rel.relation_icon}</span>
					<span class="relation-label">{rel.relation_label}</span>
					<span class="relation-target">{rel.target_name}</span>
					{#if onRemoveRelation}
						<button class="remove-rel-btn" onclick={() => onRemoveRelation(rel.id)} title="移除">&times;</button>
					{/if}
				</div>
			{/each}
		</div>
	{/if}

	{#if onAddRelation && relationTypes.length > 0}
		<div class="relation-add">
			<select id="relation-type-select" class="small-select">
				{#each relationTypes as rt}
					<option value={rt.id}>{rt.icon} {rt.label}</option>
				{/each}
			</select>
			<button
				class="small"
				onclick={async () => {
					const sel = document.getElementById('relation-type-select') as HTMLSelectElement;
					const rtId = Number(sel.value);
					const targetId = prompt('输入目标物品 ID:');
					if (targetId) {
						onAddRelation({ target_item_id: Number(targetId), relation_type_id: rtId });
					}
				}}
			>+ 关联物品</button>
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
	.relation-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 0;
		font-size: 13px;
	}
	.relation-icon {
		font-size: 14px;
	}
	.relation-label {
		font-size: 11px;
		color: var(--text-secondary);
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0 4px;
	}
	.relation-target {
		flex: 1;
		font-weight: 500;
	}
	.remove-rel-btn {
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
	.remove-rel-btn:hover {
		background: var(--danger);
		color: white;
		border-color: var(--danger);
	}
	.relation-add {
		padding: 6px 12px;
		display: flex;
		gap: 6px;
		align-items: center;
	}
	.small-select {
		padding: 2px 6px;
		font-size: 12px;
		border: 1px solid var(--border);
		border-radius: 4px;
	}
</style>
