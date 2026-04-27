<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Activity, ActivityItem, Item, Tip } from '$lib/types';

	let activities = $state<Activity[]>([]);
	let allItems = $state<Item[]>([]);
	let showForm = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state({ name: '', description: '', icon: '' });

	// Detail view
	let selectedId = $state<number | null>(null);
	let activityItems = $state<ActivityItem[]>([]);
	let tips = $state<Tip[]>([]);
	let addItemId = $state<number | null>(null);
	let newTip = $state('');

	async function load() {
		[activities, allItems] = await Promise.all([
			api.get<Activity[]>('/activities'),
			api.get<Item[]>('/items')
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
		[activityItems, tips] = await Promise.all([
			api.get<ActivityItem[]>(`/activities/${id}/items`),
			api.get<Tip[]>(`/activities/${id}/tips`)
		]);
	}

	async function addItem() {
		if (!addItemId || !selectedId) return;
		await api.post(`/activities/${selectedId}/items`, { item_id: addItemId });
		addItemId = null;
		activityItems = await api.get<ActivityItem[]>(`/activities/${selectedId}/items`);
	}

	async function removeItem(itemId: number) {
		if (!selectedId) return;
		await api.del(`/activities/${selectedId}/items/${itemId}`);
		activityItems = await api.get<ActivityItem[]>(`/activities/${selectedId}/items`);
	}

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

	function itemName(id: number) {
		const it = allItems.find((i) => i.id === id);
		return it ? `${it.name}${it.brand ? ' ' + it.brand : ''}${it.model ? ' ' + it.model : ''}` : `#${id}`;
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

<div style="display: flex; gap: 20px;">
	<!-- Activity list -->
	<div style="flex: 1;">
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
			<div style="flex: 1;">
				<h3>{selected.icon} {selected.name} - 物品列表</h3>

				<div style="display: flex; gap: 8px; margin: 12px 0;">
					<select bind:value={addItemId} style="flex: 1;">
						<option value={null}>选择物品添加...</option>
						{#each allItems as it}
							<option value={it.id}>{it.name} {it.brand} {it.model}</option>
						{/each}
					</select>
					<button class="primary" onclick={addItem} disabled={!addItemId}>添加</button>
				</div>

				{#each activityItems as ai}
					<div class="card" style="display: flex; justify-content: space-between; align-items: center; padding: 8px 14px;">
						<span>{itemName(ai.item_id)}{ai.default_qty > 1 ? ` x${ai.default_qty}` : ''}</span>
						<button class="small danger" onclick={() => removeItem(ai.item_id)}>移除</button>
					</div>
				{/each}

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
