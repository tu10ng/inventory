export interface Category {
	id: number;
	name: string;
	icon: string;
	sort_order: number;
}

export interface Tag {
	id: number;
	name: string;
	category_id: number;
	sort_order: number;
}

export interface Item {
	id: number;
	category_id: number;
	tag_id: number | null;
	attrs: Record<string, unknown>;
}

// Helper: get a string attr from Item
export function itemName(item: Item): string {
	return String(item.attrs?.name ?? '');
}
export function itemBrand(item: Item): string {
	return String(item.attrs?.brand ?? '');
}
export function itemModel(item: Item): string {
	return String(item.attrs?.model ?? '');
}

export interface AttributeDefinition {
	id: number;
	key: string;
	label: string;
	attr_type: string;
	config: string;
	category_scope: string;
	tag_scope: string;
	sort_order: number;
}

export interface Activity {
	id: number;
	name: string;
	description: string;
	icon: string;
}

export interface ActivitySlot {
	id: number;
	activity_id: number;
	slot_name: string;
	category_id: number;
	is_essential: boolean;
	default_qty: number;
	notes: string;
	sort_order: number;
	default_item_id: number | null;
}

export interface ActivitySlotWithTags extends ActivitySlot {
	tags: Tag[];
}

export interface Tip {
	id: number;
	activity_id: number;
	content: string;
	sort_order: number;
}

export interface Person {
	id: number;
	name: string;
}

export interface Trip {
	id: number;
	name: string;
	activity_id: number | null;
	start_date: string;
	end_date: string;
	notes: string;
	status: string;
}

export interface TripItem {
	id: number;
	trip_id: number;
	item_id: number | null;
	custom_name: string;
	person_id: number | null;
	qty: number;
	checked: boolean;
	item_status: ItemStatus;
	notes: string;
	sort_order: number;
	is_essential: boolean;
	slot_id: number | null;
}

export type ItemStatus = string;

export interface StatusDefinition {
	id: number;
	scope: string;
	value: string;
	label: string;
	color: string;
	icon: string;
	sort_order: number;
}

export interface SlotInfo {
	id: number;
	slot_name: string;
	category_id: number;
	is_essential: boolean;
}

export interface TripItemEnriched extends TripItem {
	slot: SlotInfo | null;
	candidates: Item[];
}

export interface TripItemWithInfo extends TripItem {
	item_info?: Item | null;
	category?: Category | null;
}

export interface ItemUsageCount {
	item_id: number;
	trip_count: number;
}

export interface ItemUsageStats {
	item_id: number;
	trips: TripRef[];
}

export interface TripRef {
	id: number;
	name: string;
	status: string;
}

export interface BulkUpdateTripItems {
	ids: number[];
	checked?: boolean;
	person_id?: number | null;
	item_status?: ItemStatus;
}

export interface ResyncPreviewItem {
	trip_item_id: number | null;
	slot_name: string | null;
	item_name: string | null;
	custom_name: string | null;
	reason: string;
}

export interface ResyncPreview {
	items_to_remove: ResyncPreviewItem[];
	items_to_add: ResyncPreviewItem[];
}

export interface DndItem {
	id: string;
	item_id: number;
	name: string;
	category_id: number;
	category_icon: string;
	already_added: boolean;
	attrs: Record<string, unknown>;
}

export interface AiParsedItem {
	category_name: string | null;
	tag_name: string | null;
	category_id: number | null;
	tag_id: number | null;
	attrs: Record<string, unknown>;
}

export interface AiParseResponse {
	items: AiParsedItem[];
	new_tags: Tag[];
}

// ── AI Organize ──

export interface OrganizeUpdateFields {
	category_name?: string;
	tag_name?: string;
	category_id?: number;
	tag_id?: number | null;
	attrs?: Record<string, unknown>;
}

export type OrganizeAction =
	| { action_type: 'update'; item_id: number; reason: string; fields: OrganizeUpdateFields }
	| { action_type: 'split'; item_id: number; reason: string; new_items: AiParsedItem[] }
	| { action_type: 'delete'; item_id: number; reason: string };

export interface OrganizePreviewResponse {
	actions: OrganizeAction[];
	new_tags: Tag[];
}

export interface OrganizeApplyResponse {
	updated: number;
	created: number;
	deleted: number;
}

// ── Import / Export ──

export interface ExportData {
	version: number;
	exported_at: string;
	categories: Category[];
	tags: Tag[];
	attribute_definitions: AttributeDefinition[];
	items: Item[];
}

export interface ImportPreviewResult {
	total_items: number;
	new_items: number;
	skip_or_update_items: number;
	preview_items: ImportItemPreview[];
}

export interface ImportItemPreview {
	name: string;
	action: 'new' | 'skip' | 'update';
	existing_id: number | null;
}

export interface ImportResult {
	categories_created: number;
	tags_created: number;
	attribute_definitions_created: number;
	items_created: number;
	items_updated: number;
	items_skipped: number;
}
