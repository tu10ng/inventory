import { describe, it, expect } from 'vitest';
import { attrMatchesScope } from './columns';
import type { AttributeDefinition } from '$lib/types';

describe('attrMatchesScope', () => {
	const globalDef: AttributeDefinition = {
		id: 1, key: 'name', label: '名称', attr_type: 'text',
		config: '{}', category_scope: '', tag_scope: '', sort_order: 1
	};

	const catScopedDef: AttributeDefinition = {
		id: 2, key: 'warmth_rating', label: '保暖', attr_type: 'bar',
		config: '{"max":50}', category_scope: '1', tag_scope: '', sort_order: 3
	};

	const tagScopedDef: AttributeDefinition = {
		id: 3, key: 'special', label: '特殊', attr_type: 'text',
		config: '{}', category_scope: '', tag_scope: '5', sort_order: 10
	};

	const multiScopeDef: AttributeDefinition = {
		id: 4, key: 'multi', label: '多范围', attr_type: 'text',
		config: '{}', category_scope: '1,2', tag_scope: '3', sort_order: 11
	};

	it('global (empty scope) matches any', () => {
		expect(attrMatchesScope(globalDef, 1, null)).toBe(true);
		expect(attrMatchesScope(globalDef, null, null)).toBe(true);
		expect(attrMatchesScope(globalDef, 5, 10)).toBe(true);
	});

	it('category_scope filters correctly', () => {
		expect(attrMatchesScope(catScopedDef, 1, null)).toBe(true);
		expect(attrMatchesScope(catScopedDef, 2, null)).toBe(false);
	});

	it('tag_scope filters correctly', () => {
		expect(attrMatchesScope(tagScopedDef, null, 5)).toBe(true);
		expect(attrMatchesScope(tagScopedDef, null, 3)).toBe(false);
	});

	it('multi scope matches any of the specified', () => {
		expect(attrMatchesScope(multiScopeDef, 1, null)).toBe(true);
		expect(attrMatchesScope(multiScopeDef, 2, null)).toBe(true);
		expect(attrMatchesScope(multiScopeDef, null, 3)).toBe(true);
		expect(attrMatchesScope(multiScopeDef, 3, null)).toBe(false);
		expect(attrMatchesScope(multiScopeDef, null, 4)).toBe(false);
	});
});
