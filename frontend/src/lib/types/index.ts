export interface Category {
	id: number;
	name: string;
	icon: string;
	sort_order: number;
}

export interface Type {
	id: number;
	name: string;
	category_id: number;
	sort_order: number;
	parent_id: number | null;
}

export interface TypeTreeNode {
	id: number;
	name: string;
	category_id: number;
	sort_order: number;
	parent_id: number | null;
	children: TypeTreeNode[];
}

export interface Item {
	id: number;
	category_id: number;
	type_id: number | null;
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
	type_scope: string;
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

export interface ActivitySlotWithTypes extends ActivitySlot {
	types: Type[];
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
	type_name: string | null;
	category_id: number | null;
	type_id: number | null;
	attrs: Record<string, unknown>;
}

export interface AiParseResponse {
	items: AiParsedItem[];
	new_types: Type[];
}

// ── AI Organize ──

export interface OrganizeUpdateFields {
	category_name?: string;
	type_name?: string;
	category_id?: number;
	type_id?: number | null;
	attrs?: Record<string, unknown>;
}

export type OrganizeAction =
	| { action_type: 'update'; item_id: number; reason: string; fields: OrganizeUpdateFields }
	| { action_type: 'split'; item_id: number; reason: string; new_items: AiParsedItem[] }
	| { action_type: 'delete'; item_id: number; reason: string };

export interface OrganizePreviewResponse {
	actions: OrganizeAction[];
	new_types: Type[];
}

export interface OrganizeApplyResponse {
	updated: number;
	created: number;
	deleted: number;
}

// ── Display Rules ──

export interface DisplayRule {
	id: number;
	name: string;
	category_id: number | null;
	group_by_key: string;
	sort_by_key: string;
	sort_dir: string;
	visible_columns: string;
	sort_order: number;
	config: string;
}

// ── Relation Types ──

export interface RelationType {
	id: number;
	name: string;
	label: string;
	color: string;
	icon: string;
	bidirectional: boolean;
	sort_order: number;
}

export interface ItemRelation {
	id: number;
	source_item_id: number;
	target_item_id: number;
	relation_type_id: number;
	notes: string;
}

export interface ItemRelationEnriched {
	id: number;
	source_item_id: number;
	target_item_id: number;
	relation_type_id: number;
	notes: string;
	target_name: string;
	relation_label: string;
	relation_color: string;
	relation_icon: string;
}

export interface CreateItemRelation {
	target_item_id: number;
	relation_type_id: number;
	notes?: string;
}

// ── Activity Includes ──

export interface ActivityInclude {
	id: number;
	activity_id: number;
	included_activity_id: number;
	sort_order: number;
}

export interface CreateActivityInclude {
	included_activity_id: number;
	sort_order?: number;
}

export interface ActivityIncludeEnriched {
	id: number;
	activity_id: number;
	included_activity_id: number;
	sort_order: number;
	included_name: string;
	included_icon: string;
}

// ── Display Rule Config ──

export interface DisplayRuleConfig {
	mode: 'list' | 'summary';
	summary_fields: string[];
}

export function parseDisplayRuleConfig(config: string): DisplayRuleConfig {
	try {
		const parsed = JSON.parse(config);
		return {
			mode: parsed.mode || 'list',
			summary_fields: parsed.summary_fields || [],
		};
	} catch {
		return { mode: 'list', summary_fields: [] };
	}
}

// ── Batch Items ──

export interface BatchItemsRequest {
	ids: number[];
	action: 'delete' | 'update';
	changes?: Record<string, unknown>;
}

export interface BatchItemsResponse {
	updated: number;
	deleted: number;
}

// ── Excel Import ──

export interface ExcelPreviewResponse {
	file_name: string;
	sheet_names: string[];
	active_sheet: string;
	headers: string[];
	rows: string[][];
	total_rows: number;
}

// ── Import / Export ──

export interface ExportData {
	version: number;
	exported_at: string;
	categories: Category[];
	types: Type[];
	attribute_definitions: AttributeDefinition[];
	items: Item[];
	display_rules: DisplayRule[];
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
	types_created: number;
	attribute_definitions_created: number;
	items_created: number;
	items_updated: number;
	items_skipped: number;
	display_rules_created: number;
}
