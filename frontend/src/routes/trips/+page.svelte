<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Trip, Activity } from '$lib/types';
	import { TRIP_STATUS_LABELS } from '$lib/utils/status';

	let trips = $state<Trip[]>([]);
	let activities = $state<Activity[]>([]);
	let showForm = $state(false);
	let form = $state({ name: '', activity_id: null as number | null, start_date: '', end_date: '', notes: '' });

	async function load() {
		[trips, activities] = await Promise.all([
			api.get<Trip[]>('/trips'),
			api.get<Activity[]>('/activities')
		]);
	}

	async function createTrip() {
		await api.post('/trips', form);
		form = { name: '', activity_id: null, start_date: '', end_date: '', notes: '' };
		showForm = false;
		await load();
	}

	async function remove(id: number) {
		await api.del(`/trips/${id}`);
		await load();
	}

	function activityName(id: number | null) {
		if (!id) return '';
		const a = activities.find((a) => a.id === id);
		return a ? `${a.icon} ${a.name}` : '';
	}

	$effect(() => { load(); });
</script>

<div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
	<h1>行程列表</h1>
	<button class="primary" onclick={() => (showForm = !showForm)}>
		{showForm ? '取消' : '+ 新建行程'}
	</button>
</div>

{#if showForm}
	<div class="card">
		<div style="display: flex; flex-direction: column; gap: 10px;">
			<input bind:value={form.name} placeholder="行程名称" />
			<div style="display: flex; gap: 10px;">
				<select bind:value={form.activity_id} style="flex: 1;">
					<option value={null}>选择活动模板（可选）</option>
					{#each activities as a}
						<option value={a.id}>{a.icon} {a.name}</option>
					{/each}
				</select>
				<input type="date" bind:value={form.start_date} />
				<input type="date" bind:value={form.end_date} />
			</div>
			<textarea bind:value={form.notes} placeholder="备注" rows="2"></textarea>
			<button class="primary" onclick={createTrip} disabled={!form.name}>创建</button>
		</div>
	</div>
{/if}

{#if trips.length === 0}
	<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
		还没有行程
	</div>
{:else}
	{#each trips as trip}
		<div class="card" style="display: flex; justify-content: space-between; align-items: center;">
			<a href="/trips/{trip.id}" style="text-decoration: none; color: inherit; flex: 1;">
				<div style="display: flex; align-items: center; gap: 12px;">
					<strong>{trip.name}</strong>
					<span class="badge {trip.status}">{TRIP_STATUS_LABELS[trip.status]}</span>
					{#if trip.activity_id}
						<span style="color: var(--text-secondary); font-size: 13px;">{activityName(trip.activity_id)}</span>
					{/if}
				</div>
				{#if trip.start_date}
					<div style="color: var(--text-secondary); font-size: 13px; margin-top: 2px;">
						{trip.start_date}{#if trip.end_date} ~ {trip.end_date}{/if}
					</div>
				{/if}
			</a>
			<button class="small danger" onclick={() => remove(trip.id)}>删除</button>
		</div>
	{/each}
{/if}
