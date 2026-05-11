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

export interface SseCallbacks {
	onThinking: (text: string) => void;
	onProgress: (msg: string) => void;
	onResult: (data: any) => void;
	onError: (msg: string) => void;
}

export function aiPostStream(
	path: string,
	body: unknown,
	callbacks: SseCallbacks
): AbortController {
	const controller = new AbortController();

	fetch(`${BASE}${path}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body),
		signal: controller.signal
	})
		.then(async (response) => {
			if (!response.ok) {
				const text = await response.text();
				let message: string;
				try {
					const err = JSON.parse(text);
					message = err.error || `${response.status}: ${JSON.stringify(err)}`;
				} catch {
					message = text || `请求失败 (${response.status})`;
				}
				callbacks.onError(message);
				return;
			}

			const reader = response.body?.getReader();
			if (!reader) {
				callbacks.onError('Response body is empty');
				return;
			}

			const decoder = new TextDecoder();
			let buffer = '';

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				buffer += decoder.decode(value, { stream: true });

				// Process complete SSE lines
				const lines = buffer.split('\n');
				buffer = lines.pop() || '';

				for (const line of lines) {
					const trimmed = line.trim();
					if (!trimmed || trimmed.startsWith(':')) continue;

					if (trimmed.startsWith('data: ')) {
						const data = trimmed.slice(6);
						if (data === '[DONE]') continue;

						try {
							const event = JSON.parse(data);
							switch (event.type) {
								case 'thinking':
									callbacks.onThinking(event.content);
									break;
								case 'progress':
									callbacks.onProgress(event.message);
									break;
								case 'result':
									callbacks.onResult(event);
									break;
								case 'error':
									callbacks.onError(event.message);
									break;
							}
						} catch {
							// Skip unparseable events
						}
					}
				}
			}
		})
		.catch((e) => {
			if (e.name !== 'AbortError') {
				callbacks.onError(e.message || 'Stream request failed');
			}
		});

	return controller;
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
