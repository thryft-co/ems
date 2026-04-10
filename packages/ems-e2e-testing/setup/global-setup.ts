import { ApiClient } from '../utils/api-client';
import dotenv from 'dotenv';
import path from 'path';

/**
 * Global setup: runs once before any test files.
 *
 * - Verifies backend and frontend are reachable
 * - When launched via `python run.py e2e`, servers are already started by the runner
 * - When launched standalone (`npx playwright test`), waits for externally-started servers
 */
async function globalSetup(): Promise<void> {
  // Load .env.test
  dotenv.config({ path: path.resolve(__dirname, '..', '.env.test') });

  const apiClient = new ApiClient();
  const baseURL = process.env.BASE_URL || 'http://localhost:3001';

  console.log('╔══════════════════════════════════════════════╗');
  console.log('║     EMS E2E Testing Suite — Global Setup     ║');
  console.log('╚══════════════════════════════════════════════╝');
  console.log(`\n  Backend URL:  ${process.env.API_URL || 'http://localhost:5002'}`);
  console.log(`  Frontend URL: ${baseURL}\n`);

  // Quick health check (short timeout — servers should already be up via run.py)
  console.log('⏳ Verifying backend is healthy...');
  await apiClient.waitForHealthy(10, 1000);

  console.log('⏳ Verifying frontend is reachable...');
  let frontendReady = false;
  for (let attempt = 1; attempt <= 10; attempt++) {
    try {
      const response = await fetch(baseURL, { signal: AbortSignal.timeout(3000) });
      if (response.ok || response.status === 200) {
        frontendReady = true;
        console.log(`✓ Frontend is reachable (attempt ${attempt})`);
        break;
      }
    } catch {
      // Not ready yet
    }
    if (attempt < 10) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  if (!frontendReady) {
    throw new Error(
      'Frontend is not reachable. If running standalone, start servers first:\n' +
      '  python run.py dev\n' +
      'Or use the integrated runner:\n' +
      '  python run.py e2e',
    );
  }

  console.log('\n✓ Global setup complete. Starting tests...\n');
}

export default globalSetup;
