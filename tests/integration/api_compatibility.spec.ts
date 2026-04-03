import { test, expect } from '@playwright/test';

test.describe('API Compatibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Wait for backend to be ready
    await page.goto('/api/health');
    const health = await page.textContent('body');
    expect(health).toBe('OK');
  });

  test('GET /api/v1/accounts/:address returns 200', async ({ request }) => {
    // Create a test account first via API or use fixture
    const response = await request.get('/api/v1/accounts/test-account-123');
    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('balance');
    expect(body).toHaveProperty('address');
  });

  test('POST /api/v1/transactions returns 201 with txid', async ({ request }) => {
    const txPayload = {
      sender: 'test-sender-address',
      recipient: 'test-recipient-address',
      amount: 100000000, // 1 NRC
      fee: 100000,
      attachment: '',
    };

    const response = await request.post('/api/v1/transactions', {
      data: txPayload,
    });

    expect(response.status()).toBe(201);
    const body = await response.json();
    expect(body).toHaveProperty('transactionId');
    expect(body).toHaveProperty('status', 'PENDING');
  });

  test('GET /api/v1/blocks/latest returns block data', async ({ request }) => {
    const response = await request.get('/api/v1/blocks/latest');
    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body).toHaveProperty('height');
    expect(body).toHaveProperty('hash');
    expect(body).toHaveProperty('timestamp');
    expect(body).toHaveProperty('transactions');
  });

  test('GET /api/v1/blocks/:height returns specific block', async ({ request }) => {
    // Assuming at least 1 block exists
    const response = await request.get('/api/v1/blocks/1');
    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.height).toBe(1);
  });

  test('GET /api/v1/transactions/:txid returns transaction details', async ({ request }) => {
    // First create a transaction
    const txResponse = await request.post('/api/v1/transactions', {
      data: {
        sender: 'test-account',
        recipient: 'another-account',
        amount: 50000000,
        fee: 100000,
      },
    });
    const tx = await txResponse.json();
    const txId = tx.transactionId;

    // Then fetch it
    const getResponse = await request.get(`/api/v1/transactions/${txId}`);
    expect(getResponse.status()).toBe(200);

    const body = await getResponse.json();
    expect(body.transactionId).toBe(txId);
  });

  test('response format matches OpenAPI schema', async ({ request }) => {
    const response = await request.get('/api/v1/accounts/test-account');
    expect(response.status()).toBe(200);

    const body = await response.json();

    // Check response has correct content-type
    expect(response.headers()['content-type']).toContain('application/json');

    // Validate required fields
    expect(typeof body).toBe('object');
    expect(['address', 'balance', 'unconfirmedBalance']).toContain(
      Object.keys(body)[0]
    );
  });

  test('invalid endpoint returns 404', async ({ request }) => {
    const response = await request.get('/api/v1/invalid-endpoint');
    expect(response.status()).toBe(404);
  });

  test('missing required parameters return 400', async ({ request }) => {
    const response = await request.post('/api/v1/transactions', {
      data: {
        // Missing required fields
      },
    });
    expect(response.status()).toBe(400);
  });

  test('CORS headers present', async ({ request }) => {
    const response = await request.get('/api/v1/blocks/latest');
    const headers = response.headers();

    // Should have CORS headers if enabled
    expect(headers['access-control-allow-origin']).toBeDefined();
  });
});
