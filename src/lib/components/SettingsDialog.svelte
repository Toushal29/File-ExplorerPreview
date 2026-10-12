<script>
	import Icon from './Icon.svelte';

	export let startPath = '';
	export let hideDotfiles = false;
	export let viewMode = 'list';
	export let theme = 'system';
	export let darkMode = false;
	export let shortcuts = {};
	export let mouseMiddleOtherPreview = true;
	export let mouseSideNavigation = true;
	export let message = '';
	export let onSave = () => {};
	export let onClose = () => {};

	let recording = null;
	const shortcutActions = [
		{ id: 'search', label: 'Search files' },
		{ id: 'quickOpen', label: 'Open file picker' },
		{ id: 'nextTab', label: 'Next preview tab' },
		{ id: 'previousTab', label: 'Previous preview tab' },
		{ id: 'editPath', label: 'Edit location' },
		{ id: 'toggleSplit', label: 'Split previews' },
		{ id: 'toggleSidebar', label: 'Sidebar' },
		{ id: 'togglePreview', label: 'Preview pane' },
		{ id: 'newFolder', label: 'New folder' },
		{ id: 'rename', label: 'Rename selected file' },
		{ id: 'delete', label: 'Delete selected file' },
		{ id: 'previousItem', label: 'Previous item' },
		{ id: 'nextItem', label: 'Next item' },
		{ id: 'refresh', label: 'Refresh' },
		{ id: 'back', label: 'Back' },
		{ id: 'forward', label: 'Forward' },
		{ id: 'up', label: 'Parent folder' }
	];

	function bindingsFor(id) {
		const stored = shortcuts[id];
		return Array.isArray(stored) ? [stored[0] || '', stored[1] || ''] : [stored || '', ''];
	}

	function recordShortcut(event) {
		if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); recording = null; return; }
		if (['Control', 'Alt', 'Shift', 'Meta', 'Tab'].includes(event.key)) return;
		event.preventDefault();
		event.stopPropagation();
		const parts = [];
		if (event.ctrlKey || event.metaKey) parts.push('Ctrl');
		if (event.altKey) parts.push('Alt');
		if (event.shiftKey) parts.push('Shift');
		const names = { ArrowLeft: 'Left', ArrowRight: 'Right', ArrowUp: 'Up', ArrowDown: 'Down', ' ': 'Space' };
		parts.push(names[event.key] || (event.key.length === 1 ? event.key.toUpperCase() : event.key));
		const [id, slot] = recording;
		const next = bindingsFor(id);
		next[slot] = parts.join('+');
		shortcuts = { ...shortcuts, [id]: next };
		recording = null;
	}

	function clearShortcut(id, slot) {
		const next = bindingsFor(id);
		next[slot] = '';
		shortcuts = { ...shortcuts, [id]: next };
	}
</script>

