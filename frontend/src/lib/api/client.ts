const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...options
	});
	if (!res.ok) {
		let message: string;
		try {
			const body = await res.json();
			message = body.error || `${res.status}: ${JSON.stringify(body)}`;
		} catch {
			const text = await res.text();
			message = text || `请求失败 (${res.status})`;
		}
		throw new Error(message);
	}
	if (res.status === 204 || res.headers.get('content-length') === '0') {
		return undefined as T;
	}
	return res.json();
}

export const api = {
	get: <T>(path: string) => request<T>(path),
	post: <T>(path: string, body?: unknown) =>
		request<T>(path, { method: 'POST', body: body ? JSON.stringify(body) : undefined }),
	put: <T>(path: string, body: unknown) =>
		request<T>(path, { method: 'PUT', body: JSON.stringify(body) }),
	patch: <T>(path: string, body: unknown) =>
		request<T>(path, { method: 'PATCH', body: JSON.stringify(body) }),
	del: <T>(path: string) => request<T>(path, { method: 'DELETE' }),

	async downloadExport(path: string): Promise<void> {
		const res = await fetch(`${BASE}${path}`);
		if (!res.ok) {
			let message: string;
			try {
				const body = await res.json();
				message = body.error || `${res.status}: ${JSON.stringify(body)}`;
			} catch {
				const text = await res.text();
				message = text || `请求失败 (${res.status})`;
			}
			throw new Error(message);
		}
		const blob = await res.blob();
		const url = URL.createObjectURL(blob);
		const now = new Date();
		const dateStr = now.toISOString().slice(0, 10);
		const a = document.createElement('a');
		a.href = url;
		a.download = `inventory-backup-${dateStr}.json`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
};
