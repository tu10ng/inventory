# Frontend — SvelteKit 2 + Svelte 5 + TypeScript

## 技术栈

- **SvelteKit 2** — 应用框架（adapter-static，SPA 模式，`ssr = false`）
- **Svelte 5** — 组件框架（runes 模式：`$state` / `$derived` / `$effect` / `$props`）
- **TypeScript** — 类型系统
- **Vite 8** — 构建工具（dev server proxy `/api` → `localhost:3000`）
- **svelte-dnd-action** — 拖拽库（已安装，用于物品库拖拽）

## 目录结构

```
src/
├── app.css                    全局样式 + 暗色物品栏主题 + 打印样式
├── app.html                   HTML 入口
├── lib/
│   ├── api/client.ts          fetch 封装（get/post/put/patch/del）
│   ├── types/index.ts         所有 TS 接口定义
│   ├── utils/status.ts        状态常量（STATUS_LABELS, STATUS_OPTIONS, TRIP_STATUS_LABELS）
│   └── components/            11 个可复用组件
├── routes/
│   ├── +layout.svelte         全局布局（侧边栏导航 + .content 容器）
│   ├── +layout.ts             ssr = false, prerender = true
│   ├── +page.svelte           首页
│   ├── trips/+page.svelte     行程列表
│   ├── trips/[id]/+page.svelte  行程详情（双栏：清单 + 物品库）
│   ├── items/+page.svelte     物品库管理
│   └── activities/+page.svelte  活动模板管理
```

## 开发约定

### 状态管理
- 纯 Svelte 5 runes，无外部状态库
- 页面数据在 `$effect` 中调用 `load()` 加载，存入 `$state` 变量
- 父子通信用 `$props` + 回调函数，不用 store

### API 调用
- 统一通过 `$lib/api/client.ts` 的 `api` 对象
- 路径不含 `/api` 前缀（client 会自动加）
- 错误处理：`api.get/post/put/patch/del` 抛异常，调用方自行 catch

### 组件模式
- `$props()` 解构所有参数，含类型注解
- 回调用 `onXxx` 命名（`onchange`、`onToggleCheck`）
- snippet slot 用 `{@render children()}` 模式
- 样式 scoped（`<style>` 块），全局样式在 `app.css`

### 类型检查
```bash
pnpm check    # svelte-kit sync && svelte-check
```

## CSS 变量

### 基础主题（app.css :root）
`--bg`, `--surface`, `--border`, `--text`, `--text-secondary`, `--primary`, `--primary-hover`, `--success`, `--warning`, `--danger`

### 物品栏暗色主题
`--inventory-bg: #1a1a2e`, `--inventory-surface: #16213e`, `--inventory-border: #0f3460`, `--inventory-text`, `--inventory-text-secondary`, `--inventory-accent: #e94560`, `--inventory-highlight: #533483`

## 布局

- 侧边栏宽度：`--sidebar-width: 220px`
- 内容区最大宽度：`1400px`（容纳双栏布局）
- 移动端 `< 768px`：双栏变 tab 切换
