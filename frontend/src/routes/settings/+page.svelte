<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Category, Tag, Person } from '$lib/types';

	// ── Data ──
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let people = $state<Person[]>([]);

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

	async function load() {
		[categories, tags, people] = await Promise.all([
			api.get<Category[]>('/categories'),
			api.get<Tag[]>('/tags'),
			api.get<Person[]>('/people')
		]);
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
		} catch (e: any) {
			alert('删除失败：' + e.message);
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
		} catch (e: any) {
			alert('删除失败：' + e.message);
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
		} catch (e: any) {
			alert('删除失败：' + e.message);
		}
	}

	$effect(() => { load(); });
</script>

<h1>设置</h1>

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
</style>
