<script>
	import { afterUpdate } from 'svelte';
	import { HighlightAuto } from 'svelte-highlight';
	import 'svelte-highlight/styles/atom-one-dark.css';
	import Icon from './Icon.svelte';

	export let selectedFile = null;
	export let previewType = null;
	export let previewError = '';
	export let textContent = '';
	export let mediaUrl = null;
	export let imageZoom = 1;
	export let active = false;
	export let paneTitle = 'Preview';
	export let paneIndex = 0;
	export let incomingTabDrag = false;
	export let tabs = [];
	export let activeTabId = '';
	export let onClose = () => {};
	export let onOpenFile = () => {};
	export let onSelectTab = () => {};
	export let onCloseTab = () => {};
	export let onTabPointerDown = () => {};
	export let onZoomIn = () => {};
	export let onZoomOut = () => {};
	export let onZoomReset = () => {};
	export let onActivate = () => {};
	export let onPreviewError = () => {};
	let tabBarElement;
	let lastActiveTabId = '';

	const noteExtensions = new Set(['txt', 'text', 'md', 'markdown', 'rst', 'nfo', 'log', 'csv', 'tsv', 'org', 'adoc', 'rtf', 'note']);
	$: extension = selectedFile?.name?.toLowerCase().split('.').at(-1) || '';
	$: readableNote = noteExtensions.has(extension) || selectedFile?.name?.toLowerCase() === 'readme';
	$: formattedText = extension === 'rtf' ? stripRtf(textContent) : textContent;
	$: markdownReadable = ['md', 'markdown'].includes(extension) && formattedText.length <= 300_000;
	$: formattedMarkdown = markdownReadable ? markdownToHtml(formattedText) : '';
	$: syntaxHighlightable = textContent.length <= 80_000;
	afterUpdate(() => {
		if (activeTabId && activeTabId !== lastActiveTabId) {
			lastActiveTabId = activeTabId;
			tabBarElement?.querySelector('.preview-tab.current .tab-name')?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
		}
	});
	function handleTabPointerDown(event, tabId) {
		if (event.button !== 0 || !event.isPrimary) return;
		onTabPointerDown(paneIndex, tabId, event);
	}

	function escapeHtml(value) {
		return value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;').replaceAll("'", '&#39;');
	}
	function inlineMarkdown(value) {
		return escapeHtml(value)
			.replace(/`([^`]+)`/g, '<code>$1</code>')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/__(.+?)__/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em>$1</em>')
			.replace(/_([^_]+)_/g, '<em>$1</em>')
			.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '$1 <span class="note-url">($2)</span>');
	}
	function markdownToHtml(source) {
		const output = [];
		let paragraph = [];
		let listType = '';
		let codeLines = [];
		let inCode = false;
		const closeList = () => { if (listType) output.push(`</${listType}>`); listType = ''; };
		const flushParagraph = () => { if (paragraph.length) output.push(`<p>${inlineMarkdown(paragraph.join(' '))}</p>`); paragraph = []; };
		for (const line of source.replace(/\r/g, '').split('\n')) {
			if (line.trimStart().startsWith('```')) {
				flushParagraph(); closeList();
				if (inCode) { output.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`); codeLines = []; }
				inCode = !inCode;
				continue;
			}
			if (inCode) { codeLines.push(line); continue; }
			if (!line.trim()) { flushParagraph(); closeList(); continue; }
			const heading = line.match(/^\s{0,3}(#{1,6})\s+(.+)$/);
			if (heading) { flushParagraph(); closeList(); const level = heading[1].length; output.push(`<h${level}>${inlineMarkdown(heading[2])}</h${level}>`); continue; }
			if (/^\s{0,3}([-*_])\1\1+\s*$/.test(line)) { flushParagraph(); closeList(); output.push('<hr>'); continue; }
			const listItem = line.match(/^\s*(?:([-*+])\s+|(\d+)\.\s+)(.+)$/);
			if (listItem) {
				flushParagraph();
				const nextType = listItem[2] ? 'ol' : 'ul';
				if (listType !== nextType) { closeList(); listType = nextType; output.push(`<${listType}>`); }
				output.push(`<li>${inlineMarkdown(listItem[3])}</li>`); continue;
			}
			if (line.startsWith('> ')) { flushParagraph(); closeList(); output.push(`<blockquote>${inlineMarkdown(line.slice(2))}</blockquote>`); continue; }
			closeList(); paragraph.push(line.trim());
		}
		flushParagraph(); closeList();
		if (inCode) output.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`);
		return output.join('');
	}
	function stripRtf(source) {
		return source
			.replace(/\{\\\*[^{}]*\}/g, '')
			.replace(/\\par[d]?\b ?/gi, '\n')
			.replace(/\\line\b ?/gi, '\n')
			.replace(/\\tab\b ?/gi, '\t')
			.replace(/\\'[0-9a-f]{2}/gi, (match) => String.fromCharCode(parseInt(match.slice(2), 16)))
			.replace(/\\[a-z]+-?\d* ?/gi, '')
			.replace(/\\([{}\\])/g, '$1')
			.replace(/[{}]/g, '')
			.trim();
	}
</script>

<aside class:active class:drop-target={incomingTabDrag} class="preview-pane" data-preview-index={paneIndex} aria-label={paneTitle + ' file preview'} onpointerdown={onActivate}>
	<div class="preview-tab-row">
		<div class="preview-tabs" bind:this={tabBarElement} role="tablist" aria-label={paneTitle + ' open files'}>
			{#each tabs as tab (tab.tabId)}
				<div class:current={tab.tabId === activeTabId} class="preview-tab">
					<button class="tab-name" type="button" role="tab"  aria-selected={tab.tabId === activeTabId} title={(tab.selectedFile?.path || 'File tab') + '\nDrag to another preview pane'} onclick={() => onSelectTab(tab.tabId)} onpointerdown={(event) => handleTabPointerDown(event, tab.tabId)} onauxclick={(event) => { if (event.button === 1) { event.preventDefault(); onCloseTab(tab.tabId); } }}>{tab.selectedFile?.name || 'Loading…'}</button>
					<button class="tab-close" type="button" aria-label={'Close ' + (tab.selectedFile?.name || 'file')} title="Close file tab" onclick={(event) => { event.stopPropagation(); onCloseTab(tab.tabId); }}><Icon name="close" size={12} /></button>
				</div>
			{/each}
		</div>
		<div class="preview-actions">
			<button type="button" title="Open another file (Ctrl+P)" aria-label="Open another file" onclick={onOpenFile}><Icon name="plus" size={15} /></button>
			<button type="button" title="Close preview pane" aria-label="Close preview pane" onclick={onClose}><Icon name="close" size={16} /></button>
		</div>
	</div>
	{#if selectedFile}
		{#if previewType === 'image'}<div class="preview-tools"><button onclick={onZoomOut} aria-label="Zoom out">−</button><span>{Math.round(imageZoom * 100)}%</span><button onclick={onZoomIn} aria-label="Zoom in">+</button><button onclick={onZoomReset}>Fit</button></div>{/if}
		<div class:note-canvas={readableNote} class="preview-canvas">
			{#if previewType === 'text'}
				{#if readableNote}
					{#if markdownReadable}<article class="note-document markdown-document">{@html formattedMarkdown}</article>
					{:else}<pre class="note-document">{formattedText}</pre>{/if}
				{:else}<div class="code-scroll">{#if syntaxHighlightable}<HighlightAuto code={textContent} />{:else}<pre class="large-code">{textContent}</pre>{/if}</div>{/if}
			{:else if previewType === null}<p>Loading preview…</p>
				{:else if previewType === 'image'}<div class="image-scroll"><img src={mediaUrl} alt={selectedFile.name} style={`width:${imageZoom * 100}%;`} onerror={() => onPreviewError('This image could not be displayed.', selectedFile.path)} /></div>
			{:else if previewType === 'pdf'}<iframe title={`Preview of ${selectedFile.name}`} src={mediaUrl}></iframe>
				{:else if previewType === 'audio'}<div class="media-message">Audio preview<audio controls src={mediaUrl} onerror={() => onPreviewError('This audio file could not be played.', selectedFile.path)}></audio></div>
				{:else if previewType === 'video'}<!-- svelte-ignore a11y_media_has_caption --><video controls src={mediaUrl} onerror={() => onPreviewError('This video could not be played.', selectedFile.path)}></video>
			{:else}<p>No preview available for this file type.</p>{/if}
		</div>
		{#if previewError}<p class="preview-error">{previewError}</p>{/if}
	{:else}<div class="preview-empty">Open a file here or select one from the folder list.</div>{/if}
	{#if incomingTabDrag}<div class="tab-drop-shield" aria-hidden="true"><span>Move tab here</span></div>{/if}
</aside>

<style>
	.preview-pane { position: relative; min-width: 0; min-height: 0; display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--line); background: var(--paper); }
	.preview-pane.active { border-color: var(--accent); }
	.preview-pane.drop-target { outline: 2px dashed var(--focus); outline-offset: -3px; }
	.tab-drop-shield { position: absolute; z-index: 5; inset: 0; display: grid; place-items: center; border: 2px dashed var(--focus); color: var(--ink); background: color-mix(in srgb, var(--canvas) 45%, transparent); font-size: 12px; pointer-events: auto; }
	.preview-tab-row { min-width: 0; min-height: 32px; flex: 0 0 auto; display: flex; align-items: stretch; border-bottom: 1px solid var(--line); background: var(--canvas); }
	.preview-actions { flex: 0 0 auto; display: flex; align-items: center; gap: 3px; padding: 0 5px; }
	.preview-actions button, .preview-tools button { min-width: 26px; min-height: 24px; display: inline-flex; align-items: center; justify-content: center; gap: 5px; border: 1px solid var(--line); border-radius: 3px; color: var(--ink); background: var(--paper); cursor: pointer; }
	.preview-tabs { min-width: 0; min-height: 32px; flex: 1; display: flex; gap: 2px; overflow: auto hidden; padding: 3px 4px 0; }
	.preview-tab { min-width: 0; max-width: 180px; display: flex; flex: 0 1 150px; align-items: center; border: 1px solid transparent; border-bottom: 0; border-radius: 3px 3px 0 0; color: var(--muted); }
	.preview-tab.current { border-color: var(--line); color: var(--ink); background: var(--paper); }
	.preview-tab button { height: 27px; border: 0; color: inherit; background: transparent; cursor: pointer; }
	.preview-tab .tab-name { min-width: 0; flex: 1; overflow: hidden; padding: 0 6px; text-align: left; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; cursor: grab; user-select: none; touch-action: none; }
	.preview-tab .tab-name:active { cursor: grabbing; }
	.preview-tab .tab-close { width: 23px; flex: 0 0 auto; display: grid; place-items: center; opacity: .65; }
	.preview-tab .tab-close:hover { opacity: 1; background: var(--canvas); }
	.preview-tools { display: flex; justify-content: center; align-items: center; gap: 7px; padding: 5px; border-bottom: 1px solid var(--line); font-size: 10px; }
	.preview-canvas { min-width: 0; min-height: 0; flex: 1; display: flex; align-items: center; justify-content: center; overflow: auto; background: var(--canvas); }
	.preview-canvas.note-canvas { display: block; background: var(--paper); }
	.note-document { width: 100%; min-height: 100%; margin: 0; padding: 18px 20px; overflow-wrap: anywhere; color: var(--ink); background: var(--paper); white-space: pre-wrap; font: 13px/1.65 "Segoe UI", sans-serif; }
	.markdown-document { white-space: normal; }
	.markdown-document :global(h1), .markdown-document :global(h2), .markdown-document :global(h3), .markdown-document :global(h4) { margin: 1.1em 0 .45em; line-height: 1.25; }
	.markdown-document :global(h1) { margin-top: 0; font-size: 21px; }
	.markdown-document :global(h2) { font-size: 17px; }
	.markdown-document :global(p), .markdown-document :global(ul), .markdown-document :global(ol) { margin: .6em 0; }
	.markdown-document :global(li) { padding: 2px 0; }
	.markdown-document :global(blockquote) { margin: 10px 0; padding-left: 12px; border-left: 3px solid var(--line); color: var(--muted); }
	.markdown-document :global(code) { padding: 1px 3px; background: var(--canvas); font-family: Consolas, monospace; font-size: .9em; }
	.markdown-document :global(pre) { overflow: auto; padding: 10px; background: var(--canvas); white-space: pre-wrap; }
	.markdown-document :global(hr) { border: 0; border-top: 1px solid var(--line); }
	.markdown-document :global(.note-url) { color: var(--muted); font-size: .9em; }
	.code-scroll { width: 100%; height: 100%; overflow: auto; padding: 8px; background: var(--canvas); }
	.code-scroll :global(.hljs) { background: var(--canvas); }
	.code-scroll :global(pre) { margin: 0; white-space: pre-wrap; overflow-wrap: anywhere; }
	.code-scroll :global(code) { font: 10px/1.6 Consolas, monospace; }
	.code-scroll .large-code { font: 10px/1.6 Consolas, monospace; }
	.image-scroll { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; overflow: auto; padding: 10px; }
	.image-scroll img { height: auto; max-width: none; }
	.preview-canvas iframe { width: 100%; height: 100%; border: 0; background: var(--canvas); }
	.preview-canvas video { max-width: 100%; max-height: 100%; }
	.media-message { width: 100%; display: grid; justify-items: center; gap: 12px; color: var(--muted); font-size: 11px; }
	.media-message audio { width: 90%; }
	.preview-canvas > p, .preview-empty { padding: 16px; color: var(--muted); text-align: center; font-size: 11px; }
	.preview-empty { margin: auto; }
	.preview-error { margin: 6px; padding: 7px; color: var(--danger-ink); background: var(--danger-bg); font-size: 10px; }
</style>

