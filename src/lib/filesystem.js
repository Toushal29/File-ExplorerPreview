import { convertFileSrc, invoke } from '@tauri-apps/api/core';

/** @typedef {{ name: string, path: string, is_dir: boolean, file_type: string, size: number, modified: number | null }} FileEntry */
/** @typedef {{ start_path: string | null, hide_dotfiles: boolean, favorites: string[], view_mode: 'list' | 'grid', theme?: string, sidebar_collapsed?: boolean, sidebar_width?: number, preview_width?: number, split_ratio?: number, split_mode?: boolean, split_path?: string | null, mouse_middle_other_preview?: boolean, mouse_side_navigation?: boolean, shortcuts?: Record<string, string[]> }} AppSettings */

export const filesystem = {
	/** @param {string} path */
	listDirectory: (path) => invoke('list_directory', { path }),
	getHomeDirectory: () => invoke('get_home_dir'),
	getParentDirectory: (/** @type {string} */ path) => invoke('get_parent_directory', { path }),
	getPreviewType: (/** @type {string} */ path) => invoke('get_preview_type', { path }),
	readTextFile: (/** @type {string} */ path) => invoke('read_text_file', { path }),
	fileExists: (/** @type {string} */ path) => invoke('file_exists', { path }),
	loadSettings: () => invoke('load_settings'),
	saveSettings: (/** @type {AppSettings} */ settings) => invoke('save_settings', { settings }),
	createDirectory: (/** @type {string} */ path) => invoke('create_directory', { path }),
	renamePath: (/** @type {string} */ path, /** @type {string} */ newName) => invoke('rename_path', { path, newName }),
	deletePath: (/** @type {string} */ path) => invoke('delete_path', { path }),
	/** @param {string} path */
	mediaUrl: (path) => convertFileSrc(path)
};
