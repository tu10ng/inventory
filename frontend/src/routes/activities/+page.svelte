<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Activity, ActivitySlotWithTags, Item, Tag, Tip, Category } from '$lib/types';

	let activities = $state<Activity[]>([]);
	let allItems = $state<Item[]>([]);
	let tags = $state<Tag[]>([]);
	let categories = $state<Category[]>([]);
	let showForm = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state({ name: '', description: '', icon: '' });

	// Detail view
	let selectedId = $state<number | null>(null);
	let slots = $state<ActivitySlotWithTags[]>([]);
	let tips = $state<Tip[]>([]);
	let newTip = $state('');

	// Slot form
	let showSlotForm = $state(false);
	let editingSlotId = $state<number | null>(null);
	let slotForm = $state({
		slot_name: '',
		category_id: 0,
		is_essential: true,
		default_qty: 1,
		default_item_id: null as number | null,
		notes: '',
		sort_order: 0,
		tag_ids: [] as number[]
	});

	async function load() {
		[activities, allItems, tags, categories] = await Promise.all([
			api.get<Activity[]>('/activities'),
			api.get<Item[]>('/items'),
			api.get<Tag[]>('/tags'),
			api.get<Category[]>('/categories')
		]);
	}

	function resetForm() {
		form = { name: '', description: '', icon: '' };
		editingId = null;
		showForm = false;
	}

	function startEdit(a: Activity) {
		form = { name: a.name, description: a.description, icon: a.icon };
		editingId = a.id;
		showForm = true;
	}

	async function save() {
		if (editingId) {
			await api.put(`/activities/${editingId}`, form);
		} else {
			await api.post('/activities', form);
		}
		resetForm();
		await load();
	}

	async function remove(id: number) {
		await api.del(`/activities/${id}`);
		if (selectedId === id) selectedId = null;
		await load();
	}

	async function selectActivity(id: number) {
		selectedId = id;
		[slots, tips] = await Promise.all([
			api.get<ActivitySlotWithTags[]>(`/activities/${id}/slots`),
			api.get<Tip[]>(`/activities/${id}/tips`)
		]);
	}

	// ── Slot management ──

	function resetSlotForm() {
		slotForm = {
			slot_name: '',
			category_id: categories[0]?.id ?? 0,
			is_essential: true,
			default_qty: 1,
			default_item_id: null,
			notes: '',
			sort_order: slots.length,
			tag_ids: []
		};
		editingSlotId = null;
		showSlotForm = false;
	}

	function startEditSlot(slot: ActivitySlotWithTags) {
		slotForm = {
			slot_name: slot.slot_name,
			category_id: slot.category_id,
			is_essential: slot.is_essential,
			default_qty: slot.default_qty,
			default_item_id: slot.default_item_id,
			notes: slot.notes,
			sort_order: slot.sort_order,
			tag_ids: slot.tags.map(t => t.id)
		};
		editingSlotId = slot.id;
		showSlotForm = true;
	}

	async function saveSlot() {
		if (!selectedId) return;
		if (editingSlotId) {
			await api.put(`/activity-slots/${editingSlotId}`, slotForm);
		} else {
			await api.post(`/activities/${selectedId}/slots`, slotForm);
		}
		resetSlotForm();
		slots = await api.get<ActivitySlotWithTags[]>(`/activities/${selectedId}/slots`);
	}

	async function removeSlot(id: number) {
		await api.del(`/activity-slots/${id}`);
		if (selectedId) {
			slots = await api.get<ActivitySlotWithTags[]>(`/activities/${selectedId}/slots`);
		}
	}

	async function toggleSlotEssential(slot: ActivitySlotWithTags) {
		await api.put(`/activity-slots/${slot.id}`, { is_essential: !slot.is_essential });
		if (selectedId) {
			slots = await api.get<ActivitySlotWithTags[]>(`/activities/${selectedId}/slots`);
		}
	}

	function toggleTagId(tagId: number) {
		if (slotForm.tag_ids.includes(tagId)) {
			slotForm.tag_ids = slotForm.tag_ids.filter(id => id !== tagId);
		} else {
			slotForm.tag_ids = [...slotForm.tag_ids, tagId];
		}
	}

	// ── Tips ──

	async function addTip() {
		if (!newTip || !selectedId) return;
		await api.post(`/activities/${selectedId}/tips`, { content: newTip });
		newTip = '';
		tips = await api.get<Tip[]>(`/activities/${selectedId}/tips`);
	}

	async function removeTip(id: number) {
		await api.del(`/tips/${id}`);
		if (selectedId) tips = await api.get<Tip[]>(`/activities/${selectedId}/tips`);
	}

	// ── Helpers ──

	const slotFormTags = $derived.by(() => {
		return tags.filter(t => t.category_id === slotForm.category_id);
	});

	const candidateItems = $derived.by(() => {
		if (slotForm.tag_ids.length === 0) return allItems;
		return allItems.filter(i => i.tag_id && slotForm.tag_ids.includes(i.tag_id));
	});

	function getCategoryName(catId: number): string {
		const c = categories.find(c => c.id === catId);
		return c ? `${c.icon} ${c.name}` : '';
	}

	function getItemDisplay(itemId: number | null): string {
		if (!itemId) return '无';
		const it = allItems.find(i => i.id === itemId);
		return it ? `${it.name}${it.brand ? ' ' + it.brand : ''}${it.model ? ' ' + it.model : ''}` : `#${itemId}`;
	}

	$effect(() => { load(); });
