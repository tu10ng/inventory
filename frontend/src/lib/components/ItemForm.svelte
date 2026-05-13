<script lang="ts">
	import type { Item, Category, Tag, AttributeDefinition } from '$lib/types';
	import { itemName } from '$lib/types';
	import { getAttrConfig } from '$lib/utils/attrs';
	import { attrMatchesScope } from '$lib/utils/columns';

	let { item = null, categories, tags, attrDefs = [], onSave, onCancel }: {
		item?: Partial<Item> | null;
		categories: Category[];
		tags: Tag[];
		attrDefs?: AttributeDefinition[];
		onSave: (data: Record<string, unknown>) => void;
		onCancel: () => void;
	} = $props();

	// svelte-ignore state_referenced_locally
	let category_id = $state(item?.category_id ?? categories[0]?.id ?? 0);
	// svelte-ignore state_referenced_locally
	let tag_id = $state<number | null>(item?.tag_id ?? null);

	// Unified attrs: name, brand, model, default_qty, notes all live here
	// svelte-ignore state_referenced_locally
	let attrs = $state<Record<string, unknown>>({ ...(item?.attrs ?? {}) });

	const categoryTags = $derived(tags.filter(t => t.category_id === category_id));

	// Current item_type value
	const itemType = $derived(String(attrs.item_type ?? '') || '实体');

	// Scoped attribute definitions (registered + matching scope)
	const scopedAttrDefs = $derived(
		attrDefs
			.filter(ad => attrMatchesScope(ad, category_id, tag_id))
			.filter(ad => {
				// Virtual-item-specific attributes only show when item_type is '虚拟'
				if (ad.key === 'expiry_date' || ad.key === 'file_url') {
					return itemType === '虚拟';
				}
				return true;
			})
	);

	// Known attrs for the dynamic section – filter out 'name' since it's rendered at the top
	const knownAttrDefs = $derived(
		scopedAttrDefs.filter(ad => ad.key !== 'name')
	);

	// Ad-hoc keys: keys in attrs that are NOT in any attrDef
	const allDefKeys = $derived(new Set(attrDefs.map(ad => ad.key)));
	const adHocKeys = $derived(
		Object.keys(attrs).filter(k => !allDefKeys.has(k))
	);

	function onTagChange(newTagId: number | null) {
		tag_id = newTagId;
		if (newTagId) {
			const t = tags.find(t => t.id === newTagId);
			if (t) category_id = t.category_id;
		}
	}

	function getAttrValue(key: string, defaultVal: unknown = ''): unknown {
		return attrs[key] ?? defaultVal;
	}

	function setAttr(key: string, value: unknown) {
		attrs = { ...attrs, [key]: value };
	}

	function togglePillValue(key: string, pill: string) {
		const current = String(getAttrValue(key, '') ?? '');
		const parts = current ? current.split(',').filter(Boolean) : [];
		const idx = parts.indexOf(pill);
		if (idx >= 0) parts.splice(idx, 1);
		else parts.push(pill);
		setAttr(key, parts.join(','));
	}

	// Ad-hoc key management
	function setAdHocKey(oldKey: string, newKey: string) {
		if (!newKey.trim() || oldKey === newKey) return;
		const value = attrs[oldKey];
		const newAttrs: Record<string, unknown> = {};
		for (const [k, v] of Object.entries(attrs)) {
			if (k === oldKey) {
				newAttrs[newKey.trim()] = value;
			} else {
				newAttrs[k] = v;
			}
		}
		attrs = newAttrs;
	}

	function removeAdHocKey(key: string) {
		const newAttrs = { ...attrs };
		delete newAttrs[key];
		attrs = newAttrs;
	}

	function addAdHocKey() {
		let candidate = 'new_key';
		let idx = 1;
		while (candidate in attrs) {
			candidate = `new_key_${idx}`;
			idx++;
		}
		attrs = { ...attrs, [candidate]: '' };
	}

	function handleSave() {
		onSave({
			category_id, tag_id, attrs,
		});
	}

	const isEdit = $derived(!!item?.id);

	// Validate name via the itemName helper (reads attrs.name)
	const hasName = $derived(!!itemName({ attrs } as unknown as Item));
</script>

