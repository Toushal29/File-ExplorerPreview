<script>
	import { afterUpdate, onDestroy, onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import { fileCategory, formatDate, formatSize } from '../format.js';

	export let files = [];
	export let selectedPath = '';
	export let selectedPaths = [];
	export let viewMode = 'list';
	export let sortKey = 'name';
	export let sortDirection = 'asc';
	export let hasFilters = false;
	export let loading = false;
	export let error = '';
	export let onSelect = () => {};
	export let onSort = () => {};
	export let onContextMenu = () => {};
	export let onAuxSelect = () => {};
	export let onSelectionChange = () => {};

	const listRowHeight = 31;
	const gridRowHeight = 92;
	const overscan = 8;
	let panelElement;
	let scrollElement;
	let resizeObserver;
	let scrollTop = 0;
	let viewportHeight = 500;
	let gridColumns = 4;
	let lastSelectedPath = '';
	let selectionDrag = null;
	let selectionMarquee = null;
	let selectionActive = false;
	let dragSelectedPaths = [];
	$: firstListRow = Math.max(0, Math.floor(scrollTop / listRowHeight) - overscan);
	$: lastListRow = Math.min(files.length, Math.ceil((scrollTop + viewportHeight) / listRowHeight) + overscan);
	$: listTopSpace = firstListRow * listRowHeight;
	$: listBottomSpace = Math.max(0, (files.length - lastListRow) * listRowHeight);
	$: firstGridRow = Math.max(0, Math.floor(scrollTop / gridRowHeight) - overscan);
	$: gridRowCount = Math.ceil(files.length / gridColumns);
	$: lastGridRow = Math.min(gridRowCount, Math.ceil((scrollTop + viewportHeight) / gridRowHeight) + overscan);
	$: firstGridItem = firstGridRow * gridColumns;
	$: lastGridItem = Math.min(files.length, lastGridRow * gridColumns);
	$: gridTopSpace = firstGridRow * gridRowHeight;
	$: gridBottomSpace = Math.max(0, (gridRowCount - lastGridRow) * gridRowHeight);
	$: visibleItems = files.slice(firstListRow, lastListRow);
	$: visibleGridItems = files.slice(firstGridItem, lastGridItem);
	$: displayedSelectedPaths = selectionActive ? dragSelectedPaths : selectedPaths;

	onMount(() => {
		resizeObserver = new ResizeObserver(measureViewport);
		if (panelElement) resizeObserver.observe(panelElement);
		measureViewport();
	});
	onDestroy(() => {
		resizeObserver?.disconnect();
		if (scrollElement) scrollElement.removeEventListener('scroll', recordScroll);
	});
	afterUpdate(() => {
		const next = panelElement?.querySelector(viewMode === 'grid' ? '.file-grid' : '.file-rows');
		if (next !== scrollElement) {
			if (scrollElement) scrollElement.removeEventListener('scroll', recordScroll);
			scrollElement = next || null;
			scrollTop = 0;
			lastSelectedPath = '';
			if (scrollElement) {
				scrollElement.scrollTop = 0;
				scrollElement.addEventListener('scroll', recordScroll, { passive: true });
			}
			measureViewport();
		}
		if (selectedPath !== lastSelectedPath) {
			lastSelectedPath = selectedPath;
			const index = files.findIndex((file) => file.path === selectedPath);
			if (index >= 0 && scrollElement) {
				const row = viewMode === 'grid' ? Math.floor(index / gridColumns) : index;
				const height = viewMode === 'grid' ? gridRowHeight : listRowHeight;
				const top = row * height;
				if (top < scrollElement.scrollTop) scrollElement.scrollTop = top;
				else if (top + height > scrollElement.scrollTop + scrollElement.clientHeight) scrollElement.scrollTop = top + height - scrollElement.clientHeight;
			}
		}
	});

	function measureViewport() {
		if (!panelElement) return;
		if (scrollElement) viewportHeight = scrollElement.clientHeight;
		if (viewMode === 'grid') gridColumns = Math.max(1, Math.floor((panelElement.clientWidth - 16 + 2) / 107));
	}
	function recordScroll() {
		scrollTop = scrollElement?.scrollTop || 0;
		viewportHeight = scrollElement?.clientHeight || viewportHeight;
	}

	function startSelectionDrag(event) {
		if (event.button !== 0 || event.pointerType === 'touch' || event.target.closest('[data-path]')) return;
		selectionDrag = {
			pointerId: event.pointerId,
			startX: event.clientX,
			startY: event.clientY,
			x: event.clientX,
			y: event.clientY,
			originalPaths: [...selectedPaths],
			basePaths: event.ctrlKey || event.metaKey ? [...selectedPaths] : [],
			paths: [...selectedPaths],
			active: false
		};
		selectionActive = false;
		dragSelectedPaths = [...selectedPaths];
		event.currentTarget.setPointerCapture(event.pointerId);
	}

	function moveSelectionDrag(event) {
		if (!selectionDrag || event.pointerId !== selectionDrag.pointerId) return;
		selectionDrag.x = event.clientX;
		selectionDrag.y = event.clientY;
		if (!selectionDrag.active && Math.hypot(selectionDrag.x - selectionDrag.startX, selectionDrag.y - selectionDrag.startY) < 4) return;
		selectionDrag.active = true;
		selectionActive = true;
		event.preventDefault();
		const left = Math.min(selectionDrag.startX, selectionDrag.x);
		const top = Math.min(selectionDrag.startY, selectionDrag.y);
		const right = Math.max(selectionDrag.startX, selectionDrag.x);
		const bottom = Math.max(selectionDrag.startY, selectionDrag.y);
		selectionMarquee = { left, top, width: right - left, height: bottom - top };

		const bounds = scrollElement?.getBoundingClientRect();
		if (bounds) {
			if (selectionDrag.y < bounds.top + 28) scrollElement.scrollTop -= 20;
			else if (selectionDrag.y > bounds.bottom - 28) scrollElement.scrollTop += 20;
		}
		const selected = new Set(selectionDrag.basePaths);
		for (const element of scrollElement?.querySelectorAll('[data-path]') || []) {
			const rect = element.getBoundingClientRect();
			if (rect.right >= left && rect.left <= right && rect.bottom >= top && rect.top <= bottom) selected.add(element.dataset.path);
		}
		const paths = files.filter((file) => selected.has(file.path)).map((file) => file.path);
		selectionDrag.paths = paths;
		dragSelectedPaths = paths;
	}

	function endSelectionDrag(event, cancelled = false) {
		if (!selectionDrag || event.pointerId !== selectionDrag.pointerId) return;
		const wasActive = selectionDrag.active;
		if (cancelled) onSelectionChange(selectionDrag.originalPaths, true);
		else if (wasActive) onSelectionChange(selectionDrag.paths, true);
		else onSelectionChange([], true);
		if (scrollElement?.hasPointerCapture(event.pointerId)) scrollElement.releasePointerCapture(event.pointerId);
		selectionDrag = null;
		selectionMarquee = null;
		selectionActive = false;
	}

	function iconFor(item) {
		const category = fileCategory(item);
		if (category === 'folder') return 'folder';
		if (category === 'image') return 'image';
		if (category === 'video') return 'video';
		if (category === 'audio') return 'music';
		return 'file';
	}
	function sortLabel(key, label) {
		return `${label}${sortKey === key ? (sortDirection === 'asc' ? ' ↑' : ' ↓') : ''}`;
	}
</script>

<section class="browser-panel" bind:this={panelElement} aria-label="Folder contents" oncontextmenu={(event) => onContextMenu(event, null)}>
	{#if error}<div class="browser-state error-state"><h3>Couldn’t open this folder</h3><p>{error}</p></div>
	{:else if loading}<div class="browser-state">Opening folder…</div>
	{:else if files.length === 0}<div class="browser-state">{hasFilters ? 'No matching files.' : 'This folder is empty.'}</div>
	{:else if viewMode === 'grid'}
		<div class="file-grid" style={`--grid-columns:${gridColumns}`} onpointerdown={startSelectionDrag} onpointermove={moveSelectionDrag} onpointerup={endSelectionDrag} onpointercancel={(event) => endSelectionDrag(event, true)}>
			{#if gridTopSpace > 0}<div class="virtual-space" style={`height:${gridTopSpace - 2}px`}></div>{/if}
			{#each visibleGridItems as item (item.path)}
				<button class:chosen={displayedSelectedPaths.includes(item.path)} class="grid-card" type="button" title={item.path} data-path={item.path} onclick={(event) => onSelect(item, event)} onauxclick={(event) => event.button === 1 && onAuxSelect(item, event)} oncontextmenu={(event) => { event.stopPropagation(); onContextMenu(event, item); }}>
					<Icon name={iconFor(item)} size={22} /><span class="grid-name">{item.name}</span><span class="grid-meta">{item.is_dir ? 'Folder' : formatSize(item.size)}</span>
				</button>
			{/each}
			{#if gridBottomSpace > 0}<div class="virtual-space" style={`height:${gridBottomSpace - 2}px`}></div>{/if}
		</div>
	{:else}
		<div class="table-wrap">
			<div class="table-header"><button class="name-heading" onclick={() => onSort('name')}>{sortLabel('name', 'Name')}</button><button onclick={() => onSort('type')}>{sortLabel('type', 'Type')}</button><button onclick={() => onSort('size')}>{sortLabel('size', 'Size')}</button><button onclick={() => onSort('modified')}>{sortLabel('modified', 'Date modified')}</button></div>
			<div class="file-rows" role="list" onpointerdown={startSelectionDrag} onpointermove={moveSelectionDrag} onpointerup={endSelectionDrag} onpointercancel={(event) => endSelectionDrag(event, true)}>
				{#if listTopSpace > 0}<div class="virtual-space" style={`height:${listTopSpace}px`}></div>{/if}
				{#each visibleItems as item (item.path)}
					<button class:chosen={displayedSelectedPaths.includes(item.path)} class="file-row" type="button" title={item.path} data-path={item.path} onclick={(event) => onSelect(item, event)} onauxclick={(event) => event.button === 1 && onAuxSelect(item, event)} oncontextmenu={(event) => { event.stopPropagation(); onContextMenu(event, item); }} role="listitem">
						<span class="name-cell"><Icon name={iconFor(item)} size={17} /><span class="file-name">{item.name}</span></span>
						<span class="type-cell">{item.file_type}</span><span class="size-cell">{item.is_dir ? '—' : formatSize(item.size)}</span><span class="date-cell">{formatDate(item.modified)}</span>
					</button>
				{/each}
				{#if listBottomSpace > 0}<div class="virtual-space" style={`height:${listBottomSpace}px`}></div>{/if}
			</div>
		</div>
	{/if}
</section>
{#if selectionMarquee}<div class="selection-marquee" aria-hidden="true" style={`left:${selectionMarquee.left}px;top:${selectionMarquee.top}px;width:${selectionMarquee.width}px;height:${selectionMarquee.height}px`}></div>{/if}

<style>
	.browser-panel { min-width: 0; min-height: 0; flex: 1; display: flex; overflow: hidden; flex-direction: column; background: var(--paper); container-type: inline-size; }
	.table-wrap, .file-rows { min-width: 0; min-height: 0; flex: 1; display: flex; flex-direction: column; overflow: hidden; }
	.table-header, .file-row { display: grid; grid-template-columns: minmax(140px, 1fr) minmax(55px, 100px) minmax(50px, 75px) minmax(100px, 140px); align-items: center; gap: 10px; }
	.table-header { min-height: 30px; padding: 0 10px; border-bottom: 1px solid var(--line); background: var(--canvas); }
	.table-header button { overflow: hidden; padding: 3px 0; border: 0; color: var(--muted); background: transparent; text-align: left; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; cursor: pointer; }
	.file-rows { overflow: auto; }
	.file-row { width: 100%; height: 31px; min-height: 31px; flex: 0 0 31px; padding: 2px 10px; border: 0; border-bottom: 1px solid transparent; color: var(--ink); background: var(--paper); text-align: left; font-size: 11px; cursor: default; }
	.file-row:hover, .grid-card:hover { background: var(--canvas); }
	.file-row.chosen, .grid-card.chosen { outline: 1px solid var(--focus); background: var(--selected); }
	.file-rows, .file-grid { user-select: none; }
	.selection-marquee { position: fixed; z-index: 100; border: 1px solid var(--focus); background: color-mix(in srgb, var(--selected) 55%, transparent); pointer-events: none; }
	.name-cell { min-width: 0; display: flex; align-items: center; gap: 8px; }
	.file-name, .type-cell, .size-cell, .date-cell { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.name-cell :global(.icon) { color: var(--icon); }
	.type-cell, .size-cell, .date-cell { color: var(--muted); font-size: 10px; }
	.file-grid { min-height: 0; flex: 1; display: grid; grid-template-columns: repeat(var(--grid-columns, 4), minmax(0, 1fr)); grid-auto-rows: minmax(90px, auto); align-content: start; gap: 2px; overflow: auto; padding: 8px; }
	.virtual-space { min-height: 0; grid-column: 1 / -1; flex: 0 0 auto; pointer-events: none; }
	.grid-card { min-width: 0; min-height: 90px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 5px; padding: 8px; border: 1px solid transparent; color: var(--ink); background: transparent; cursor: default; }
	.grid-card :global(.icon) { color: var(--icon); }
	.grid-name { width: 100%; overflow: hidden; text-align: center; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; }
	.grid-meta { color: var(--muted); font-size: 9px; }
	.browser-state { flex: 1; display: grid; place-content: center; gap: 6px; padding: 20px; color: var(--muted); text-align: center; font-size: 12px; }
	.browser-state h3, .browser-state p { margin: 0; font-size: inherit; }
	.error-state { color: var(--danger-ink); }
	@container (max-width: 500px) {
		.table-header, .file-row { grid-template-columns: minmax(0, 1fr) 62px; }
		.table-header button:nth-child(2), .table-header button:nth-child(4), .file-row .type-cell, .file-row .date-cell { display: none; }
	}
	@container (max-width: 370px) {
		.table-header, .file-row { grid-template-columns: minmax(0, 1fr); }
		.table-header button:not(:first-child), .file-row .type-cell, .file-row .size-cell, .file-row .date-cell { display: none; }
	}
</style>