<div class:dark={darkMode} class="dialog-backdrop" role="presentation" onclick={(event) => event.target === event.currentTarget && onClose()}>
	<section class="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title">
		<header class="dialog-header"><h2 id="settings-title">Preferences</h2><button type="button" class="dialog-close" aria-label="Close preferences" onclick={onClose}><Icon name="close" size={17} /></button></header>

		<label class="setting-field"><span>Appearance</span><select bind:value={theme}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
		<label class="setting-field"><span>Starting folder</span><input class="text-input" type="text" bind:value={startPath} placeholder="C:\Users\You\Documents" /></label>
		<div class="setting-row"><span>Hide hidden files</span><input type="checkbox" bind:checked={hideDotfiles} /></div>
		<label class="setting-field"><span>Default layout</span><select bind:value={viewMode}><option value="list">List</option><option value="grid">Grid</option></select></label>
		<div class="setting-row"><span>Middle-click previews in the other pane</span><input type="checkbox" bind:checked={mouseMiddleOtherPreview} /></div>
		<div class="setting-row"><span>Mouse side buttons navigate history</span><input type="checkbox" bind:checked={mouseSideNavigation} /></div>

		<details class="shortcut-details" open>
			<summary>Keyboard shortcuts <small>Each action can have two bindings.</small></summary>
			<div class="shortcut-head"><span>Action</span><span>Option 1</span><span>Option 2</span></div>
			<div class="shortcut-list">
				{#each shortcutActions as action (action.id)}
					<div class="shortcut-row"><span>{action.label}</span>
						{#each [0, 1] as slot}
							<div class="shortcut-input"><input readonly value={recording?.[0] === action.id && recording?.[1] === slot ? 'Press keys…' : bindingsFor(action.id)[slot] || 'Not set'} aria-label={`Shortcut option ${slot + 1} for ${action.label}`} title="Click to record a shortcut" onfocus={() => recording = [action.id, slot]} onkeydown={recordShortcut} onblur={() => recording = null} /><button type="button" aria-label={`Clear shortcut option ${slot + 1} for ${action.label}`} onclick={() => clearShortcut(action.id, slot)}>×</button></div>
						{/each}
					</div>
				{/each}
			</div>
		</details>

		{#if message}<p class="settings-message" role="status">{message}</p>{/if}
		<footer class="dialog-footer"><button class="cancel-button" type="button" onclick={onClose}>Close</button><button class="save-button" type="button" onclick={onSave}>Save preferences</button></footer>
	</section>
</div>

<style>
	.dialog-backdrop { position: fixed; z-index: 20; inset: 0; display: grid; place-items: center; padding: 16px; background: #0005; }
	.settings-dialog { width: min(840px, 100%); max-height: calc(100vh - 32px); overflow: auto; padding: 20px; border: 1px solid var(--line); border-radius: 6px; color: var(--ink); background: var(--paper); }
	.dialog-header, .setting-row, .dialog-footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
	.dialog-header { margin-bottom: 16px; }
	h2 { margin: 0; font-size: 17px; }
	.dialog-close { width: 30px; height: 30px; border: 1px solid var(--line); border-radius: 5px; background: var(--paper); cursor: pointer; }
	.setting-field { display: grid; grid-template-columns: 140px minmax(0, 1fr); align-items: center; gap: 12px; margin: 11px 0; font-size: 12px; }
	.setting-field select, .text-input { min-width: 0; min-height: 32px; padding: 5px 8px; border: 1px solid var(--line); border-radius: 5px; color: var(--ink); background: var(--paper); }
	.setting-row { min-height: 34px; border-top: 1px solid var(--line); font-size: 12px; }
	.shortcut-details { margin-top: 14px; border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); }
	.shortcut-details summary { padding: 12px 0; font-size: 13px; font-weight: 600; cursor: pointer; }
	.shortcut-details summary small { margin-left: 8px; color: var(--muted); font-size: 11px; font-weight: 400; }
	.shortcut-head, .shortcut-row { display: grid; grid-template-columns: minmax(86px, 1.2fr) minmax(65px, 1fr) minmax(65px, 1fr); align-items: center; gap: 6px; }
	.shortcut-head { padding: 6px 0; color: var(--muted); font-size: 10px; }
	.shortcut-list { display: grid; grid-template-columns: 1fr 1fr; gap: 5px 18px; padding-bottom: 12px; }
	.shortcut-row { min-width: 0; font-size: 10px; }
	.shortcut-input { min-width: 0; display: flex; }
	.shortcut-input input { width: 100%; min-width: 0; height: 28px; padding: 0 5px; border: 1px solid var(--line); border-right: 0; border-radius: 4px 0 0 4px; color: var(--accent-strong); background: var(--canvas); text-align: center; font-size: 10px; cursor: pointer; }
	.shortcut-input button { width: 24px; height: 28px; border: 1px solid var(--line); border-radius: 0 4px 4px 0; color: var(--muted); background: var(--paper); cursor: pointer; }
	.settings-message { color: var(--muted); font-size: 12px; }
	.dialog-footer { justify-content: flex-end; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--line); }
	.dialog-footer button { min-height: 32px; padding: 0 12px; border: 1px solid var(--line); border-radius: 5px; color: var(--ink); background: var(--paper); font-size: 11px; cursor: pointer; }
	.dialog-footer .save-button { border-color: var(--accent); color: white; background: var(--accent); }
	@media (max-width: 560px) { .shortcut-list { grid-template-columns: 1fr; } .shortcut-head { display: none; } }
</style>