</script>

<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
	<h1>活动模板</h1>
	<button class="primary" onclick={() => { if (showForm) resetForm(); else showForm = true; }}>
		{showForm ? '取消' : '+ 新建活动'}
	</button>
</div>

{#if showForm}
	<div class="card">
		<div style="display: flex; flex-direction: column; gap: 10px;">
			<div style="display: flex; gap: 10px;">
				<input bind:value={form.icon} placeholder="图标" style="width: 60px;" />
				<input bind:value={form.name} placeholder="活动名称" style="flex: 1;" />
			</div>
			<input bind:value={form.description} placeholder="描述" />
			<button class="primary" onclick={save} disabled={!form.name}>
				{editingId ? '更新' : '创建'}
			</button>
		</div>
	</div>
{/if}

<div class="activities-layout">
	<!-- Activity list -->
	<div class="activities-list">
		{#each activities as a}
			<div
				class="card"
				style="cursor: pointer; {selectedId === a.id ? 'border-color: var(--primary);' : ''}"
				onclick={() => selectActivity(a.id)}
				role="button"
				tabindex="0"
				onkeydown={(e) => e.key === 'Enter' && selectActivity(a.id)}
			>
				<div style="display: flex; justify-content: space-between; align-items: center;">
					<span><strong>{a.icon} {a.name}</strong></span>
					<div style="display: flex; gap: 6px;">
						<button class="small" onclick={(e) => { e.stopPropagation(); startEdit(a); }}>编辑</button>
						<button class="small danger" onclick={(e) => { e.stopPropagation(); remove(a.id); }}>删除</button>
					</div>
				</div>
				{#if a.description}
					<div style="color: var(--text-secondary); font-size: 14px; margin-top: 4px;">{a.description}</div>
				{/if}
			</div>
		{/each}
		{#if activities.length === 0}
			<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
				还没有活动模板
			</div>
		{/if}
	</div>

	<!-- Detail panel -->
	{#if selectedId}
		{@const selected = activities.find((a) => a.id === selectedId)}
		{#if selected}
			<div class="detail-panel">
				<div class="detail-header">
					<h3>{selected.icon} {selected.name} - 槽位列表</h3>
					<button class="primary small" onclick={() => { if (showSlotForm) resetSlotForm(); else { resetSlotForm(); showSlotForm = true; } }}>
						{showSlotForm && !editingSlotId ? '取消' : '+ 添加槽位'}
					</button>
				</div>

				{#if showSlotForm}
					<div class="card slot-form">
						<div class="slot-form-row">
							<input bind:value={slotForm.slot_name} placeholder="槽位名称（如：硬壳/雨衣）" style="flex: 2;" />
							<select bind:value={slotForm.category_id} style="flex: 1;">
								{#each categories as c}
									<option value={c.id}>{c.icon} {c.name}</option>
								{/each}
							</select>
						</div>
						<div class="slot-form-row">
							<label class="inline-label">
								<input type="checkbox" bind:checked={slotForm.is_essential} />
								必备
							</label>
							<label class="inline-label">
								数量
								<input type="number" bind:value={slotForm.default_qty} min="1" style="width: 50px;" />
							</label>
						</div>
						<div class="tag-select">
							<span class="tag-select-label">接受标签：</span>
							{#each slotFormTags as t}
								<button
									class="tag-chip"
									class:selected={slotForm.tag_ids.includes(t.id)}
									onclick={() => toggleTagId(t.id)}
								>
									{t.name}
								</button>
							{/each}
							{#if slotFormTags.length === 0}
								<span style="color: var(--text-secondary); font-size: 13px;">该分类无标签</span>
							{/if}
						</div>
						<div class="slot-form-row">
							<select value={slotForm.default_item_id ?? ''} onchange={(e) => slotForm.default_item_id = e.currentTarget.value ? Number(e.currentTarget.value) : null} style="flex: 1;">
								<option value="">默认物品（无）</option>
								{#each candidateItems as it}
									<option value={it.id}>{it.name}{it.brand ? ` ${it.brand}` : ''}{it.model ? ` ${it.model}` : ''}</option>
								{/each}
							</select>
						</div>
						<input bind:value={slotForm.notes} placeholder="备注" />
						<button class="primary" onclick={saveSlot} disabled={!slotForm.slot_name}>
							{editingSlotId ? '更新槽位' : '添加槽位'}
						</button>
					</div>
				{/if}

				{#each slots as slot (slot.id)}
					<div class="card slot-row">
						<div class="slot-main">
							<button
								class="small essential-btn"
								style="color: {slot.is_essential ? 'var(--warning)' : 'var(--text-secondary)'};"
								onclick={() => toggleSlotEssential(slot)}
								title={slot.is_essential ? '必备（点击取消）' : '非必备（点击标记为必备）'}
							>
								{slot.is_essential ? '★' : '☆'}
							</button>
							<div class="slot-info">
								<strong>{slot.slot_name}</strong>
								{#if slot.default_qty > 1}
									<span class="slot-qty">x{slot.default_qty}</span>
								{/if}
								<span class="slot-category">{getCategoryName(slot.category_id)}</span>
							</div>
						</div>
						<div class="slot-tags">
							{#each slot.tags as t}
								<span class="tag-chip-small">{t.name}</span>
							{/each}
						</div>
						<div class="slot-default">
							默认: {getItemDisplay(slot.default_item_id)}
						</div>
						<div class="slot-actions">
							<button class="small" onclick={() => startEditSlot(slot)}>编辑</button>
							<button class="small danger" onclick={() => removeSlot(slot.id)}>删除</button>
						</div>
					</div>
				{/each}

				{#if slots.length === 0}
					<div class="card" style="text-align: center; color: var(--text-secondary); padding: 24px;">
						暂无槽位，点击上方按钮添加
					</div>
				{/if}

				<h3 style="margin-top: 20px;">提示</h3>
				<div style="display: flex; gap: 8px; margin: 12px 0;">
					<input bind:value={newTip} placeholder="添加提示，如：不穿含棉衣物" style="flex: 1;" />
					<button class="primary" onclick={addTip} disabled={!newTip}>添加</button>
				</div>
				{#each tips as tip}
					<div class="card" style="display: flex; justify-content: space-between; align-items: center; padding: 8px 14px;">
						<span>{tip.content}</span>
						<button class="small danger" onclick={() => removeTip(tip.id)}>删除</button>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<style>
	.activities-layout {
		display: flex;
		gap: 20px;
	}
	.activities-list {
		flex: 1;
		min-width: 0;
	}
	.detail-panel {
		flex: 1.5;
		min-width: 0;
	}
	.detail-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}
	.slot-form {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 12px;
	}
	.slot-form-row {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.inline-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 14px;
		white-space: nowrap;
	}
	.tag-select {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}
	.tag-select-label {
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
	}
	.tag-chip {
		font-size: 12px;
		padding: 2px 8px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface);
		cursor: pointer;
	}
	.tag-chip.selected {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}
	.tag-chip-small {
		font-size: 11px;
		padding: 1px 6px;
		border-radius: 8px;
		background: var(--surface);
		border: 1px solid var(--border);
		color: var(--text-secondary);
	}
	.slot-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 14px;
		flex-wrap: wrap;
	}
	.slot-main {
		display: flex;
		align-items: center;
		gap: 4px;
		flex: 1;
		min-width: 0;
	}
	.essential-btn {
		flex-shrink: 0;
	}
	.slot-info {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.slot-qty {
		color: var(--text-secondary);
		font-size: 13px;
	}
	.slot-category {
		font-size: 12px;
		color: var(--text-secondary);
	}
	.slot-tags {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.slot-default {
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
	}
	.slot-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}
	@media (max-width: 768px) {
		.activities-layout {
			flex-direction: column;
		}
	}
</style>
