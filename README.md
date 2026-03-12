# ShopWire / BlueShoeMart Prototype

Agent-first e-commerce affiliate API skeleton focused on structured responses and low-latency search.

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
  - Invalidate `search:v1:*` Redis keys for fast freshness.

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
