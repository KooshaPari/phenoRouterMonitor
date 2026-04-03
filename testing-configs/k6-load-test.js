// K6 Load Testing Configuration
// Modern performance testing

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');

// Test configuration
export const options = {
  // Stages for load ramping
  stages: [
    { duration: '30s', target: 10 },   // Ramp up
    { duration: '1m', target: 10 },    // Steady state
    { duration: '30s', target: 50 },   // Stress test
    { duration: '1m', target: 50 },    // Steady high load
    { duration: '30s', target: 0 },   // Ramp down
  ],

  // Thresholds
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% under 500ms
    http_req_failed: ['rate<0.01'],    // Less than 1% failures
    errors: ['rate<0.05'],             // Less than 5% error rate
  },

  // Summary
  summaryTrendStats: ['avg', 'min', 'max', 'p(95)', 'p(99)'],
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Test different endpoints
  const endpoints = [
    '/api/health',
    '/api/status',
    '/api/version',
  ];

  const endpoint = endpoints[Math.floor(Math.random() * endpoints.length)];
  const url = `${BASE_URL}${endpoint}`;

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'X-Request-ID': `k6-${__VU}-${__ITER}`,
    },
  };

  // Make request
  const res = http.get(url, params);

  // Track metrics
  responseTime.add(res.timings.duration);
  errorRate.add(res.status !== 200);

  // Assertions
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response has body': (r) => r.body && r.body.length > 0,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });

  sleep(1);
}

// Handle setup
export function setup() {
  console.log('Starting k6 load test...');
  return { startTime: new Date().toISOString() };
}

// Handle teardown
export function teardown(data) {
  console.log(`Load test completed. Started at: ${data.startTime}`);
}
