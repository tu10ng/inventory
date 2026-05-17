<script lang="ts">
	import { api } from '$lib/api/client';
	import type { ExcelPreviewResponse, Category, Type, AttributeDefinition, AiParsedItem } from '$lib/types';

	let {
		categories,
		types,
		attrDefs,
		onDone,
		onClose,
		onOpenAiModal
	}: {
		categories: Category[];
		types: Type[];
		attrDefs: AttributeDefinition[];
		onDone: (created: number) => void;
		onClose: () => void;
		onOpenAiModal?: (text: string) => void;
	} = $props();

	type Mode = 'ai' | 'manual';
	type Stage = 'upload' | 'parsing' | 'preview-raw' | 'manual-mapping' | 'importing' | 'done';

	let stage = $state<Stage>('upload');
	let mode = $state<Mode>('ai');
	let errorMsg = $state('');

	// Parsed Excel data
	let preview = $state<ExcelPreviewResponse | null>(null);

	// Manual mapping
	let columnMappings = $state<Map<number, string>>(new Map());
	let manualItems = $state<AiParsedItem[]>([]);
	let manualNewAttrs = $state<{ key: string; label: string; attr_type: string }[]>([]);

	// Importing
	let importProgress = $state({ current: 0, total: 0 });
	let importResult = $state({ created: 0, failed: 0 });
	let abortImport = $state(false);

	// Column mapping options
	let mappingOptions = $derived.by(() => {
		const options: { value: string; label: string }[] = [
			{ value: '', label: '— 忽略此列 —' },
			{ value: '_name', label: '→ name (名称)' },
			{ value: '_brand', label: '→ brand (品牌)' },
			{ value: '_model', label: '→ model (型号)' },
			{ value: '_notes', label: '→ notes (备注)' },
			{ value: '_default_qty', label: '→ default_qty (默认数量)' },
			{ value: '_category', label: '→ 分类 (按名称匹配)' },
			{ value: '_type', label: '→ 类型 (按名称匹配)' }
		];
		for (const ad of attrDefs) {
			options.push({ value: ad.key, label: `→ ${ad.label} (${ad.key})` });
		}
		options.push({ value: '_new', label: '+ 新建属性...' });
		return options;
	});

	// Preview rows (limited for display)
	let previewRows = $derived(preview ? preview.rows.slice(0, 20) : []);

	// Reset
	function reset() {
		preview = null;
		columnMappings = new Map();
		manualItems = [];
		manualNewAttrs = [];
		importProgress = { current: 0, total: 0 };
		importResult = { created: 0, failed: 0 };
		abortImport = false;
		errorMsg = '';
	}

	async function handleUpload(file: File) {
		stage = 'parsing';
		errorMsg = '';
		reset();

		try {
			preview = await api.uploadExcelPreview(file);
			stage = 'preview-raw';
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : '解析失败';
			stage = 'upload';
		}
	}

	function formatExcelAsText(headers: string[], rows: string[][]): string {
		const lines: string[] = [];
		for (const row of rows) {
			const parts: string[] = [];
			for (let i = 0; i < headers.length; i++) {
				if (row[i]) {
					parts.push(`${headers[i]}: ${row[i]}`);
				}
			}
			if (parts.length > 0) {
				lines.push(parts.join(' | '));
			}
		}
		return lines.join('\n');
	}

	function handleAiParse() {
		if (!preview) return;
		const text = formatExcelAsText(preview.headers, preview.rows);
		onOpenAiModal?.(text);
		onClose();
	}

	function startManualMapping() {
		if (!preview) return;
		stage = 'manual-mapping';
		// Auto-map columns by header name
		const m = new Map<number, string>();
		const headerLower = preview.headers.map(h => h.toLowerCase().trim());
		for (let i = 0; i < headerLower.length; i++) {
			const h = headerLower[i];
			if (h.includes('名称') || h.includes('商品') || h.includes('产品') || h === 'name' || h === '品名') {
				m.set(i, '_name');
			} else if (h.includes('品牌') || h.includes('牌子') || h === 'brand' || h === '商标') {
				m.set(i, '_brand');
			} else if (h.includes('型号') || h.includes('规格') || h === 'model' || h.includes('款式') || h.includes('货号')) {
				m.set(i, '_model');
			} else if (h.includes('备注') || h.includes('说明') || h === 'notes' || h === '描述') {
				m.set(i, '_notes');
			} else if (h.includes('数量') || h.includes('个数') || h === 'qty' || h === 'quantity') {
				m.set(i, '_default_qty');
			} else if (h.includes('价格') || h.includes('金额') || h === 'price' || h.includes('售价')) {
				const priceAttr = attrDefs.find(a => a.key === 'price');
				if (priceAttr) m.set(i, 'price');
				else m.set(i, '_new');
			}
		}
		columnMappings = m;
		updateManualPreview(m);
	}

	function updateManualPreview(m: Map<number, string>) {
		if (!preview) return;

		const newKeys: { key: string; label: string; attr_type: string }[] = [];
		const items: AiParsedItem[] = [];

		for (let ri = 0; ri < preview.rows.length; ri++) {
			const row = preview.rows[ri];
			const attrs: Record<string, unknown> = {};
			let categoryName: string | null = null;
			let typeName: string | null = null;

			for (let ci = 0; ci < preview.headers.length; ci++) {
				const mapping = m.get(ci);
				const val = row[ci] || '';
				if (!mapping || !val) continue;

				if (mapping === '_name') {
					attrs.name = val;
				} else if (mapping === '_brand') {
					attrs.brand = val;
				} else if (mapping === '_model') {
					attrs.model = val;
				} else if (mapping === '_notes') {
					attrs.notes = val;
				} else if (mapping === '_default_qty') {
					const num = parseInt(val, 10);
					attrs.default_qty = isNaN(num) ? 1 : num;
				} else if (mapping === '_category') {
					categoryName = val;
				} else if (mapping === '_type') {
					typeName = val;
				} else if (mapping === '_new') {
					// Skip - will need user to create new attr first
				} else {
					// Map to existing attr key
					const ad = attrDefs.find(a => a.key === mapping);
					if (ad) {
						if (ad.attr_type === 'number' || ad.attr_type === 'weight') {
							const num = parseFloat(val.replace(/[￥¥元克gGkK\s]/g, ''));
							if (!isNaN(num)) attrs[mapping] = num;
							else attrs[mapping] = val;
						} else {
							attrs[mapping] = val;
						}
					}
				}
			}

			// Only include rows that have at least a name
			if (!attrs.name) continue;

			// Try to match category and type
			let categoryId: number | null = null;
			let typeId: number | null = null;

			if (categoryName) {
				const cat = categories.find(c => c.name === categoryName || c.name.includes(categoryName));
				if (cat) categoryId = cat.id;
				else categoryId = categories.find(c => c.name === '其他')?.id || categories[0]?.id || null;
			} else {
				categoryId = categories.find(c => c.name === '其他')?.id || categories[0]?.id || null;
			}

			if (typeName && categoryId) {
				const type = types.find(t => t.name === typeName && t.category_id === categoryId);
				if (type) typeId = type.id;
			}

			items.push({
				category_name: categoryName,
				type_name: typeName,
				category_id: categoryId,
				type_id: typeId,
				attrs
			});
		}

		manualItems = items;
		manualNewAttrs = newKeys;
	}

	function handleMappingChange(colIdx: number, value: string) {
		const m = new Map(columnMappings);
		if (value === '') {
			m.delete(colIdx);
		} else {
			m.set(colIdx, value);
		}
		columnMappings = m;
		// If _new is selected, prompt for key/label
		if (value === '_new') {
			const key = prompt('请输入新属性的 key (英文 snake_case):');
			if (key) {
				const label = prompt('请输入新属性的中文类型:');
				if (label) {
					const newKey = key.trim().toLowerCase().replace(/\s+/g, '_');
					m.set(colIdx, newKey);
					columnMappings = m;
					manualNewAttrs = [...manualNewAttrs, { key: newKey, label, attr_type: 'text' }];
				} else {
					m.delete(colIdx);
					columnMappings = m;
				}
			} else {
				m.delete(colIdx);
				columnMappings = m;
			}
		}
		updateManualPreview(m);
	}

	async function handleImport(items: AiParsedItem[]) {
		stage = 'importing';
		importProgress = { current: 0, total: items.length };
		importResult = { created: 0, failed: 0 };
		abortImport = false;

		for (let i = 0; i < items.length; i++) {
			if (abortImport) break;

			const item = items[i];
			try {
				const categoryId = item.category_id || categories.find(c => c.name === '其他')?.id || 1;
				const attrs = { ...item.attrs } as Record<string, unknown>;
				const body: Record<string, unknown> = {
					category_id: categoryId,
					type_id: item.type_id || null,
					attrs
				};
				// Ensure name exists
				if (!attrs.name) {
					attrs.name = '未命名物品';
				}
				await api.post('/items', body);
				importResult = { ...importResult, created: importResult.created + 1 };
			} catch {
				importResult = { ...importResult, failed: importResult.failed + 1 };
			}
			importProgress = { ...importProgress, current: i + 1 };
		}

		stage = 'done';
	}

	function close() {
		onClose();
	}

	function done() {
		onDone(importResult.created);
		onClose();
	}

	function getCategoryName(item: AiParsedItem): string {
		if (item.category_name) return item.category_name;
		if (item.category_id) {
			const cat = categories.find(c => c.id === item.category_id);
			return cat ? cat.name : '未知';
		}
		return '—';
	}

	function getTypeName(item: AiParsedItem): string {
		if (item.type_name) return item.type_name;
		if (item.type_id) {
			const type = types.find(t => t.id === item.type_id);
			return type ? type.name : '—';
		}
		return '—';
	}
