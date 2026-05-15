<script lang="ts">
	import { api } from '$lib/api/client';

	let {
		onClose,
		onOpenAiModal
	}: {
		onClose: () => void;
		onOpenAiModal?: (text: string) => void;
	} = $props();

	type Stage = 'upload' | 'loading' | 'ocr-result' | 'ai-loading';
	let stage = $state<Stage>('upload');
	let ocrText = $state('');
	let errorMsg = $state('');
	let files = $state<{ name: string; url: string; file: File }[]>([]);
	let progress = $state({ current: 0, total: 0 });
	let dropActive = $state(false);

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dropActive = true;
	}
	function handleDragLeave() {
		dropActive = false;
	}
	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dropActive = false;
		if (e.dataTransfer?.files) {
			addFiles(e.dataTransfer.files);
		}
	}
	function handleBrowse(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files) addFiles(input.files);
		input.value = '';
	}
	const MAX_SIZE = 50 * 1024 * 1024; // 50MB per image
	const MAX_TOTAL = 100 * 1024 * 1024; // 100MB total

	function formatSize(bytes: number): string {
		if (bytes < 1024) return bytes + ' B';
		if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
		return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
	}

	function addFiles(fileList: FileList) {
		for (let i = 0; i < fileList.length; i++) {
			const file = fileList[i];
			if (!file.type.startsWith('image/')) continue;
			if (file.size > MAX_SIZE) {
				errorMsg = `"${file.name}" 大小 ${formatSize(file.size)}，超过单张限制 50MB`;
				continue;
			}
			files = [...files, {
				name: file.name,
				url: URL.createObjectURL(file),
				file
			}];
		}
		const totalSize = files.reduce((sum, f) => sum + f.file.size, 0);
		if (totalSize > MAX_TOTAL) {
			errorMsg = `总大小 ${formatSize(totalSize)}，超过限制 100MB，请减少图片数量`;
		}
	}
	function removeFile(index: number) {
		URL.revokeObjectURL(files[index].url);
		files = files.filter((_, i) => i !== index);
	}

	async function handleOcr() {
		if (files.length === 0) return;
		stage = 'loading';
		errorMsg = '';
		progress = { current: 0, total: files.length };

		try {
			const formData = new FormData();
			for (const f of files) {
				formData.append('images[]', f.file, f.name);
			}

			progress = { current: 1, total: files.length };
			// Simulate progress (actual OCR is server-side parallel)
			const timer = setInterval(() => {
				progress = {
					current: Math.min(progress.current + 1, progress.total),
					total: progress.total
				};
			}, 500);

			const BASE = '/api';
			const res = await fetch(`${BASE}/ai/ocr`, {
				method: 'POST',
				body: formData
			});

			clearInterval(timer);
			progress = { current: files.length, total: files.length };

			if (!res.ok) {
				let message: string;
				try {
					const body = await res.json();
					message = body.error || `${res.status}: ${JSON.stringify(body)}`;
				} catch {
					message = `请求失败 (${res.status})`;
				}
				throw new Error(message);
			}

			const data = await res.json();
			ocrText = data.ocr_text;
			stage = 'ocr-result';
		} catch (e) {
			const msg = (e as Error).message;
			if (msg.includes('write EPIPE') || msg.includes('Failed to fetch') || msg.includes('413')) {
				errorMsg = '图片太大，服务器拒绝接收（单次上传总大小不超过 50MB）。请压缩图片或分批上传。';
			} else {
				errorMsg = msg;
			}
			stage = 'upload';
		}
	}

	function handleAiParse() {
		if (onOpenAiModal && ocrText.trim()) {
			onOpenAiModal(ocrText.trim());
			onClose();
		}
	}

	function copyToClipboard() {
		navigator.clipboard.writeText(ocrText).catch(() => {});
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	function goBack() {
		stage = 'upload';
		errorMsg = '';
		ocrText = '';
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		<div class="modal-header">
			<h2>OCR 导入订单</h2>
			<button class="close-btn" onclick={onClose}>&times;</button>
		</div>

		{#if stage === 'upload'}
			<div class="modal-body">
				<p class="hint">上传订单截图（支持多张，长订单可分屏截图后一起上传，单张最大 50MB）</p>

				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="drop-zone"
					class:active={dropActive}
					ondragover={handleDragOver}
					ondragleave={handleDragLeave}
					ondrop={handleDrop}
				>
					<div class="drop-icon">📷</div>
					<p>拖拽图片到此处，或点击下方按钮选择文件</p>
					<label class="browse-btn">
						选择图片
						<input type="file" accept="image/*" multiple hidden onchange={handleBrowse} />
					</label>
				</div>

				{#if files.length > 0}
					<div class="image-list">
						{#each files as f, i}
							<div class="image-item">
								<img src={f.url} alt={f.name} />
								<span class="image-name">{f.name}</span>
								<span class="image-size">{formatSize(f.file.size)}</span>
								<button class="remove-btn" onclick={() => removeFile(i)}>&times;</button>
							</div>
						{/each}
						<label class="add-more-btn">
							+ 添加更多
							<input type="file" accept="image/*" multiple hidden onchange={handleBrowse} />
						</label>
					</div>
				{/if}

				{#if errorMsg}
					<div class="error">{errorMsg}</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button onclick={onClose}>取消</button>
				<button class="primary" onclick={handleOcr} disabled={files.length === 0}>
					OCR 识别
				</button>
			</div>

		{:else if stage === 'loading'}
			<div class="modal-body loading-body">
				<div class="spinner"></div>
				<p>正在 OCR 识别 {progress.current}/{progress.total}...</p>
				<p class="hint">正在调用 Tesseract 识别图片中的文字</p>
			</div>

		{:else if stage === 'ocr-result'}
			<div class="modal-body ocr-result-body">
				<div class="two-col">
					<div class="ocr-text-col">
						<h3>OCR 识别结果</h3>
						<p class="hint">可编辑修正 OCR 识别错误</p>
						<textarea bind:value={ocrText} rows="12" class="ocr-textarea"></textarea>
						<button class="small" onclick={copyToClipboard}>复制文本</button>
					</div>
					<div class="actions-col">
						<h3>下一步</h3>
						<p class="hint">OCR 已识别完毕，请选择下一步操作：</p>
						<button class="primary wide" onclick={handleAiParse}>
							AI 智能解析
						</button>
						<p class="hint-small">将 OCR 文本发送给 AI，自动解析出结构化物品信息</p>
						<hr />
						<button class="wide" onclick={copyToClipboard}>
							手动添加
						</button>
						<p class="hint-small">关闭弹窗后，可在"AI 添加"中粘贴文本手动解析</p>
					</div>
				</div>
			</div>
			<div class="modal-footer">
				<button onclick={goBack}>返回重试</button>
				<button onclick={onClose}>关闭</button>
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
	.hint-small {
		color: var(--text-secondary);
		font-size: 12px;
		margin-top: 4px;
	}

	.drop-zone {
		border: 2px dashed var(--border);
		border-radius: 10px;
		padding: 40px 20px;
		text-align: center;
		transition: border-color 0.2s, background 0.2s;
	}
	.drop-zone.active {
		border-color: var(--primary);
		background: color-mix(in srgb, var(--primary), transparent 92%);
	}
	.drop-zone p {
		color: var(--text-secondary);
		font-size: 14px;
		margin: 8px 0;
	}
	.drop-icon {
		font-size: 40px;
		margin-bottom: 8px;
	}
	.browse-btn {
		display: inline-block;
		padding: 6px 16px;
		border: 1px solid var(--border);
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
		margin-top: 8px;
		color: var(--text);
		background: var(--bg);
	}
	.browse-btn:hover {
		border-color: var(--primary);
	}

	.image-list {
		margin-top: 16px;
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: flex-start;
	}
	.image-item {
		position: relative;
		width: 100px;
		text-align: center;
	}
	.image-item img {
		width: 100px;
		height: 100px;
		object-fit: cover;
		border-radius: 6px;
		border: 1px solid var(--border);
	}
	.image-name {
		display: block;
		font-size: 11px;
		color: var(--text-secondary);
		margin-top: 4px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.image-size {
		display: block;
		font-size: 10px;
		color: var(--text-secondary);
		opacity: 0.7;
	}
	.remove-btn {
		position: absolute;
		top: -6px;
		right: -6px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--danger);
		color: white;
		border: none;
		cursor: pointer;
		font-size: 12px;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.add-more-btn {
		width: 100px;
		height: 100px;
		border: 1px dashed var(--border);
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		font-size: 13px;
		color: var(--text-secondary);
	}
	.add-more-btn:hover {
		border-color: var(--primary);
		color: var(--primary);
	}

	.error {
		color: var(--danger);
		margin-top: 12px;
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
		to { transform: rotate(360deg); }
	}

	.ocr-result-body {
		padding: 16px;
	}
	.two-col {
		display: flex;
		gap: 20px;
	}
	.ocr-text-col {
		flex: 1;
		min-width: 0;
	}
	.ocr-text-col h3 {
		font-size: 14px;
		margin-bottom: 4px;
	}
	.actions-col {
		width: 220px;
		flex-shrink: 0;
	}
	.actions-col h3 {
		font-size: 14px;
		margin-bottom: 4px;
	}
	.actions-col hr {
		border: none;
		border-top: 1px solid var(--border);
		margin: 12px 0;
	}
	.ocr-textarea {
		width: 100%;
		resize: vertical;
		font-size: 13px;
		line-height: 1.5;
		padding: 10px;
		border: 1px solid var(--border);
		border-radius: 6px;
		font-family: inherit;
		box-sizing: border-box;
	}
	.wide {
		width: 100%;
		margin-bottom: 4px;
	}

	@media (max-width: 768px) {
		.modal {
			width: 95vw;
			max-height: 90vh;
		}
		.two-col {
			flex-direction: column;
		}
		.actions-col {
			width: 100%;
		}
	}
</style>
