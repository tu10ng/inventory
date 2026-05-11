import type { AttributeDefinition } from '$lib/types';

export function getAttrConfig(ad: AttributeDefinition): { max?: number; suffix?: string; options?: string[] } {
	try { return JSON.parse(ad.config || '{}'); } catch { return {}; }
}
