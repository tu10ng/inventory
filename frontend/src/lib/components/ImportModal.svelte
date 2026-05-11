<script lang="ts">
	import type { ImportPreviewResult, ImportResult } from '$lib/types';
	import { api } from '$lib/api/client';

	type Stage = 'select' | 'preview' | 'importing' | 'result';

	let {
		onClose,
		onDone
	}: {
		onClose: () => void;
		onDone: () => void;
	} = $props();

	let stage = $state<Stage>('select');
	let errorMsg = $state('');
	let strategy = $state<'skip' | 'update'>('skip');

	let importData = $state<object | null>(null);
	let preview = $state<ImportPreviewResult | null>(null);
	let result = $state<ImportResult | null>(null);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	async function handleFileSelected(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		errorMsg = '';
		try {
			const text = await file.text();
			const data = JSON.parse(text);

			// Basic structure validation
			if (!data.version || !data.items || !Array.isArray(data.items)) {
				errorMsg = '无效的导出文件：缺少 version 或 items 字段';
				return;
			}
			if (!data.categories || !Array.isArray(data.categories)) {
				errorMsg = '无效的导出文件：缺少 categories 字段';
				return;
			}
			if (!data.tags || !Array.isArray(data.tags)) {
				errorMsg = '无效的导出文件：缺少 tags 字段';
				return;
			}
			if (!data.attribute_definitions || !Array.isArray(data.attribute_definitions)) {
				errorMsg = '无效的导出文件：缺少 attribute_definitions 字段';
				return;
			}

			importData = data;
			await loadPreview(data);
		} catch (err) {
			if (err instanceof SyntaxError) {
				errorMsg = '无法解析 JSON 文件，请检查文件格式';
			} else {
				errorMsg = (err as Error).message;
			}
		}
	}

	async function loadPreview(data: object) {
		stage = 'preview';
		errorMsg = '';
		preview = null;
		try {
			const payload = { ...data, strategy };
			preview = await api.post<ImportPreviewResult>('/items/import-preview', payload);
		} catch (err) {
			errorMsg = (err as Error).message;
			stage = 'select';
		}
	}

	async function handleImport() {
		if (!importData) return;
		stage = 'importing';
		errorMsg = '';
		try {
			const payload = { ...importData, strategy };
			result = await api.post<ImportResult>('/items/import', payload);
			stage = 'result';
		} catch (err) {
			errorMsg = (err as Error).message;
			stage = 'preview';
		}
	}

	function handleDone() {
		onDone();
		onClose();
	}

	function handleStrategyChange(newStrategy: 'skip' | 'update') {
		strategy = newStrategy;
		if (importData) {
			loadPreview(importData);
		}
	}

	function goBack() {
		stage = 'select';
		importData = null;
		preview = null;
		errorMsg = '';
	}

	function actionLabel(action: string): string {
		switch (action) {
			case 'new': return '新增';
			case 'skip': return '跳过';
			case 'update': return '更新';
			default: return action;
		}
	}

	function actionClass(action: string): string {
		return `action-${action}`;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		{#if stage === 'select'}
			<div class="modal-header">
				<h2>导入物品库</h2>
				<button class="close-btn" onclick={onClose}>&times;</button>
			</div>
			<div class="modal-body">
				<p class="hint">选择一个之前导出的 JSON 备份文件，导入其中的物品、分类、标签和属性定义。</p>

				<div class="strategy-section">
					<span class="section-label">冲突处理策略</span>
					<div class="radio-group">
						<label class="radio-label">
							<input type="radio" name="strategy" value="skip" checked={strategy === 'skip'}
								onchange={() => handleStrategyChange('skip')} />
							跳过已存在的物品（只添加新物品）
						</label>
						<label class="radio-label">
							<input type="radio" name="strategy" value="update" checked={strategy === 'update'}
								onchange={() => handleStrategyChange('update')} />
							更新已存在的物品（用导入数据覆盖）
						</label>
					</div>
				</div>

				<div class="file-upload">
					<input type="file" accept=".json" id="import-file" onchange={handleFileSelected}
						class="file-input" />
					<label for="import-file" class="file-label">
						<span class="file-icon">📁</span>
						<span>选择 JSON 文件</span>
					</label>
				</div>

				{#if errorMsg}
					<div class="error">{errorMsg}</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button onclick={onClose}>取消</button>
			</div>

		{:else if stage === 'preview'}
			<div class="modal-header">
				<h2>导入预览</h2>
				<button class="close-btn" onclick={onClose}>&times;</button>
			</div>
			{#if preview == null}
				<div class="modal-body loading-body">
					<div class="spinner"></div>
					<p>正在分析导入数据...</p>
				</div>
			{:else}
				<div class="modal-body preview-body">
					<div class="summary">
						<div class="summary-item">
							<span class="summary-label">总物品数</span>
							<span class="summary-value">{preview.total_items}</span>
						</div>
						<div class="summary-item new">
							<span class="summary-label">新增</span>
							<span class="summary-value">{preview.new_items}</span>
						</div>
						<div class="summary-item existing">
							<span class="summary-label">{strategy === 'skip' ? '跳过' : '更新'}</span>
							<span class="summary-value">{preview.skip_or_update_items}</span>
						</div>
					</div>

					<p class="strategy-note">
						策略：<strong>{strategy === 'skip' ? '跳过已存在' : '更新已存在'}</strong>
						<button class="link-btn" onclick={goBack}>修改</button>
					</p>

					{#if preview.preview_items.length > 0}
						<div class="table-wrapper">
							<table>
								<thead>
									<tr>
										<th>名称</th>
										<th>品牌</th>
										<th>型号</th>
										<th>操作</th>
									</tr>
								</thead>
								<tbody>
									{#each preview.preview_items as item}
										<tr>
											<td>{item.name}</td>
											<td>{item.brand || '-'}</td>
											<td>{item.model || '-'}</td>
											<td><span class="action-badge {actionClass(item.action)}">{actionLabel(item.action)}</span></td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						{#if preview.total_items > 50}
							<p class="truncated-note">还有 {preview.total_items - 50} 条未显示</p>
						{/if}
					{/if}

					{#if errorMsg}
						<div class="error">{errorMsg}</div>
					{/if}
				</div>
				<div class="modal-footer">
					<button onclick={goBack}>重新选择</button>
					<button onclick={onClose}>取消</button>
					<button class="primary" onclick={handleImport}>
						确认导入
					</button>
				</div>
			{/if}

		{:else if stage === 'importing'}
			<div class="modal-header">
				<h2>导入中...</h2>
			</div>
			<div class="modal-body loading-body">
				<div class="spinner"></div>
				<p>正在导入数据...</p>
			</div>

		{:else if stage === 'result'}
			<div class="modal-header">
				<h2>导入完成</h2>
				<button class="close-btn" onclick={handleDone}>&times;</button>
			</div>
			<div class="modal-body">
				<div class="result-grid">
					{#if result!.categories_created > 0}
						<div class="result-item">
							<span class="result-label">分类</span>
							<span class="result-value">+{result!.categories_created}</span>
						</div>
					{/if}
					{#if result!.tags_created > 0}
						<div class="result-item">
							<span class="result-label">标签</span>
							<span class="result-value">+{result!.tags_created}</span>
						</div>
					{/if}
					{#if result!.attribute_definitions_created > 0}
						<div class="result-item">
							<span class="result-label">属性定义</span>
							<span class="result-value">+{result!.attribute_definitions_created}</span>
						</div>
					{/if}
					<div class="result-item">
						<span class="result-label">物品新增</span>
						<span class="result-value">+{result!.items_created}</span>
					</div>
					{#if result!.items_updated > 0}
						<div class="result-item">
							<span class="result-label">物品更新</span>
							<span class="result-value">{result!.items_updated}</span>
						</div>
					{/if}
					{#if result!.items_skipped > 0}
						<div class="result-item">
							<span class="result-label">物品跳过</span>
							<span class="result-value">{result!.items_skipped}</span>
						</div>
					{/if}
				</div>
			</div>
			<div class="modal-footer">
				<button class="primary" onclick={handleDone}>完成</button>
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
		margin-bottom: 16px;
	}

	.error {
		color: var(--danger);
		margin-top: 12px;
		font-size: 14px;
	}

	.strategy-section {
		margin-bottom: 16px;
	}

	.section-label {
		display: block;
		font-weight: 600;
		margin-bottom: 8px;
		font-size: 14px;
	}

	.radio-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.radio-label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		cursor: pointer;
	}

	.file-upload {
		margin-bottom: 8px;
	}

	.file-input {
		display: none;
	}

	.file-label {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px 24px;
		border: 2px dashed var(--border);
		border-radius: 8px;
		cursor: pointer;
		font-size: 15px;
		color: var(--text-secondary);
		transition: border-color 0.2s;
	}

	.file-label:hover {
		border-color: var(--primary);
		color: var(--text);
	}

	.file-icon {
		font-size: 20px;
	}

	/* Preview stage */
	.preview-body {
		padding: 16px 20px;
	}

	.summary {
		display: flex;
		gap: 16px;
		margin-bottom: 16px;
	}

	.summary-item {
		flex: 1;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 12px;
		text-align: center;
	}

	.summary-item.new {
		border-color: var(--success);
	}

	.summary-item.existing {
		border-color: var(--warning);
	}

	.summary-label {
		display: block;
		font-size: 12px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}

	.summary-value {
		font-size: 24px;
		font-weight: 700;
	}

	.strategy-note {
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 12px;
	}

	.link-btn {
		background: none;
		border: none;
		color: var(--primary);
		cursor: pointer;
		font-size: 13px;
		padding: 0 4px;
		text-decoration: underline;
	}

	.table-wrapper {
		max-height: 300px;
		overflow-y: auto;
		border: 1px solid var(--border);
		border-radius: 8px;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	th {
		text-align: left;
		padding: 8px 10px;
		border-bottom: 2px solid var(--border);
		font-weight: 600;
		white-space: nowrap;
		color: var(--text-secondary);
		font-size: 12px;
		position: sticky;
		top: 0;
		background: var(--surface);
	}

	td {
		padding: 6px 10px;
		border-bottom: 1px solid var(--border);
	}

	.action-badge {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 10px;
		font-size: 12px;
		font-weight: 600;
	}

	.action-new {
		background: var(--success);
		color: #fff;
	}

	.action-skip {
		background: var(--border);
		color: var(--text-secondary);
	}

	.action-update {
		background: var(--warning);
		color: #fff;
	}

	.truncated-note {
		font-size: 12px;
		color: var(--text-secondary);
		margin-top: 8px;
		text-align: center;
	}

	/* Loading */
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
		to { transform: rotate(360deg); }
	}

	/* Result */
	.result-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.result-item {
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 16px;
		text-align: center;
	}

	.result-label {
		display: block;
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}

	.result-value {
		font-size: 22px;
		font-weight: 700;
		color: var(--success);
	}

	@media (max-width: 768px) {
		.modal {
			width: 95vw;
			max-height: 90vh;
		}
	}
</style>
