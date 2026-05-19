<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Type, Person, DisplayRule, AttributeDefinition, RelationType, LlmConfig } from '$lib/types';

	// ── Data ──
	let types = $state<Type[]>([]);
	let people = $state<Person[]>([]);
	let displayRules = $state<DisplayRule[]>([]);
	let relationTypes = $state<RelationType[]>([]);
	let attrDefs = $state<AttributeDefinition[]>([]);
	let allColumns = $state<{ key: string; label: string }[]>([]);
	let llmConfigs = $state<LlmConfig[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// ── Type form ──
	let showTypeForm = $state(false);
	let editingTypeId = $state<number | null>(null);
	let typeForm = $state({ name: '', parent_id: null as number | null, sort_order: 0 });

	// ── Person form ──
	let showPersonForm = $state(false);
	let editingPersonId = $state<number | null>(null);
	let personForm = $state({ name: '' });

	// ── Display Rule form ──
	let showRuleForm = $state(false);
	let editingRuleId = $state<number | null>(null);
	let ruleForm = $state({
		name: '',
		group_by_key: '',
		sort_by_key: '',
		sort_dir: 'asc',
		visible_columns: '[]' as string,
		sort_order: 0,
		config: '{}'
	});
	let ruleVisibleCols = $state<string[]>([]);

	// ── LLM Config form ──
	let editingLlmId = $state<number | null>(null);
	let llmForm = $state({ provider_name: '', base_url: '', api_key: '', model: '', is_active: true });
	let llmFormTask = $state('');

	// ── Relation Type form ──
	let showRtForm = $state(false);
	let editingRtId = $state<number | null>(null);
	let rtForm = $state({ name: '', label: '', color: '', icon: '', bidirectional: false, sort_order: 0 });

	function toggleRuleCol(key: string) {
		if (ruleVisibleCols.includes(key)) {
			ruleVisibleCols = ruleVisibleCols.filter(k => k !== key);
		} else {
			ruleVisibleCols = [...ruleVisibleCols, key];
		}
	}

	async function load() {
		try {
			loading = true;
			error = null;
			[types, people, displayRules, relationTypes, attrDefs, llmConfigs] = await Promise.all([
				api.get<Type[]>('/types'),
				api.get<Person[]>('/people'),
				api.get<DisplayRule[]>('/display-rules'),
				api.get<RelationType[]>('/relation-types'),
				api.get<AttributeDefinition[]>('/attribute-definitions'),
				api.getLlmConfigs<LlmConfig[]>()
			]);
			allColumns = [{ key: 'type', label: '类型' }, ...attrDefs.map(a => ({ key: a.key, label: a.label }))];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : '加载失败';
		} finally {
			loading = false;
		}
	}

	// ── Helpers ──
	function taskLabel(task: string): string {
		switch (task) {
			case 'parse': return '物品解析';
			case 'organize': return 'AI 整理';
			case 'ocr': return '图片识别';
			default: return task;
		}
	}

	// ── Types ──

	const rootTypes = $derived(types.filter(t => t.parent_id === null).sort((a, b) => a.sort_order - b.sort_order));

	// Depth helper for tree display
	function getTypeDepth(typeId: number): number {
		let depth = 0;
		let pid: number | null = types.find(t => t.id === typeId)?.parent_id ?? null;
		while (pid != null) {
			depth++;
			pid = types.find(t => t.id === pid)?.parent_id ?? null;
		}
		return depth;
	}

	function resetTypeForm() {
		typeForm = { name: '', parent_id: null, sort_order: 0 };
		editingTypeId = null;
		showTypeForm = false;
	}

	function startEditType(t: Type) {
		typeForm = { name: t.name, parent_id: t.parent_id, sort_order: t.sort_order };
		editingTypeId = t.id;
		showTypeForm = true;
	}

	async function saveType() {
		if (editingTypeId) {
			await api.put(`/types/${editingTypeId}`, typeForm);
		} else {
			await api.post('/types', typeForm);
		}
		resetTypeForm();
		await load();
	}

	async function removeType(id: number) {
		const children = types.filter(t => t.parent_id === id);
		if (children.length > 0) {
			alert(`无法删除：此类型下有 ${children.length} 个子类型（${children.map(c => c.name).join('、')}），请先删除子类型。`);
			return;
		}
		if (!confirm('删除类型？如果有物品引用此类型，删除会失败。')) return;
		try {
			await api.del(`/types/${id}`);
			await load();
		} catch (e: unknown) {
			alert('删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	// ── People ──

	function resetPersonForm() {
		personForm = { name: '' };
		editingPersonId = null;
		showPersonForm = false;
	}

	function startEditPerson(p: Person) {
		personForm = { name: p.name };
		editingPersonId = p.id;
		showPersonForm = true;
	}

	async function savePerson() {
		if (editingPersonId) {
			await api.put(`/people/${editingPersonId}`, personForm);
		} else {
			await api.post('/people', personForm);
		}
		resetPersonForm();
		await load();
	}

	async function removePerson(id: number) {
		if (!confirm('删除人员？')) return;
		try {
			await api.del(`/people/${id}`);
			await load();
		} catch (e: unknown) {
			alert('删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	// ── Display Rules ──

	function resetRuleForm() {
		ruleForm = { name: '', group_by_key: '', sort_by_key: '', sort_dir: 'asc', visible_columns: '[]', sort_order: 0, config: '{}' };
		ruleVisibleCols = [];
		editingRuleId = null;
		showRuleForm = false;
	}

	function startEditRule(r: DisplayRule) {
		ruleForm = { name: r.name, group_by_key: r.group_by_key, sort_by_key: r.sort_by_key, sort_dir: r.sort_dir, visible_columns: r.visible_columns, sort_order: r.sort_order, config: r.config };
		try { ruleVisibleCols = JSON.parse(r.visible_columns); } catch { ruleVisibleCols = []; }
		editingRuleId = r.id;
		showRuleForm = true;
	}

	async function saveRule() {
		const payload = { ...ruleForm, visible_columns: JSON.stringify(ruleVisibleCols) };
		if (editingRuleId) {
			await api.put(`/display-rules/${editingRuleId}`, payload);
		} else {
			await api.post('/display-rules', payload);
		}
		resetRuleForm();
		await load();
	}

	async function removeRule(id: number) {
		if (!confirm('删除展示规则？')) return;
		try {
			await api.del(`/display-rules/${id}`);
			await load();
		} catch (e: unknown) {
			alert('删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	// ── LLM Configs ──

	function startEditLlm(cfg: LlmConfig) {
		llmForm = {
			provider_name: cfg.provider_name,
			base_url: cfg.base_url,
			api_key: '',  // Must re-enter key
			model: cfg.model,
			is_active: cfg.is_active
		};
		llmFormTask = cfg.task;
		editingLlmId = cfg.id;
	}

	function cancelEditLlm() {
		editingLlmId = null;
		llmFormTask = '';
		llmForm = { provider_name: '', base_url: '', api_key: '', model: '', is_active: true };
	}

	async function saveLlm() {
		if (!editingLlmId) return;
		await api.updateLlmConfig(editingLlmId, llmForm);
		cancelEditLlm();
		await load();
	}

	// ── Relation Types ──

	function resetRtForm() {
		rtForm = { name: '', label: '', color: '', icon: '', bidirectional: false, sort_order: relationTypes.length };
		editingRtId = null;
		showRtForm = false;
	}

	function startEditRt(rt: RelationType) {
		rtForm = { name: rt.name, label: rt.label, color: rt.color, icon: rt.icon, bidirectional: rt.bidirectional, sort_order: rt.sort_order };
		editingRtId = rt.id;
		showRtForm = true;
	}

	async function saveRt() {
		if (editingRtId) {
			await api.put(`/relation-types/${editingRtId}`, rtForm);
		} else {
			await api.post('/relation-types', rtForm);
		}
		resetRtForm();
		await load();
	}

	async function removeRt(id: number) {
		if (!confirm('删除关系类型？')) return;
		try {
			await api.del(`/relation-types/${id}`);
			await load();
		} catch (e: unknown) {
			alert('删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	$effect(() => { load(); });
</script>

<h1>设置</h1>

{#if loading}
	<div class="spinner">加载中...</div>
{:else if error}
	<div class="error-banner">{error}</div>
	<button class="primary" onclick={load}>重试</button>
{:else}

<!-- ── Section: Types ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>类型</h2>
		<button class="primary small" onclick={() => { if (showTypeForm && !editingTypeId) resetTypeForm(); else { resetTypeForm(); showTypeForm = true; } }}>
			{showTypeForm && !editingTypeId ? '取消' : '+ 新建类型'}
		</button>
	</div>

	{#if showTypeForm}
		<div class="card inline-form">
			<input bind:value={typeForm.name} placeholder="类型名称" style="flex: 1;" />
			<select bind:value={typeForm.parent_id}>
				<option value={null}>无（顶级类型）</option>
				{#each types.filter(t => t.id !== editingTypeId) as pt (pt.id)}
					<option value={pt.id}>{'--'.repeat(getTypeDepth(pt.id))}{pt.name}</option>
				{/each}
			</select>
			<input type="number" bind:value={typeForm.sort_order} placeholder="排序" style="width: 70px;" title="排序" />
			<button class="primary" onclick={saveType} disabled={!typeForm.name}>
				{editingTypeId ? '更新' : '创建'}
			</button>
			{#if editingTypeId}
				<button onclick={resetTypeForm}>取消</button>
			{/if}
		</div>
	{/if}

	{#if types.length === 0}
		<div class="card empty">还没有类型</div>
	{:else}
		<!-- Show types by root type -->
		{#each rootTypes as root (root.id)}
			<div class="type-group">
				<div class="type-group-header">{root.name}</div>
				<div class="type-group-items">
					<div class="type-item" style="padding-left: 24px">
						<span class="type-name">{root.name}</span>
						<div class="list-actions">
							<button class="small" onclick={() => startEditType(root)}>编辑</button>
							<button class="small danger" onclick={() => removeType(root.id)}>删除</button>
						</div>
					</div>
					{#each types.filter(t => t.parent_id === root.id) as child (child.id)}
						<div class="type-item" style="padding-left: {24 + getTypeDepth(child.id) * 16}px">
							<span class="type-name">{'--'.repeat(getTypeDepth(child.id))}{child.name}</span>
							<div class="list-actions">
								<button class="small" onclick={() => startEditType(child)}>编辑</button>
								<button class="small danger" onclick={() => removeType(child.id)}>删除</button>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/each}
	{/if}
</section>

<!-- ── Section: LLM Configs ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>AI 模型配置</h2>
	</div>

	<div class="section-desc">
		为不同任务配置不同的 AI 模型。API Key 仅显示最后 4 位，保存时不填表示保持不变。
	</div>

	<div class="llm-table">
		<div class="llm-table-header">
			<span class="col-task">任务</span>
			<span class="col-provider">提供商</span>
			<span class="col-model">模型</span>
			<span class="col-status">状态</span>
			<span class="col-actions"></span>
		</div>
		{#each llmConfigs as cfg (cfg.id)}
			<div class="llm-row">
				{#if editingLlmId === cfg.id}
					<div class="llm-edit-form">
						<div class="llm-edit-row">
							<label for="llm-task">任务</label>
							<span id="llm-task" class="llm-task-label">{taskLabel(cfg.task)}</span>
						</div>
						<div class="llm-edit-row">
							<label for="llm-provider">提供商</label>
							<input id="llm-provider" bind:value={llmForm.provider_name} placeholder="如 DeepSeek" />
						</div>
						<div class="llm-edit-row">
							<label for="llm-url">API 地址</label>
							<input id="llm-url" bind:value={llmForm.base_url} placeholder="https://api.deepseek.com/v1" />
						</div>
						<div class="llm-edit-row">
							<label for="llm-key">API Key</label>
							<input id="llm-key" type="password" bind:value={llmForm.api_key} placeholder="留空表示保持不变" />
						</div>
						<div class="llm-edit-row">
							<label for="llm-model">模型</label>
							<input id="llm-model" bind:value={llmForm.model} placeholder="deepseek-chat" />
						</div>
						<div class="llm-edit-row">
							<span class="checkbox-label">
								<input id="llm-active" type="checkbox" bind:checked={llmForm.is_active} />
								<label for="llm-active">启用</label>
							</span>
						</div>
						<div class="llm-edit-actions">
							<button class="primary small" onclick={saveLlm}>保存</button>
							<button class="small" onclick={cancelEditLlm}>取消</button>
						</div>
					</div>
				{:else}
					<span class="col-task">{taskLabel(cfg.task)}</span>
					<span class="col-provider">{cfg.provider_name}</span>
					<span class="col-model">{cfg.model}</span>
					<span class="col-status">
						{#if cfg.is_active}
							<span class="badge active">启用</span>
						{:else}
							<span class="badge inactive">停用</span>
						{/if}
					</span>
					<span class="col-actions">
						<button class="small" onclick={() => startEditLlm(cfg)}>编辑</button>
					</span>
				{/if}
			</div>
		{/each}
	</div>
</section>

<!-- ── Section: People ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>人员</h2>
		<button class="primary small" onclick={() => { if (showPersonForm && !editingPersonId) resetPersonForm(); else { resetPersonForm(); showPersonForm = true; } }}>
			{showPersonForm && !editingPersonId ? '取消' : '+ 新建人员'}
		</button>
	</div>

	{#if showPersonForm}
		<div class="card inline-form">
			<input bind:value={personForm.name} placeholder="姓名" style="flex: 1;" />
			<button class="primary" onclick={savePerson} disabled={!personForm.name}>
				{editingPersonId ? '更新' : '创建'}
			</button>
			{#if editingPersonId}
				<button onclick={resetPersonForm}>取消</button>
			{/if}
		</div>
	{/if}

	{#if people.length === 0}
		<div class="card empty">还没有人员</div>
	{:else}
		<div class="list">
			{#each people as p (p.id)}
				<div class="list-item">
					<span class="list-name">{p.name}</span>
					<div class="list-actions">
						<button class="small" onclick={() => startEditPerson(p)}>编辑</button>
						<button class="small danger" onclick={() => removePerson(p.id)}>删除</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</section>

<!-- ── Section: Relation Types ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>关系类型</h2>
		<button class="primary small" onclick={() => { if (showRtForm && !editingRtId) resetRtForm(); else { resetRtForm(); showRtForm = true; } }}>
			{showRtForm && !editingRtId ? '取消' : '+ 新建关系类型'}
		</button>
	</div>

	{#if showRtForm}
		<div class="card inline-form">
			<input bind:value={rtForm.name} placeholder="名称 (英文key)" style="width: 130px;" />
			<input bind:value={rtForm.label} placeholder="显示名" style="width: 100px;" />
			<input bind:value={rtForm.icon} placeholder="图标" style="width: 60px;" />
			<input bind:value={rtForm.color} placeholder="颜色" style="width: 80px;" />
			<label class="checkbox-label">
				<input type="checkbox" bind:checked={rtForm.bidirectional} />
				双向
			</label>
			<input type="number" bind:value={rtForm.sort_order} placeholder="排序" style="width: 70px;" title="排序" />
			<button class="primary" onclick={saveRt} disabled={!rtForm.name || !rtForm.label}>
				{editingRtId ? '更新' : '创建'}
			</button>
			{#if editingRtId}
				<button onclick={resetRtForm}>取消</button>
			{/if}
		</div>
	{/if}

	{#if relationTypes.length === 0}
		<div class="card empty">还没有关系类型，点击上方按钮创建</div>
	{:else}
		<div class="list">
			{#each relationTypes as rt (rt.id)}
				<div class="list-item">
					<span class="list-icon" style="color: {rt.color || 'var(--text)'};">{rt.icon || '🔗'}</span>
					<span class="list-name">{rt.label}</span>
					<span class="list-meta">{rt.name}{#if rt.bidirectional} · 双向{/if}</span>
					<div class="list-actions">
						<button class="small" onclick={() => startEditRt(rt)}>编辑</button>
						<button class="small danger" onclick={() => removeRt(rt.id)}>删除</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</section>

<!-- ── Section: Display Rules ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>展示规则</h2>
		<button class="primary small" onclick={() => { if (showRuleForm && !editingRuleId) resetRuleForm(); else { resetRuleForm(); showRuleForm = true; } }}>
			{showRuleForm && !editingRuleId ? '取消' : '+ 新建规则'}
		</button>
	</div>

	<div class="section-desc">
		展示规则用于一键切换物品库的分组、排序和列显示。
	</div>

	{#if showRuleForm}
		<div class="card rule-form">
			<div class="rule-form-row">
				<label>名称</label>
				<input bind:value={ruleForm.name} placeholder="规则名称" style="flex: 1;" />
			</div>
			<div class="rule-form-row">
				<label>分组依据</label>
				<select bind:value={ruleForm.group_by_key}>
					<option value="">无</option>
					{#each allColumns.filter(c => c.key !== 'type') as col}
						<option value={col.key}>{col.label}</option>
					{/each}
				</select>
			</div>
			<div class="rule-form-row">
				<label>排序字段</label>
				<select bind:value={ruleForm.sort_by_key}>
					<option value="">无</option>
					{#each allColumns as col}
						<option value={col.key}>{col.label}</option>
					{/each}
				</select>
				<label style="margin-left: 8px">方向</label>
				<select bind:value={ruleForm.sort_dir}>
					<option value="asc">升序</option>
					<option value="desc">降序</option>
				</select>
			</div>
			<div class="rule-form-row">
				<label>排序权重</label>
				<input type="number" bind:value={ruleForm.sort_order} style="width: 80px;" />
			</div>
			<div class="rule-form-row">
				<label>可见列</label>
				<div class="rule-col-checkboxes">
					{#each allColumns as col}
						<label class="col-checkbox">
							<input type="checkbox" checked={ruleVisibleCols.includes(col.key)} onchange={() => toggleRuleCol(col.key)} />
							{col.label}
						</label>
					{/each}
				</div>
			</div>
			<div class="rule-form-actions">
				<button class="primary" onclick={saveRule} disabled={!ruleForm.name}>
					{editingRuleId ? '更新' : '创建'}
				</button>
				{#if editingRuleId}
					<button onclick={resetRuleForm}>取消</button>
				{/if}
			</div>
		</div>
	{/if}

	{#if displayRules.length === 0}
		<div class="card empty">还没有展示规则</div>
	{:else}
		<div class="list">
			{#each displayRules as r (r.id)}
				<div class="list-item">
					<span class="list-name">{r.name}</span>
					<span class="list-meta">
						{#if r.group_by_key}
							分组: {allColumns.find(c => c.key === r.group_by_key)?.label ?? r.group_by_key}
						{/if}
					</span>
					<div class="list-actions">
						<button class="small" onclick={() => startEditRule(r)}>编辑</button>
						<button class="small danger" onclick={() => removeRule(r.id)}>删除</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</section>
{/if}

<style>
	.settings-section {
		margin-bottom: 32px;
	}
	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}
	.inline-form {
		display: flex;
		gap: 8px;
		align-items: center;
		margin-bottom: 12px;
	}
	.empty {
		text-align: center;
		color: var(--text-secondary);
		padding: 24px;
	}
	.list {
		display: flex;
		flex-direction: column;
	}
	.list-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 14px;
		border-bottom: 1px solid var(--border);
	}
	.list-item:last-child {
		border-bottom: none;
	}
	.list-icon {
		font-size: 18px;
		width: 28px;
		text-align: center;
	}
	.list-name {
		flex: 1;
		font-weight: 500;
	}
	.list-meta {
		color: var(--text-secondary);
		font-size: 13px;
	}
	.list-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}
	.type-group {
		margin-bottom: 12px;
	}
	.type-group-header {
		font-weight: 600;
		font-size: 14px;
		color: var(--text-secondary);
		padding: 6px 14px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
	}
	.type-group-items {
		display: flex;
		flex-direction: column;
	}
	.type-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 14px;
		border-bottom: 1px solid var(--border);
	}
	.type-item:last-child {
		border-bottom: none;
	}
	.type-name {
		flex: 1;
	}
	.spinner {
		text-align: center;
		padding: 40px;
		color: var(--text-secondary);
	}
	.error-banner {
		text-align: center;
		padding: 16px;
		background: #fdf0f0;
		border: 1px solid var(--danger);
		border-radius: 8px;
		color: var(--danger);
		margin-bottom: 16px;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
		cursor: pointer;
	}
	.checkbox-label input[type='checkbox'] {
		width: 14px;
		height: 14px;
		accent-color: var(--primary);
	}

	.section-desc {
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 12px;
		line-height: 1.5;
	}

	.rule-form {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-bottom: 12px;
	}

	.rule-form-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.rule-form-row label {
		font-size: 13px;
		color: var(--text-secondary);
		min-width: 60px;
		flex-shrink: 0;
	}

	.rule-form-row select {
		font-size: 13px;
		padding: 2px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}

	.rule-form-row input {
		font-size: 13px;
		padding: 2px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}

	.rule-col-checkboxes {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.col-checkbox {
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 12px;
		color: var(--text);
		cursor: pointer;
	}

	.rule-form-actions {
		display: flex;
		gap: 8px;
		margin-top: 4px;
	}

	/* LLM Config table */
	.llm-table {
		margin-bottom: 12px;
	}
	.llm-table-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 14px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
	}
	.llm-row {
		border-bottom: 1px solid var(--border);
	}
	.llm-row:last-child {
		border-bottom: none;
	}
	.col-task { width: 90px; flex-shrink: 0; }
	.col-provider { flex: 1; }
	.col-model { flex: 1.5; }
	.col-status { width: 60px; flex-shrink: 0; text-align: center; }
	.col-actions { width: 60px; flex-shrink: 0; text-align: right; }

	.llm-row:not(:has(.llm-edit-form)) {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 14px;
	}

	.llm-edit-form {
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.llm-edit-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.llm-edit-row label {
		font-size: 13px;
		color: var(--text-secondary);
		min-width: 60px;
		flex-shrink: 0;
	}
	.llm-edit-row input {
		flex: 1;
		font-size: 13px;
		padding: 4px 8px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}
	.llm-task-label {
		font-size: 13px;
		color: var(--text);
	}
	.llm-edit-actions {
		display: flex;
		gap: 8px;
		margin-top: 4px;
	}

	.badge.active {
		background: var(--success);
		color: #fff;
		padding: 1px 8px;
		border-radius: 10px;
		font-size: 11px;
		white-space: nowrap;
	}
	.badge.inactive {
		background: var(--text-secondary);
		color: #fff;
		padding: 1px 8px;
		border-radius: 10px;
		font-size: 11px;
		white-space: nowrap;
	}
</style>
