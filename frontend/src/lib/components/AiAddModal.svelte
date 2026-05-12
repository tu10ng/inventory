<script lang="ts">
	import type { Category, Tag, AiParsedItem } from '$lib/types';
	import { aiPostStream } from '$lib/api/client';

	let {
		categories,
		tags,
		onConfirm,
		onClose,
		onNewTags,
		prefillAiText = ''
	}: {
		categories: Category[];
		tags: Tag[];
		onConfirm: (items: AiParsedItem[]) => void;
		onClose: () => void;
		onNewTags?: (tags: Tag[]) => void;
		prefillAiText?: string;
	} = $props();

	type Stage = 'input' | 'loading' | 'preview';
	let stage = $state<Stage>('input');
	let inputText = $state('');
	let errorMsg = $state('');
	let parsedItems = $state<AiParsedItem[]>([]);
	let thinkingText = $state('');
	let progressMsg = $state('');
	let streamController = $state<AbortController | null>(null);

	$effect(() => {
		if (prefillAiText) {
			inputText = prefillAiText;
		}
	});

	function handleParse() {
		if (!inputText.trim()) return;
		stage = 'loading';
		errorMsg = '';
		thinkingText = '';
		progressMsg = '';

		streamController = aiPostStream(
			'/ai/parse-items-stream',
			{ text: inputText },
			{
				onThinking(text: string) {
					thinkingText += text;
				},
				onProgress(msg: string) {
					progressMsg = msg;
				},
				onResult(data: { items: AiParsedItem[]; new_tags: Tag[] }) {
					parsedItems = data.items.map(item => ({
						...item,
						attrs: item.attrs || {}
					}));
					if (data.new_tags && data.new_tags.length > 0) {
						onNewTags?.(data.new_tags);
					}
					stage = 'preview';
				},
				onError(msg: string) {
					errorMsg = msg;
					stage = 'input';
				}
			}
		);
	}

	function cancelStream() {
		streamController?.abort();
		streamController = null;
	}

	function handleClose() {
		cancelStream();
		onClose();
	}

	function handleCancel() {
		cancelStream();
		goBack();
	}

	function removeItem(index: number) {
		parsedItems = parsedItems.filter((_, i) => i !== index);
	}

	function handleConfirm() {
		if (parsedItems.length === 0) return;
		onConfirm(parsedItems);
	}

	function goBack() {
		stage = 'input';
	}

	function getCategoryName(catId: number | null): string {
		if (catId == null) return '-';
		return categories.find(c => c.id === catId)?.name ?? '-';
	}

	function getTagName(tagId: number | null): string {
		if (tagId == null) return '-';
		return tags.find(t => t.id === tagId)?.name ?? '-';
	}

	function handleCategoryChange(index: number, value: string) {
		const catId = parseInt(value);
		parsedItems[index].category_id = isNaN(catId) ? null : catId;
		const currentTag = tags.find(t => t.id === parsedItems[index].tag_id);
		if (currentTag && currentTag.category_id !== catId) {
			parsedItems[index].tag_id = null;
		}
	}

	function handleTagChange(index: number, value: string) {
		const tagId = parseInt(value);
		parsedItems[index].tag_id = isNaN(tagId) ? null : tagId;
	}

	function availableTags(catId: number | null): Tag[] {
		if (catId == null) return tags;
		return tags.filter(t => t.category_id === catId);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			cancelStream();
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={handleClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		{#if stage === 'input'}
			<div class="modal-header">
				<h2>AI 智能添加</h2>
				<button class="close-btn" onclick={handleClose}>&times;</button>
			</div>
			<div class="modal-body">
				<p class="hint">用自然语言描述你要添加的物品，AI 会自动解析出结构化信息。</p>
				<textarea
					bind:value={inputText}
					placeholder="例如：始祖鸟 Beta LT 冲锋衣, 黑钻登山杖, Garmin 飞耐时7手表, 海尔兄弟防晒霜"
					rows="5"
				></textarea>
				{#if errorMsg}
					<div class="error">{errorMsg}</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button onclick={handleClose}>取消</button>
				<button class="primary" onclick={handleParse} disabled={!inputText.trim()}>
					解析
				</button>
			</div>

		{:else if stage === 'loading'}
			<div class="modal-header">
				<h2>AI 智能添加</h2>
				<button class="close-btn" onclick={handleClose}>&times;</button>
			</div>
			<div class="modal-body loading-body">
				<div class="thinking-indicator">
					<span class="pulse-dot"></span>
					{progressMsg || 'AI 思考中...'}
				</div>
				<pre class="thinking-text">{thinkingText}<span class="cursor-blink">|</span></pre>
			</div>
			<div class="modal-footer">
				<button onclick={handleCancel}>取消</button>
			</div>

		{:else if stage === 'preview'}
			<div class="modal-header">
				<h2>解析结果预览</h2>
				<button class="close-btn" onclick={handleClose}>&times;</button>
			</div>
			<div class="modal-body preview-body">
				{#if parsedItems.length === 0}
					<p class="hint">没有解析到任何物品。</p>
				{:else}
					<div class="table-wrapper">
						<table>
							<thead>
								<tr>
									<th>名称</th>
									<th>品牌</th>
									<th>型号</th>
									<th>分类</th>
									<th>标签</th>
									<th></th>
								</tr>
							</thead>
							<tbody>
								{#each parsedItems as item, i}
									<tr>
										<td>
											<input type="text" bind:value={item.attrs.name} class="cell-input" />
										</td>
										<td>
											<input type="text" bind:value={item.attrs.brand} class="cell-input" />
										</td>
										<td>
											<input type="text" bind:value={item.attrs.model} class="cell-input" />
										</td>
										<td>
											<select
												value={item.category_id?.toString() ?? ''}
												onchange={(e) => handleCategoryChange(i, e.currentTarget.value)}
												class="cell-select"
											>
												<option value="">-</option>
												{#each categories as cat}
													<option value={cat.id.toString()}>{cat.name}</option>
												{/each}
											</select>
										</td>
										<td>
											<select
												value={item.tag_id?.toString() ?? ''}
												onchange={(e) => handleTagChange(i, e.currentTarget.value)}
												class="cell-select"
											>
												<option value="">-</option>
												{#each availableTags(item.category_id) as tag}
													<option value={tag.id.toString()}>{tag.name}</option>
												{/each}
											</select>
										</td>
										<td>
											<button class="small danger" onclick={() => removeItem(i)}>删除</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button onclick={goBack}>返回修改</button>
				<button onclick={handleClose}>取消</button>
				<button class="primary" onclick={handleConfirm} disabled={parsedItems.length === 0}>
					确认添加 ({parsedItems.length} 件)
				</button>
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
		max-width: 900px;
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
		margin-bottom: 12px;
	}

	textarea {
		width: 100%;
		resize: vertical;
		font-size: 15px;
		line-height: 1.6;
		padding: 12px;
		border: 1px solid var(--border);
		border-radius: 8px;
		font-family: inherit;
		box-sizing: border-box;
	}

	.error {
		color: var(--danger);
		margin-top: 8px;
		font-size: 14px;
	}

	.loading-body {
		display: flex;
		flex-direction: column;
		padding: 16px 20px;
		gap: 12px;
	}

	.thinking-indicator {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		color: var(--text-secondary);
	}

	.pulse-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--primary);
		animation: pulse 1.2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 0.3; transform: scale(0.8); }
		50% { opacity: 1; transform: scale(1.2); }
	}

	.thinking-text {
		background: var(--inventory-bg, #1a1a2e);
		color: var(--inventory-text, #e0e0e0);
		border: 1px solid var(--inventory-border, #0f3460);
		border-radius: 8px;
		padding: 16px;
		font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
		font-size: 13px;
		line-height: 1.7;
		white-space: pre-wrap;
		word-wrap: break-word;
		overflow-y: auto;
		max-height: 300px;
		min-height: 120px;
		margin: 0;
	}

	.cursor-blink {
		color: var(--primary);
		animation: blink 0.8s step-end infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0; }
	}

	.preview-body {
		padding: 12px;
	}

	.table-wrapper {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	th {
		text-align: left;
		padding: 6px 8px;
		border-bottom: 2px solid var(--border);
		font-weight: 600;
		white-space: nowrap;
		color: var(--text-secondary);
		font-size: 12px;
	}

	td {
		padding: 4px 4px;
		border-bottom: 1px solid var(--border);
	}

	.cell-input {
		width: 100%;
		padding: 4px 6px;
		border: 1px solid transparent;
		border-radius: 4px;
		font-size: 13px;
		background: transparent;
		box-sizing: border-box;
	}

	.cell-input:hover, .cell-input:focus {
		border-color: var(--border);
		background: var(--bg);
	}

	.cell-select {
		width: 100%;
		padding: 4px 4px;
		border: 1px solid transparent;
		border-radius: 4px;
		font-size: 13px;
		background: transparent;
		cursor: pointer;
	}

	.cell-select:hover, .cell-select:focus {
		border-color: var(--border);
		background: var(--bg);
	}

	@media (max-width: 768px) {
		.modal {
			width: 95vw;
			max-height: 90vh;
		}
	}
</style>
