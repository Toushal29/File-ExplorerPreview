<script>
	import { filesystem } from '../filesystem.js';
	import Icon from './Icon.svelte';
	import { pathLabel } from '../format.js';

	export let currentPath = '';
	export let favorites = [];
	export let homePath = '';
	export let collapsed = false;
	export let sidebarWidth = 190;
	export let refreshToken = 0;
	export let onNavigate = () => {};
	export let onToggleCollapse = () => {};

	let treeRoot = '';
	let rootInitialized = '';
	let lastRevealed = '';
	let childrenByPath = {};
	let expandedPaths = {};
	let loadingPaths = {};
	let treeError = '';
	let lastRefreshToken = 0;
	let treeExpanded = false;
	let treeReady = false;
	$: if (treeExpanded && homePath && rootInitialized !== homePath) { rootInitialized = homePath; prepareTree(homePath); }
	$: if (treeExpanded && treeReady && treeRoot && currentPath && lastRevealed !== currentPath) { lastRevealed = currentPath; revealCurrentPath(currentPath); }
	$: if (treeExpanded && treeReady && treeRoot && refreshToken !== lastRefreshToken) { lastRefreshToken = refreshToken; refreshExpandedTree(); }
	$: treeRows = flattenTree(treeRoot, expandedPaths, childrenByPath);

	function toggleTree() { treeExpanded = !treeExpanded; }

	function normalize(path) { return path.replaceAll('/', '\\').replace(/\\+$/, '').toLowerCase(); }
	async function loadChildren(path) {
		if (Object.hasOwn(childrenByPath, path) || loadingPaths[path]) return childrenByPath[path] || [];
		loadingPaths = { ...loadingPaths, [path]: true };
		try {
			const entries = await filesystem.listDirectory(path);
			const folders = entries.filter((entry) => entry.is_dir).sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base', numeric: true }));
			childrenByPath = { ...childrenByPath, [path]: folders };
			return folders;
		} catch (reason) {
			treeError = String(reason);
			childrenByPath = { ...childrenByPath, [path]: [] };
			return [];
		} finally {
			const next = { ...loadingPaths }; delete next[path]; loadingPaths = next;
		}
	}
	async function prepareTree(path) {
		treeReady = false;
		const chain = [path];
		let cursor = path;
		for (let step = 0; step < 40; step += 1) {
			const parent = await filesystem.getParentDirectory(cursor);
			if (!parent || parent === cursor) break;
			chain.unshift(parent); cursor = parent;
		}
		treeRoot = chain[0];
		for (const folder of chain) {
			expandedPaths = { ...expandedPaths, [folder]: true };
		}
		await Promise.all(chain.map((folder) => loadChildren(folder)));
		treeReady = true;
	}
	async function refreshExpandedTree() {
		const expanded = Object.keys(expandedPaths);
		childrenByPath = {};
		treeError = '';
		await Promise.all(expanded.map((path) => loadChildren(path)));
	}
	async function revealCurrentPath(path) {
		let cursor = path;
		const chain = [];
		for (let step = 0; step < 40 && normalize(cursor) !== normalize(treeRoot); step += 1) {
			chain.unshift(cursor);
			const parent = await filesystem.getParentDirectory(cursor);
			if (!parent || parent === cursor) return;
			cursor = parent;
		}
		if (normalize(cursor) !== normalize(treeRoot)) return;
		expandedPaths = { ...expandedPaths, [treeRoot]: true };
		let parentPath = treeRoot;
		for (const childPath of chain) {
			const children = await loadChildren(parentPath);
			if (!children.some((child) => normalize(child.path) === normalize(childPath))) break;
			expandedPaths = { ...expandedPaths, [parentPath]: true, [childPath]: true };
			parentPath = childPath;
		}
		await loadChildren(parentPath);
	}
	async function toggleFolder(path, hasChildren) {
		if (!hasChildren) return;
		if (expandedPaths[path]) {
			const next = { ...expandedPaths }; delete next[path]; expandedPaths = next;
		} else {
			await loadChildren(path);
			expandedPaths = { ...expandedPaths, [path]: true };
		}
	}
	function flattenTree(root, expanded, childrenByFolder) {
		if (!root) return [];
		const rows = [];
		const visited = new Set();
		function visit(path, name, depth) {
			if (depth > 12 || rows.length > 400 || visited.has(normalize(path))) return;
			visited.add(normalize(path));
			const children = childrenByFolder[path];
			const hasChildren = !children || children.length > 0;
			rows.push({ path, name, depth, hasChildren });
			if (expanded[path] && children) for (const child of children) visit(child.path, child.name, depth + 1);
		}
		visit(root, pathLabel(root), 0);
		return rows;
	}
</script>

