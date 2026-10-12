<script>
	export let name;
	export let size = 18;
	export let strokeWidth = 1.8;
</script>

<span class="icon" aria-hidden="true">
	<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={strokeWidth} stroke-linecap="round" stroke-linejoin="round">
		{#if name === 'folder'}
			<path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H10l2 2h6.5A2.5 2.5 0 0 1 21 9.5v8a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17.5z" /><path d="M3 10h18" />
		{:else if name === 'home'}
			<path d="m3 10 9-7 9 7" /><path d="M5 9v11h14V9M9 20v-7h6v7" />
		{:else if name === 'star'}
			<path d="m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9z" />
		{:else if name === 'settings'}
			<circle cx="12" cy="12" r="3" /><path d="m19.4 15 .1.1 1.3 1-1.3 2.3-1.6-.5a8 8 0 0 1-1.8 1l-.3 1.7h-2.7l-.3-1.7a8 8 0 0 1-1.8-1l-1.6.5-1.3-2.3 1.3-1a7 7 0 0 1 0-2l-1.3-1 1.3-2.3 1.6.5a8 8 0 0 1 1.8-1l.3-1.7h2.7l.3 1.7a8 8 0 0 1 1.8 1l1.6-.5 1.3 2.3-1.3 1a7 7 0 0 1-.1 2Z" transform="translate(-1 -1)" />
		{:else if name === 'chevron-left'}
			<path d="m15 18-6-6 6-6" />
		{:else if name === 'chevron-right'}
			<path d="m9 18 6-6-6-6" />
		{:else if name === 'chevron-up'}
			<path d="m18 15-6-6-6 6" />
		{:else if name === 'refresh'}
			<path d="M20 7v5h-5M4 17v-5h5" /><path d="M5.5 9a7 7 0 0 1 12-2L20 12M4 12l2.5 5a7 7 0 0 0 12-2" />
		{:else if name === 'search'}
			<circle cx="10.8" cy="10.8" r="6.8" /><path d="m16 16 4.5 4.5" />
		{:else if name === 'list'}
			<path d="M9 6h11M9 12h11M9 18h11" /><path d="M4 6h.01M4 12h.01M4 18h.01" />
		{:else if name === 'grid'}
			<rect x="4" y="4" width="6" height="6" rx="1" /><rect x="14" y="4" width="6" height="6" rx="1" /><rect x="4" y="14" width="6" height="6" rx="1" /><rect x="14" y="14" width="6" height="6" rx="1" />
		{:else if name === 'file'}
			<path d="M6 3h8l5 5v13H6z" /><path d="M14 3v5h5M9 13h7M9 17h7" />
		{:else if name === 'image'}
			<rect x="3" y="4" width="18" height="16" rx="2" /><circle cx="8.5" cy="9" r="1.5" /><path d="m21 15-5-5L5 20" />
		{:else if name === 'video'}
			<rect x="3" y="5" width="13" height="14" rx="2" /><path d="m16 10 5-3v10l-5-3z" />
		{:else if name === 'music'}
			<path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />
		{:else if name === 'pdf'}
			<path d="M6 3h8l5 5v13H6z" /><path d="M14 3v5h5M9 13h6M9 17h6" />
		{:else if name === 'close'}
			<path d="m18 6-12 12M6 6l12 12" />
		{:else if name === 'zoom-in'}
			<circle cx="10.8" cy="10.8" r="6.8" /><path d="m16 16 4.5 4.5M10.8 7.8v6M7.8 10.8h6" />
		{:else if name === 'zoom-out'}
			<circle cx="10.8" cy="10.8" r="6.8" /><path d="m16 16 4.5 4.5M7.8 10.8h6" />
		{:else if name === 'arrow-up'}
			<path d="M12 19V5M5 12l7-7 7 7" />
		{:else if name === 'pin'}
			<path d="m16 3 5 5-4 1-4 4-1 4-2-2-6 6M8 7l9 9" />
		{:else if name === 'clock'}
			<circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" />
		{:else if name === 'back'}
			<path d="M9 14 4 9l5-5" /><path d="M4 9h9a7 7 0 0 1 7 7v2" />
		{:else if name === 'more'}
			<circle cx="5" cy="12" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" />
		{:else if name === 'edit'}
			<path d="m14 5 5 5M4 20l4.5-1 11-11a2.1 2.1 0 0 0-3-3l-11 11z" />
		{:else if name === 'split'}
			<rect x="3" y="4" width="18" height="16" rx="2" /><path d="M12 4v16M7 8h2M15 8h2M7 12h2M15 12h2M7 16h2M15 16h2" />
		{:else if name === 'plus'}
			<path d="M12 5v14M5 12h14" />
		{:else if name === 'copy'}
			<rect x="8" y="8" width="12" height="12" rx="2" /><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3" />
		{:else if name === 'trash'}
			<path d="M4 7h16M10 11v6M14 11v6M6 7l1 14h10l1-14M9 7V4h6v3" />
		{:else if name === 'folder-plus'}
			<path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H10l2 2h6.5A2.5 2.5 0 0 1 21 9.5v8a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17.5z" /><path d="M12 11v6M9 14h6" />
		{:else if name === 'sun'}
			<circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" />
		{:else if name === 'moon'}
			<path d="M20.5 15.5A8.5 8.5 0 0 1 8.5 3.5 8.5 8.5 0 1 0 20.5 15.5Z" />
		{:else if name === 'monitor'}
			<rect x="3" y="4" width="18" height="13" rx="2" /><path d="M8 21h8M12 17v4" />
		{/if}
	</svg>
</span>
