# ShopWire / BlueShoeMart Prototype

Agent-first e-commerce affiliate API skeleton focused on structured responses and low-latency search.

## Problem statement
Current shopping agents depend on HTML scraping pipelines (browser automation + parsing + LLM post-processing). That creates:
- High latency (seconds, not milliseconds)
- Fragile integrations (DOM/layout changes break flows)
- Large payload overhead (HTML/JS vs structured API contracts)
- Lower answer accuracy for price/color/stock
- Higher infra and proxy cost

For affiliate commerce, this makes agent checkout discovery unreliable, expensive, and hard to scale.

## Solution
ShopWire provides an agent-native product API for structured search and inventory retrieval:
- Stable machine-readable contracts (OpenAPI + Protobuf)
- Fast path with Redis cache and lightweight JSON responses
- Postgres-backed filtering/sorting for deterministic results
- Internal ingestion endpoint for feed upserts
- Cache invalidation on ingest for rapid freshness
- Redirect URLs to merchant (Walmart) for final checkout

The prototype is designed for Render deployment so teams can iterate quickly and validate real agent traffic.

## Architecture
```text
[AI Agent / LLM Client]
        |
        | HTTP JSON
        v
[Render Web Service: Rust + Actix]
        |            \
        |             \-- GET /agent/discovery
        |             \-- GET /v1/schema
        |             \-- POST /v1/search
        |             \-- POST /internal/ingest/upsert
        |
        +--> [Redis (Render Key Value)]
        |       - search cache (search:v*:*)
        |       - cache invalidation after ingest
        |
        +--> [Postgres (Render)]
                - products
                - price_history
                - ingestion_runs
```

### Search request lifecycle
1. Agent calls `POST /v1/search`.
2. Service checks Redis search key.
3. On miss, service queries Postgres with dynamic filters/sort.
4. Response is returned and cached.
5. Metadata includes source (`cache`, `db`, `fallback`) and latency.

### Ingestion lifecycle
1. Internal system calls `POST /internal/ingest/upsert`.
2. Service upserts products in Postgres.
3. Service writes `price_history` snapshots and `ingestion_runs` summary.
4. Service invalidates `search:v1:*` keys in Redis.
5. Next search reflects updated product state.

## Implemented in this scaffold
- Rust `actix-web` service structure.
- Endpoints:
  - `GET /healthz`
  - `GET /agent/discovery`
  - `GET /v1/schema`
  - `POST /v1/search`
- OpenAPI contract in `docs/openapi.yaml`.
- Protobuf contract in `contracts/search.proto`.
- Initial PostgreSQL migrations in `migrations/`.
- Docker compose for Postgres + Redis.
- API smoke tests in `tests/api_smoke.rs`.

## Run locally
1. Install Rust toolchain (cargo/rustc).
2. Start dependencies:
   - `docker compose up -d`
3. Configure env:
   - `cp .env.example .env`
4. Run migrations:
   - `psql "$DATABASE_URL" -f migrations/0001_init.sql`
   - `psql "$DATABASE_URL" -f migrations/0002_seed_products.sql`
5. Run the service:
   - `cargo run`
6. Run tests:
   - `cargo test`

## Deploy to Render (prototype)
1. Push this repo to GitHub.
2. In Render, create from Blueprint and select `render.yaml`.
3. Render will provision:
   - Web service `shopwire-api`
   - Postgres `shopwire-db`
   - Redis `shopwire-redis`
4. After first deploy, run SQL in Render Postgres shell:
   - `\i migrations/0001_init.sql`
   - `\i migrations/0002_seed_products.sql`
5. Verify:
   - `GET /healthz`
   - `GET /agent/discovery`
   - `POST /v1/search`

## Environment variables
- `DATABASE_URL` required for DB-backed search.
- `REDIS_URL` optional but recommended for cache hit latency.
- `RUST_LOG=info` recommended in prototype.
- `INTERNAL_API_KEY` optional, but recommended to protect internal ingest endpoints.

## Current search flow
- Redis lookup by normalized query key.
- Postgres fallback query with dynamic filters/sort.
- Local in-process fallback response if DB is unavailable.

## Internal ingestion endpoint
- Endpoint: `POST /internal/ingest/upsert`
- Header (if configured): `x-internal-key: <INTERNAL_API_KEY>`
- Purpose:
  - Upsert incoming product snapshots into `products`.
  - Append rows to `price_history`.
  - Record run summary in `ingestion_runs`.
  - Invalidate `search:v*:*` Redis keys for fast freshness.

Example:
```bash
curl -X POST "$BASE_URL/internal/ingest/upsert" \
  -H "content-type: application/json" \
  -H "x-internal-key: $INTERNAL_API_KEY" \
  -d '{
    "source": "walmart-feed-prototype",
    "invalidate_cache": true,
    "products": [
      {
        "sku": "nike-pg40-royal",
        "name": "Nike Air Zoom Pegasus 40",
        "brand": "nike",
        "category": "shoes",
        "color": "royal-blue",
        "size": "9",
        "price": 47.99,
        "original_price": 89.99,
        "stock": 21,
        "image_url": "https://i5.walmartimages.com/asr/example1.jpg",
        "walmart_url": "https://www.walmart.com/ip/456789123",
        "active": true
      }
    ]
  }'
```

## Notes
- This is a scaffold and prototype for fast iteration on Render.
- Performance/cost goals still need benchmark and load-test validation.
- Completion gates are tracked in `IMPLEMENTATION_PLAN.md`.
