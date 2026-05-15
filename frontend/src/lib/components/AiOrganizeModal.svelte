<script lang="ts">
	import type {
		Category,
		Tag,
		Item,
		OrganizeAction,
		OrganizePreviewResponse,
		OrganizeApplyResponse,
		AttributeDefinition
	} from '$lib/types';
	import { api } from '$lib/api/client';

	let {
		items,
		categories,
		tags,
		attrDefs = [],
		itemIds = undefined,
		onDone,
		onClose,
		onNewTags
	}: {
		items: Item[];
		categories: Category[];
		tags: Tag[];
		attrDefs?: AttributeDefinition[];
		itemIds?: number[];
		onDone: () => void;
		onClose: () => void;
		onNewTags?: (tags: Tag[]) => void;
	} = $props();

	const isSelectiveMode = $derived(itemIds != null && itemIds.length > 0);
	const modalTitle = $derived(isSelectiveMode ? 'AI 整理选中项' : 'AI 智能整理');

	type Stage = 'loading' | 'preview' | 'applying';
	let stage = $state<Stage>('loading');
	let errorMsg = $state('');
	let actions = $state<OrganizeAction[]>([]);
	let selected = $state<Set<number>>(new Set());
	let applyResult = $state<OrganizeApplyResponse | null>(null);

	function getItemName(itemId: number): string {
		const item = items.find((i) => i.id === itemId);
		return String(item?.attrs?.name ?? `#${itemId}`);
	}

	function getItemInfo(itemId: number): Item | undefined {
		return items.find((i) => i.id === itemId);
	}

	function getCategoryName(catId: number | undefined | null): string {
		if (catId == null) return '';
		return categories.find((c) => c.id === catId)?.name ?? '';
	}

	function getTagName(tagId: number | undefined | null): string {
		if (tagId == null) return '';
		return tags.find((t) => t.id === tagId)?.name ?? '';
	}

	async function loadPreview() {
		stage = 'loading';
		errorMsg = '';
		try {
			const body = isSelectiveMode ? { item_ids: itemIds } : {};
			const resp = await api.post<OrganizePreviewResponse>('/ai/organize-preview', body);
			actions = resp.actions;
			if (resp.new_tags.length > 0) {
				onNewTags?.(resp.new_tags);
			}
			// Select all by default
			selected = new Set(actions.map((_, i) => i));
			stage = 'preview';
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : 'AI 分析失败';
			stage = 'preview';
		}
	}

	function toggleSelect(index: number) {
		const next = new Set(selected);
		if (next.has(index)) next.delete(index);
		else next.add(index);
		selected = next;
	}

	function toggleAll() {
		if (selected.size === actions.length) {
			selected = new Set();
		} else {
			selected = new Set(actions.map((_, i) => i));
		}
	}

	const selectedCount = $derived(selected.size);

	async function handleApply() {
		const selectedActions = actions.filter((_, i) => selected.has(i));
		if (selectedActions.length === 0) return;
		stage = 'applying';
		try {
			const resp = await api.post<OrganizeApplyResponse>('/ai/organize-apply', {
				actions: selectedActions
			});
			applyResult = resp;
			stage = 'preview';
			// Tags are already created during preview phase, no need to re-emit
			onDone();
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : '应用失败';
			stage = 'preview';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	// Start loading immediately
	$effect(() => {
		loadPreview();
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		{#if stage === 'loading'}
			<div class="modal-header">
				<h2>{modalTitle}</h2>
			</div>
			<div class="modal-body loading-body">
				<div class="spinner"></div>
				<p>AI 正在分析物品库...</p>
				<p class="hint">这可能需要十几秒</p>
			</div>
		{:else if stage === 'applying'}
			<div class="modal-header">
				<h2>{modalTitle}</h2>
			</div>
			<div class="modal-body loading-body">
				<div class="spinner"></div>
				<p>正在应用修改...</p>
			</div>
		{:else if applyResult}
			<div class="modal-header">
				<h2>整理完成</h2>
				<button class="close-btn" onclick={onClose}>&times;</button>
			</div>
			<div class="modal-body">
				<div class="result-summary">
					<p>修改了 {applyResult.updated} 个物品</p>
					<p>新建了 {applyResult.created} 个物品</p>
					<p>删除了 {applyResult.deleted} 个物品</p>
				</div>
			</div>
			<div class="modal-footer">
				<button class="primary" onclick={onClose}>关闭</button>
			</div>
		{:else}
			<div class="modal-header">
				<h2>{modalTitle}</h2>
				<button class="close-btn" onclick={onClose}>&times;</button>
			</div>
			<div class="modal-body preview-body">
				{#if errorMsg}
					<div class="error">{errorMsg}</div>
				{/if}
				{#if actions.length === 0}
					<p class="hint">物品库状态良好，没有发现需要整理的问题。</p>
				{:else}
					<div class="select-all">
						<label>
							<input
								type="checkbox"
								checked={selected.size === actions.length}
								onchange={toggleAll}
							/>
							全选 ({actions.length} 项)
						</label>
					</div>
					<div class="action-list">
						{#each actions as action, i}
							{@const item = getItemInfo(action.item_id)}
							<div class="action-card" class:deselected={!selected.has(i)}>
								<div class="action-header">
									<label class="action-check">
										<input
											type="checkbox"
											checked={selected.has(i)}
											onchange={() => toggleSelect(i)}
										/>
									</label>
									<span
										class="action-badge"
										class:badge-update={action.action_type === 'update'}
										class:badge-split={action.action_type === 'split'}
										class:badge-delete={action.action_type === 'delete'}
									>
										{#if action.action_type === 'update'}
											修改
										{:else if action.action_type === 'split'}
											拆分
										{:else}
											删除
										{/if}
									</span>
									<span class="action-item-name">{getItemName(action.item_id)}</span>
								</div>
								<div class="action-reason">{action.reason}</div>
								<div class="action-detail">
									{#if action.action_type === 'update'}
										<div class="diff-list">
											{#if action.fields.attrs?.name != null}
												<div class="diff-row">
													<span class="diff-label">名称</span>
													<span class="diff-old">{String(item?.attrs?.name ?? '')}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new">{String(action.fields.attrs?.name ?? '')}</span>
												</div>
											{/if}
											{#if action.fields.attrs?.brand != null}
												<div class="diff-row">
													<span class="diff-label">品牌</span>
													<span class="diff-old">{String(item?.attrs?.brand ?? '')}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new">{String(action.fields.attrs?.brand ?? '')}</span>
												</div>
											{/if}
											{#if action.fields.attrs?.model != null}
												<div class="diff-row">
													<span class="diff-label">型号</span>
													<span class="diff-old">{String(item?.attrs?.model ?? '')}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new">{String(action.fields.attrs?.model ?? '')}</span>
												</div>
											{/if}
											{#if action.fields.category_name != null || action.fields.category_id != null}
												<div class="diff-row">
													<span class="diff-label">分类</span>
													<span class="diff-old">{getCategoryName(item?.category_id)}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new"
														>{action.fields.category_name ||
															getCategoryName(action.fields.category_id)}</span
													>
												</div>
											{/if}
											{#if action.fields.tag_name != null || action.fields.tag_id !== undefined}
												<div class="diff-row">
													<span class="diff-label">标签</span>
													<span class="diff-old">{getTagName(item?.tag_id) || '-'}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new"
														>{action.fields.tag_name ||
															getTagName(action.fields.tag_id) ||
															'-'}</span
													>
												</div>
											{/if}
											{#if action.fields.attrs?.notes != null}
												<div class="diff-row">
													<span class="diff-label">备注</span>
													<span class="diff-old">{String(item?.attrs?.notes ?? '')}</span>
													<span class="diff-arrow">→</span>
													<span class="diff-new">{String(action.fields.attrs?.notes ?? '')}</span>
												</div>
											{/if}
											{#if action.fields.attrs}
												{#each Object.entries(action.fields.attrs) as [key, value]}
													{#if key !== 'name' && key !== 'brand' && key !== 'model' && key !== 'notes'}
														{@const def = attrDefs.find((d) => d.key === key)}
														{@const label = def?.label ?? key}
														<div class="diff-row">
															<span class="diff-label">{label}</span>
															<span class="diff-old">{String(item?.attrs?.[key] ?? '')}</span>
															<span class="diff-arrow">→</span>
															<span class="diff-new">{String(value ?? '')}</span>
														</div>
													{/if}
												{/each}
											{/if}
										</div>
									{:else if action.action_type === 'split'}
										<div class="split-list">
											<span class="split-label">拆分为：</span>
											{#each action.new_items as newItem}
												<span class="split-item"
													>{String(newItem.attrs?.name ?? '')}{#if newItem.category_name}
														<span class="split-cat">({newItem.category_name})</span
														>{/if}</span
												>
											{/each}
										</div>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button onclick={onClose}>取消</button>
				{#if actions.length > 0}
					<button class="primary" onclick={handleApply} disabled={selectedCount === 0}>
						应用选中的 {selectedCount} 项
					</button>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 12px;
		width: 90vw;
		max-width: 700px;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}

	.modal-header h2 {
		margin: 0;
		font-size: 18px;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 24px;
		cursor: pointer;
		color: var(--text-secondary);
		padding: 0 4px;
		line-height: 1;
	}

	.modal-body {
		padding: 20px;
		overflow-y: auto;
		flex: 1;
		min-height: 0;
	}

	.modal-footer {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
		padding: 16px 20px;
		border-top: 1px solid var(--border);
		flex-shrink: 0;
	}

	.hint {
		color: var(--text-secondary);
		font-size: 13px;
	}

	.error {
		color: var(--danger);
		margin-bottom: 12px;
		font-size: 14px;
	}

	.loading-body {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 60px 20px;
		gap: 12px;
	}

	.spinner {
		width: 36px;
		height: 36px;
		border: 3px solid var(--border);
		border-top-color: var(--primary);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.preview-body {
		padding: 12px 16px;
	}

	.select-all {
		margin-bottom: 12px;
		font-size: 13px;
		color: var(--text-secondary);
	}

	.select-all label {
		display: flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
	}

	.action-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.action-card {
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 12px;
		transition: opacity 0.15s;
	}

	.action-card.deselected {
		opacity: 0.45;
	}

	.action-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 4px;
	}

	.action-check {
		cursor: pointer;
		display: flex;
		align-items: center;
	}

	.action-badge {
		font-size: 11px;
		font-weight: 600;
		padding: 2px 8px;
		border-radius: 4px;
		white-space: nowrap;
	}

	.badge-update {
		background: #dbeafe;
		color: #1e40af;
	}
	.badge-split {
		background: #fed7aa;
		color: #9a3412;
	}
	.badge-delete {
		background: #fecaca;
		color: #991b1b;
	}

	.action-item-name {
		font-weight: 600;
		font-size: 14px;
	}

	.action-reason {
		font-size: 13px;
		color: var(--text-secondary);
		margin-left: 28px;
		margin-bottom: 6px;
	}

	.action-detail {
		margin-left: 28px;
	}

	.diff-list {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-size: 13px;
	}

	.diff-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.diff-label {
		color: var(--text-secondary);
		min-width: 36px;
		font-size: 12px;
	}

	.diff-old {
		text-decoration: line-through;
		color: var(--danger);
	}

	.diff-arrow {
		color: var(--text-secondary);
	}

	.diff-new {
		color: var(--success);
		font-weight: 500;
	}

	.split-list {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		font-size: 13px;
	}

	.split-label {
		color: var(--text-secondary);
	}

	.split-item {
		background: var(--bg);
		border: 1px solid var(--border);
		padding: 2px 8px;
		border-radius: 4px;
	}

	.split-cat {
		color: var(--text-secondary);
		font-size: 11px;
		margin-left: 2px;
	}

	.result-summary {
		font-size: 15px;
		line-height: 2;
	}

	@media (max-width: 768px) {
		.modal {
			width: 95vw;
			max-height: 90vh;
		}
	}
</style>
