<script>
	import Icon from './Icon.svelte';
	export let x = 0;
	export let y = 0;
	export let items = [];
	export let darkMode = false;
	export let onAction = () => {};
</script>

<div class:dark={darkMode} class="context-menu" style={`left:${x}px;top:${y}px`} role="menu" aria-label="File options" oncontextmenu={(event) => event.preventDefault()}>
	{#each items as item (item.id)}
		{#if item.separator}<div class="menu-separator"></div>{:else}<button type="button" role="menuitem" disabled={item.disabled} onclick={() => onAction(item.id)}><Icon name={item.icon || 'file'} size={15} /><span>{item.label}</span>{#if item.shortcut}<kbd>{item.shortcut}</kbd>{/if}</button>{/if}
	{/each}
</div>

<style>
	.context-menu { position: fixed; z-index: 40; width: 230px; padding: 4px; border: 1px solid var(--line); border-radius: 4px; color: var(--ink); background: var(--paper); box-shadow: 0 4px 14px #0003; }
	.context-menu button { width: 100%; min-height: 29px; display: flex; align-items: center; gap: 9px; padding: 0 8px; border: 0; border-radius: 2px; color: inherit; background: transparent; text-align: left; font-size: 11px; cursor: pointer; }
	.context-menu button:hover:not(:disabled) { background: var(--hover); }
	.context-menu button:disabled { opacity: .45; cursor: default; }
	.context-menu button :global(.icon) { color: var(--icon); }
	.context-menu button span:not(.icon) { flex: 1; }
	.context-menu kbd { padding: 2px 4px; border: 1px solid var(--line); border-radius: 4px; color: var(--muted); background: var(--canvas); font-size: 8px; }
	.menu-separator { height: 1px; margin: 3px 4px; background: var(--line); }
</style>
