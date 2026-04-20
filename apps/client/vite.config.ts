import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import fs from 'fs';

// Load environment variables from config/.env at project root
function loadProjectEnv() {
  const projectRoot = path.resolve(__dirname, '../..');
  const envFile = path.join(projectRoot, 'config', '.env');
  
  if (fs.existsSync(envFile)) {
    const envContent = fs.readFileSync(envFile, 'utf-8');
    const envVars: Record<string, string> = {};
    
    envContent.split('\n').forEach(line => {
      line = line.trim();
      if (line && !line.startsWith('#') && line.includes('=')) {
        const [key, value] = line.split('=', 2);
        envVars[key.trim()] = value.trim();
      }
    });
    
    return envVars;
  }
  
  return {};
}

// https://vitejs.dev/config/
export default defineConfig(() => {
  const projectEnv = loadProjectEnv();
  
  // Get ports from environment with defaults
  const frontendPort = parseInt(projectEnv.FRONTEND_PORT || '3001');
  const backendPort = parseInt(projectEnv.BACKEND_PORT || '5002');
  
  return {
    plugins: [
      react(),
      tailwindcss(),
    ],
    server: {
      port: frontendPort,
      proxy: {
        '/api': {
          target: `http://localhost:${backendPort}`,
          changeOrigin: true,
        },
      },
    },
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
  };
}); 
