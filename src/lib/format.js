/** @param {number} size */
export function formatSize(size) {
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	let value = size;
	let index = 0;
	while (value >= 1024 && index < units.length - 1) {
		value /= 1024;
		index += 1;
	}
	return `${value >= 10 || index === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[index]}`;
}

/** @param {number | null} timestamp */
export function formatDate(timestamp) {
	if (timestamp === null) return '—';
	return new Intl.DateTimeFormat(undefined, {
		month: 'short', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit'
	}).format(new Date(timestamp * 1000));
}

/** @param {string} path */
export function pathLabel(path) {
	const parts = path.split(/[\\/]+/).filter(Boolean);
	return parts.at(-1) || path || 'Home';
}

/** @param {string} directory @param {string} name */
export function joinPath(directory, name) {
	const separator = directory.includes('\\') ? '\\' : '/';
	return `${directory.replace(/[\\/]+$/, '')}${separator}${name}`;
}

const imageExtensions = new Set(['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'svg', 'ico', 'avif', 'apng', 'tif', 'tiff']);
const audioExtensions = new Set(['mp3', 'wav', 'ogg', 'oga', 'opus', 'flac', 'm4a', 'aac', 'weba']);
const videoExtensions = new Set(['mp4', 'webm', 'mkv', 'mov', 'avi', 'm4v', 'ogv', 'wmv']);
const documentExtensions = new Set([
	'txt', 'text', 'log', 'csv', 'tsv', 'rst', 'nfo', 'org', 'adoc', 'ini', 'conf', 'cfg', 'json', 'jsonl', 'xml', 'yaml', 'yml', 'toml', 'lock', 'env',
	'gitignore', 'dockerfile', 'js', 'mjs', 'cjs', 'ts', 'jsx', 'tsx', 'rs', 'py', 'java', 'kt', 'swift', 'c', 'cpp', 'cc',
	'h', 'hpp', 'cs', 'go', 'php', 'rb', 'sh', 'bat', 'ps1', 'html', 'htm', 'css', 'scss', 'sass', 'less', 'md', 'markdown',
	'svelte', 'sql', 'r', 'lua', 'pl', 'dart', 'vue', 'pdf', 'doc', 'docx', 'odt', 'rtf'
]);

/** @param {{ name: string, is_dir: boolean }} entry */
export function fileCategory(entry) {
	if (entry.is_dir) return 'folder';
	const filename = entry.name.toLowerCase();
	if (['dockerfile', 'makefile', '.gitignore', '.env'].includes(filename) || filename.startsWith('.env.')) return 'document';
	const extension = filename.split('.').at(-1) || '';
	if (imageExtensions.has(extension)) return 'image';
	if (audioExtensions.has(extension)) return 'audio';
	if (videoExtensions.has(extension)) return 'video';
	if (documentExtensions.has(extension)) return 'document';
	return 'other';
}

/** @param {string} path */
export function getBreadcrumbs(path) {
	if (!path) return [];
	const windows = path.includes('\\');
	const normalized = path.replace(/[\\/]+$/, '');
	const uncRoot = windows ? normalized.match(/^\\\\[^\\]+\\[^\\]+/)?.[0] : null;
	const drive = windows && !uncRoot ? normalized.match(/^[a-z]:/i)?.[0] : null;
	const root = uncRoot || (drive ? `${drive}\\` : path.startsWith('/') ? '/' : '');
	let remainder = uncRoot ? normalized.slice(uncRoot.length) : drive ? normalized.slice(drive.length) : normalized;
	const segments = remainder.split(/[\\/]+/).filter(Boolean);
	const crumbs = [];
	if (root) crumbs.push({ name: drive || (uncRoot ? pathLabel(uncRoot) : 'Root'), path: root });
	let current = root;
	for (const segment of segments) {
		current = current ? `${current.replace(/[\\/]+$/, '')}${windows ? '\\' : '/'}${segment}` : segment;
		crumbs.push({ name: segment, path: current });
	}
	return crumbs;
}
