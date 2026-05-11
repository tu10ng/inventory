import { api } from '$lib/api/client';
import type { StatusDefinition } from '$lib/types';

let itemStatuses: StatusDefinition[] | null = null;
let tripStatuses: StatusDefinition[] | null = null;
let allStatuses: StatusDefinition[] | null = null;

async function ensureLoaded(): Promise<void> {
	if (allStatuses) return;
	allStatuses = await api.get<StatusDefinition[]>('/status-definitions');
	itemStatuses = allStatuses.filter((s) => s.scope === 'item');
	tripStatuses = allStatuses.filter((s) => s.scope === 'trip');
}

export async function getItemStatuses(): Promise<StatusDefinition[]> {
	await ensureLoaded();
	return itemStatuses!;
}

export async function getTripStatuses(): Promise<StatusDefinition[]> {
	await ensureLoaded();
	return tripStatuses!;
}

export function invalidateStatusCache(): void {
	allStatuses = null;
	itemStatuses = null;
	tripStatuses = null;
}

// Synchronous fallbacks for use after initial load
export function getItemStatusLabel(value: string): string {
	if (!itemStatuses) return value;
	const def = itemStatuses.find((s) => s.value === value);
	return def?.label ?? value;
}

export function getTripStatusLabel(value: string): string {
	if (!tripStatuses) return value;
	const def = tripStatuses.find((s) => s.value === value);
	return def?.label ?? value;
}
