<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category } from '$lib/types';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let showForm = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state({ name: '', brand: '', model: '', category_id: 0, default_qty: 1, notes: '' });

	async function load() {
		[items, categories] = await Promise.all([
			api.get<Item[]>('/items'),
			api.get<Category[]>('/categories')
		]);
		if (form.category_id === 0 && categories.length > 0) {
			form.category_id = categories[0].id;
		}
	}

	function resetForm() {
		form = { name: '', brand: '', model: '', category_id: categories[0]?.id ?? 0, default_qty: 1, notes: '' };
		editingId = null;
		showForm = false;
	}

	function startEdit(item: Item) {
		form = { name: item.name, brand: item.brand, model: item.model, category_id: item.category_id, default_qty: item.default_qty, notes: item.notes };
		editingId = item.id;
		showForm = true;
	}

	async function save() {
		if (editingId) {
			await api.put(`/items/${editingId}`, form);
		} else {
			await api.post('/items', form);
		}
		resetForm();
		await load();
	}

	async function remove(id: number) {
		await api.del(`/items/${id}`);
		await load();
	}

	function categoryName(id: number) {
		return categories.find((c) => c.id === id);
	}

	$effect(() => { load(); });
</script>

<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
	<h1>物品库</h1>
	<button class="primary" onclick={() => { if (showForm) resetForm(); else showForm = true; }}>
		{showForm ? '取消' : '+ 添加物品'}
	</button>
</div>

{#if showForm}
	<div class="card">
		<div style="display: flex; flex-direction: column; gap: 10px;">
			<div style="display: flex; gap: 10px;">
				<input bind:value={form.name} placeholder="物品名称" style="flex: 1;" />
				<select bind:value={form.category_id}>
					{#each categories as c}
						<option value={c.id}>{c.icon} {c.name}</option>
					{/each}
				</select>
			</div>
			<div style="display: flex; gap: 10px;">
				<input bind:value={form.brand} placeholder="品牌" style="flex: 1;" />
				<input bind:value={form.model} placeholder="型号" style="flex: 1;" />
				<input type="number" bind:value={form.default_qty} min="1" style="width: 60px;" />
			</div>
			<input bind:value={form.notes} placeholder="备注" />
			<button class="primary" onclick={save} disabled={!form.name}>
				{editingId ? '更新' : '添加'}
			</button>
		</div>
	</div>
{/if}

{#each categories as cat}
	{@const catItems = items.filter((i) => i.category_id === cat.id)}
	{#if catItems.length > 0}
		<div style="margin-bottom: 16px;">
			<h3 style="margin-bottom: 8px;">{cat.icon} {cat.name} ({catItems.length})</h3>
			{#each catItems as item}
				<div class="card" style="display: flex; justify-content: space-between; align-items: center; padding: 10px 16px;">
					<div>
						<strong>{item.name}</strong>
						{#if item.brand || item.model}
							<span style="color: var(--text-secondary); margin-left: 8px;">
								{item.brand} {item.model}
							</span>
						{/if}
						{#if item.default_qty > 1}
							<span style="color: var(--text-secondary);"> x{item.default_qty}</span>
						{/if}
						{#if item.notes}
							<span style="color: var(--text-secondary); font-size: 13px; margin-left: 8px;">{item.notes}</span>
						{/if}
					</div>
					<div style="display: flex; gap: 6px;">
						<button class="small" onclick={() => startEdit(item)}>编辑</button>
						<button class="small danger" onclick={() => remove(item.id)}>删除</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
{/each}

{#if items.length === 0}
	<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
		物品库为空，点击上方按钮添加物品
	</div>
{/if}
