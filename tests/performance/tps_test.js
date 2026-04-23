import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 20 },   // 30秒内增加到 20 VU
    { duration: '1m', target: 50 },    // 1分钟内增加到 50 VU
    { duration: '2m', target: 100 },   // 2分钟内增加到 100 VU
    { duration: '1m', target: 100 },   // 保持 100 VU 1分钟
    { duration: '30s', target: 0 },    // 30秒内降到 0
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% 的请求在 500ms 内完成
    http_req_failed: ['rate<0.05'],   // 错误率低于 5%
  },
};

const RUST_API = __ENV.RUST_NRCS_HOST || 'localhost';
const RUST_PORT = __ENV.RUST_NRCS_PORT || '17976';
const JAVA_API = __ENV.JAVA_NRCS_HOST || '192.168.2.164';
const JAVA_PORT = __ENV.JAVA_NRCS_PORT || '17976';

export default function () {
  // 测试 Rust 节点
  const rustUrl = `http://${RUST_API}:${RUST_PORT}/nrcs`;
  const rustPayload = 'requestType=getBlockchainStatus';
  
  const rustResp = http.post(rustUrl, rustPayload);
  
  check(rustResp, {
    'Rust status is 200': (r) => r.status === 200,
    'Rust response has numberOfBlocks': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.numberOfBlocks !== undefined;
      } catch (e) {
        return false;
      }
    },
    'Rust response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  sleep(1);
  
  // 测试 Java 节点（如果可用）
  const javaUrl = `http://${JAVA_API}:${JAVA_PORT}/nrcs`;
  const javaPayload = 'requestType=getBlockchainStatus';
  
  const javaResp = http.post(javaUrl, javaPayload);
  
  check(javaResp, {
    'Java status is 200': (r) => r.status === 200,
    'Java response has numberOfBlocks': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.numberOfBlocks !== undefined;
      } catch (e) {
        return false;
      }
    },
    'Java response time < 300ms': (r) => r.timings.duration < 300,
  });
  
  sleep(1);
}

export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'tests/reports/performance_report.json': JSON.stringify(data, null, 2),
  };
}

function textSummary(data, options) {
  const indent = options.indent || '';
  const enableColors = options.enableColors || false;
  
  let summary = '\n' + indent + '='.repeat(50) + '\n';
  summary += indent + '性能测试报告\n';
  summary += indent + '='.repeat(50) + '\n\n';
  
  // HTTP 请求统计
  if (data.metrics.http_req_duration) {
    const avg = data.metrics.http_req_duration.values.avg;
    const min = data.metrics.http_req_duration.values.min;
    const med = data.metrics.http_req_duration.values.med;
    const max = data.metrics.http_req_duration.values.max;
    const p90 = data.metrics.http_req_duration.values['p(90)'];
    const p95 = data.metrics.http_req_duration.values['p(95)'];
    
    summary += indent + 'HTTP 请求延迟:\n';
    summary += indent + `  平均: ${avg.toFixed(2)}ms\n`;
    summary += indent + `  最小: ${min.toFixed(2)}ms\n`;
    summary += indent + `  中位数: ${med.toFixed(2)}ms\n`;
    summary += indent + `  最大: ${max.toFixed(2)}ms\n`;
    summary += indent + `  P90: ${p90.toFixed(2)}ms\n`;
    summary += indent + `  P95: ${p95.toFixed(2)}ms\n\n`;
  }
  
  // 请求速率
  if (data.metrics.http_reqs) {
    const rate = data.metrics.http_reqs.values.rate;
    summary += indent + `请求速率: ${rate.toFixed(2)} req/s\n\n`;
  }
  
  // 错误率
  if (data.metrics.http_req_failed) {
    const failRate = data.metrics.http_req_failed.values.rate * 100;
    summary += indent + `错误率: ${failRate.toFixed(2)}%\n\n`;
  }
  
  summary += indent + '='.repeat(50) + '\n';
  
  return summary;
}
