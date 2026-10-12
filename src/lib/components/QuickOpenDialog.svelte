<script>
	import { onMount } from 'svelte';
	import { filesystem } from '../filesystem.js';
	import { pathLabel } from '../format.js';
	import Icon from './Icon.svelte';

	export let currentPath = '';
	export let homePath = '';
	export let favorites = [];
	export let recentFiles = [];
	export let initialPane = 0;
	export let splitMode = false;
	export let darkMode = false;
	export let onOpen = () => {};
	export let onClose = () => {};

	let folderPath = currentPath || homePath;
	let folderFiles = [];
	let query = '';
	let selectedPath = '';
	let targetPane = initialPane;
	let loading = false;
	let error = '';
	let searchElement;
	let folderRequestId = 0;
	$: normalizedQuery = query.trim().toLocaleLowerCase();
	$: recentMatches = recentFiles.filter((file) => file.name.toLocaleLowerCase().includes(normalizedQuery) || file.path.toLocaleLowerCase().includes(normalizedQuery)).slice(0, normalizedQuery ? 30 : 8);
	$: folderMatches = folderFiles.filter((file) => file.name.toLocaleLowerCase().includes(normalizedQuery));
	$: folderFileMatches = folderMatches.filter((file) => !file.is_dir);
	$: folderMatchesForBrowse = folderMatches.filter((file) => file.is_dir);
	$: pickerItems = [...folderFileMatches.slice(0, 100), ...folderMatchesForBrowse.slice(0, 20)];
	$: shownFiles = [...recentMatches, ...pickerItems.filter((file) => !recentMatches.some((recent) => recent.path === file.path))].slice(0, 120);

	onMount(() => {
		searchElement?.focus();
		loadFolder(folderPath);
	});

	async function loadFolder(path) {
		if (!path) return false;
		const requestId = ++folderRequestId;
		loading = true;
		error = '';
		try {
			const entries = await filesystem.listDirectory(path);
			if (requestId !== folderRequestId) return false;
			folderFiles = entries;
			folderPath = path;
			selectedPath = '';
			requestAnimationFrame(() => searchElement?.focus());
			return true;
		} catch (reason) {
			if (requestId !== folderRequestId) return false;
			error = String(reason);
			folderFiles = [];
			return false;
		} finally {
			if (requestId === folderRequestId) loading = false;
		}
	}

	async function openLocation() {
		const path = folderPath.trim();
		if (await loadFolder(path)) return;
		const parent = await filesystem.getParentDirectory(path);
		if (!parent) return;
		try {
			const entries = await filesystem.listDirectory(parent);
			const wanted = path.replaceAll('/', '\\').replace(/\\+$/, '').toLocaleLowerCase();
			const item = entries.find((entry) => entry.path.replaceAll('/', '\\').replace(/\\+$/, '').toLocaleLowerCase() === wanted);
			if (item && !item.is_dir) onOpen(item, targetPane);
		} catch { /* Keep the location error visible in the picker. */ }
	}

	async function goUp() {
		const parent = await filesystem.getParentDirectory(folderPath);
		if (parent && parent !== folderPath) await loadFolder(parent);
	}

	function openItem(item) {
		if (item.is_dir) return;
		onOpen(item, targetPane);
	}

	function handleRowClick(item, event) {
		selectedPath = item.path;
		if (item.is_dir && event.detail > 1) loadFolder(item.path);
		else if (!item.is_dir) openItem(item);
	}

	function handleKeydown(event) {
		if (event.key === 'Escape') { event.preventDefault(); onClose(); }
		if (event.target === searchElement && (event.key === 'ArrowDown' || event.key === 'ArrowUp')) {
			event.preventDefault();
			const current = shownFiles.findIndex((item) => item.path === selectedPath);
			const direction = event.key === 'ArrowDown' ? 1 : -1;
			const next = current < 0 ? (direction > 0 ? 0 : shownFiles.length - 1) : Math.max(0, Math.min(shownFiles.length - 1, current + direction));
			selectedPath = shownFiles[next]?.path || '';
		}
		if (event.key === 'Enter' && event.target === searchElement) {
			event.preventDefault();
			const item = shownFiles.find((entry) => entry.path === selectedPath) || shownFiles.find((entry) => !entry.is_dir);
			if (item?.is_dir) loadFolder(item.path);
			else if (item) openItem(item);
		}
	}
</script>

