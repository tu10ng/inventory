export interface Category {
	id: number;
	name: string;
	icon: string;
	sort_order: number;
}

export interface Item {
	id: number;
	name: string;
	brand: string;
	model: string;
	category_id: number;
	default_qty: number;
	notes: string;
}

export interface Activity {
	id: number;
	name: string;
	description: string;
	icon: string;
}

export interface ActivityItem {
	id: number;
	activity_id: number;
	item_id: number;
	is_essential: boolean;
	default_qty: number;
	notes: string;
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
	status: 'planning' | 'packing' | 'done';
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
}

export type ItemStatus = '' | 'need_buy' | 'need_find' | 'need_charge' | 'need_fetch' | 'need_give';

// Extended types for new features

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

export interface DndItem {
	id: string;
	item_id: number;
	name: string;
	brand: string;
	model: string;
	category_id: number;
	category_icon: string;
	default_qty: number;
	already_added: boolean;
}
