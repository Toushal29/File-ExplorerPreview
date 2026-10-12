# File Explorer-Preview

A Windows desktop file browser for opening a folder, selecting a file, and reading or viewing it in the same window. Split preview lets you compare two files side by side.

## Features

- Windows Explorer style folder tree, breadcrumbs, editable path, and back/forward navigation.
- Search the current folder, sort its contents, and switch between list and icon views.
- Preview notes and source files, Markdown, images, PDFs, audio, and video.
- Open files from a quick picker without changing the main folder; browse by path, Quick Access, or recent previews.
- Keep separate file tabs in each preview pane and compare two files side by side. Resize the folder tree, preview area, and split panes.
- Quick access folders, light/dark/system themes, hidden file visibility, and context menu actions.
- Customizable keyboard shortcuts with two binding options per action, including Ctrl+P to open the file picker and Ctrl+Tab to switch preview tabs.
- Middle mouse opens a file in the other preview pane; mouse side buttons can navigate folder history.

## Architecture

- src/routes/+page.svelte coordinates navigation, filtering, history, settings, shortcuts, and the two preview panes.
- src/lib/components/ contains the lazy folder tree, virtualized file list, quick-open picker, per-pane previews, settings, context menu, and icons.
- src/lib/filesystem.js calls the Tauri file and settings commands; src/lib/format.js contains display formatting and breadcrumbs.
- src-tauri/src/filesystem.rs handles directory listing, preview metadata, text reads, and file operations.
- src-tauri/src/settings.rs stores preferences and migrates the previous shortcut and mouse settings.

The frontend reuses Tauri's existing filesystem commands and opener plugin, the installed svelte-highlight component for source previews, and browser APIs for clipboard access and system theme detection.

## Development

Install dependencies and run the desktop app with pnpm install and pnpm tauri dev.

Build a distributable app with pnpm tauri build.