<div class="item-form">
	<h3 class="form-title">{isEdit ? '编辑物品' : '添加物品'}</h3>

	<div class="form-group">
		<label for="item-form-name">名称 *</label>
		<input id="item-form-name" value={attrs.name ?? ''} oninput={(e) => setAttr('name', e.currentTarget.value)} placeholder="物品名称" />
	</div>

	<div class="form-row">
		<div class="form-group" style="flex:1">
			<label for="item-form-category">分类</label>
			<select id="item-form-category" bind:value={category_id}>
				{#each categories as c}
					<option value={c.id}>{c.icon} {c.name}</option>
				{/each}
			</select>
		</div>
		<div class="form-group" style="flex:1">
			<label for="item-form-tag">标签</label>
			<select id="item-form-tag" value={tag_id ?? ''} onchange={(e) => onTagChange(e.currentTarget.value ? Number(e.currentTarget.value) : null)}>
				<option value="">无标签</option>
				{#each categoryTags as t}
					<option value={t.id}>{t.name}</option>
				{/each}
				{#if tag_id && !categoryTags.find(t => t.id === tag_id)}
					{@const otherTag = tags.find(t => t.id === tag_id)}
					{#if otherTag}
						<option value={otherTag.id}>{otherTag.name} (其他分类)</option>
					{/if}
				{/if}
			</select>
		</div>
	</div>

	<!-- Known attributes (scoped, excluding 'name' which is rendered at top) -->
	{#if knownAttrDefs.length > 0}
		<hr class="form-divider" />
		<h4 class="sub-title">已知属性</h4>

		{#each knownAttrDefs as ad (ad.id)}
			{@const config = getAttrConfig(ad)}
			{#if ad.attr_type === 'number' || ad.attr_type === 'weight'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label for="item-form-attr-{ad.key}">{ad.label}{config.suffix ? ` (${config.suffix})` : ''}</label>
						<input id="item-form-attr-{ad.key}" type="number" value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))} min="0" />
					</div>
				</div>
			{:else if ad.attr_type === 'bar'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label for="item-form-attr-{ad.key}">{ad.label} (0-{config.max ?? 10})</label>
						<input id="item-form-attr-{ad.key}" type="number" value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))} min="0" max={config.max ?? 10} />
					</div>
				</div>
			{:else if ad.attr_type === 'stars'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label for="item-form-attr-{ad.key}">{ad.label} (0-{config.max ?? 5})</label>
						<select id="item-form-attr-{ad.key}" value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))}>
							{#each Array.from({length: (config.max ?? 5) + 1}, (_, i) => i) as v}
								<option value={v}>{v}</option>
							{/each}
						</select>
					</div>
				</div>
			{:else if ad.attr_type === 'bool'}
				<div class="form-row">
					<label class="toggle-label">
						<input type="checkbox" checked={Number(getAttrValue(ad.key, 0)) > 0} onchange={(e) => setAttr(ad.key, e.currentTarget.checked ? 1 : 0)} />
						{ad.label}
					</label>
				</div>
			{:else if ad.attr_type === 'text' && config.options}
				<div class="form-group">
					<label for="item-form-attr-{ad.key}">{ad.label}</label>
					<div class="checkbox-row">
						{#each config.options as opt, idx}
							{@const current = String(getAttrValue(ad.key, ''))}
							{@const selected = current.split(',').filter(Boolean).includes(opt)}
							<label class="checkbox-label">
								<input type="checkbox" id={idx === 0 ? `item-form-attr-${ad.key}` : undefined} checked={selected} onchange={() => togglePillValue(ad.key, opt)} />
								{opt}
							</label>
						{/each}
					</div>
				</div>
			{:else}
				<div class="form-group">
					<label for="item-form-attr-{ad.key}">{ad.label}</label>
					<input id="item-form-attr-{ad.key}" value={getAttrValue(ad.key, '')} onchange={(e) => setAttr(ad.key, e.currentTarget.value)} placeholder={ad.label} />
				</div>
			{/if}
		{/each}
	{/if}

	<!-- Ad-hoc attributes (not registered) -->
	{#if adHocKeys.length > 0}
		<hr class="form-divider" />
		<h4 class="sub-title">其他属性</h4>
		{#each adHocKeys as key (key)}
			<div class="form-row adhoc-row">
				<div class="form-group" style="flex:1">
					<label for="item-form-adhoc-key-{key}">键</label>
					<input
						id="item-form-adhoc-key-{key}"
						value={key}
						onchange={(e) => setAdHocKey(key, e.currentTarget.value)}
						placeholder="属性名"
					/>
				</div>
				<div class="form-group" style="flex:2">
					<label for="item-form-adhoc-value-{key}">值</label>
					<input
						id="item-form-adhoc-value-{key}"
						value={String(getAttrValue(key, ''))}
						onchange={(e) => setAttr(key, e.currentTarget.value)}
						placeholder="属性值"
					/>
				</div>
				<button class="remove-btn" onclick={() => removeAdHocKey(key)} title="删除">&times;</button>
			</div>
		{/each}
	{/if}

	<div class="adhoc-add">
		<button class="small" onclick={addAdHocKey}>+ 添加属性</button>
	</div>

	<div class="form-actions">
		<button onclick={onCancel}>取消</button>
		<button class="primary" onclick={handleSave} disabled={!hasName}>{isEdit ? '更新' : '添加'}</button>
	</div>
</div>

<style>
	.item-form {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 16px;
	}
	.form-title {
		font-size: 16px;
		margin-bottom: 12px;
	}
	.form-group {
		margin-bottom: 10px;
	}
	.form-group label {
		display: block;
		font-size: 12px;
		color: var(--text-secondary);
		margin-bottom: 3px;
	}
	.form-group input,
	.form-group select {
		width: 100%;
	}
	.form-row {
		display: flex;
		gap: 10px;
		margin-bottom: 10px;
	}
	.adhoc-row {
		align-items: flex-end;
	}
	.adhoc-row .remove-btn {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: transparent;
		color: var(--text-secondary);
		border: 1px solid var(--border);
		cursor: pointer;
		font-size: 14px;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		margin-bottom: 10px;
	}
	.adhoc-row .remove-btn:hover {
		background: var(--danger);
		color: white;
		border-color: var(--danger);
	}
	.adhoc-add {
		margin-bottom: 10px;
	}
	.form-divider {
		border: none;
		border-top: 1px solid var(--border);
		margin: 16px 0 12px;
	}
	.sub-title {
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 10px;
	}
	.checkbox-row {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
	}
	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		cursor: pointer;
	}
	.checkbox-label input[type="checkbox"] {
		width: auto;
	}
	.toggle-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		cursor: pointer;
	}
	.toggle-label input[type="checkbox"] {
		width: auto;
	}
	.form-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
		margin-top: 14px;
	}
</style>
