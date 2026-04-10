/**
 * Global teardown: runs once after all test files have finished.
 *
 * Currently a lightweight hook. In the future, this can be extended to:
 * - Clean up test data from the database
 * - Generate summary reports
 * - Notify external systems
 */
async function globalTeardown(): Promise<void> {
  console.log('\n╔══════════════════════════════════════════════════╗');
  console.log('║     EMS E2E Testing Suite — Global Teardown      ║');
  console.log('╚══════════════════════════════════════════════════╝');

  const skipCleanup = process.env.SKIP_CLEANUP === 'true';

  if (skipCleanup) {
    console.log('ℹ SKIP_CLEANUP is set — skipping test data cleanup');
  } else {
    console.log('ℹ Test data cleanup is managed per-test via fixtures');
  }

  console.log('✓ Global teardown complete.\n');
}

export default globalTeardown;
