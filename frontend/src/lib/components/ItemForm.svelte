<script lang="ts">
	import type { Item, Category, Tag } from '$lib/types';

	let { item = null, categories, tags, onSave, onCancel }: {
		item?: Partial<Item> | null;
		categories: Category[];
		tags: Tag[];
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

	// Physical attributes
	let warmth_rating = $state(item?.warmth_rating ?? 0);
	let material = $state(item?.material ?? '');
	let encumbrance = $state(item?.encumbrance ?? 0);
	let waterproof = $state(item?.waterproof ?? 0);
	let weight_grams = $state(item?.weight_grams ?? 0);
	let season = $state(item?.season ?? '');
	let body_parts = $state(item?.body_parts ?? '');
	let env_protection = $state(item?.env_protection ?? 0);
	let durability = $state(item?.durability ?? 0);
	let storage_ml = $state(item?.storage_ml ?? 0);
	let breathable = $state(item?.breathable ?? 0);

	const allSeasons = ['春', '夏', '秋', '冬'];
	const allBodyParts = ['头', '躯干', '腿', '脚', '手'];

	let selectedSeasons = $state<Set<string>>(new Set(season ? season.split(',').filter(Boolean) : []));
	let selectedBodyParts = $state<Set<string>>(new Set(body_parts ? body_parts.split(',').filter(Boolean) : []));

	const categoryTags = $derived(tags.filter(t => t.category_id === category_id));

	function onTagChange(newTagId: number | null) {
		tag_id = newTagId;
		if (newTagId) {
			const t = tags.find(t => t.id === newTagId);
			if (t) category_id = t.category_id;
		}
	}

	function toggleSeason(s: string) {
		const next = new Set(selectedSeasons);
		if (next.has(s)) next.delete(s); else next.add(s);
		selectedSeasons = next;
	}

	function toggleBodyPart(bp: string) {
		const next = new Set(selectedBodyParts);
		if (next.has(bp)) next.delete(bp); else next.add(bp);
		selectedBodyParts = next;
	}

	function handleSave() {
		onSave({
			name, category_id, tag_id, brand, model, default_qty, notes,
			warmth_rating, material, encumbrance, waterproof, weight_grams,
			season: [...selectedSeasons].join(','),
			body_parts: [...selectedBodyParts].join(','),
			env_protection, durability, storage_ml, breathable,
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

	<hr class="form-divider" />
	<h4 class="sub-title">物理属性</h4>

	<div class="form-row">
		<div class="form-group" style="flex:1">
			<label>保暖 (0-50)</label>
			<input type="number" bind:value={warmth_rating} min="0" max="50" />
		</div>
		<div class="form-group" style="flex:1">
			<label>累赘 (0-10)</label>
			<input type="number" bind:value={encumbrance} min="0" max="10" />
		</div>
		<div class="form-group" style="flex:1">
			<label>重量 (g)</label>
			<input type="number" bind:value={weight_grams} min="0" />
		</div>
	</div>

	<div class="form-group">
		<label>材质 (逗号分隔)</label>
		<input bind:value={material} placeholder="如: 涤纶,氨纶" />
	</div>

	<div class="form-group">
		<label>适用季节</label>
		<div class="checkbox-row">
			{#each allSeasons as s}
				<label class="checkbox-label">
					<input type="checkbox" checked={selectedSeasons.has(s)} onchange={() => toggleSeason(s)} />
					{s}
				</label>
			{/each}
		</div>
	</div>

	<div class="form-group">
		<label>覆盖部位</label>
		<div class="checkbox-row">
			{#each allBodyParts as bp}
				<label class="checkbox-label">
					<input type="checkbox" checked={selectedBodyParts.has(bp)} onchange={() => toggleBodyPart(bp)} />
					{bp}
				</label>
			{/each}
		</div>
	</div>

	<div class="form-row">
		<div class="form-group" style="flex:1">
			<label>环境防护 (0-5)</label>
			<select bind:value={env_protection}>
				{#each [0,1,2,3,4,5] as v}
					<option value={v}>{v}</option>
				{/each}
			</select>
		</div>
		<div class="form-group" style="flex:1">
			<label>耐久 (0-5)</label>
			<select bind:value={durability}>
				{#each [0,1,2,3,4,5] as v}
					<option value={v}>{v}</option>
				{/each}
			</select>
		</div>
		<div class="form-group" style="flex:1">
			<label>容量 (ml)</label>
			<input type="number" bind:value={storage_ml} min="0" />
		</div>
	</div>

	<div class="form-row">
		<label class="toggle-label">
			<input type="checkbox" checked={waterproof > 0} onchange={(e) => waterproof = e.currentTarget.checked ? 1 : 0} />
			防水
		</label>
		<label class="toggle-label">
			<input type="checkbox" checked={breathable > 0} onchange={(e) => breathable = e.currentTarget.checked ? 1 : 0} />
			透气
		</label>
	</div>

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
