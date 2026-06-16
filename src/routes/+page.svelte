<script>
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	import { HighlightAuto } from "svelte-highlight";
	// import 'svelte-highlight/styles/github.css';
	import 'svelte-highlight/styles/atom-one-dark.css';

	/** @typedef {{ name: string, path: string, is_dir: boolean, file_type: string, size: number, modified: number | null }} FileEntry */
	/** @typedef {'text' | 'image' | 'pdf' | 'audio' | 'video' | 'unsupported' | null} PreviewType */
	/** @typedef {{ start_path: string | null, hide_dotfiles: boolean }} AppSettings */

	let currentPath = '';
	let files = /** @type {FileEntry[]} */ ([]);
	let selectedFile = /** @type {FileEntry | null} */ (null);
	let previewType = /** @type {PreviewType} */ (null);
	let previewError = '';
	let textContent = '';
	let settingsStartPath = '';
	let hideDotfiles = false;
	let settingsMessage = '';
	let imageZoom = 1;
	let settingsVisible = false;
	let previewVisible = false;

	let imageUrl = /** @type {string | null} */ (null);
	let pdfUrl = /** @type {string | null} */ (null);
	let audioUrl = /** @type {string | null} */ (null);
	let videoUrl = /** @type {string | null} */ (null);

	$: visibleFiles = hideDotfiles
		? files.filter((file) => !file.name.startsWith('.'))
		: files;

	function clearPreview() {
		textContent = '';
		imageUrl = null;
		pdfUrl = null;
		audioUrl = null;
		videoUrl = null;
		previewError = '';
		imageZoom = 1;
	}

	/**
	 * @param {string} path
	 */
	function mediaUrl(path) {
		return convertFileSrc(path);
	}

	// file size
	/**
	 * @param {number} size
	 */
	function formatSize(size) {
		if (size === 0) {
			return '-';
		}

		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let value = size;
		let unitIndex = 0;

		while (value >= 1024 && unitIndex < units.length - 1) {
			value /= 1024;
			unitIndex += 1;
		}

		return `${value >= 10 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`;
	}

	// date time
	/**
	 * @param {number | null} modified
	 */
	function formatDate(modified) {
		if (!modified) {
			return '-';
		}

		return new Date(modified * 1000).toLocaleString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Image functions
	function zoomIn() {
		imageZoom = Math.min(6, Number((imageZoom + 0.25).toFixed(2)));
	}

	function zoomOut() {
		imageZoom = Math.max(0.25, Number((imageZoom - 0.25).toFixed(2)));
	}

	function resetZoom() {
		imageZoom = 1;
	}

	/**
   * @param {{ deltaY: number; }} e
   */
	function mouseZoom(e) {
		if (e.deltaY < 0) {
                zoomIn();
        } else {
                zoomOut();
        }
    }


	// Setting container
	function toggleSettings() {
		settingsVisible = !settingsVisible;
	}

	function togglePreview() {
		previewVisible = !previewVisible;
	}

	/**
	 * @param {string} path
	 */
	async function loadDirectory(path, addToHistory = true) {
		// selectedFile = null;
		// previewType = null;
		// clearPreview();

		try {
			files = await invoke('list_directory', { path });
			currentPath = path;
			settingsMessage = '';
		}
		catch (err) {
			previewError = `Could not open folder: ${err}`;
		}
	}

	/**
	 * @param {FileEntry} item
	 */
	async function openItem(item) {
		if (item.is_dir) {
			await loadDirectory(item.path);
			return;
		}

		selectedFile = item;
		previewVisible = true
		clearPreview();

		try {
			const exists = await invoke('file_exists', { path: item.path });

			if (!exists) {
				previewType = 'unsupported';
				previewError = 'This file no longer exists or cannot be read.';
				return;
			}

			const info = /** @type {{ preview_type: Exclude<PreviewType, null> }} */ (
				await invoke('get_preview_type', { path: item.path })
			);
			previewType = info.preview_type;

			if (previewType === 'text') {
				textContent = await invoke('read_text_file', { path: item.path });
			}
			else if (previewType === 'image') {
				imageUrl = mediaUrl(item.path);
			}
			else if (previewType === 'pdf') {
				pdfUrl = mediaUrl(item.path);
			}
			else if (previewType === 'audio') {
				audioUrl = mediaUrl(item.path);
			}
			else if (previewType === 'video') {
				videoUrl = mediaUrl(item.path);
			}
			else {
				previewError = 'No preview is available for this file type.';
			}
		}
		catch (err) {
			previewType = 'unsupported';
			previewError = `Could not load preview: ${err}`;
		}
	}

	// go back up directory
	async function goBack() {
		const parts = currentPath.split('\\');

		if (parts.length > 2) {
			parts.pop();
			await loadDirectory(parts.join('\\'));
		}
	}

	// refresh function
	async function refreshDirectory() {
		if (currentPath) {
			await loadDirectory(currentPath);
		}
	}

	// saving the user state - path + hidden files
	async function saveSettings() {
		const startPath = settingsStartPath.trim();
		const settings = /** @type {AppSettings} */ ({
			start_path: startPath ? startPath : null,
			hide_dotfiles: hideDotfiles
		});

		try {
			await invoke('save_settings', { settings });
			settingsMessage = 'Settings saved.';
		}
		catch (err) {
			settingsMessage = `Could not save settings: ${err}`;
		}
	}

	onMount(async () => {
		try {
			const homeDir = /** @type {string} */ (await invoke('get_home_dir'));
			const settings = /** @type {AppSettings} */ (await invoke('load_settings'));
			settingsStartPath = settings.start_path || homeDir;
			hideDotfiles = settings.hide_dotfiles;

			await loadDirectory(settingsStartPath);
		}
		catch (err) {
			previewError = `Could not load settings: ${err}`;
		}
	});
</script>

<main class="container">
	<header class="path-header">
		<p>PATH -- {currentPath}</p>
	</header>
	<div class="path-actions">
		<button class="tool-button" type="button" title="Back" onclick={goBack}>↩︎</button>
		<button class="tool-button" type="button" title="Refresh Directory" onclick={refreshDirectory}>⟳</button>
		<button class="tool-button" type="button" title="Show/Hide Preview Panel" onclick={togglePreview}>
			{previewVisible ? '◨' : '◫'}
		</button>
		<button class="tool-button" type="button" title="Settings" onclick={toggleSettings} >⋮</button>
	</div>

	{#if settingsVisible}
		<section class="settings-panel" aria-label="Settings">
			<label class="start-path-field">
				<span>Start path</span>
				<input
					type="text"
					placeholder="C:\Users\YourName\Documents"
					bind:value={settingsStartPath}
				/>
			</label>
			<label class="checkbox-field">
				<span>Hide files and folders starting with "."</span>
				<input type="checkbox" bind:checked={hideDotfiles} />			
			</label>
			<span>Save </span>
			<button class="save-button" type="button" title="Save" onclick={saveSettings}>🖫</button>

			{#if settingsMessage}
				<p class="settings-message">{settingsMessage}</p>
			{/if}
		</section>
	{/if}

	<div class="explorer-container">
		<section class="file-list" aria-label="Files">
			<div class="table-header">
				<span>NAME</span>
				<span>TYPE</span>
				<span>SIZE</span>
				<span>LAST MODIFIED</span>
			</div>

			<ul>
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<li class="file-row" style="font-weight: bolder;" onclick={goBack} title="Go Back up">...</li>
				{#each visibleFiles as item}
					<li class:selected={selectedFile?.path === item.path}>
						<button class="file-row" type="button" onclick={() => openItem(item)}>
							<span class="file-name-cell">
								<span class:folder-icon={item.is_dir} class:file-icon={!item.is_dir}></span>
								<span class="file-name">{item.name}</span>
							</span>
							<span class="file-type">{item.file_type}</span>
							<span class="file-size">{item.is_dir ? '-' : formatSize(item.size)}</span>
							<span class="file-modified">{formatDate(item.modified)}</span>
						</button>
					</li>
				{/each}
			</ul>

			{#if visibleFiles.length === 0}
				<p class="empty-list">No files to show.</p>
			{/if}
		</section>

		{#if previewVisible}
			<aside class="preview-panel" aria-label="Preview">
				{#if selectedFile}
					{#if previewType === 'image'}
						<div class="zoom-toolbar" aria-label="Image zoom">
							<button type="button" onclick={zoomOut}>-</button>
							<span>{Math.round(imageZoom * 100)}%</span>
							<button type="button" onclick={zoomIn}>+</button>
							<button type="button" onclick={resetZoom}>⟲</button>
						</div>
					{/if}

					<div
						class:preview-stage={previewType !== 'text'}
						class:text-stage={previewType === 'text'}
						class:image-stage={previewType === 'image'}
					>
						{#if previewType === 'text'}
							{#key selectedFile?.path}
								<HighlightAuto code={textContent} class="text-stage"/>
							{/key}
						{:else if previewType === 'image'}
							<div class="image-stage-inner" class:zoomed={imageZoom > 1} onwheel={mouseZoom}>
							
								<img
									class="image-preview"
									src={imageUrl}
									alt={selectedFile.name}
									style={`width: ${imageZoom * 100}%;`}
									onerror={() => previewError = 'The image could not be rendered.'}
								/>
							</div>
						{:else if previewType === 'pdf'}
							<iframe
								title={selectedFile.name}
								class="pdf-preview"
								src={pdfUrl}
								onerror={() => previewError = 'The PDF could not be rendered.'}
							></iframe>
						{:else if previewType === 'audio'}
							<audio
								class="audio-preview"
								controls
								src={audioUrl}
								onerror={() => previewError = 'The audio file could not be rendered.'}
							></audio>
						{:else if previewType === 'video'}
							<!-- svelte-ignore a11y_media_has_caption -->
							<video
								class="video-preview"
								controls
								src={videoUrl}
								onerror={() => previewError = 'The video file could not be rendered.'}
							></video>
						{:else}
							{#if previewError}
								<p class="preview-error">{previewError}</p>
							{:else}
								<p class="empty-preview">No preview available.</p>
							{/if}
						{/if}
					</div>
				{:else}
					<div class="empty-preview empty-preview-large">
						<p>Select a file to preview it.</p>
					</div>
				{/if}
			</aside>
		{/if}
	</div>
</main>

<style>
	/* ------GLOBAL CSS------ */
	:global(html),
	:global(body) {
		margin: 0;
		padding: 0;
		height: 100%;
		overflow: hidden;
		color: black;
		background: #eceef0;
		font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
	}

	/* Buttons and Inputs css */
	button,
	input {
		font: inherit;
	}

	/* ------Container<main>------ */
	.container {
		height: 100vh;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
		overflow: hidden;
		/* Test V */
		/* background: greenyellow; */
	}

	/* ------HEADER------ */
	.path-header {
		flex-shrink: 0;
		align-items: center;
		padding: 10px 10px;
		/* Test V */
		/* background: green; */
	}

	.path-header p {
		font-weight: 600;
		margin: 0px;
		color: #491a70;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		/* Test V */
		/* background: wheat; */
	}

	.path-actions {
		display: flex;
		/* padding: 5px; */
		margin-left: 10px;
		gap: 10px;
		flex-shrink: 0;
		font-size: 20px;
		/* width: 200px; */
		/* Test V */
		/* background-color: chocolate; */
	}

	.tool-button {
		height: 40px;
		width: 40px;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		cursor: pointer;
		background: white;
		font-weight: 600;
		margin-bottom: 8px;
		/* Test V */
		/* background: yellow; */
	}

	.tool-button:hover {
		background: #f3f6f9;
	}

	/* ------SETTING------ */
	.settings-panel {
		margin: 0px 8px 8px 8px;
		border-radius: 8px;
		border-style: solid;
		border-width: 1px;
		background: white;
		padding: 8px;
	}

	.start-path-field {
		display: flex;
		flex-direction: column;
		/* Test V */
		/* background: brown; */
	}

	.start-path-field input {
		height: 30px;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		padding: 0 8px;
		color: #1d2733;
	}

	.checkbox-field {
		margin-top: 10px;
		display: flex;
		gap: 25px;
		margin-bottom: 10px;
		align-items: center;
	}

	.save-button {
		height: 40px;
		width: 40px;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		background: white;
		cursor: pointer;
		font-size: 20px;
		transition: 100ms;
	}

	.save-button:hover {
		background: #f3f6f9;
		transform: scale(1.1);
		transition: 100ms;
	}

	.settings-message {
		margin: 5px 0px 0px 0px;
		color: #607080;
		font-size: 12px;
	}

	/* ------File Explorer------ */
	.explorer-container {
		flex: 1;
		display: flex;
		min-height: 0;
		/* overflow: hidden; */
		/* Test V */
		/* background: pink; */
	}

	.file-list {
		flex: 1;
		min-width: 0;
		overflow: auto;
	}
	
	.table-header {
		display: grid;
		grid-template-columns: minmax(220px, 1fr) 130px 95px 170px;
		align-items: center;
		gap: 10px;
		position: sticky;
		top: 0;
		z-index: 2;
		padding: 12px 20px;
		/* border-bottom: 1px solid #e3e7eb; */
		background: #f8fafc;
		color: #607080;
		font-size: 12px;
		font-weight: 700;
	}

	ul {
		margin: 0;
		padding: 6px;
		/* Test V */
		/* background: blue; */
	}

	li {
		list-style: none;
		border-radius: 8px;
		/* Test V */
		/* background: palegreen; */
	}

	li:hover {
		text-decoration: underline ;
		background: #e6e9ec;
	}

	li.selected {
		background: #dae8ff;
		font-weight: 600;
	}

	.file-row {
		display: grid;
		grid-template-columns: minmax(220px, 1fr) 130px 95px 170px;
		align-items: center;
		gap: 10px;
		width: 100%;
		border: 0;
		background: transparent;
		color: inherit;
		padding: 8px 8px;
		text-align: left;
		cursor: pointer;
		/* Test V */
		/* background: red; */
	}

	.file-name-cell {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		/* Test V */
		/* background: pink; */
	}

	.folder-icon {
		width: 25px;
		height: 20px;
		border-radius: 5px;
		flex-shrink: 0;
		background: linear-gradient(#e6b94f 0 32%, #d79a2b 32%);
	}

	.file-icon {
		width: 25px;
		height: 20px;
		border-radius: 5px;
		flex-shrink: 0;
		border: 1px solid #b6c2cf;
		background: linear-gradient(135deg, #ffffff 0 72%, #d7dce2 72%);
	}

	.file-name,
	.file-type,
	.file-size,
	.file-modified {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		/* Test V */
		/* background: coral; */
	}

	.file-type,
	.file-size,
	.file-modified {
		color: #607080;
		font-size: 14px;
	}

	.empty-list {
		color: #607080;
		margin: 28px;
		/* Test V */
		/* background: violet; */
	}

	/* ------Preview Panel------ */
	.preview-panel {
		margin-top: -50px;
		margin-bottom: 20px;
		margin-right: 5px;
		margin-left: 5px;
		width: 50%;
		min-width: 330px;
		max-width: calc(100% - 200px);
		display: flex;
		flex-direction: column;
		/* padding: 5px; */
		box-sizing: border-box;
		/* background: #ffffff; */
		/* border-left: 1px solid #d7dce2; */
		resize: horizontal;
		overflow: auto;
		direction: rtl;
	}

	.preview-panel * {
		direction: ltr;
	}

	.preview-error {
		color: #8a1f11;
		background: #fff2ef;
		border: 1px solid #f1b4aa;
		border-radius: 8px;
		padding: 8px 8px;
		margin: 0;
	}

	/* ------Preview Panel - Image Panel Zoom------ */
	.zoom-toolbar {
		justify-content: center;
		padding: 6px;
		background-color: #d3d5d8;
		margin: 0px 0px 5px 0px;
		border-radius: 8px;
		border: 1px solid #cbd3dc;
		display: flex;
		gap: 8px;
		align-items: center;
		flex-shrink: 0;
		font-size: 18px;
		font-weight: 600;
		/* Test V */
		/* background: chocolate; */
	}

	.zoom-toolbar span {
		min-width: 40px;
		text-align: center;
		color: #607080;
		font-size: 18px;
		/* Test V */
		/* background: brown; */
	}

	.zoom-toolbar button {
		height: 30px;
		width: 30px;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		background: white;
		cursor: pointer;
	}

	.zoom-toolbar button:hover {
		background: #f3f6f9;
	}

	/* ------Preview Panel - Text Panel------ */
	.text-stage {
		flex: 1;
		min-height: 0;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		background: rgb(255, 255, 255);
		/* padding: 5px; */
		overflow: auto;
		/* Test V */
		/* background: palevioletred; */
	}

	.text-stage :global(pre) {
		margin: 0;
		font-size: 14px;
		white-space: pre-wrap;
		word-break: break-word;
		background: transparent;
	}

	.text-stage :global(code) {
		padding: 5px;
		height: calc(100vh - 75px);
		font-family:
			'JetBrains Mono',
			'Fira Code',
			'Cascadia Code',
			Consolas,
			monospace;
	}

	/* ------Preview Stage------ */
	.preview-stage {
		flex: 1;
		min-height: 0;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
		background: white;
		overflow: auto;
		display: flex;
		align-items: center;
		justify-content: center;
		/* padding: 14px; */
		/* Test V */
		/* background: paleturquoise; */
	}

	.image-stage {
		display: block;
	}

	.image-stage-inner {
		min-width: 100%;
		min-height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 5px;
		box-sizing: border-box;
		/* background: rgba(0, 0, 0, 0.8); */
	}

	.image-stage-inner.zoomed {
		align-items: flex-start;
		justify-content: flex-start;
		min-width: 100%;
		min-height: 100%;
	}

	.image-stage-inner:not(.zoomed) .image-preview {
		width: auto;
		max-width: 100%;
		max-height: calc(100vh - 200px);
	}

	.image-preview {
		display: block;
		height: auto;
		max-width: none;
		max-height: none;
		object-fit: contain;
		transition: 200ms ease;
	}

	/* ------Preview Panel - PDF Panel------ */
	.pdf-preview {
		width: 100%;
		height: calc(100vh - 70px);
		/* min-height: 420px; */
		border: 0;
		/* background: rgb(0, 0, 0); */
	}

	/* ------Preview Panel - Video Panel------ */
	.audio-preview,
	.video-preview {
		width: 100%;
	}

	.video-preview {
		max-height: calc(100vh - 100px);
		background: black;
	}

	.empty-preview {
		margin: 0;
		text-align: center;
		background: white;
	}

	.empty-preview-large {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid #cbd3dc;
		border-radius: 8px;
	}


	/* ------Media------ */
	@media (max-width: 700px) {
		.path-header,
		.settings-panel {
			align-items: stretch;
		}

		.path-header,
		.settings-panel {
			flex-direction: column;
		}

		.path-actions {
			width: 100%;
			flex-wrap: wrap;
		}

		.explorer-container {
			flex-direction: column;
		}

		.preview-panel {
			width: 100%;
			max-width: none;
			min-width: 0;
			height: 70%;
			border-left: 0;
			resize: vertical;
		}

		.pdf-preview {
			height: calc(70vh - 100px);
		}

		.table-header,
		.file-row {
			grid-template-columns: minmax(180px, 1fr) 100px 80px;
		}

		.table-header span:nth-child(4),
		.file-modified {
			display: none;
		}
	}
</style>