<div class:dark={darkMode} class="quick-open-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && onClose()}>
	<section class="quick-open" role="dialog" aria-modal="true" aria-labelledby="quick-open-title" tabindex="-1" onkeydown={handleKeydown}>
		<header><h2 id="quick-open-title">Open a file</h2><button type="button" aria-label="Close" onclick={onClose}><Icon name="close" size={16} /></button></header>
		<label class="quick-search"><Icon name="search" size={16} /><input bind:this={searchElement} bind:value={query} placeholder="Find a recent file or search this folder" aria-label="Find a file" /><kbd>Enter</kbd></label>
		<div class="quick-location">
			<button type="button" aria-label="Parent folder" title="Parent folder" onclick={goUp}><Icon name="arrow-up" size={15} /></button>
			<input bind:value={folderPath} aria-label="Folder to browse" onkeydown={(event) => { if (event.key === 'Enter') { event.stopPropagation(); openLocation(); } }} />
			<select bind:value={targetPane} aria-label="Open in preview pane"><option value={0}>Preview 1</option>{#if splitMode}<option value={1}>Preview 2</option>{/if}</select>
		</div>
		{#if homePath || favorites.length}
			<div class="quick-locations" aria-label="Quick locations">
				{#if homePath}<button type="button" title={homePath} onclick={() => loadFolder(homePath)}><Icon name="home" size={14} />Home</button>{/if}
				{#each favorites as favorite (favorite)}<button type="button" title={favorite} onclick={() => loadFolder(favorite)}><Icon name="pin" size={14} />{pathLabel(favorite)}</button>{/each}
			</div>
		{/if}
		<div class="quick-list" aria-label="Files">
			{#if loading}<p class="quick-state">Loading folder…</p>
			{:else if error}<p class="quick-state error">{error}</p>
			{:else if shownFiles.length === 0}<p class="quick-state">No matching files here or in recent previews.</p>
			{:else}
				{#each shownFiles as item (item.path)}
					<button class:selected={selectedPath === item.path} class="quick-file" type="button" title={item.path} onclick={(event) => handleRowClick(item, event)}>
						<Icon name={item.is_dir ? 'folder' : 'file'} size={16} /><span>{item.name}</span><small>{item.is_dir ? 'Folder' : pathLabel(item.path)}</small>
					</button>
				{/each}
			{/if}
		</div>
		<footer><span>Search recent previews and this folder. Use the location field to browse elsewhere.</span><button type="button" onclick={onClose}>Close</button></footer>
	</section>
</div>

<style>
	.quick-open-backdrop { position: fixed; z-index: 30; inset: 0; display: grid; place-items: start center; padding: min(12vh, 90px) 14px 14px; background: #0006; }
	.quick-open { width: min(680px, 100%); max-height: 78vh; display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--line); border-radius: 6px; color: var(--ink); background: var(--paper); box-shadow: 0 8px 28px #0004; }
	.quick-open > header, .quick-open > footer { min-height: 42px; display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 0 12px; border-bottom: 1px solid var(--line); }
	.quick-open h2 { margin: 0; font-size: 13px; font-weight: 600; }
	.quick-open header button, .quick-open footer button, .quick-location button { min-height: 27px; display: inline-flex; align-items: center; justify-content: center; padding: 0 7px; border: 1px solid var(--line); border-radius: 3px; color: var(--ink); background: var(--paper); cursor: pointer; }
	.quick-search { min-height: 38px; display: flex; align-items: center; gap: 8px; margin: 10px 12px 6px; padding: 0 9px; border: 1px solid var(--line); border-radius: 4px; color: var(--muted); background: var(--canvas); }
	.quick-search input, .quick-location input { width: 100%; min-width: 0; border: 0; outline: 0; color: var(--ink); background: transparent; font-size: 12px; }
	.quick-search kbd { color: var(--muted); font-size: 9px; }
	.quick-location { display: flex; gap: 6px; padding: 0 12px 8px; }
	.quick-location input { height: 29px; padding: 0 7px; border: 1px solid var(--line); border-radius: 3px; background: var(--paper); font-size: 10px; }
	.quick-location select { width: 104px; border: 1px solid var(--line); border-radius: 3px; color: var(--ink); background: var(--paper); font-size: 10px; }
	.quick-locations { display: flex; gap: 4px; overflow: auto; padding: 0 12px 8px; }
	.quick-locations button { max-width: 150px; min-height: 25px; display: inline-flex; align-items: center; gap: 5px; overflow: hidden; padding: 0 7px; border: 1px solid var(--line); border-radius: 3px; color: var(--ink); background: var(--paper); text-overflow: ellipsis; white-space: nowrap; font-size: 10px; cursor: pointer; }
	.quick-list { min-height: 90px; flex: 1; overflow: auto; padding: 2px 8px; }
	.quick-file { width: 100%; min-height: 32px; display: grid; grid-template-columns: 20px minmax(80px, 1fr) minmax(60px, 1fr); align-items: center; gap: 8px; padding: 0 7px; border: 0; color: var(--ink); background: transparent; text-align: left; cursor: pointer; }
	.quick-file:hover, .quick-file.selected { background: var(--selected); }
	.quick-file > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }
	.quick-file small { overflow: hidden; color: var(--muted); text-overflow: ellipsis; white-space: nowrap; font-size: 9px; }
	.quick-state { padding: 22px; color: var(--muted); text-align: center; font-size: 11px; }
	.quick-state.error { color: var(--danger-ink); }
	.quick-open > footer { min-height: 36px; justify-content: space-between; border-top: 1px solid var(--line); border-bottom: 0; color: var(--muted); font-size: 9px; }
	.quick-open > footer button { min-height: 25px; font-size: 10px; }
	.quick-open.dark { color: var(--ink); }
</style>