</script>

{#if stage !== 'done'}
<button class="backdrop" onclick={close} aria-label="关闭"></button>
{/if}

<div class="modal excel-modal">
	<header>
		<h2>
			{#if stage === 'upload'}
				Excel 导入
			{:else if stage === 'parsing'}
				正在解析 Excel...
			{:else if stage === 'preview-raw'}
				Excel 预览 — {preview?.file_name}
			{:else if stage === 'manual-mapping'}
				手动列映射 — {manualItems.length} 个物品
			{:else if stage === 'importing'}
				正在导入... {importProgress.current}/{importProgress.total}
			{:else if stage === 'done'}
				导入完成
			{/if}
		</h2>
		{#if stage !== 'importing' && stage !== 'done'}
		<button class="close-btn" onclick={close}>✕</button>
		{/if}
	</header>

	<div class="modal-body">
		<!-- Stage: upload -->
		{#if stage === 'upload'}
			<div class="upload-area">
				<div class="mode-tabs">
					<button class="mode-tab" class:active={mode === 'ai'} onclick={() => mode = 'ai'}>
						🤖 AI 智能导入
					</button>
					<button class="mode-tab" class:active={mode === 'manual'} onclick={() => mode = 'manual'}>
						📋 手动映射列
					</button>
				</div>
				<p class="mode-desc">
					{#if mode === 'ai'}
						AI 将自动识别列含义、提取物品信息、清洗数据格式。适合列名不规范、需要批量处理的情况。
					{:else}
						你自己指定每一列对应什么属性。适合列名已规范、只需快速导入的情况。
					{/if}
				</p>
				<label class="file-drop">
					<input
						type="file"
						accept=".xlsx,.xls"
						onchange={(e: Event) => {
							const target = e.target as HTMLInputElement;
							const file = target.files?.[0];
							if (file) handleUpload(file);
						}}
					/>
					<div class="drop-content">
						<span class="drop-icon">📁</span>
						<span>点击选择 Excel 文件</span>
						<span class="drop-hint">支持 .xlsx .xls</span>
					</div>
				</label>
			</div>
		{/if}

		<!-- Stage: parsing -->
		{#if stage === 'parsing'}
			<div class="spinner-area">
				<div class="spinner"></div>
				<span>正在读取 Excel 文件...</span>
			</div>
		{/if}

		<!-- Stage: preview-raw -->
		{#if stage === 'preview-raw' && preview}
			<div class="preview-info">
				<div class="info-row">
					<span>列数: <strong>{preview.headers.length}</strong></span>
					<span>数据行: <strong>{preview.total_rows}</strong></span>
					<span>工作表: <strong>{preview.active_sheet}</strong></span>
				</div>
			</div>

			<div class="preview-table-wrap">
				<table class="preview-table">
					<thead>
						<tr>
							<th>#</th>
							{#each preview.headers as header}
								<th>{header}</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each previewRows as row, ri}
							<tr>
								<td class="row-num">{ri + 1}</td>
								{#each row as cell}
									<td>{cell}</td>
								{/each}
							</tr>
						{/each}
						{#if preview.total_rows > 20}
							<tr>
								<td class="row-more" colspan={preview.headers.length + 1}>
									... 还有 {preview.total_rows - 20} 行未显示
								</td>
							</tr>
						{/if}
					</tbody>
				</table>
			</div>

			<div class="preview-actions">
				<button class="btn btn-secondary" onclick={close}>取消</button>
				<button class="btn btn-secondary" onclick={startManualMapping}>
					📋 手动映射列
				</button>
				<button class="btn btn-primary" onclick={handleAiParse}>
					🤖 AI 智能解析
				</button>
			</div>
		{/if}

		<!-- Stage: manual-mapping -->
		{#if stage === 'manual-mapping' && preview}
			<div class="mapping-area">
				<div class="mapping-list">
					{#each preview.headers as header, ci}
						<div class="mapping-row">
							<span class="mapping-col">{header}</span>
							<select
								value={columnMappings.get(ci) || ''}
								onchange={(e: Event) => handleMappingChange(ci, (e.target as HTMLSelectElement).value)}
							>
								{#each mappingOptions as opt}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
						</div>
					{/each}
				</div>

				{#if manualItems.length > 0}
					<div class="mapping-preview">
						<h3>预览（{manualItems.length} 个物品）</h3>
						<div class="preview-table-wrap">
							<table class="preview-table">
								<thead>
									<tr>
										<th>#</th>
										<th>名称</th>
										<th>品牌</th>
										<th>型号</th>
										<th>分类</th>
										<th>类型</th>
									</tr>
								</thead>
								<tbody>
									{#each manualItems.slice(0, 20) as item, i}
										<tr>
											<td class="row-num">{i + 1}</td>
											<td>{item.attrs.name}</td>
											<td>{item.attrs.brand ?? ''}</td>
											<td>{item.attrs.model ?? ''}</td>
											<td>{getCategoryName(item)}</td>
											<td>{getTypeName(item)}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				{/if}
			</div>

			<div class="preview-actions">
				<button class="btn btn-secondary" onclick={() => stage = 'preview-raw'}>返回</button>
				<button
					class="btn btn-primary"
					disabled={manualItems.length === 0}
					onclick={() => handleImport(manualItems)}
				>
					导入 {manualItems.length} 个物品
				</button>
			</div>
		{/if}

		<!-- Stage: importing -->
		{#if stage === 'importing'}
			<div class="importing-area">
				<div class="progress-bar-wrap">
					<div class="progress-bar" style="width: {importProgress.total > 0 ? (importProgress.current / importProgress.total * 100) : 0}%"></div>
				</div>
				<span class="progress-text">{importProgress.current} / {importProgress.total}</span>
				<button
					class="btn btn-secondary"
					style="margin-top: 12px"
					onclick={() => abortImport = true}
				>
					停止
				</button>
			</div>
		{/if}

		<!-- Stage: done -->
		{#if stage === 'done'}
			<div class="done-area">
				<div class="done-icon">✅</div>
				<div class="done-text">
					成功导入 <strong>{importResult.created}</strong> 个物品
					{#if importResult.failed > 0}
						，{importResult.failed} 个失败
					{/if}
				</div>
				<button class="btn btn-primary" onclick={done}>完成</button>
			</div>
		{/if}

		<!-- Error display -->
		{#if errorMsg}
			<div class="error-msg">{errorMsg}</div>
		{/if}
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		z-index: 1000;
		border: none;
		width: 100%;
		cursor: pointer;
	}

	.excel-modal {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		background: var(--surface);
		border-radius: 12px;
		box-shadow: 0 8px 40px rgba(0,0,0,0.3);
		z-index: 1001;
		width: min(900px, 95vw);
		max-height: 90vh;
		display: flex;
		flex-direction: column;
	}

	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border);
	}

	header h2 {
		margin: 0;
		font-size: 1.1rem;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 1.3rem;
		cursor: pointer;
		color: var(--text-secondary);
	}

	.close-btn:hover {
		color: var(--text);
	}

	.modal-body {
		padding: 20px;
		overflow-y: auto;
		flex: 1;
	}

	.upload-area {
		text-align: center;
	}

	.mode-tabs {
		display: flex;
		gap: 0;
		margin-bottom: 12px;
		border-radius: 8px;
		overflow: hidden;
		border: 1px solid var(--border);
	}

	.mode-tab {
		flex: 1;
		padding: 10px 16px;
		border: none;
		background: var(--bg);
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 0.9rem;
		transition: background 0.15s;
	}

	.mode-tab.active {
		background: var(--primary);
		color: #fff;
	}

	.mode-desc {
		color: var(--text-secondary);
		font-size: 0.85rem;
		margin-bottom: 20px;
	}

	.file-drop {
		display: block;
		border: 2px dashed var(--border);
		border-radius: 12px;
		padding: 40px;
		cursor: pointer;
		transition: border-color 0.2s;
	}

	.file-drop:hover {
		border-color: var(--primary);
	}

	.file-drop input { display: none; }

	.drop-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		color: var(--text-secondary);
	}

	.drop-icon { font-size: 2rem; }

	.drop-hint {
		font-size: 0.8rem;
		color: var(--text-secondary);
		opacity: 0.7;
	}

	.spinner-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 40px;
		color: var(--text-secondary);
	}

	.spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--border);
		border-top-color: var(--primary);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin { to { transform: rotate(360deg); } }

	.preview-info {
		margin-bottom: 12px;
	}

	.info-row {
		display: flex;
		gap: 20px;
		font-size: 0.85rem;
		color: var(--text-secondary);
	}

	.preview-table-wrap {
		overflow-x: auto;
		margin-bottom: 16px;
		max-height: 400px;
		overflow-y: auto;
		border: 1px solid var(--border);
		border-radius: 8px;
	}

	.preview-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.82rem;
	}

	.preview-table th,
	.preview-table td {
		padding: 6px 10px;
		text-align: left;
		border-bottom: 1px solid var(--border);
		white-space: nowrap;
	}

	.preview-table th {
		background: var(--bg);
		position: sticky;
		top: 0;
		font-weight: 600;
	}

	.preview-table .row-num {
		color: var(--text-secondary);
		text-align: right;
		min-width: 30px;
	}

	.preview-table .row-more {
		text-align: center;
		color: var(--text-secondary);
		padding: 10px;
		font-style: italic;
	}

	.preview-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}

	.btn {
		padding: 8px 16px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: var(--surface);
		color: var(--text);
		cursor: pointer;
		font-size: 0.85rem;
		transition: opacity 0.15s;
	}

	.btn:hover { opacity: 0.85; }

	.btn:disabled { opacity: 0.4; cursor: not-allowed; }

	.btn-primary {
		background: var(--primary);
		color: #fff;
		border-color: var(--primary);
	}

	.btn-secondary {
		background: var(--bg);
	}

	.mapping-area {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.mapping-list {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
	}

	.mapping-row {
		display: contents;
	}

	.mapping-col {
		font-size: 0.82rem;
		font-weight: 500;
		padding: 4px 0;
	}

	.mapping-row select {
		font-size: 0.82rem;
		padding: 4px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}

	.mapping-preview h3 {
		font-size: 0.9rem;
		margin: 0 0 8px;
	}

	.importing-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 30px;
	}

	.progress-bar-wrap {
		width: 100%;
		height: 8px;
		background: var(--bg);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: var(--primary);
		transition: width 0.2s;
		border-radius: 4px;
	}

	.progress-text {
		font-size: 0.9rem;
		color: var(--text-secondary);
	}

	.done-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 30px;
	}

	.done-icon { font-size: 3rem; }

	.done-text {
		font-size: 1rem;
		text-align: center;
	}

	.error-msg {
		margin-top: 12px;
		padding: 8px 12px;
		background: var(--danger);
		color: #fff;
		border-radius: 6px;
		font-size: 0.85rem;
	}
</style>
