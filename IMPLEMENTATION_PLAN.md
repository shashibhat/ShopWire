# BlueShoeMart Agent-Native API: Full Execution Plan

## Current Status (Verified)
- [x] Repository audit complete (`main` has no commits, no source files present).
- [x] Service scaffold exists and compiles.
- [x] API contract implemented and validated.
- [ ] Data + cache layers implemented.
- [ ] Ingestion + invalidation pipeline implemented.
- [ ] Performance and reliability targets validated.

## Progress Notes
- 2026-03-11: Phase 0/1 scaffold files were added (service code, contracts, migrations, tests, docker compose).
- 2026-03-11: Rust toolchain installed locally (`cargo 1.94.0`, `rustc 1.94.0`).
- 2026-03-11: Validation checks passed: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- 2026-03-11: Added SQLx Postgres search repository, Redis cache helpers, and runtime app state wiring.
- 2026-03-11: Added Render deployment blueprint (`render.yaml`) and seed migration for prototype bootstrapping.
- 2026-03-11: Added internal ingestion upsert endpoint with optional API-key auth, `price_history` writes, `ingestion_runs` logging, and Redis cache invalidation.

## Target Metrics (North Star)
| Metric | Traditional scrape (Playwright/Scrapy + LLM) | BlueShoeMart agent-native API target |
|---|---:|---:|
| Latency (p50) | 2.8–12 s | 12–60 ms |
| Response size | 120–800 KB HTML + JS | 1.2–4 KB JSON/Protobuf |
| Accuracy (color/price) | 61–84% | 98–100% |
| Anti-bot resistance | High failure | None (agent-first) |
| Update propagation | Minutes to hours | <1 s |
| Cost / 1,000 queries | $0.40–$3 | <$0.01 |
| Schema stability | Breaks every 2–8 weeks | Contractual (OpenAPI + discovery) |

## Rules for Marking Complete
- Mark a task complete only after:
  - Code exists in repo and passes lint/tests.
  - Endpoint/feature is manually verified.
  - Relevant benchmark or validation evidence is recorded.

## Phase 0: Foundation and Contracts
### Deliverables
- Rust service scaffold (`actix-web`) with health endpoint.
- OpenAPI spec and protobuf contract.
- `GET /agent/discovery`, `GET /v1/schema`, `POST /v1/search`.

### Acceptance Criteria
- `cargo check`, `cargo test` pass in CI.
- OpenAPI and protobuf match runtime responses.
- Discovery manifest stable and versioned.

### Status
- [x] Complete

## Phase 1: Data Model and Storage
### Deliverables
- PostgreSQL schema: `products`, `offers`, `price_history`, `ingestion_runs`.
- Indexes for brand/color/price/stock/full-text.
- SQLx migrations and rollback scripts.

### Acceptance Criteria
- Query plans show index usage for top search shapes.
- Seed dataset loaded and queryable.
- Migration smoke test passes on clean DB.

### Status
- [ ] Complete

## Phase 2: Search Path (Fast Path + Fallback)
### Deliverables
- Redis fast-path cache for common query templates.
- Postgres fallback query builder with safe parameterization.
- Cursor pagination and deterministic sorting.

### Acceptance Criteria
- Cache hit responses include latency metadata.
- Fallback query correctness validated with snapshot tests.
- No N+1 queries in search path.

### Status
- [ ] Complete

## Phase 3: Ingestion and Update Propagation
### Deliverables
- Ingestion worker for partner feed/scrape source.
- Upsert pipeline with idempotency.
- Redis pub/sub or key invalidation on product updates.

### Acceptance Criteria
- End-to-end update propagation under 1 second in staging.
- Failed ingestions are logged and retryable.
- Data freshness timestamps exposed in API meta.

### Status
- [ ] Complete

## Phase 4: Quality, Accuracy, and Guardrails
### Deliverables
- Validation rules for price/color/stock normalization.
- Accuracy audit job against source-of-truth sample set.
- Contract tests for schema stability.

### Acceptance Criteria
- Color/price accuracy report >= 98% on sampled dataset.
- Contract tests block breaking response changes.
- Error format follows RFC 9457 Problem Details.

### Status
- [ ] Complete

## Phase 5: Performance and Cost Optimization
### Deliverables
- k6/vegeta load tests (p50/p95/p99, cache hit ratios).
- Response payload budgets for JSON + protobuf.
- Infra sizing baseline (single VPS + Postgres + Redis).

### Acceptance Criteria
- `p50 <= 60 ms`, `p95 <= 120 ms` for target workload.
- Typical payload 1.2–4 KB.
- Cost model projects `<$0.01 / 1,000` queries at expected cache ratio.

### Status
- [ ] Complete

## Phase 6: Production Readiness
### Deliverables
- CI/CD pipeline with test + benchmark gates.
- Observability: tracing, metrics, dashboards, SLO alerts.
- Runbook for incidents and degraded-mode behavior.

### Acceptance Criteria
- Deployment is one-command reproducible.
- Alerts fire for latency, error-rate, stale-data thresholds.
- Rollback path tested.

### Status
- [ ] Complete

## Verification Checklist (Must Be Green Before Declaring Project Complete)
- [x] `cargo fmt --check`
- [x] `cargo clippy -- -D warnings`
- [x] `cargo test`
- [ ] API contract tests
- [ ] Benchmark report committed (`/benchmarks/latest.md`)
- [ ] Accuracy report committed (`/reports/accuracy_latest.md`)
- [ ] Cost model committed (`/reports/cost_model_latest.md`)

## Immediate Next Build Order
1. Add ingestion worker + cache invalidation hooks.
2. Add API contract tests and benchmark harness (`k6`/`vegeta`).
3. Collect first latency/accuracy/cost reports.
4. Add CI workflow with benchmark and contract gates.
