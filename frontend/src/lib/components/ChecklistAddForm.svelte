<script lang="ts">
	import type { Item } from '$lib/types';

	let {
		allItems,
		onAdd
	}: {
		allItems: Item[];
		onAdd: (itemId: number | null, customName: string, qty: number) => void;
	} = $props();

	let addItemId = $state<number | null>(null);
	let addCustomName = $state('');
	let addQty = $state(1);

	function handleAdd() {
		onAdd(addItemId, addCustomName, addQty);
		addItemId = null;
		addCustomName = '';
		addQty = 1;
	}
</script>

<div class="card add-form">
	<div class="add-row">
		<div class="add-field" style="flex: 1;">
			<div class="field-label">从物品库选择</div>
			<select bind:value={addItemId} style="width: 100%;">
				<option value={null}>选择物品...</option>
				{#each allItems as it}
					<option value={it.id}>{String(it.attrs?.name ?? '')} {String(it.attrs?.brand ?? '')} {String(it.attrs?.model ?? '')}</option>
				{/each}
			</select>
		</div>
		<div class="add-field">
			<div class="field-label">或自定义名称</div>
			<input bind:value={addCustomName} placeholder="自定义物品" />
		</div>
		<div class="add-field">
			<div class="field-label">数量</div>
			<input type="number" bind:value={addQty} min="1" style="width: 60px;" />
		</div>
		<button class="primary" onclick={handleAdd} disabled={!addItemId && !addCustomName} style="align-self: flex-end;">添加</button>
	</div>
</div>

<style>
	.add-form {
		margin-bottom: 16px;
	}
	.add-row {
		display: flex;
		gap: 10px;
		align-items: end;
	}
	.add-field {
		display: flex;
		flex-direction: column;
	}
	.field-label {
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}
</style>
