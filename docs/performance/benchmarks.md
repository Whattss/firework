# 📊 Benchmarks

Performance benchmarks for Firework.

---

## Official Benchmarks

```
Plain Text:    170,056 req/s  (732µs latency)
JSON Response: 166,509 req/s  (748µs latency)
Path Params:   165,996 req/s  (750µs latency)
Optimized:     201,325 req/s  (1.27ms latency)
Peak:          260,962 req/s  🚀
```

---

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench routing_bench
cargo bench --bench request_bench
cargo bench --bench server_bench
```

---

## Benchmark Suites

### 1. Router Benchmarks
- Route insertion: <1µs per route
- Route lookup: <100µs
- Scaling: O(log n) with radix tree

### 2. Request/Response Benchmarks
- Request creation: <10µs
- Response building: <5µs
- JSON serialization: <20µs

### 3. Server Throughput Benchmarks
- Simple GET: 200k+ req/s
- JSON response: 166k req/s
- Real HTTP: 170k+ req/s

---

## Comparison

| Framework | Req/s | Notes |
|-----------|-------|-------|
| Firework | 200k+ | Optimized |
| Actix-web | 250k+ | Industry standard |
| Axum | 180k | Tokio-based |
| Rocket | 120k | Higher-level |

---

## Tools

```bash
# bombardier
bombardier -c 125 -n 10000000 http://localhost:8080/

# wrk
wrk -t12 -c400 -d30s http://localhost:8080/

# Apache Bench
ab -n 100000 -c 100 http://localhost:8080/
```
