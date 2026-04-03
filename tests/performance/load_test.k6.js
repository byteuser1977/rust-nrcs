// k6 load test script for NRCS backend API
// Run: k6 run --vus 100 --duration 30s tests/performance/load_test.k6.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const balanceCheckLatency = new Trend('balance_check_latency');
const txSubmitLatency = new Trend('tx_submit_latency');

export const options = {
  stages: [
    { duration: '10s', target: 50 },  // Ramp up to 50 VUs
    { duration: '20s', target: 50 },  // Stay at 50 VUs
    { duration: '10s', target: 100 }, // Ramp up to 100 VUs
    { duration: '30s', target: 100 }, // Stay at 100 VUs
    { duration: '10s', target: 0 },   // Ramp down
  ],
  thresholds: {
    errors: ['rate<0.05'], // Error rate < 5%
    balance_check_latency: ['p(95)<200'], // 95% < 200ms
    tx_submit_latency: ['p(95)<500'], // 95% < 500ms
    http_req_duration: ['p(99)<1000'], // 99% < 1s
  },
};

const BASE_URL = 'http://localhost:17976/api/v1';
const TEST_ACCOUNT = 'test-account-123';

// Pre-test setup: create test account or use fixture
export function setup() {
  // Could create test account here
  return {};
}

export default function(data) {
  // 1. Health check
  const healthRes = http.get(`${BASE_URL.replace('/api/v1', '')}/health`);
  check(healthRes, {
    'health status 200': (r) => r.status === 200,
  });

  // 2. Get account balance (read-heavy operation)
  const balanceUrl = `${BASE_URL}/accounts/${TEST_ACCOUNT}`;
  const balanceRes = http.get(balanceUrl);
  balanceCheckLatency.add(balanceRes.timings.duration);

  const balanceOk = check(balanceRes, {
    'balance response 200': (r) => r.status === 200,
    'balance has amount': (r) => {
      const body = r.json();
      return body && typeof body.balance === 'number';
    },
  });

  errorRate.add(!balanceOk);

  // 3. Submit a transaction (write operation)
  const txPayload = JSON.stringify({
    sender: TEST_ACCOUNT,
    recipient: 'another-test-account',
    amount: 1000000, // 0.01 NRC
    fee: 100000,
  });

  const txRes = http.post(`${BASE_URL}/transactions`, txPayload, {
    headers: { 'Content-Type': 'application/json' },
  });
  txSubmitLatency.add(txRes.timings.duration);

  const txOk = check(txRes, {
    'tx submit 201': (r) => r.status === 201,
    'tx has transactionId': (r) => {
      const body = r.json();
      return body && typeof body.transactionId === 'string';
    },
  });

  errorRate.add(!txOk);

  // 4. Get latest block
  const blockRes = http.get(`${BASE_URL}/blocks/latest`);
  check(blockRes, {
    'block response 200': (r) => r.status === 200,
    'block has height': (r) => {
      const body = r.json();
      return body && typeof body.height === 'number';
    },
  });

  sleep(1); // Think time
}

export function teardown(data) {
  // Cleanup
}
