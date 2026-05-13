<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Category, Tag, Person, DisplayRule, AttributeDefinition, RelationType } from '$lib/types';

	// ── Data ──
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let people = $state<Person[]>([]);
	let displayRules = $state<DisplayRule[]>([]);
	let relationTypes = $state<RelationType[]>([]);
	let attrDefs = $state<AttributeDefinition[]>([]);
	let allColumns = $state<{ key: string; label: string }[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// ── Category form ──
	let showCatForm = $state(false);
	let editingCatId = $state<number | null>(null);
	let catForm = $state({ name: '', icon: '', sort_order: 0 });

	// ── Tag form ──
	let showTagForm = $state(false);
	let editingTagId = $state<number | null>(null);
	let tagForm = $state({ name: '', category_id: 0, sort_order: 0 });

	// ── Person form ──
	let showPersonForm = $state(false);
	let editingPersonId = $state<number | null>(null);
	let personForm = $state({ name: '' });

	// ── Display Rule form ──
	let showRuleForm = $state(false);
	let editingRuleId = $state<number | null>(null);
	let ruleForm = $state({
		name: '',
		category_id: null as number | null,
		group_by_key: '',
		sort_by_key: '',
		sort_dir: 'asc',
		visible_columns: '[]' as string,
		sort_order: 0,
		config: '{}'
	});
	let ruleVisibleCols = $state<string[]>([]);

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
			[categories, tags, people, displayRules, relationTypes, attrDefs] = await Promise.all([
				api.get<Category[]>('/categories'),
				api.get<Tag[]>('/tags'),
				api.get<Person[]>('/people'),
				api.get<DisplayRule[]>('/display-rules'),
				api.get<RelationType[]>('/relation-types'),
				api.get<AttributeDefinition[]>('/attribute-definitions')
			]);
			allColumns = [{ key: 'tag', label: '标签' }, ...attrDefs.map(a => ({ key: a.key, label: a.label }))];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : '加载失败';
		} finally {
			loading = false;
		}
	}

	// ── Categories ──

	function resetCatForm() {
		catForm = { name: '', icon: '', sort_order: categories.length };
		editingCatId = null;
		showCatForm = false;
	}

	function startEditCat(c: Category) {
		catForm = { name: c.name, icon: c.icon, sort_order: c.sort_order };
		editingCatId = c.id;
		showCatForm = true;
	}

	async function saveCat() {
		if (editingCatId) {
			await api.put(`/categories/${editingCatId}`, catForm);
		} else {
			await api.post('/categories', catForm);
		}
		resetCatForm();
		await load();
	}

	async function removeCat(id: number) {
		if (!confirm('删除分类？如果有物品或标签引用此分类，删除会失败。')) return;
		try {
			await api.del(`/categories/${id}`);
			await load();
		} catch (e: unknown) {
			alert('删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	// ── Tags ──

	const tagsByCategory = $derived.by(() => {
		const map = new Map<number, Tag[]>();
		for (const t of tags) {
			if (!map.has(t.category_id)) map.set(t.category_id, []);
			map.get(t.category_id)!.push(t);
		}
		return map;
	});

	function resetTagForm() {
		tagForm = { name: '', category_id: categories[0]?.id ?? 0, sort_order: 0 };
		editingTagId = null;
		showTagForm = false;
	}

	function startEditTag(t: Tag) {
		tagForm = { name: t.name, category_id: t.category_id, sort_order: t.sort_order };
		editingTagId = t.id;
		showTagForm = true;
	}

	async function saveTag() {
		if (editingTagId) {
			await api.put(`/tags/${editingTagId}`, tagForm);
		} else {
			await api.post('/tags', tagForm);
		}
		resetTagForm();
		await load();
	}

	async function removeTag(id: number) {
		if (!confirm('删除标签？如果有物品引用此标签，删除会失败。')) return;
		try {
			await api.del(`/tags/${id}`);
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
		ruleForm = { name: '', category_id: null, group_by_key: '', sort_by_key: '', sort_dir: 'asc', visible_columns: '[]', sort_order: 0, config: '{}' };
		ruleVisibleCols = [];
		editingRuleId = null;
		showRuleForm = false;
	}

	function startEditRule(r: DisplayRule) {
		ruleForm = { name: r.name, category_id: r.category_id, group_by_key: r.group_by_key, sort_by_key: r.sort_by_key, sort_dir: r.sort_dir, visible_columns: r.visible_columns, sort_order: r.sort_order, config: r.config };
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

<!-- ── Section: Categories ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>物品分类</h2>
		<button class="primary small" onclick={() => { if (showCatForm && !editingCatId) resetCatForm(); else { resetCatForm(); showCatForm = true; } }}>
			{showCatForm && !editingCatId ? '取消' : '+ 新建分类'}
		</button>
	</div>

	{#if showCatForm}
		<div class="card inline-form">
			<input bind:value={catForm.icon} placeholder="图标" style="width: 60px;" />
			<input bind:value={catForm.name} placeholder="分类名称" style="flex: 1;" />
			<input type="number" bind:value={catForm.sort_order} placeholder="排序" style="width: 70px;" title="排序" />
			<button class="primary" onclick={saveCat} disabled={!catForm.name}>
				{editingCatId ? '更新' : '创建'}
			</button>
			{#if editingCatId}
				<button onclick={resetCatForm}>取消</button>
			{/if}
		</div>
	{/if}

	{#if categories.length === 0}
		<div class="card empty">还没有分类，点击上方按钮创建</div>
	{:else}
		<div class="list">
			{#each categories as c (c.id)}
				<div class="list-item">
					<span class="list-icon">{c.icon}</span>
					<span class="list-name">{c.name}</span>
					<span class="list-meta">排序: {c.sort_order}</span>
					<div class="list-actions">
						<button class="small" onclick={() => startEditCat(c)}>编辑</button>
						<button class="small danger" onclick={() => removeCat(c.id)}>删除</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</section>

<!-- ── Section: Tags ── -->
<section class="settings-section">
	<div class="section-header">
		<h2>标签</h2>
		<button class="primary small" onclick={() => { if (showTagForm && !editingTagId) resetTagForm(); else { resetTagForm(); showTagForm = true; } }} disabled={categories.length === 0}>
			{showTagForm && !editingTagId ? '取消' : '+ 新建标签'}
		</button>
	</div>

	{#if categories.length === 0}
		<div class="card empty">请先创建分类</div>
	{:else}
		{#if showTagForm}
			<div class="card inline-form">
				<input bind:value={tagForm.name} placeholder="标签名称" style="flex: 1;" />
				<select bind:value={tagForm.category_id}>
					{#each categories as c}
						<option value={c.id}>{c.icon} {c.name}</option>
					{/each}
				</select>
				<input type="number" bind:value={tagForm.sort_order} placeholder="排序" style="width: 70px;" title="排序" />
				<button class="primary" onclick={saveTag} disabled={!tagForm.name}>
					{editingTagId ? '更新' : '创建'}
				</button>
				{#if editingTagId}
					<button onclick={resetTagForm}>取消</button>
				{/if}
			</div>
		{/if}

		{#if tags.length === 0}
			<div class="card empty">还没有标签</div>
		{:else}
			{#each categories as cat (cat.id)}
				{@const catTags = tagsByCategory.get(cat.id)}
				{#if catTags && catTags.length > 0}
					<div class="tag-group">
						<div class="tag-group-header">{cat.icon} {cat.name}</div>
						<div class="tag-group-items">
							{#each catTags as t (t.id)}
								<div class="tag-item">
									<span class="tag-name">{t.name}</span>
									<div class="list-actions">
										<button class="small" onclick={() => startEditTag(t)}>编辑</button>
										<button class="small danger" onclick={() => removeTag(t.id)}>删除</button>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			{/each}
		{/if}
	{/if}
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
		展示规则用于一键切换物品库的分组、排序和列显示。例如："服装按部位"会筛选服装分类，按 body_parts 分组，按名称排序。
	</div>

	{#if showRuleForm}
		<div class="card rule-form">
			<div class="rule-form-row">
				<label>名称</label>
				<input bind:value={ruleForm.name} placeholder="规则名称" style="flex: 1;" />
			</div>
			<div class="rule-form-row">
				<label>筛选分类</label>
				<select bind:value={ruleForm.category_id}>
					<option value={null}>全部（不筛选）</option>
					{#each categories as c}
						<option value={c.id}>{c.icon} {c.name}</option>
					{/each}
				</select>
			</div>
			<div class="rule-form-row">
				<label>分组依据</label>
				<select bind:value={ruleForm.group_by_key}>
					<option value="">无</option>
					{#each allColumns.filter(c => c.key !== 'tag') as col}
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
				{@const cat = categories.find(c => c.id === r.category_id)}
				<div class="list-item">
					<span class="list-name">{r.name}</span>
					<span class="list-meta">
						分类: {cat ? cat.icon + ' ' + cat.name : '全部'}
						{#if r.group_by_key}
							| 分组: {allColumns.find(c => c.key === r.group_by_key)?.label ?? r.group_by_key}
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
	.tag-group {
		margin-bottom: 12px;
	}
	.tag-group-header {
		font-weight: 600;
		font-size: 14px;
		color: var(--text-secondary);
		padding: 6px 14px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
	}
	.tag-group-items {
		display: flex;
		flex-direction: column;
	}
	.tag-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 14px 6px 28px;
		border-bottom: 1px solid var(--border);
	}
	.tag-item:last-child {
		border-bottom: none;
	}
	.tag-name {
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
</style>
