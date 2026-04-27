<script lang="ts">
	import '../app.css';

	let { children } = $props();

	const nav = [
		{ href: '/', label: '首页', icon: '🏠' },
		{ href: '/items', label: '物品库', icon: '📦' },
		{ href: '/activities', label: '活动模板', icon: '🏔️' },
		{ href: '/trips', label: '行程', icon: '🗺️' }
	];
</script>

<div class="layout">
	<aside class="sidebar">
		<h2 class="logo">出行清单</h2>
		<nav>
			{#each nav as item}
				<a href={item.href} class="nav-item">
					<span class="nav-icon">{item.icon}</span>
					{item.label}
				</a>
			{/each}
		</nav>
	</aside>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	.layout {
		display: flex;
		min-height: 100vh;
	}

	.sidebar {
		width: var(--sidebar-width);
		background: var(--surface);
		border-right: 1px solid var(--border);
		padding: 20px 0;
		flex-shrink: 0;
	}

	.logo {
		padding: 0 20px 20px;
		font-size: 18px;
		border-bottom: 1px solid var(--border);
		margin-bottom: 12px;
	}

	nav {
		display: flex;
		flex-direction: column;
	}

	.nav-item {
		padding: 10px 20px;
		text-decoration: none;
		color: var(--text);
		display: flex;
		align-items: center;
		gap: 8px;
		transition: background 0.15s;
	}

	.nav-item:hover {
		background: var(--bg);
	}

	.nav-icon {
		font-size: 16px;
		width: 24px;
		text-align: center;
	}

	.content {
		flex: 1;
		padding: 24px 32px;
		max-width: 960px;
	}
</style>
