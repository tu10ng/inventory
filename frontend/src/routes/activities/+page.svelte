<script lang="ts">
	import { api } from '$lib/api/client';
	import CategoryGroup from '$lib/components/CategoryGroup.svelte';
	import type { Activity, ActivitySlotWithTypes, ActivityIncludeEnriched, Type, Tip, Category } from '$lib/types';

	let activities = $state<Activity[]>([]);
	let types = $state<Type[]>([]);
	let categories = $state<Category[]>([]);
	let showForm = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state({ name: '', description: '', icon: '' });
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Detail view
	let selectedId = $state<number | null>(null);
	let slots = $state<ActivitySlotWithTypes[]>([]);
	let tips = $state<Tip[]>([]);
	let newTip = $state('');

	// Includes
	let includes = $state<ActivityIncludeEnriched[]>([]);
	let showIncludeAdd = $state(false);
	let includeSearch = $state('');
	let expandedIncludes = $state<Set<number>>(new Set());
	let includedSlots = $state<Map<number, ActivitySlotWithTypes[]>>(new Map());

	// Slot form
	let showSlotForm = $state(false);
	let editingSlotId = $state<number | null>(null);
	let slotForm = $state({
		slot_name: '',
		category_id: 0,
		is_essential: true,
		default_qty: 1,
		notes: '',
		sort_order: 0,
		type_ids: [] as number[]
	});

	async function load() {
		try {
			loading = true;
			error = null;
			[activities, types, categories] = await Promise.all([
				api.get<Activity[]>('/activities'),
				api.get<Type[]>('/types'),
				api.get<Category[]>('/categories')
			]);
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loading = false;
		}
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
		try {
			if (editingId) {
				await api.put(`/activities/${editingId}`, form);
			} else {
				await api.post('/activities', form);
			}
			resetForm();
			await load();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function remove(id: number) {
		try {
			await api.del(`/activities/${id}`);
			if (selectedId === id) selectedId = null;
			await load();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function selectActivity(id: number) {
		try {
			selectedId = id;
			[slots, tips, includes] = await Promise.all([
				api.get<ActivitySlotWithTypes[]>(`/activities/${id}/slots`),
				api.get<Tip[]>(`/activities/${id}/tips`),
				api.get<ActivityIncludeEnriched[]>(`/activities/${id}/includes`)
			]);
			expandedIncludes = new Set();
			includedSlots = new Map();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	// ── Slot management ──

	function resetSlotForm() {
		slotForm = {
			slot_name: '',
			category_id: categories[0]?.id ?? 0,
			is_essential: true,
			default_qty: 1,
			notes: '',
			sort_order: slots.length,
			type_ids: []
		};
		editingSlotId = null;
		showSlotForm = false;
	}

	function startEditSlot(slot: ActivitySlotWithTypes) {
		slotForm = {
			slot_name: slot.slot_name,
			category_id: slot.category_id,
			is_essential: slot.is_essential,
			default_qty: slot.default_qty,
			notes: slot.notes,
			sort_order: slot.sort_order,
			type_ids: slot.types.map(t => t.id)
		};
		editingSlotId = slot.id;
		showSlotForm = true;
	}

	async function saveSlot() {
		if (!selectedId) return;
		try {
			if (editingSlotId) {
				await api.put(`/activity-slots/${editingSlotId}`, slotForm);
			} else {
				await api.post(`/activities/${selectedId}/slots`, slotForm);
			}
			resetSlotForm();
			slots = await api.get<ActivitySlotWithTypes[]>(`/activities/${selectedId}/slots`);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function removeSlot(id: number) {
		try {
			await api.del(`/activity-slots/${id}`);
			if (selectedId) {
				slots = await api.get<ActivitySlotWithTypes[]>(`/activities/${selectedId}/slots`);
			}
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function toggleSlotEssential(slot: ActivitySlotWithTypes) {
		try {
			await api.put(`/activity-slots/${slot.id}`, { is_essential: !slot.is_essential });
			if (selectedId) {
				slots = await api.get<ActivitySlotWithTypes[]>(`/activities/${selectedId}/slots`);
			}
		} catch (e) {
			alert((e as Error).message);
		}
	}

	function toggleTypeId(typeId: number) {
		if (slotForm.type_ids.includes(typeId)) {
			slotForm.type_ids = slotForm.type_ids.filter(id => id !== typeId);
		} else {
			slotForm.type_ids = [...slotForm.type_ids, typeId];
		}
	}

	// ── Tips ──

	async function addTip() {
		if (!newTip || !selectedId) return;
		try {
			await api.post(`/activities/${selectedId}/tips`, { content: newTip });
			newTip = '';
			tips = await api.get<Tip[]>(`/activities/${selectedId}/tips`);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function removeTip(id: number) {
		try {
			await api.del(`/tips/${id}`);
			if (selectedId) tips = await api.get<Tip[]>(`/activities/${selectedId}/tips`);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	// ── Includes ──

	async function addInclude(includedActivityId: number) {
		if (!selectedId) return;
		try {
			await api.post(`/activities/${selectedId}/includes`, {
				included_activity_id: includedActivityId
			});
			includes = await api.get<ActivityIncludeEnriched[]>(`/activities/${selectedId}/includes`);
			showIncludeAdd = false;
			includeSearch = '';
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function removeInclude(incId: number) {
		try {
			await api.del(`/activity-includes/${incId}`);
			if (selectedId) {
				includes = await api.get<ActivityIncludeEnriched[]>(`/activities/${selectedId}/includes`);
			}
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function toggleIncludePreview(inc: ActivityIncludeEnriched) {
		const incId = inc.id;
		if (expandedIncludes.has(incId)) {
			expandedIncludes.delete(incId);
			expandedIncludes = new Set(expandedIncludes);
		} else {
			expandedIncludes = new Set([...expandedIncludes, incId]);
			if (!includedSlots.has(incId)) {
				try {
					const s = await api.get<ActivitySlotWithTypes[]>(`/activities/${inc.included_activity_id}/slots`);
					includedSlots = new Map(includedSlots).set(incId, s);
				} catch {
					// ignore
				}
			}
		}
	}

	const includableActivities = $derived(
		activities.filter(a => {
			if (a.id === selectedId) return false;
			if (includes.some(inc => inc.included_activity_id === a.id)) return false;
			if (!includeSearch) return true;
			return a.name.includes(includeSearch);
		})
	);

	// ── Helpers ──

	const slotFormTypes = $derived.by(() => {
		return types.filter(t => t.category_id === slotForm.category_id);
	});

	const groupedSlots = $derived.by(() => {
		const catMap = new Map<number, ActivitySlotWithTypes[]>();
		for (const slot of slots) {
			if (!catMap.has(slot.category_id)) catMap.set(slot.category_id, []);
			catMap.get(slot.category_id)!.push(slot);
		}
		const groups: { category: Category; slots: ActivitySlotWithTypes[] }[] = [];
		for (const cat of categories) {
			const catSlots = catMap.get(cat.id);
			if (catSlots && catSlots.length > 0) {
				groups.push({ category: cat, slots: catSlots });
			}
		}
		return groups;
	});

	let slotCollapsed = $state<Record<number, boolean>>({});

	$effect(() => { load(); });
</script>

{#if loading}
	<div class="loading-state">加载中...</div>
{:else if error}
	<div class="error-state">
		<p>{error}</p>
		<button onclick={load}>重试</button>
	</div>
{:else}
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
						<div class="type-select">
							<span class="type-select-label">接受类型：</span>
							{#each slotFormTypes as t}
								<button
									class="type-chip"
									class:selected={slotForm.type_ids.includes(t.id)}
									onclick={() => toggleTypeId(t.id)}
								>
									{t.name}
								</button>
							{/each}
							{#if slotFormTypes.length === 0}
								<span style="color: var(--text-secondary); font-size: 13px;">该分类无类型</span>
							{/if}
						</div>
						<input bind:value={slotForm.notes} placeholder="备注" />
						<button class="primary" onclick={saveSlot} disabled={!slotForm.slot_name}>
							{editingSlotId ? '更新槽位' : '添加槽位'}
						</button>
					</div>
				{/if}

				{#each groupedSlots as group (group.category.id)}
					{@const essentialCount = group.slots.filter(s => s.is_essential).length}
					<CategoryGroup
						icon={group.category.icon}
						name={group.category.name}
						checked={essentialCount}
						total={group.slots.length}
						collapsed={slotCollapsed[group.category.id] ?? false}
						onToggle={() => slotCollapsed[group.category.id] = !slotCollapsed[group.category.id]}
					>
						{#each group.slots as slot (slot.id)}
							<div class="slot-row">
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
									</div>
								</div>
								<div class="slot-types">
									{#each slot.types as t}
										<span class="type-chip-small">{t.name}</span>
									{/each}
								</div>
								<div class="slot-actions">
									<button class="small" onclick={() => startEditSlot(slot)}>编辑</button>
									<button class="small danger" onclick={() => removeSlot(slot.id)}>删除</button>
								</div>
							</div>
						{/each}
					</CategoryGroup>
				{/each}

				{#if slots.length === 0}
					<div class="card" style="text-align: center; color: var(--text-secondary); padding: 24px;">
						暂无槽位，点击上方按钮添加
					</div>
				{/if}

				<!-- Included activities -->
				<h3 style="margin-top: 20px;">包含的活动</h3>
				{#if includes.length > 0}
					{#each includes as inc (inc.id)}
						<div class="include-card card">
							<div class="include-header" onclick={() => toggleIncludePreview(inc)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && toggleIncludePreview(inc)}>
								<span class="include-expand">{expandedIncludes.has(inc.id) ? '▼' : '▶'}</span>
								<span>{inc.included_icon} {inc.included_name}</span>
								<button class="small danger" onclick={(e) => { e.stopPropagation(); removeInclude(inc.id); }}>移除</button>
							</div>
							{#if expandedIncludes.has(inc.id) && includedSlots.has(inc.id)}
								<div class="include-slots">
									{#each includedSlots.get(inc.id)! as slot (slot.id)}
										<div class="include-slot-row">
											<span class="include-slot-name">{slot.is_essential ? '★' : '☆'} {slot.slot_name}</span>
											{#if slot.default_qty > 1}
												<span class="include-slot-qty">x{slot.default_qty}</span>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				{/if}
				<div class="include-add-bar">
					{#if showIncludeAdd}
						<div class="include-add-form">
							<input bind:value={includeSearch} placeholder="搜索活动..." style="flex: 1;" />
							<button class="small" onclick={() => { showIncludeAdd = false; includeSearch = ''; }}>取消</button>
						</div>
						<div class="include-candidates">
							{#each includableActivities as a (a.id)}
								<button class="small include-candidate" onclick={() => addInclude(a.id)}>
									{a.icon} {a.name}
								</button>
							{/each}
							{#if includableActivities.length === 0}
								<span style="font-size: 12px; color: var(--text-secondary);">没有可选的活动</span>
							{/if}
						</div>
					{:else}
						<button class="small" onclick={() => showIncludeAdd = true}>+ 引用其他活动</button>
					{/if}
				</div>

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
{/if}

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
	.type-select {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}
	.type-select-label {
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
	}
	.type-chip {
		font-size: 12px;
		padding: 2px 8px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: var(--surface);
		cursor: pointer;
	}
	.type-chip.selected {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}
	.type-chip-small {
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
		border-top: 1px solid var(--border);
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
	.slot-types {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.slot-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}
	.loading-state {
		text-align: center;
		padding: 40px;
		color: var(--text-secondary);
	}
	.error-state {
		text-align: center;
		padding: 40px;
		color: var(--danger);
	}
	.error-state button {
		margin-top: 12px;
	}
	.include-card {
		padding: 6px 12px;
		margin-bottom: 4px;
	}
	.include-header {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}
	.include-expand {
		font-size: 10px;
		width: 14px;
		text-align: center;
		color: var(--text-secondary);
	}
	.include-header > button {
		margin-left: auto;
	}
	.include-slots {
		margin-top: 6px;
		margin-left: 22px;
		padding-left: 12px;
		border-left: 2px solid var(--border);
	}
	.include-slot-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 0;
		font-size: 13px;
	}
	.include-slot-name {
		color: var(--text);
	}
	.include-slot-qty {
		color: var(--text-secondary);
		font-size: 11px;
	}
	.include-add-bar {
		margin-top: 8px;
	}
	.include-add-form {
		display: flex;
		gap: 6px;
		margin-bottom: 6px;
	}
	.include-candidates {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		margin-bottom: 8px;
	}
	.include-candidate {
		font-size: 12px;
	}
	@media (max-width: 768px) {
		.activities-layout {
			flex-direction: column;
		}
	}
</style>
