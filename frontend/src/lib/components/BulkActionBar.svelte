<script lang="ts">
	export interface BatchAttrOption {
		key: string;
		label: string;
		type: string; // text | number | weight | bar | stars | bool | select
		config?: { max?: number; suffix?: string; options?: string[] };
		selectOptions?: { value: string | number; label: string }[];
	}

	export interface BulkAction {
		label: string;
		action: () => Promise<void> | void;
		variant?: 'default' | 'danger';
	}

	let {
		selectedCount,
		attrOptions = [],
		actions = [],
		onBatchDelete,
		onBatchUpdateAttr,
		onAiOrganize,
	}: {
		selectedCount: number;
		attrOptions?: BatchAttrOption[];
		actions?: BulkAction[];
		onBatchDelete?: () => Promise<void>;
		onBatchUpdateAttr?: (attrKey: string, value: unknown) => Promise<void>;
		onAiOrganize?: () => void;
	} = $props();

	// Legacy mode: use `actions` prop
	const legacyMode = $derived(actions.length > 0);

	let selectedAttrKey = $state<string | null>(null);
	let editingValue = $state<unknown>(null);
	let processing = $state(false);

	const selectedAttr = $derived(
		selectedAttrKey ? attrOptions.find(a => a.key === selectedAttrKey) ?? null : null
	);

	function selectAttr(key: string) {
		selectedAttrKey = key;
		// Initialize editing value based on type
		const attr = attrOptions.find(a => a.key === key);
		if (!attr) return;
		switch (attr.type) {
			case 'bool': editingValue = false; break;
			case 'bar':
			case 'weight':
			case 'number': editingValue = 0; break;
			case 'stars': editingValue = 0; break;
			case 'select': editingValue = attr.selectOptions?.[0]?.value ?? ''; break;
			case 'text':
			default:
				if (attr.config?.options) editingValue = attr.config.options[0];
				else editingValue = '';
				break;
		}
	}

	async function apply() {
		if (!selectedAttrKey || processing || !onBatchUpdateAttr) return;
		processing = true;
		try {
			await onBatchUpdateAttr(selectedAttrKey, editingValue);
			selectedAttrKey = null;
			editingValue = null;
		} finally {
			processing = false;
		}
	}

	async function handleDelete() {
		if (processing || !onBatchDelete) return;
		processing = true;
		try {
			await onBatchDelete();
		} finally {
			processing = false;
		}
	}
</script>

{#if selectedCount > 0}
	<div class="bulk-bar card">
		<span class="bulk-count">已选 {selectedCount} 项</span>

		{#if legacyMode}
			{#each actions as act}
				<button
					class="small {act.variant === 'danger' ? 'danger' : ''}"
					onclick={() => act.action()}
				>
					{act.label}
				</button>
			{/each}
		{:else}
			<span class="attr-label">属性</span>
			<select class="attr-select" value={selectedAttrKey ?? ''} onchange={(e) => selectAttr(e.currentTarget.value)}>
				<option value="" disabled>选择属性...</option>
				{#each attrOptions as opt (opt.key)}
					<option value={opt.key}>{opt.label}</option>
				{/each}
			</select>

			{#if selectedAttr}
				<span class="eq-sign">=</span>

				{#if selectedAttr.type === 'bool'}
					<label class="bool-label">
						<input type="checkbox" checked={!!editingValue} onchange={(e) => editingValue = e.currentTarget.checked} />
					</label>
				{:else if selectedAttr.type === 'text' && selectedAttr.config?.options}
					<div class="pill-group">
						{#each selectedAttr.config.options as opt}
							<button
								class="pill"
								class:active={editingValue === opt}
								onclick={() => editingValue = opt}
							>{opt}</button>
						{/each}
					</div>
				{:else if selectedAttr.type === 'stars'}
					<div class="stars-row">
						{#each Array(selectedAttr.config?.max ?? 5) as _, i}
							<button
								class="star-btn"
								class:filled={(editingValue as number) > i}
								onclick={() => editingValue = i + 1}
							>{ (editingValue as number) > i ? '★' : '☆' }</button>
						{/each}
					</div>
				{:else if selectedAttr.type === 'bar' || selectedAttr.type === 'weight' || selectedAttr.type === 'number'}
					<input
						type="number"
						class="num-input"
						value={editingValue as number}
						oninput={(e) => editingValue = Number(e.currentTarget.value)}
						min={0}
						max={selectedAttr.config?.max ?? undefined}
					/>
					{#if selectedAttr.config?.suffix}
						<span class="suffix">{selectedAttr.config.suffix}</span>
					{/if}
				{:else if selectedAttr.type === 'select'}
					<select class="attr-select" value={editingValue as string | number} onchange={(e) => editingValue = e.currentTarget.value}>
						{#each selectedAttr.selectOptions ?? [] as opt}
							<option value={opt.value}>{opt.label}</option>
						{/each}
					</select>
				{:else}
					<!-- text (freeform) -->
					<input
						type="text"
						class="text-input"
						value={editingValue as string}
						oninput={(e) => editingValue = e.currentTarget.value}
						placeholder="输入值..."
					/>
				{/if}

				<button class="small primary" onclick={apply} disabled={processing}>
					{processing ? '...' : '应用'}
				</button>
			{/if}

			{#if onAiOrganize}
				<button class="small" onclick={() => onAiOrganize()}>
					AI 整理选中
				</button>
			{/if}

			<button class="small danger" onclick={handleDelete} disabled={processing}>
				批量删除
			</button>
		{/if}
	</div>
{/if}

<style>
	.bulk-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #e8f0fe;
		margin-bottom: 12px;
		flex-wrap: wrap;
	}
	.bulk-count {
		font-size: 13px;
		font-weight: 500;
		color: var(--primary);
		white-space: nowrap;
	}
	.attr-label {
		font-size: 12px;
		color: var(--text-secondary);
		white-space: nowrap;
	}
	.attr-select {
		font-size: 12px;
		padding: 3px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}
	.eq-sign {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-secondary);
	}

	.bool-label {
		display: flex;
		align-items: center;
	}
	.bool-label input[type='checkbox'] {
		width: 18px;
		height: 18px;
		accent-color: var(--primary);
		cursor: pointer;
	}

	.pill-group {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.pill {
		font-size: 12px;
		padding: 2px 10px;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: var(--surface);
		color: var(--text);
		cursor: pointer;
		transition: all 0.15s;
	}
	.pill:hover {
		border-color: var(--primary);
	}
	.pill.active {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}

	.stars-row {
		display: flex;
		gap: 0;
	}
	.star-btn {
		background: none;
		border: none;
		font-size: 16px;
		cursor: pointer;
		padding: 0 1px;
		color: var(--border);
		transition: color 0.1s;
	}
	.star-btn.filled {
		color: #f59e0b;
	}

	.num-input,
	.text-input {
		width: 80px;
		font-size: 12px;
		padding: 3px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}
	.suffix {
		font-size: 12px;
		color: var(--text-secondary);
	}
</style>
