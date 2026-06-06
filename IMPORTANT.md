## Start the development server

Since i am using `pnpm`:
```bash
cd <app-name>
pnpm install
pnpm tauri dev
```


You’ll now see a new window open with your app running.

`pnpm tauri dev` The first time you run this command, the Rust package manager may need several minutes to download and build all the required packages. Since they are cached, subsequent builds are much faster, as only your code needs rebuilding.

Once Rust has finished building, the webview opens, displaying your web app. You can make changes to your web app, and if your tooling supports it, the webview should update automatically, just like a browser.

## Build the app

```bash
pnpm tauri build
```