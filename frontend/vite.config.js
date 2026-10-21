import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd());

  return {
    plugins: [sveltekit()],
    server: {
      host: env.VITE_DEV_HOST || 'localhost',
      port: 5173,
      allowedHosts: true,
      proxy: {
        '/api': {
          target: env.VITE_API_PROXY || 'http://localhost:8951',
          changeOrigin: true,
          secure: false,
          ws: false
        }
      }
    },
    build: {
      chunkSizeWarningLimit: 900
    }
  };
});
