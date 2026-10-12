<script>
	import { onMount } from 'svelte';
	import { openPath } from '@tauri-apps/plugin-opener';
	import { filesystem } from '$lib/filesystem.js';
	import { getBreadcrumbs, joinPath, pathLabel } from '$lib/format.js';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import ExplorerSidebar from '$lib/components/ExplorerSidebar.svelte';
	import FileBrowser from '$lib/components/FileBrowser.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import PreviewPane from '$lib/components/PreviewPane.svelte';
	import QuickOpenDialog from '$lib/components/QuickOpenDialog.svelte';
	import SettingsDialog from '$lib/components/SettingsDialog.svelte';

	const defaultShortcuts = {
		search: ['Ctrl+F', ''],
		quickOpen: ['Ctrl+P', ''],
		nextTab: ['Ctrl+Tab', ''],
		previousTab: ['Ctrl+Shift+Tab', ''],
		editPath: ['Ctrl+L', ''],
		toggleSplit: ['Ctrl+Shift+S', ''],
		toggleSidebar: ['Ctrl+B', ''],
		togglePreview: ['Ctrl+Shift+P', ''],
		newFolder: ['Ctrl+Shift+N', ''],
		rename: ['F2', ''],
		delete: ['Delete', ''],
		previousItem: ['Up', ''],
		nextItem: ['Down', ''],
		refresh: ['Ctrl+R', 'F5'],
		back: ['Alt+Left', ''],
		forward: ['Alt+Right', ''],
		up: ['Alt+Up', '']
	};

	function blankPreview() {
		return { selectedFile: null, previewType: null, previewError: '', textContent: '', mediaUrl: null, imageZoom: 1, tabs: [], activeTabId: '' };
	}

	let homePath = '';
	let currentPath = '';
	let directoryRequestId = 0;
	let files = [];
	let visibleFiles = [];
	let sortedFiles = [];
	let orderedFiles = [];
	let selectedPath = '';
	let selectedPaths = [];
	let loading = false;
	let directoryError = '';
	let settingsVisible = false;
	let startPath = '';
	let hideDotfiles = false;
	let viewMode = 'list';
	let theme = 'system';
	let systemDarkMode = false;
	let sidebarCollapsed = false;
	let sidebarWidth = 190;
	let previewWidth = 400;
	let splitRatio = 50;
	let previewVisible = false;
	let splitMode = false;
	let activePreview = 0;
	let previews = [blankPreview(), blankPreview()];
	let previewTabSequence = 0;
	let draggedPreviewTab = null;
	let pendingPreviewTabPointer = null;
	let recentFiles = [];
	let quickOpenVisible = false;
	let favorites = [];
	let treeRefreshToken = 0;
	let shortcuts = { ...defaultShortcuts };
	let mouseMiddleOtherPreview = true;
	let mouseSideNavigation = true;
	let searchQuery = '';
	let searchDraft = '';
	let searchDebounce;
	let addressEditing = false;
	let addressValue = '';
	let settingsMessage = '';
	let statusMessage = '';
	let contextMenu = null;
	let contextItem = null;
	let resizing = null;
	let addressInput;
	let searchInput;
	let sidebarElement;
	let contentElement;
	let previewAreaElement;
	let historyPaths = [];
	let historyIndex = -1;
	let sortKey = 'name';
	let sortDirection = 'asc';

	$: darkMode = theme === 'dark' || (theme === 'system' && systemDarkMode);
	$: mainPreviewIndex = splitMode ? 0 : activePreview;
	$: visibleFiles = hideDotfiles ? files.filter((file) => !file.name.startsWith('.')) : files;
	$: sortedFiles = [...visibleFiles].sort((a, b) => {
			if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
			let order = 0;
			if (sortKey === 'size') order = a.size - b.size;
			else if (sortKey === 'modified') order = (a.modified || 0) - (b.modified || 0);
			else if (sortKey === 'type') order = a.file_type.localeCompare(b.file_type, undefined, { sensitivity: 'base' });
			else order = a.name.localeCompare(b.name, undefined, { sensitivity: 'base', numeric: true });
			return sortDirection === 'asc' ? order : -order;
		});
	$: orderedFiles = sortedFiles.filter((file) => file.name.toLocaleLowerCase().includes(searchQuery.trim().toLocaleLowerCase()));
	$: breadcrumbs = getBreadcrumbs(currentPath);
	$: selectedFile = visibleFiles.find((file) => file.path === selectedPath) || null;
	function mergeShortcuts(stored = {}) {
		const merged = { ...defaultShortcuts };
		for (const [id, value] of Object.entries(stored || {})) {
			if (Object.hasOwn(defaultShortcuts, id)) {
				merged[id] = Array.isArray(value) ? [value[0] || '', value[1] || ''] : [value || '', ''];
			}
		}
		return merged;
	}

	async function loadDirectory(path, addToHistory = true) {
		if (!path) return false;
		const requestId = ++directoryRequestId;
		loading = true;
		directoryError = '';
		try {
			const entries = await filesystem.listDirectory(path);
			if (requestId !== directoryRequestId) return false;
			const sameFolder = path === currentPath;
			files = entries;
			currentPath = path;
			if (!sameFolder || !entries.some((entry) => entry.path === selectedPath)) { selectedPath = ''; selectedPaths = []; }
			else selectedPaths = selectedPaths.filter((selected) => entries.some((entry) => entry.path === selected));
			if (!sameFolder) { searchQuery = ''; searchDraft = ''; clearTimeout(searchDebounce); }
			if (addToHistory) {
				if (historyPaths[historyIndex] !== path) {
					historyPaths = [...historyPaths.slice(0, historyIndex + 1), path];
					historyIndex = historyPaths.length - 1;
				}
			}
			return true;
		} catch (error) {
			if (requestId !== directoryRequestId) return false;
			directoryError = String(error);
			return false;
		} finally {
			if (requestId === directoryRequestId) loading = false;
		}
	}

	async function navigate(path) {
		if (!path || path === currentPath) return;
		await loadDirectory(path);
	}

	async function goBack() {
		if (historyIndex <= 0) return;
		historyIndex -= 1;
		await loadDirectory(historyPaths[historyIndex], false);
	}

	async function goForward() {
		if (historyIndex >= historyPaths.length - 1) return;
		historyIndex += 1;
		await loadDirectory(historyPaths[historyIndex], false);
	}

	async function goUp() {
		if (!currentPath) return;
		const parent = await filesystem.getParentDirectory(currentPath);
		if (parent && parent !== currentPath) await navigate(parent);
	}

	async function refreshDirectory() {
		if (currentPath) await loadDirectory(currentPath, false);
	}

	function updatePreview(index, changes) {
		const preview = previews[index];
		const replacesTabs = Object.hasOwn(changes, 'tabs') || Object.hasOwn(changes, 'activeTabId');
		const tabs = replacesTabs ? (changes.tabs || preview.tabs) : preview.tabs.map((tab) => tab.tabId === preview.activeTabId ? { ...tab, ...changes } : tab);
		previews = previews.map((entry, position) => position === index ? { ...entry, ...changes, tabs } : entry);
	}

	function updateTab(index, tabId, changes) {
		const actualIndex = previews[index]?.tabs.some((tab) => tab.tabId === tabId) ? index : previews.findIndex((entry) => entry.tabs.some((tab) => tab.tabId === tabId));
		if (actualIndex < 0) return;
		const preview = previews[actualIndex];
		const tabs = preview.tabs.map((tab) => tab.tabId === tabId ? { ...tab, ...changes } : tab);
		if (preview.activeTabId === tabId) {
			const active = tabs.find((tab) => tab.tabId === tabId);
			previews = previews.map((entry, position) => position === actualIndex ? { ...active, tabs, activeTabId: tabId } : entry);
		} else {
			previews = previews.map((entry, position) => position === actualIndex ? { ...entry, tabs } : entry);
		}
	}

	function activatePreviewTab(index, tabId) {
		const preview = previews[index];
		const tab = preview.tabs.find((entry) => entry.tabId === tabId);
		if (!tab) return;
		activePreview = index;
		selectedPath = tab.selectedFile?.path || selectedPath;
		if (selectedPath) selectedPaths = [selectedPath];
		previews = previews.map((entry, position) => position === index ? { ...tab, tabs: preview.tabs, activeTabId: tabId } : entry);
	}

	function movePreviewTab(direction) {
		const index = activePreview;
		const tabs = previews[index].tabs;
		if (tabs.length < 2) return;
		const current = tabs.findIndex((tab) => tab.tabId === previews[index].activeTabId);
		const next = (current + direction + tabs.length) % tabs.length;
		activatePreviewTab(index, tabs[next].tabId);
	}

	function closePreviewTab(index, tabId) {
		const preview = previews[index];
		const oldIndex = preview.tabs.findIndex((tab) => tab.tabId === tabId);
		const tabs = preview.tabs.filter((tab) => tab.tabId !== tabId);
		if (!tabs.length) {
			previews = previews.map((entry, position) => position === index ? blankPreview() : entry);
			if (!splitMode) previewVisible = false;
			return;
		}
		if (preview.activeTabId === tabId) {
			const next = tabs[Math.max(0, Math.min(oldIndex - 1, tabs.length - 1))];
			previews = previews.map((entry, position) => position === index ? { ...next, tabs, activeTabId: next.tabId } : entry);
			selectedPath = next.selectedFile?.path || selectedPath;
			if (selectedPath) selectedPaths = [selectedPath];
		} else {
			previews = previews.map((entry, position) => position === index ? { ...entry, tabs } : entry);
		}
	}

	function movePreviewTabBetweenPanes(sourceIndex, tabId, targetIndex) {
		if (!previews[sourceIndex] || !previews[targetIndex] || sourceIndex === targetIndex) return;
		const source = previews[sourceIndex];
		const sourceTab = source.tabs.find((tab) => tab.tabId === tabId);
		if (!sourceTab) return;

		const target = previews[targetIndex];
		const sourceTabIndex = source.tabs.findIndex((tab) => tab.tabId === tabId);
		const sourceTabs = source.tabs.filter((tab) => tab.tabId !== tabId);
		let sourcePreview;
		if (!sourceTabs.length) sourcePreview = blankPreview();
		else if (source.activeTabId === tabId) {
			const next = sourceTabs[Math.max(0, Math.min(sourceTabIndex - 1, sourceTabs.length - 1))];
			sourcePreview = { ...next, tabs: sourceTabs, activeTabId: next.tabId };
		} else sourcePreview = { ...source, tabs: sourceTabs };

		const targetTabs = [...target.tabs, sourceTab];
		const targetPreview = { ...sourceTab, tabs: targetTabs, activeTabId: sourceTab.tabId };
		previews = previews.map((entry, position) => position === sourceIndex ? sourcePreview : position === targetIndex ? targetPreview : entry);
		activePreview = targetIndex;
	}

	function beginPreviewTabDrag(sourceIndex, tabId, event) {
		pendingPreviewTabPointer = {
			sourceIndex,
			tabId,
			pointerId: event.pointerId,
			startX: event.clientX,
			startY: event.clientY,
			dragging: false
		};
	}

	function movePreviewTabDrag(event) {
		const pending = pendingPreviewTabPointer;
		if (!pending || pending.pointerId !== event.pointerId) return;
		if (!pending.dragging && Math.hypot(event.clientX - pending.startX, event.clientY - pending.startY) < 5) return;
		if (!pending.dragging) {
			pending.dragging = true;
			draggedPreviewTab = { sourceIndex: pending.sourceIndex, tabId: pending.tabId };
		}
	}

	function finishPreviewTabDrag(event) {
		const pending = pendingPreviewTabPointer;
		if (!pending || pending.pointerId !== event.pointerId) return;
		pendingPreviewTabPointer = null;
		if (!pending.dragging) {
			draggedPreviewTab = null;
			return;
		}
		const targetPane = document.elementFromPoint(event.clientX, event.clientY)?.closest('.preview-pane');
		const targetIndex = Number(targetPane?.dataset.previewIndex);
		if (Number.isInteger(targetIndex) && targetIndex !== pending.sourceIndex) {
			movePreviewTabBetweenPanes(pending.sourceIndex, pending.tabId, targetIndex);
		}
		draggedPreviewTab = null;
	}

	function handleGlobalPointerMove(event) {
		moveResize(event);
		movePreviewTabDrag(event);
	}

	function handleGlobalPointerUp(event) {
		stopResize();
		finishPreviewTabDrag(event);
	}

	function handleGlobalPointerCancel(event) {
		stopResize();
		if (pendingPreviewTabPointer && pendingPreviewTabPointer.pointerId !== event.pointerId) return;
		pendingPreviewTabPointer = null;
		draggedPreviewTab = null;
	}

	function setPreviewError(index, message, path) {
		const tab = previews[index].tabs.find((entry) => entry.selectedFile?.path === path) || previews.flatMap((entry) => entry.tabs).find((entry) => entry.selectedFile?.path === path);
		if (tab) updateTab(index, tab.tabId, { previewError: message });
		else if (!path) updatePreview(index, { previewError: message });
	}

	async function openPreview(item, requestedIndex = null, preserveSelection = false) {
		if (!item || item.is_dir) return;
		previewVisible = true;
		if (!preserveSelection) requestAnimationFrame(adjustLayout);
		const index = requestedIndex === null ? activePreview : requestedIndex;
		activePreview = index;
		if (!preserveSelection) {
			selectedPath = item.path;
			selectedPaths = [item.path];
		}
		recentFiles = [item, ...recentFiles.filter((file) => file.path !== item.path)].slice(0, 40);
		const existingTab = previews[index].tabs.find((tab) => tab.selectedFile?.path === item.path);
		if (existingTab) {
			if (preserveSelection) previews = previews.map((entry, position) => position === index ? { ...existingTab, tabs: previews[index].tabs, activeTabId: existingTab.tabId } : entry);
			else activatePreviewTab(index, existingTab.tabId);
			return;
		}
		const tabId = `preview-${++previewTabSequence}`;
		const tab = { ...blankPreview(), selectedFile: item, tabId };
		const tabs = [...previews[index].tabs, tab];
		previews = previews.map((entry, position) => position === index ? { ...tab, tabs, activeTabId: tabId } : entry);

		try {
			const info = await filesystem.getPreviewType(item.path);
			const previewType = info.preview_type;
			if (previewType === 'text') {
				const textContent = await filesystem.readTextFile(item.path);
				updateTab(index, tabId, { previewType, textContent });
			} else if (['image', 'pdf', 'audio', 'video'].includes(previewType)) {
				updateTab(index, tabId, { previewType, mediaUrl: filesystem.mediaUrl(item.path) });
			} else {
				updateTab(index, tabId, { previewType: 'unsupported', previewError: 'No preview is available for this file type.' });
			}
		} catch (error) {
			updateTab(index, tabId, { previewType: 'unsupported', previewError: 'Could not load preview: ' + error });
		}

	}

	function selectItem(item, event) {
		selectedPath = item.path;
		selectedPaths = [item.path];
		if (item.is_dir) {
			if (event.detail === 0 || event.detail > 1) navigate(item.path);
			return;
		}
		openPreview(item);
	}

	function selectAdjacentFile(direction) {
		if (!orderedFiles.length) return;
		const currentIndex = orderedFiles.findIndex((file) => file.path === selectedPath);
		const nextIndex = currentIndex < 0 ? (direction > 0 ? 0 : orderedFiles.length - 1) : Math.max(0, Math.min(orderedFiles.length - 1, currentIndex + direction));
		const item = orderedFiles[nextIndex];
		selectedPath = item.path;
		selectedPaths = [item.path];
		if (!item.is_dir) openPreview(item);
	}

	function togglePreview() {
		previewVisible = !previewVisible;
		if (previewVisible) {
			if (!previews[activePreview].selectedFile && selectedFile && !selectedFile.is_dir) openPreview(selectedFile, activePreview);
			requestAnimationFrame(adjustLayout);
		}
	}

	function toggleSplit() {
		if (!splitMode) {
			splitMode = true;
			previewVisible = true;
			requestAnimationFrame(adjustLayout);
			savePreferences(false);
			return;
		}
		splitMode = false;
		savePreferences(false);
	}

	function toggleSidebar() {
		sidebarCollapsed = !sidebarCollapsed;
		requestAnimationFrame(adjustLayout);
		savePreferences(false);
	}

	function setViewMode(mode) {
		viewMode = mode;
		savePreferences(false);
	}

	function closePreview(index) {
		if (splitMode) {
			splitMode = false;
			activePreview = index === 0 ? 1 : 0;
		} else {
			previewVisible = false;
		}
		savePreferences(false);
	}

	function openQuickOpen(index = activePreview) {
		activePreview = index;
		contextMenu = null;
		quickOpenVisible = true;
	}

	function zoom(index, amount) {
		const value = previews[index].imageZoom;
		updatePreview(index, { imageZoom: Math.max(0.25, Math.min(6, Number((value + amount).toFixed(2)))) });
	}

	function setSort(key) {
		if (sortKey === key) sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		else { sortKey = key; sortDirection = 'asc'; }
	}

	function handleSearchInput(event) {
		searchDraft = event.currentTarget.value;
		clearTimeout(searchDebounce);
		searchDebounce = setTimeout(() => searchQuery = searchDraft, 90);
	}

	function handleAuxSelect(item, event) {
		event.preventDefault();
		if (!mouseMiddleOtherPreview || item.is_dir) return;
		if (!splitMode) {
			splitMode = true;
			previewVisible = true;
			requestAnimationFrame(adjustLayout);
		}
		const target = 1 - activePreview;
		savePreferences(false);
		openPreview(item, target);
	}

	function updateFileSelection(paths, updatePrimary = true) {
		selectedPaths = paths;
		if (updatePrimary) selectedPath = paths.at(-1) || '';
	}

	function openContextMenu(event, item) {
		event.preventDefault();
		if (item && !selectedPaths.includes(item.path)) { selectedPath = item.path; selectedPaths = [item.path]; }
		contextItem = item;
		const menuWidth = 238;
		const menuHeight = item ? 340 : 160;
		contextMenu = {
			x: Math.max(4, Math.min(event.clientX, window.innerWidth - menuWidth - 4)),
			y: Math.max(4, Math.min(event.clientY, window.innerHeight - menuHeight - 4))
		};
	}

	function closeContextMenu() {
		contextMenu = null;
	}

	function getSelectedPreviewFiles(fallback = contextItem) {
		const filesByPath = new Map(files.map((file) => [file.path, file]));
		const selected = selectedPaths
			.map((path) => filesByPath.get(path))
			.filter((file) => file && !file.is_dir);
		if (!selected.length && fallback && !fallback.is_dir) selected.push(fallback);
		return [...new Map(selected.map((file) => [file.path, file])).values()];
	}

	async function openSelectionInPreview(index) {
		const items = getSelectedPreviewFiles();
		if (!items.length) return;
		previewVisible = true;
		activePreview = index;
		requestAnimationFrame(adjustLayout);
		await Promise.all(items.map((item) => openPreview(item, index, true)));
	}

	async function createFolder() {
		const name = window.prompt('New folder name');
		if (!name?.trim() || !currentPath) return;
		try {
			await filesystem.createDirectory(joinPath(currentPath, name.trim()));
			treeRefreshToken += 1;
			await refreshDirectory();
			statusMessage = 'Folder created.';
		} catch (error) {
			statusMessage = 'Could not create folder: ' + error;
		}
	}

	async function renameSelected() {
		if (!selectedFile) return;
		const name = window.prompt('Rename item', selectedFile.name);
		if (!name?.trim() || name.trim() === selectedFile.name) return;
		try {
			await filesystem.renamePath(selectedFile.path, name.trim());
			treeRefreshToken += 1;
			for (let index = 0; index < previews.length; index += 1) {
				for (const tab of [...previews[index].tabs]) if (tab.selectedFile?.path === selectedFile.path) closePreviewTab(index, tab.tabId);
			}
			await refreshDirectory();
			statusMessage = 'Item renamed.';
		} catch (error) {
			statusMessage = 'Could not rename item: ' + error;
		}
	}

	async function deleteSelected() {
		if (!selectedFile || !window.confirm('Delete "' + selectedFile.name + '"?')) return;
		try {
			await filesystem.deletePath(selectedFile.path);
			treeRefreshToken += 1;
			for (let index = 0; index < previews.length; index += 1) {
				for (const tab of [...previews[index].tabs]) if (tab.selectedFile?.path === selectedFile.path) closePreviewTab(index, tab.tabId);
			}
			await refreshDirectory();
			statusMessage = 'Item deleted.';
		} catch (error) {
			statusMessage = 'Could not delete item: ' + error;
		}
	}

	function toggleFavorite(path) {
		if (!path) return;
		favorites = favorites.includes(path) ? favorites.filter((favorite) => favorite !== path) : [...favorites, path];
		statusMessage = favorites.includes(path) ? 'Added to Quick access.' : 'Removed from Quick access.';
		savePreferences(false);
	}

	async function copyPath(path) {
		try {
			await navigator.clipboard.writeText(path);
			statusMessage = 'Path copied.';
		} catch {
			statusMessage = 'Could not copy the path.';
		}
	}

	async function handleContextAction(action) {
		closeContextMenu();
		if (action === 'open') {
			if (contextItem?.is_dir) await navigate(contextItem.path);
			else if (contextItem) await openSelectionInPreview(activePreview);
		} else if (action === 'preview-left' && contextItem) {
			splitMode = true; previewVisible = true; await openSelectionInPreview(0);
			savePreferences(false);
		} else if (action === 'preview-right' && contextItem) {
			splitMode = true; previewVisible = true; await openSelectionInPreview(1);
			savePreferences(false);
		} else if (action === 'open-external' && contextItem) {
			try { await openPath(contextItem.path); } catch (error) { statusMessage = 'Could not open item: ' + error; }
		} else if (action === 'copy-path' && contextItem) {
			await copyPath(contextItem.path);
		} else if (action === 'pin') {
			toggleFavorite(contextItem?.is_dir ? contextItem.path : currentPath);
		} else if (action === 'rename') {
			await renameSelected();
		} else if (action === 'delete') {
			await deleteSelected();
		} else if (action === 'new-folder') {
			await createFolder();
		} else if (action === 'refresh') {
			await refreshDirectory();
		} else if (action === 'up') {
			await goUp();
		} else if (action === 'toggle-view') {
			setViewMode(viewMode === 'list' ? 'grid' : 'list');
		}
	}

	function contextItems() {
		if (!contextItem) return [
			{ id: 'new-folder', label: 'New folder', icon: 'folder-plus' },
			{ id: 'refresh', label: 'Refresh', icon: 'refresh' },
			{ separator: true, id: 'sep-1' },
			{ id: 'up', label: 'Parent folder', icon: 'arrow-up' },
			{ id: 'toggle-view', label: viewMode === 'list' ? 'Show as icons' : 'Show as list', icon: viewMode === 'list' ? 'grid' : 'list' }
		];
		const items = [
			{ id: 'open', label: contextItem.is_dir ? 'Open folder' : 'Preview file', icon: contextItem.is_dir ? 'folder' : 'file' }
		];
		if (!contextItem.is_dir) {
			items.push({ id: 'preview-left', label: 'Preview in left pane', icon: 'split' });
			items.push({ id: 'preview-right', label: 'Preview in right pane', icon: 'split' });
		}
		items.push({ separator: true, id: 'sep-1' });
		items.push({ id: 'copy-path', label: 'Copy path', icon: 'copy' });
		items.push({ id: 'pin', label: favorites.includes(contextItem.is_dir ? contextItem.path : currentPath) ? 'Remove from Quick access' : 'Add to Quick access', icon: 'pin' });
		items.push({ separator: true, id: 'sep-2' });
		items.push({ id: 'open-external', label: 'Open with default app', icon: 'file' });
		items.push({ id: 'rename', label: 'Rename', icon: 'edit', shortcut: getShortcut('rename') });
		items.push({ id: 'delete', label: 'Delete', icon: 'trash', shortcut: getShortcut('delete') });
		return items;
	}

	function getShortcut(id) {
		const stored = shortcuts[id];
		const values = Array.isArray(stored) ? stored : [stored || ''];
		return values.filter(Boolean).join(' / ');
	}

	function startAddressEdit() {
		addressValue = currentPath;
		addressEditing = true;
		requestAnimationFrame(() => { addressInput?.focus(); addressInput?.select(); });
	}

	async function submitAddress(event) {
		event.preventDefault();
		const path = addressValue.trim();
		addressEditing = false;
		if (path) await navigate(path);
	}

	function canonicalKey(event) {
		const parts = [];
		if (event.ctrlKey || event.metaKey) parts.push('Ctrl');
		if (event.altKey) parts.push('Alt');
		if (event.shiftKey) parts.push('Shift');
		const names = { ArrowLeft: 'Left', ArrowRight: 'Right', ArrowUp: 'Up', ArrowDown: 'Down', ' ': 'Space' };
		parts.push(names[event.key] || (event.key.length === 1 ? event.key.toUpperCase() : event.key));
		return parts.join('+');
	}

	function handleGlobalKeydown(event) {
		if (event.defaultPrevented || event.repeat || settingsVisible || quickOpenVisible) return;
		const target = event.target;
		const editable = target instanceof HTMLElement && (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName));
		const key = canonicalKey(event);
		if (!editable || event.ctrlKey || event.metaKey || event.altKey) {
			for (const [id, value] of Object.entries(shortcuts)) {
				const bindings = Array.isArray(value) ? value : [value];
				if (!bindings.filter(Boolean).includes(key)) continue;
				event.preventDefault();
				if (id === 'search') { searchInput?.focus(); searchInput?.select(); }
				else if (id === 'quickOpen') openQuickOpen();
				else if (id === 'nextTab') movePreviewTab(1);
				else if (id === 'previousTab') movePreviewTab(-1);
				else if (id === 'editPath') startAddressEdit();
				else if (id === 'toggleSplit') toggleSplit();
				else if (id === 'toggleSidebar') toggleSidebar();
				else if (id === 'togglePreview') togglePreview();
				else if (id === 'newFolder') createFolder();
				else if (id === 'rename') renameSelected();
				else if (id === 'delete') deleteSelected();
				else if (id === 'previousItem') selectAdjacentFile(-1);
				else if (id === 'nextItem') selectAdjacentFile(1);
				else if (id === 'refresh') refreshDirectory();
				else if (id === 'back') goBack();
				else if (id === 'forward') goForward();
				else if (id === 'up') goUp();
				return;
			}
		}
		if (event.key === 'Escape') {
			if (contextMenu) closeContextMenu();
			else if (addressEditing) addressEditing = false;
			else if (searchQuery || searchDraft) { searchQuery = ''; searchDraft = ''; clearTimeout(searchDebounce); }
		} else if (event.key === 'Enter' && target === document.body && selectedFile?.is_dir) {
			navigate(selectedFile.path);
		}
	}

	function handleAuxClick(event) {
		if (!mouseSideNavigation) return;
		if (event.button === 3) { event.preventDefault(); goBack(); }
		if (event.button === 4) { event.preventDefault(); goForward(); }
	}

	function startResize(kind, event) {
		event.preventDefault();
		event.currentTarget?.setPointerCapture?.(event.pointerId);
		if (kind === 'sidebar') {
			const rect = sidebarElement?.getBoundingClientRect();
			resizing = { kind, left: rect?.left || 0 };
		} else if (kind === 'preview') {
			resizing = { kind };
		} else {
			const rect = previewAreaElement?.getBoundingClientRect();
			const vertical = window.matchMedia('(max-width: 680px)').matches;
			resizing = { kind, left: rect?.left || 0, width: rect?.width || 0, top: rect?.top || 0, height: rect?.height || 0, vertical };
		}
		document.body.style.cursor = resizing.kind === 'split' && resizing.vertical ? 'row-resize' : 'col-resize';
		document.body.style.userSelect = 'none';
	}

	function moveResize(event) {
		if (!resizing) return;
		if (resizing.kind === 'sidebar') {
			const minimumPreview = previewVisible ? (splitMode ? 240 : 160) : 0;
			const maximum = Math.max(150, window.innerWidth - minimumPreview - 120);
			sidebarWidth = Math.max(150, Math.min(maximum, event.clientX - resizing.left));
			requestAnimationFrame(adjustLayout);
		} else if (resizing.kind === 'preview') {
			const rect = contentElement.getBoundingClientRect();
			const minimum = splitMode ? 240 : 160;
			const maximum = Math.max(minimum, contentElement.clientWidth - 110);
			previewWidth = Math.max(minimum, Math.min(maximum, rect.right - event.clientX));
		} else {
			const width = Math.max(1, (resizing.vertical ? resizing.height : resizing.width) - 5);
			const leftWidth = resizing.vertical ? event.clientY - resizing.top : event.clientX - resizing.left;
			const minimum = Math.min(120, width / 2);
			splitRatio = Math.round(Math.max(minimum, Math.min(width - minimum, leftWidth)) / width * 100);
		}
	}

	function adjustLayout() {
		if (!contentElement || !previewVisible) return;
		const minimum = splitMode ? 240 : 160;
		const maximum = Math.max(minimum, contentElement.clientWidth - 110);
		previewWidth = Math.max(minimum, Math.min(maximum, previewWidth));
	}

	function stopResize() {
		if (!resizing) return;
		resizing = null;
		document.body.style.cursor = '';
		document.body.style.userSelect = '';
		requestAnimationFrame(() => { adjustLayout(); savePreferences(false); });
	}

	function settingsPayload() {
		return {
			start_path: startPath.trim() || null,
			hide_dotfiles: hideDotfiles,
			favorites,
			view_mode: viewMode,
			theme,
			sidebar_collapsed: sidebarCollapsed,
			sidebar_width: Math.round(sidebarWidth),
			preview_width: Math.round(previewWidth),
			split_ratio: splitRatio,
			split_mode: splitMode,
			split_path: null,
			mouse_middle_other_preview: mouseMiddleOtherPreview,
			mouse_side_navigation: mouseSideNavigation,
			shortcuts
		};
	}

	async function savePreferences(showMessage = true, applyStartPath = false) {
		try {
			if (applyStartPath && startPath.trim() && startPath.trim() !== currentPath) {
				const opened = await loadDirectory(startPath.trim());
				if (!opened) {
					settingsMessage = 'Could not open starting folder: ' + directoryError;
					return;
				}
			}
			await filesystem.saveSettings(settingsPayload());
			settingsMessage = showMessage ? 'Preferences saved.' : settingsMessage;
		} catch (error) {
			settingsMessage = 'Could not save preferences: ' + error;
		}
	}

	function closeSettings() {
		settingsVisible = false;
		settingsMessage = '';
	}

	onMount(async () => {
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		systemDarkMode = mediaQuery.matches;
		const updateSystemTheme = () => systemDarkMode = mediaQuery.matches;
		mediaQuery.addEventListener?.('change', updateSystemTheme);
		try {
			const loaded = await filesystem.loadSettings();
			const home = await filesystem.getHomeDirectory();
			homePath = home;
			startPath = loaded.start_path || home;
			hideDotfiles = loaded.hide_dotfiles ?? false;
			favorites = loaded.favorites || [];
			viewMode = loaded.view_mode || 'list';
			theme = loaded.theme || 'system';
			sidebarCollapsed = loaded.sidebar_collapsed ?? false;
			sidebarWidth = loaded.sidebar_width === 232 ? 190 : loaded.sidebar_width || 190;
			previewWidth = loaded.preview_width || 400;
			splitRatio = loaded.split_ratio || 50;
			splitMode = (loaded.split_mode ?? false) && !loaded.split_path;
			previewVisible = splitMode;
			mouseMiddleOtherPreview = loaded.mouse_middle_other_preview ?? loaded.mouse_middle_new_tab ?? true;
			mouseSideNavigation = loaded.mouse_side_navigation ?? true;
			shortcuts = mergeShortcuts(loaded.shortcuts);
			const opened = await loadDirectory(startPath);
			if (!opened && startPath !== homePath) {
				startPath = homePath;
				await loadDirectory(homePath);
				statusMessage = 'Starting folder was unavailable. Opened Home instead.';
			}
			requestAnimationFrame(adjustLayout);
		} catch (error) {
			directoryError = 'Could not start the folder browser: ' + error;
			try {
				homePath = await filesystem.getHomeDirectory();
				startPath = homePath;
				await loadDirectory(homePath);
			} catch (fallbackError) {
				directoryError = String(fallbackError);
			}
		}
	});
