<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Trip, Activity } from '$lib/types';
	import { TRIP_STATUS_LABELS } from '$lib/utils/status';

	let trips = $state<Trip[]>([]);
	let activities = $state<Activity[]>([]);
	let showForm = $state(false);
	let form = $state({ name: '', activity_id: null as number | null, start_date: '', end_date: '' });

	async function load() {
		[trips, activities] = await Promise.all([
			api.get<Trip[]>('/trips'),
			api.get<Activity[]>('/activities')
		]);
	}

	async function createTrip() {
		await api.post('/trips', form);
		form = { name: '', activity_id: null, start_date: '', end_date: '' };
		showForm = false;
		await load();
	}

	$effect(() => {
		load();
	});
</script>

<h1>出行物品清单</h1>
<p style="color: var(--text-secondary); margin: 8px 0 24px;">管理你的出行装备，再也不怕忘带东西</p>

<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
	<h2>最近行程</h2>
	<button class="primary" onclick={() => (showForm = !showForm)}>
		{showForm ? '取消' : '+ 新建行程'}
	</button>
</div>

{#if showForm}
	<div class="card">
		<div style="display: flex; flex-direction: column; gap: 10px;">
			<input bind:value={form.name} placeholder="行程名称，如：五一哈巴西坡" />
			<div style="display: flex; gap: 10px;">
				<select bind:value={form.activity_id}>
					<option value={null}>选择活动模板（可选）</option>
					{#each activities as a}
						<option value={a.id}>{a.icon} {a.name}</option>
					{/each}
				</select>
				<input type="date" bind:value={form.start_date} />
				<input type="date" bind:value={form.end_date} />
			</div>
			<button class="primary" onclick={createTrip} disabled={!form.name}>创建</button>
		</div>
	</div>
{/if}

{#if trips.length === 0}
	<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
		还没有行程，点击上方按钮创建第一个
	</div>
{:else}
	{#each trips as trip}
		<a href="/trips/{trip.id}" class="card trip-card" style="display: block; text-decoration: none; color: inherit;">
			<div style="display: flex; justify-content: space-between; align-items: center;">
				<strong>{trip.name}</strong>
				<span class="badge {trip.status}">{TRIP_STATUS_LABELS[trip.status]}</span>
			</div>
			{#if trip.start_date}
				<div style="color: var(--text-secondary); font-size: 14px; margin-top: 4px;">
					{trip.start_date}{#if trip.end_date} ~ {trip.end_date}{/if}
				</div>
			{/if}
		</a>
	{/each}
{/if}

<style>
	.trip-card:hover {
		border-color: var(--primary);
	}
</style>
