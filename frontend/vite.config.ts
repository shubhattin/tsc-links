import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

import { proxy } from './proxy.js';

export default defineConfig(({ command }) => ({
  plugins: [
    tailwindcss(),
    sveltekit({
      compilerOptions: {
        // Force runes mode for the project, except for libraries. Can be removed in svelte 6.
        runes: ({ filename }) =>
          filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
      },
      // Static build: only `/` is prerendered. Every other path is rewritten
      // to the Rust function by the root vercel.json (filesystem wins first).
      adapter: adapter({ pages: 'build', assets: 'build', fallback: undefined, strict: true }),
      // One env file for the whole repo (`../.env.local`).
      env: { dir: '..' },
    }),
  ],
  ...(command === 'serve'
    ? {
        server: {
          proxy,
          // proxy only during development
        },
      }
    : {}),
}));
