# API Client — fetch 封装

## 文件

`client.ts` — 唯一文件，导出 `api` 对象。

## 接口

```typescript
api.get<T>(path)           // GET
api.post<T>(path, body?)   // POST
api.put<T>(path, body)     // PUT
api.patch<T>(path, body)   // PATCH
api.del<T>(path)           // DELETE
```

## 约定

- `path` 不含 `/api` 前缀（内部自动拼接 `BASE = '/api'`）
- 所有请求默认 `Content-Type: application/json`
- 非 2xx 抛 `Error`，message 格式 `"${status}: ${body}"`
- 204 或 `content-length: 0` 返回 `undefined`
- Vite dev server 将 `/api` 代理到 `http://localhost:3000`

## 使用示例

```typescript
import { api } from '$lib/api/client';
const trips = await api.get<Trip[]>('/trips');
const newTrip = await api.post<Trip>('/trips', { name: '测试' });
await api.del('/trips/1');
```
