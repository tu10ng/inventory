<script lang="ts">
	import type { Item, Category, Tag, AttributeDefinition } from '$lib/types';

	let { item = null, categories, tags, attrDefs = [], onSave, onCancel }: {
		item?: Partial<Item> | null;
		categories: Category[];
		tags: Tag[];
		attrDefs?: AttributeDefinition[];
		onSave: (data: Record<string, unknown>) => void;
		onCancel: () => void;
	} = $props();

	let name = $state(item?.name ?? '');
	let category_id = $state(item?.category_id ?? categories[0]?.id ?? 0);
	let tag_id = $state<number | null>(item?.tag_id ?? null);
	let brand = $state(item?.brand ?? '');
	let model = $state(item?.model ?? '');
	let default_qty = $state(item?.default_qty ?? 1);
	let notes = $state(item?.notes ?? '');

	// Dynamic attrs
	let attrs = $state<Record<string, unknown>>({ ...(item?.attrs ?? {}) });

	const categoryTags = $derived(tags.filter(t => t.category_id === category_id));

	function onTagChange(newTagId: number | null) {
		tag_id = newTagId;
		if (newTagId) {
			const t = tags.find(t => t.id === newTagId);
			if (t) category_id = t.category_id;
		}
	}

	function getAttrConfig(ad: AttributeDefinition): { max?: number; suffix?: string; options?: string[] } {
		try { return JSON.parse(ad.config || '{}'); } catch { return {}; }
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

	function handleSave() {
		onSave({
			name, category_id, tag_id, brand, model, default_qty, notes,
			attrs,
		});
	}

	const isEdit = $derived(!!item?.id);
</script>

<div class="item-form">
	<h3 class="form-title">{isEdit ? '编辑物品' : '添加物品'}</h3>

	<div class="form-group">
		<label>名称 *</label>
		<input bind:value={name} placeholder="物品名称" />
	</div>

	<div class="form-row">
		<div class="form-group" style="flex:1">
			<label>分类</label>
			<select bind:value={category_id}>
				{#each categories as c}
					<option value={c.id}>{c.icon} {c.name}</option>
				{/each}
			</select>
		</div>
		<div class="form-group" style="flex:1">
			<label>标签</label>
			<select value={tag_id ?? ''} onchange={(e) => onTagChange(e.currentTarget.value ? Number(e.currentTarget.value) : null)}>
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

	<div class="form-row">
		<div class="form-group" style="flex:1">
			<label>品牌</label>
			<input bind:value={brand} placeholder="品牌" />
		</div>
		<div class="form-group" style="flex:1">
			<label>型号</label>
			<input bind:value={model} placeholder="型号" />
		</div>
		<div class="form-group" style="width:80px">
			<label>数量</label>
			<input type="number" bind:value={default_qty} min="1" />
		</div>
	</div>

	<div class="form-group">
		<label>备注</label>
		<input bind:value={notes} placeholder="备注" />
	</div>

	{#if attrDefs.length > 0}
		<hr class="form-divider" />
		<h4 class="sub-title">物理属性</h4>

		{#each attrDefs as ad (ad.id)}
			{@const config = getAttrConfig(ad)}
			{#if ad.attr_type === 'number' || ad.attr_type === 'weight'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label>{ad.label}{config.suffix ? ` (${config.suffix})` : ''}</label>
						<input type="number" value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))} min="0" />
					</div>
				</div>
			{:else if ad.attr_type === 'bar'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label>{ad.label} (0-{config.max ?? 10})</label>
						<input type="number" value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))} min="0" max={config.max ?? 10} />
					</div>
				</div>
			{:else if ad.attr_type === 'stars'}
				<div class="form-row">
					<div class="form-group" style="flex:1">
						<label>{ad.label} (0-{config.max ?? 5})</label>
						<select value={getAttrValue(ad.key, 0)} onchange={(e) => setAttr(ad.key, Number(e.currentTarget.value))}>
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
					<label>{ad.label}</label>
					<div class="checkbox-row">
						{#each config.options as opt}
							{@const current = String(getAttrValue(ad.key, ''))}
							{@const selected = current.split(',').filter(Boolean).includes(opt)}
							<label class="checkbox-label">
								<input type="checkbox" checked={selected} onchange={() => togglePillValue(ad.key, opt)} />
								{opt}
							</label>
						{/each}
					</div>
				</div>
			{:else}
				<div class="form-group">
					<label>{ad.label}</label>
					<input value={getAttrValue(ad.key, '')} onchange={(e) => setAttr(ad.key, e.currentTarget.value)} placeholder={ad.label} />
				</div>
			{/if}
		{/each}
	{/if}

	<div class="form-actions">
		<button onclick={onCancel}>取消</button>
		<button class="primary" onclick={handleSave} disabled={!name}>{isEdit ? '更新' : '添加'}</button>
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