<aside class:collapsed class="app-sidebar" style={`width:${collapsed ? 64 : sidebarWidth}px`}>
	<header class="sidebar-header">
		<div class="brand-mark"><Icon name="folder" size={19} /></div>
		{#if !collapsed}<strong>File Explorer-Preview</strong>{/if}
		<button class="collapse-button" type="button" aria-label={collapsed ? 'Expand folder tree' : 'Collapse folder tree'} title={collapsed ? 'Expand folder tree' : 'Collapse folder tree'} onclick={onToggleCollapse}><Icon name={collapsed ? 'chevron-right' : 'chevron-left'} size={15} /></button>
	</header>

	{#if homePath || favorites.length}
		<section class="quick-access"><h2>{#if !collapsed}Quick access{/if}</h2>
			{#if homePath}<button class="tree-location" title={homePath} onclick={() => onNavigate(homePath)}><Icon name="home" size={15} />{#if !collapsed}<span>Home</span>{/if}</button>{/if}
			{#each favorites as folder (folder)}<button class="tree-location" title={folder} onclick={() => onNavigate(folder)}><Icon name="pin" size={15} />{#if !collapsed}<span>{pathLabel(folder)}</span>{/if}</button>{/each}
		</section>
	{/if}

	<section class="folder-section" aria-label="Folder explorer">
		<button class="tree-location tree-toggle" type="button" aria-expanded={treeExpanded} title="Browse folders" onclick={toggleTree}><Icon name={treeExpanded ? 'chevron-up' : 'folder'} size={15} />{#if !collapsed}<span>Browse folders</span>{/if}</button>
		{#if treeError && !treeRows.length}<p class="tree-error" title={treeError}>Can’t load folders</p>{/if}
		{#if treeExpanded}<div class="folder-tree" role="tree" aria-label="Folders">
			{#each treeRows as row (row.path)}
				<div class="tree-row" style={`padding-left:${Math.min(row.depth, 10) * 13}px`} role="treeitem" aria-expanded={row.hasChildren ? !!expandedPaths[row.path] : undefined}>
					{#if !collapsed}<button class="tree-expander" type="button" aria-label={expandedPaths[row.path] ? 'Collapse folder' : 'Expand folder'} disabled={!row.hasChildren} onclick={() => toggleFolder(row.path, row.hasChildren)}><Icon name={loadingPaths[row.path] ? 'refresh' : expandedPaths[row.path] ? 'chevron-up' : 'chevron-right'} size={13} /></button>{/if}
					<button class:current={normalize(currentPath) === normalize(row.path)} class="tree-folder" type="button" title={row.path} onclick={() => onNavigate(row.path)}><Icon name="folder" size={16} />{#if !collapsed}<span>{row.name}</span>{/if}</button>
				</div>
			{/each}
		</div>{/if}
	</section>

</aside>

<style>
	.app-sidebar { min-width: 64px; height: 100%; display: flex; flex: 0 0 auto; flex-direction: column; overflow: hidden; padding: 8px 6px; border-right: 1px solid var(--line); color: var(--ink); background: var(--canvas); }
	.sidebar-header { min-height: 38px; display: flex; align-items: center; gap: 8px; padding: 0 3px; margin-bottom: 8px; }
	.sidebar-header strong { min-width: 0; flex: 1; overflow: hidden; font-size: 11px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
	.brand-mark { width: 28px; height: 28px; display: grid; flex: 0 0 auto; place-items: center; color: var(--icon); }
	.collapse-button { width: 23px; height: 24px; display: grid; place-items: center; margin-left: auto; padding: 0; border: 0; color: var(--muted); background: transparent; cursor: pointer; }
	.collapsed .sidebar-header { flex-direction: column; min-height: 64px; justify-content: center; padding: 0; }
	.collapsed .collapse-button { margin: 0; }
	h2 { margin: 5px 6px; color: var(--muted); font-size: 10px; font-weight: 600; }
	.quick-access { display: grid; gap: 1px; padding-bottom: 8px; border-bottom: 1px solid var(--line); }
	.tree-location, .tree-folder { min-width: 0; width: 100%; min-height: 29px; display: flex; align-items: center; gap: 7px; overflow: hidden; padding: 0 6px; border: 0; border-radius: 3px; color: inherit; background: transparent; text-align: left; font-size: 11px; cursor: pointer; }
	.tree-location:hover, .tree-folder:hover { background: var(--hover); }
	.tree-location span, .tree-folder span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.tree-location :global(.icon), .tree-folder :global(.icon) { color: var(--icon); }
	.folder-section { min-height: 0; display: flex; flex: 1; flex-direction: column; overflow: hidden; }
	.tree-toggle { flex: 0 0 auto; }
	.collapsed .folder-section { display: none; }
	.folder-tree { min-height: 0; flex: 1; overflow: auto; }
	.tree-row { min-width: 0; height: 28px; display: flex; align-items: center; }
	.tree-expander { width: 19px; height: 25px; display: grid; flex: 0 0 auto; place-items: center; padding: 0; border: 0; color: var(--muted); background: transparent; cursor: pointer; }
	.tree-expander:disabled { opacity: 0; cursor: default; }
	.tree-folder { flex: 1; height: 27px; }
	.tree-folder.current { color: var(--selected-ink); background: var(--selected); }
	.collapsed .tree-row { padding-left: 0 !important; justify-content: center; }
	.collapsed .tree-folder { justify-content: center; padding: 0; }
	.tree-error { margin: 6px; color: var(--danger-ink); font-size: 10px; }
	@media (max-width: 790px) { .app-sidebar { font-size: 10px; } }
</style>