</script>

<svelte:window onpointermove={handleGlobalPointerMove} onpointerup={handleGlobalPointerUp} onpointercancel={handleGlobalPointerCancel} onpointerdown={(event) => { if (!event.target.closest?.('.context-menu')) closeContextMenu(); }} onkeydown={handleGlobalKeydown} onauxclick={handleAuxClick} onresize={adjustLayout} />

<main class:theme-dark={darkMode} class="app-shell">
	<div class="sidebar-holder" bind:this={sidebarElement} style={'width:' + (sidebarCollapsed ? 64 : sidebarWidth) + 'px'}>
	<ExplorerSidebar currentPath={currentPath} {favorites} {homePath} collapsed={sidebarCollapsed} {sidebarWidth} {treeRefreshToken} onNavigate={navigate} onToggleCollapse={toggleSidebar} />
	</div>
	{#if !sidebarCollapsed}<div class="sidebar-resizer resize-handle" role="separator" aria-label="Resize folder explorer" onpointerdown={(event) => startResize('sidebar', event)}></div>{/if}

	<section class="main-column">
		<header class="command-bar">
			<div class="navigation-tools">
				<button type="button" title="Back" aria-label="Back" disabled={historyIndex <= 0} onclick={goBack}><Icon name="chevron-left" size={16} /></button>
				<button type="button" title="Forward" aria-label="Forward" disabled={historyIndex >= historyPaths.length - 1} onclick={goForward}><Icon name="chevron-right" size={16} /></button>
				<button type="button" title="Parent folder" aria-label="Parent folder" onclick={goUp}><Icon name="arrow-up" size={16} /></button>
				<button type="button" title="Refresh" aria-label="Refresh" onclick={refreshDirectory}><Icon name="refresh" size={15} /></button>
			</div>
			{#if addressEditing}
				<form class="address-bar editing" onsubmit={submitAddress}>
					<input bind:this={addressInput} bind:value={addressValue} aria-label="Folder path" onblur={() => setTimeout(() => addressEditing = false, 100)} />
				</form>
			{:else}
				<div class="address-bar" role="group" aria-label="Current folder" ondblclick={startAddressEdit}>
					{#each breadcrumbs as crumb, index (crumb.path)}
						{#if index > 0}<span class="crumb-separator">\</span>{/if}
						<button type="button" title={crumb.path} onclick={() => navigate(crumb.path)}>{crumb.name}</button>
					{/each}
					<button class="address-edit" type="button" title="Edit location (Ctrl+L)" aria-label="Edit location" onclick={startAddressEdit}><Icon name="edit" size={14} /></button>
				</div>
			{/if}
			<label class="search-box"><Icon name="search" size={15} /><input bind:this={searchInput} value={searchDraft} oninput={handleSearchInput} placeholder="Search this folder" aria-label="Search this folder" />{#if getShortcut('search')}<kbd>{getShortcut('search')}</kbd>{/if}</label>
		</header>

		<div class="command-row">
			<div class="command-group">
				<button type="button" title={'Open file in a preview ' + getShortcut('quickOpen')} onclick={() => openQuickOpen()}><Icon name="search" size={15} /><span>Open file</span></button>
				<button type="button" title="Create folder" onclick={createFolder}><Icon name="folder-plus" size={15} /><span>New folder</span></button>
				<button class:pressed={previewVisible} type="button" title={'Show or hide preview pane ' + getShortcut('togglePreview')} onclick={togglePreview}><Icon name="file" size={15} /><span>Preview</span></button>
				<button class:pressed={splitMode} type="button" title={'Show two file previews ' + getShortcut('toggleSplit')} onclick={toggleSplit}><Icon name="split" size={15} /><span>{splitMode ? 'Single preview' : 'Split preview'}</span></button>
			</div>
			<div class="command-group view-tools">
				<button class:pressed={viewMode === 'list'} type="button" title="List view" aria-label="List view" onclick={() => setViewMode('list')}><Icon name="list" size={16} /></button>
				<button class:pressed={viewMode === 'grid'} type="button" title="Icon view" aria-label="Icon view" onclick={() => setViewMode('grid')}><Icon name="grid" size={16} /></button>
				<button type="button" title="Preferences" aria-label="Preferences" onclick={() => settingsVisible = true}><Icon name="settings" size={16} /></button>
			</div>
		</div>

		<div class:with-preview={previewVisible} class:split-preview={previewVisible && splitMode} class="workspace-content" bind:this={contentElement} style={'--preview-width:' + previewWidth + 'px'}>
			<FileBrowser files={orderedFiles} {selectedPath} {selectedPaths} {viewMode} {sortKey} sortDirection={sortDirection} hasFilters={!!searchQuery} {loading} error={directoryError} onSelect={selectItem} onSort={setSort} onContextMenu={openContextMenu} onAuxSelect={handleAuxSelect} onSelectionChange={updateFileSelection} />
			{#if previewVisible}
				<div class="preview-resizer resize-handle" role="separator" aria-label="Resize preview area" onpointerdown={(event) => startResize('preview', event)}></div>
				<div class:split={splitMode} class="preview-area" bind:this={previewAreaElement} style={'--split-left:' + splitRatio + '%'}>
				<PreviewPane
					{...previews[mainPreviewIndex]}
					active={activePreview === mainPreviewIndex}
					paneIndex={mainPreviewIndex}
					paneTitle={splitMode ? 'Preview 1' : 'Preview'}
					incomingTabDrag={!!draggedPreviewTab && draggedPreviewTab.sourceIndex !== mainPreviewIndex}
					onTabPointerDown={beginPreviewTabDrag}
					onActivate={() => activePreview = mainPreviewIndex}
					onOpenFile={() => openQuickOpen(mainPreviewIndex)}
					onSelectTab={(tabId) => activatePreviewTab(mainPreviewIndex, tabId)}
					onCloseTab={(tabId) => closePreviewTab(mainPreviewIndex, tabId)}
					onClose={() => closePreview(mainPreviewIndex)}
					onPreviewError={(message, path) => setPreviewError(mainPreviewIndex, message, path)}
					onZoomIn={() => zoom(mainPreviewIndex, 0.25)}
					onZoomOut={() => zoom(mainPreviewIndex, -0.25)}
					onZoomReset={() => updatePreview(mainPreviewIndex, { imageZoom: 1 })}
				/>
					{#if splitMode}
						<div class="split-resizer resize-handle" role="separator" aria-label="Resize preview panes" onpointerdown={(event) => startResize('split', event)}></div>
						<PreviewPane {...previews[1]} active={activePreview === 1} paneIndex={1} paneTitle="Preview 2" incomingTabDrag={!!draggedPreviewTab && draggedPreviewTab.sourceIndex !== 1} onTabPointerDown={beginPreviewTabDrag} onActivate={() => activePreview = 1} onOpenFile={() => openQuickOpen(1)} onSelectTab={(tabId) => activatePreviewTab(1, tabId)} onCloseTab={(tabId) => closePreviewTab(1, tabId)} onClose={() => closePreview(1)} onPreviewError={(message, path) => setPreviewError(1, message, path)} onZoomIn={() => zoom(1, 0.25)} onZoomOut={() => zoom(1, -0.25)} onZoomReset={() => updatePreview(1, { imageZoom: 1 })} />
					{/if}
				</div>
			{/if}
		</div>

		<footer class="status-bar">
			<span>{orderedFiles.length} items{searchQuery ? ' found' : ''}</span>
			{#if selectedPaths.length}<span>{selectedPaths.length} selected</span>{/if}
			{#if selectedFile}<span>{selectedFile.name}</span>{/if}
			<span class="status-message" role="status">{statusMessage || (directoryError ? 'Folder could not be loaded.' : pathLabel(currentPath))}</span>
		</footer>
	</section>
</main>

{#if contextMenu}
	<ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextItems()} darkMode={darkMode} onAction={handleContextAction} />
{/if}

{#if settingsVisible}
	<SettingsDialog bind:startPath bind:hideDotfiles bind:viewMode bind:theme bind:shortcuts bind:mouseMiddleOtherPreview bind:mouseSideNavigation {darkMode} message={settingsMessage} onSave={() => savePreferences(true, true)} onClose={closeSettings} />
{/if}

{#if quickOpenVisible}
	<QuickOpenDialog currentPath={currentPath} {homePath} {favorites} {recentFiles} initialPane={activePreview} {splitMode} {darkMode} onOpen={(item, index) => { quickOpenVisible = false; openPreview(item, index); }} onClose={() => quickOpenVisible = false} />
{/if}

<style>
	.app-shell { width: 100vw; height: 100vh; min-width: 0; min-height: 0; display: flex; overflow: hidden; color: var(--ink); background: var(--canvas); }
	.main-column { min-width: 0; min-height: 0; flex: 1; display: flex; flex-direction: column; overflow: hidden; }
	.command-bar, .command-row, .status-bar { display: flex; flex: 0 0 auto; align-items: center; border-bottom: 1px solid var(--line); background: var(--paper); }
	.command-bar { min-height: 42px; gap: 8px; padding: 5px 9px; }
	.navigation-tools, .command-group { display: flex; flex: 0 0 auto; align-items: center; gap: 2px; }
	.navigation-tools button, .command-group button { min-width: 29px; height: 29px; display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 0 7px; border: 1px solid transparent; border-radius: 3px; color: var(--ink); background: transparent; font-size: 11px; cursor: pointer; }
	.navigation-tools button:hover, .command-group button:hover { border-color: var(--line); background: var(--canvas); }
	.navigation-tools button:disabled { opacity: .4; cursor: default; }
	.command-row button.pressed { color: var(--accent-strong); background: var(--accent-soft); }
	.address-bar { min-width: 0; height: 29px; flex: 1; display: flex; align-items: center; gap: 2px; overflow: hidden; padding: 0 5px; border: 1px solid var(--line); border-radius: 3px; background: var(--paper); }
	.address-bar button { min-width: 0; flex: 0 1 auto; overflow: hidden; padding: 3px 4px; border: 0; color: var(--ink); background: transparent; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; cursor: pointer; }
	.address-bar button:hover { background: var(--canvas); }
	.address-bar .crumb-separator { flex: 0 0 auto; color: var(--muted); font-size: 11px; }
	.address-bar .address-edit { margin-left: auto; }
	.address-bar.editing { padding: 0 5px; }
	.address-bar input { width: 100%; height: 26px; border: 0; outline: 0; color: var(--ink); background: var(--paper); font-size: 11px; }
	.search-box { width: min(250px, 28vw); min-width: 130px; height: 29px; display: flex; align-items: center; gap: 6px; padding: 0 7px; border: 1px solid var(--line); border-radius: 3px; color: var(--muted); background: var(--paper); }
	.search-box input { width: 100%; min-width: 0; border: 0; outline: 0; color: var(--ink); background: transparent; font-size: 11px; }
	.search-box kbd { flex: 0 0 auto; color: var(--muted); font-size: 9px; }
	.command-row { min-height: 37px; justify-content: space-between; padding: 3px 8px; }
	.command-group button { height: 27px; }
	.view-tools { gap: 0; }
	.workspace-content { min-width: 0; min-height: 0; flex: 1; display: grid; grid-template-columns: minmax(0, 1fr); overflow: hidden; }
	.workspace-content.with-preview { grid-template-columns: minmax(0, 1fr) 5px var(--preview-width, 400px); }
	.preview-area { min-width: 0; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr); gap: 0; overflow: hidden; background: var(--line); }
	.preview-area.split { grid-template-columns: minmax(0, calc(var(--split-left, 50%) - 2.5px)) 5px minmax(0, 1fr); }
	.resize-handle { z-index: 2; touch-action: none; background: transparent; }
	.resize-handle:hover, .resize-handle:active { background: var(--focus); }
	.sidebar-holder { height: 100%; flex: 0 0 auto; overflow: hidden; }
	.sidebar-resizer, .preview-resizer, .split-resizer { width: 5px; cursor: col-resize; }
	.status-bar { min-height: 23px; gap: 14px; padding: 2px 9px; border-top: 1px solid var(--line); border-bottom: 0; color: var(--muted); font-size: 10px; }
	.status-message { min-width: 0; margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	@media (max-width: 850px) {
		.command-bar { gap: 4px; padding-inline: 4px; }
		.search-box { width: 150px; min-width: 100px; }
		.navigation-tools button { min-width: 25px; }
	}
	@media (max-width: 680px) {
		.command-group button span { display: none; }
		.preview-area.split { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, calc(var(--split-left, 50%) - 2.5px)) 5px minmax(0, 1fr); }
		.split-resizer { width: auto; height: 5px; cursor: row-resize; }
	}
</style>
