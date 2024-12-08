import { defineConfig } from "vite";
import { svelte } from '@sveltejs/vite-plugin-svelte';
import {sveltePreprocess} from "svelte-preprocess";

// @ts-expect-error process is a node.js global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    // Vite настройки, разработанные специально для разработки Tauri и применяемые только в `tauri dev` или `tauri build`
    //
    // 1. Не допускайте, чтобы Vite скрывал ошибки, связанные с Rust
    clearScreen: false,
    // 2. Tauri ожидает фиксированный порт и завершает работу с ошибкой, если этот порт недоступен
    server: {
          port: 1420,
          strictPort: true,
          host: host || false,
          hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1421,
              }
            : undefined,
          watch: {
            // 3. Укажите Vite, чтобы он не смотрел "src-tauri`
            ignored: ["**/src-tauri/**"],
          },
    },

    // 4. Добавление плагинов
    plugins: [svelte(
        {
            preprocess: sveltePreprocess({
                scss: {
                    includePaths: ['./src'], // Путь для поиска SCSS файлов
                },
            })
        }
    )],

    root: "src"
}));
