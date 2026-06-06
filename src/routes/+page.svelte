<script>
	import { convertFileSrc, invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

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

	function zoomIn() {
		imageZoom = Math.min(4, Number((imageZoom + 0.25).toFixed(2)));
	}

	function zoomOut() {
		imageZoom = Math.max(0.25, Number((imageZoom - 0.25).toFixed(2)));
	}

	function resetZoom() {
		imageZoom = 1;
	}

	function toggleSettings() {
		settingsVisible = !settingsVisible;
	}

	function togglePreview() {
		previewVisible = !previewVisible;
	}

	/**
	 * @param {string} path
	 */
	async function loadDirectory(path) {
		selectedFile = null;
		previewType = null;
		clearPreview();

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

	async function goBack() {
		const parts = currentPath.split('\\');

		if (parts.length > 2) {
			parts.pop();
			await loadDirectory(parts.join('\\'));
		}
	}

	async function refreshDirectory() {
		if (currentPath) {
			await loadDirectory(currentPath);
		}
	}

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
		<p>PATH - {currentPath}</p>
		<div class="path-actions">
			<button class="tool-button" type="button" onclick={goBack}>↩︎</button>
			<button class="tool-button" type="button" onclick={refreshDirectory}>⟳</button>
			<button class="tool-button" type="button" onclick={togglePreview}>
				{previewVisible ? '◨' : '◫'}
			</button>
			<button class="tool-button" type="button" onclick={toggleSettings}>⋮</button>
		</div>
	</header>

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
				<input type="checkbox" bind:checked={hideDotfiles} />
				<span>Hide files and folders starting with "."</span>
			</label>

			<button class="save-button" type="button" onclick={saveSettings}>🖫</button>

			{#if settingsMessage}
				<p class="settings-message">{settingsMessage}</p>
			{/if}
		</section>
	{/if}

	<div class="explorer-container">
		<section class="file-list" aria-label="Files">
			<div class="table-header">
				<span>Name</span>
				<span>Type</span>
				<span>Size</span>
				<span>Modified</span>
			</div>

			<ul>
				<li class="file-row" style="font-weight: bolder;" onclick={goBack}>...</li>
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
							<pre class="text-preview">{textContent}</pre>
						{:else if previewType === 'image'}
							<div class="image-stage-inner" class:zoomed={imageZoom > 1}>
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
	:global(html),
	:global(body) {
		margin: 0;
		padding: 0;
		height: 100%;
		overflow: hidden;
		background: #edf0f3;
		color: #1d2733;
	}

	:global(body) {
		font-family:
			Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
	}

	button,
	input {
		font: inherit;
	}

	.container {
		height: 100vh;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
		overflow: hidden;
	}

	.path-header,
	.explorer-container {
		background: #f1f1f1;
	}

	.settings-panel {
		background: #e2e2e2;
	}

	.path-header {
		flex-shrink: 0;
		align-items: center;
		padding: 12px 14px;
	}

	.path-header p {
		font-weight: 500;
		margin: 0 0 10px 10px;
		color: #531a81;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.path-actions,
	.zoom-toolbar {
		display: flex;
		gap: 10px;
		align-items: center;
		flex-shrink: 0;
		font-size: 20px;
	}

	.tool-button,
	.save-button,
	.zoom-toolbar button {
		height: 35px;
		width: 35px;
		border: 1px solid #cbd3dc;
		border-radius: 7px;
		background: #ffffff;
		color: #172230;
		cursor: pointer;
	}

	.tool-button:hover,
	.save-button:hover,
	.zoom-toolbar button:hover {
		background: #f3f6f9;
	}

	.settings-panel {
		flex-shrink: 0;
		display: flex;
		gap: 16px;
		align-items: end;
		padding: 12px 14px;
	}

	.start-path-field,
	.checkbox-field {
		display: flex;
		gap: 8px;
		color: #000000;
		font-size: 15px;
	}

	.start-path-field {
		flex-direction: column;
		flex: 1;
		max-width: 500px;
	}

	.start-path-field input {
		height: 30px;
		border: 1px solid #cbd3dc;
		border-radius: 7px;
		padding: 0 10px;
		color: #1d2733;
	}

	.checkbox-field {
		align-items: center;
		padding-bottom: 8px;
	}

	.settings-message {
		margin: 0 0 8px;
		color: #607080;
		font-size: 13px;
	}

	.explorer-container {
		flex: 1;
		display: flex;
		min-height: 0;
		overflow: hidden;
	}

	.file-list {
		flex: 1;
		min-width: 0;
		overflow: auto;
	}

	.table-header,
	.file-row {
		display: grid;
		grid-template-columns: minmax(220px, 1fr) 130px 95px 170px;
		align-items: center;
		gap: 12px;
	}

	.table-header {
		position: sticky;
		top: 0;
		z-index: 2;
		padding: 12px 20px;
		border-bottom: 1px solid #e3e7eb;
		background: #f8fafc;
		color: #607080;
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0;
	}

	ul {
		margin: 0;
		padding: 6px;
	}

	li {
		list-style: none;
		border-radius: 7px;
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
		width: 100%;
		border: 0;
		background: transparent;
		color: inherit;
		padding: 10px 10px;
		text-align: left;
		cursor: pointer;
	}

	.file-name-cell {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}

	.folder-icon,
	.file-icon {
		width: 22px;
		height: 18px;
		border-radius: 4px;
		flex-shrink: 0;
	}

	.folder-icon {
		background: linear-gradient(#e6b94f 0 32%, #d79a2b 32%);
	}

	.file-icon {
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
	}

	.file-type,
	.file-size,
	.file-modified {
		color: #607080;
		font-size: 13px;
	}

	.empty-list {
		color: #607080;
		margin: 28px;
	}

	.preview-panel {
		margin-bottom: 20px;
		width: 50%;
		min-width: 330px;
		max-width: 90%;
		display: flex;
		flex-direction: column;
		padding: 5px;
		box-sizing: border-box;
		background: #ffffff;
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
		border-radius: 7px;
		padding: 8px 10px;
		margin: 0;
	}

	.zoom-toolbar {
		justify-content: center;
		padding: 4px 0 8px;
	}

	.zoom-toolbar span {
		min-width: 40px;
		text-align: center;
		color: #607080;
		font-size: 15px;
	}

	.preview-stage {
		flex: 1;
		min-height: 0;
		border: 1px solid #d7dce2;
		border-radius: 8px;
		background: rgb(40, 40, 40);
		overflow: auto;
	}


	.text-stage {
		flex: 1;
		min-height: 0;
		border: 1px solid #d7dce2;
		border-radius: 8px;
		background: rgb(37, 37, 37);
		overflow: auto;
	}

	.preview-stage {
		display: flex;
		align-items: center;
		justify-content: center;
		/* padding: 14px; */
	}

	.image-stage {
		display: block;
		padding: 0;
	}

	.text-preview {
		font-size: 14px;
		margin: 0;
		/* padding: 16px; */
		white-space: pre-wrap;
		word-break: break-word;
		color: #ffffff;
		line-height: 1.5;
	}

	.image-stage-inner {
		min-width: 100%;
		min-height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 10px;
		box-sizing: border-box;
	}

	.image-stage-inner.zoomed {
		align-items: flex-start;
		justify-content: flex-start;
		min-width: 100%;
		min-height: 100%;
	}

	.image-preview {
		display: block;
		height: auto;
		max-width: none;
		max-height: none;
		object-fit: contain;
		transition: width 120ms ease;
	}

	.image-stage-inner:not(.zoomed) .image-preview {
		width: auto;
		max-width: 100%;
		max-height: calc(100vh - 200px);
	}

	.pdf-preview {
		width: 100%;
		height: calc(100vh - 120px);
		/* min-height: 420px; */
		border: 0;
		background: rgb(0, 0, 0);
	}

	.audio-preview,
	.video-preview {
		width: 100%;
	}

	.video-preview {
		max-height: calc(100vh - 100px);
		background: black;
	}

	.empty-preview {
		color: #607080;
		margin: 0;
		text-align: center;
	}

	.empty-preview-large {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px dashed #cbd3dc;
		border-radius: 8px;
	}

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

		.tool-button {
			flex: 1;
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
