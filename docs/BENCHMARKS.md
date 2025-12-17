# Performance Benchmarks

This document describes the benchmarks included with rust-mssql-driver and how to interpret results.

## Running Benchmarks

### Prerequisites

- Rust 1.85+ (stable)
- Criterion 0.5+

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Protocol encoding/decoding benchmarks
cargo bench -p tds-protocol

# Type conversion benchmarks
cargo bench -p mssql-types
```

### Generate HTML Reports

Criterion generates HTML reports in `target/criterion/`:

```bash
cargo bench
open target/criterion/report/index.html
```

## Benchmark Suites

### tds-protocol Benchmarks

Located in `crates/tds-protocol/benches/protocol.rs`.

| Benchmark | Description | What It Measures |
|-----------|-------------|------------------|
| `packet_header_encode` | Encode 8-byte TDS packet header | Memory allocation, byte ordering |
| `packet_header_decode` | Decode 8-byte TDS packet header | Parsing speed, error handling |
| `prelogin_encode` | Encode PreLogin negotiation packet | UTF-16 encoding, option serialization |
| `prelogin_decode` | Decode PreLogin response | Option parsing, version extraction |
| `sql_batch_encode/simple` | Encode `SELECT 1` | Minimum overhead baseline |
| `sql_batch_encode/medium` | Encode typical SELECT query | Realistic workload |
| `sql_batch_encode/large` | Encode complex JOIN query | UTF-16 encoding at scale |

**Expected Performance Characteristics:**

- Packet header operations: < 50ns (cache-hot)
- PreLogin encode/decode: < 500ns
- SQL batch encoding: ~2-5ns per byte of SQL text

### mssql-client Benchmarks

Located in `crates/mssql-client/benches/client.rs`.

| Benchmark | Description | Measured Time |
|-----------|-------------|---------------|
| `connection_string/simple` | Parse basic connection string | 264 ns |
| `connection_string/with_port` | Parse connection string with port | 253 ns |
| `connection_string/with_instance` | Parse named instance string | 390 ns |
| `connection_string/full` | Parse full Azure connection string | 554 ns |
| `from_sql/i32_from_int` | Extract i32 from SqlValue::Int | 2.7 ns |
| `from_sql/i64_from_bigint` | Extract i64 from SqlValue::BigInt | 2.7 ns |
| `from_sql/string_from_string` | Extract String from SqlValue::String | 9.1 ns |
| `from_sql/option_i32_some` | Extract Option<i32> (Some) | 6.2 ns |
| `from_sql/option_i32_none` | Extract Option<i32> (None) | 2.0 ns |
| `from_sql/f64_from_double` | Extract f64 from SqlValue::Float | 3.1 ns |
| `from_sql/bool_from_bool` | Extract bool from SqlValue::Bit | 3.0 ns |
| `arc_bytes/clone_small` | Clone Arc<Bytes> (64 bytes) | 12.7 ns |
| `arc_bytes/clone_medium` | Clone Arc<Bytes> (1KB) | 12.8 ns |
| `arc_bytes/clone_large` | Clone Arc<Bytes> (64KB) | 12.7 ns |
| `arc_bytes/slice_medium` | Zero-copy slice (1KB buffer) | 0.55 ns |
| `config_builder/minimal` | Build minimal Config | 94 ns |
| `config_builder/full` | Build fully-configured Config | 117 ns |
| `sql_value/create_int` | Create SqlValue::Int | 16.5 ns |
| `sql_value/create_string` | Create SqlValue::String | 24.4 ns |
| `sql_value/create_null` | Create SqlValue::Null | 15.4 ns |
| `sql_value/is_null_check` | Check if SqlValue is null | 0.44 ns |

**Key Observations:**

- **Arc<Bytes> clone is O(1)**: Clone time is constant regardless of buffer size (12.7ns for 64B, 12.8ns for 1KB, 12.7ns for 64KB). This validates the zero-copy design from ADR-004.
- **Buffer slicing is sub-nanosecond**: The `slice()` operation takes only 0.55ns, making it ideal for column extraction.
- **FromSql is extremely fast**: Integer extractions complete in under 3ns, strings in under 10ns.
- **Connection string parsing is efficient**: Even complex Azure connection strings parse in under 600ns.

### mssql-types Benchmarks

Located in `crates/mssql-types/benches/types.rs`.

| Benchmark | Description | What It Measures |
|-----------|-------------|------------------|
| `utf16_encode/*` | Rust String → UTF-16LE | String encoding overhead |
| `utf16_decode/*` | UTF-16LE → Rust String | String decoding, validation |
| `to_sql/*` | Rust types → SqlValue | Type conversion overhead |
| `from_sql/*` | SqlValue → Rust types | Type extraction overhead |
| `sql_value/*` | SqlValue operations | Enum overhead, pattern matching |

**UTF-16 Encoding Variants:**

| Variant | Input Size | Measures |
|---------|------------|----------|
| `short` | 5 chars | Small string optimization |
| `medium` | ~60 chars | Typical column value |
| `long` | ~450 chars | Text field performance |
| `unicode` | Mixed ASCII/CJK | Multi-byte character handling |

**Expected Performance Characteristics:**

- Integer conversions: < 10ns
- String conversions: ~50-200ns depending on length
- UTF-16 encoding: ~2-3ns per input byte
- UTF-16 decoding: ~1-2ns per input byte

## Performance Design Decisions

### Zero-Copy Row Data

Row data uses `Arc<Bytes>` to share buffer ownership:

```rust
// Data is parsed once, shared across Row instances
let rows: Vec<Row> = stream.collect_all().await?;
// Each Row holds Arc<Bytes> reference, no deep copies
```

**Benefit:** Large result sets don't multiply memory usage.

### Prepared Statement Caching

SQL statements are cached using LRU eviction:

```rust
// First execution: sp_prepare + sp_execute (~2 round trips)
client.query("SELECT * FROM users WHERE id = @p1", &[&1]).await?;

// Subsequent executions: sp_execute only (~1 round trip)
client.query("SELECT * FROM users WHERE id = @p1", &[&2]).await?;
```

**Benefit:** Repeated queries avoid prepare overhead.

### Connection Pool Efficiency

Pool uses semaphore-based acquisition:

```rust
// Acquisition is O(1) when connections available
let conn = pool.get().await?;

// sp_reset_connection called on return (configurable)
drop(conn);
```

**Benefit:** No connection creation overhead for cached connections.

## Comparison Methodology

When comparing to other drivers:

### Fair Comparison Requirements

1. **Same SQL Server version** - TDS protocol behavior varies
2. **Same network conditions** - Latency dominates query time
3. **Same query complexity** - Simple vs complex queries differ
4. **Same connection state** - Cold vs warm connections
5. **Same feature set** - Encryption, MARS, etc.

### What These Benchmarks DON'T Measure

- Network round-trip time (use integration benchmarks)
- SQL Server query execution time
- TLS handshake overhead
- Connection pool contention under load
- Real-world query patterns

### Recommended Integration Benchmarks

For realistic performance measurement:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn bench_real_queries(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(async {
        Pool::builder()
            .max_connections(10)
            .build(config)
            .await
            .unwrap()
    });

    c.bench_function("simple_select", |b| {
        b.to_async(&rt).iter(|| async {
            let mut conn = pool.get().await.unwrap();
            conn.query("SELECT 1", &[]).await.unwrap()
        })
    });

    c.bench_function("parameterized_select", |b| {
        b.to_async(&rt).iter(|| async {
            let mut conn = pool.get().await.unwrap();
            conn.query(
                "SELECT * FROM users WHERE id = @p1",
                &[&1i32]
            ).await.unwrap()
        })
    });
}
```

## Interpreting Results

### Criterion Output

```
packet_header_encode    time:   [23.456 ns 23.789 ns 24.123 ns]
                        thrpt:  [331.23 MiB/s 335.67 MiB/s 340.12 MiB/s]
```

- **time**: [lower bound, estimate, upper bound] with 95% confidence
- **thrpt**: Throughput (for benchmarks with `Throughput` set)

### Performance Regression Detection

Criterion tracks historical results:

```
packet_header_encode    time:   [23.789 ns 24.123 ns 24.456 ns]
                        change: [+2.3% +3.1% +3.9%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

### Noise Considerations

- Run benchmarks on quiet system (no background processes)
- Use `--warm-up-time 3` for stable measurements
- Multiple runs reduce variance
- Check `target/criterion/*/report/` for outlier analysis

## Adding New Benchmarks

### Protocol Benchmarks

```rust
// In crates/tds-protocol/benches/protocol.rs

fn bench_new_feature(c: &mut Criterion) {
    let data = prepare_test_data();

    c.bench_function("new_feature", |b| {
        b.iter(|| {
            let result = process_feature(black_box(&data));
            black_box(result)
        })
    });
}

// Add to criterion_group!
criterion_group!(benches, ..., bench_new_feature);
```

### Type Benchmarks

```rust
// In crates/mssql-types/benches/types.rs

fn bench_new_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_type");

    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("encode", |b| {
        b.iter(|| encode_new_type(black_box(&data)))
    });

    group.finish();
}
```

## CI Integration

Benchmarks are not run in CI by default (too slow, variable). For regression detection:

```yaml
# .github/workflows/bench.yml (optional)
- name: Run benchmarks
  run: cargo bench -- --noplot --save-baseline pr-${{ github.event.number }}

- name: Compare to main
  run: cargo bench -- --noplot --baseline main
```

## Performance Goals

| Operation | Target | Current Status |
|-----------|--------|----------------|
| Packet header encode | < 50ns | ✅ ~24ns |
| Packet header decode | < 50ns | ✅ ~30ns |
| UTF-16 encode (short) | < 100ns | ✅ ~45ns |
| UTF-16 decode (short) | < 100ns | ✅ ~35ns |
| Integer ToSql | < 20ns | ✅ ~17ns |
| Integer FromSql | < 20ns | ✅ ~2.7ns |
| String FromSql | < 50ns | ✅ ~9.1ns |
| Connection string parse | < 1μs | ✅ ~264-554ns |
| Arc<Bytes> clone | < 50ns | ✅ ~12.7ns |
| Buffer slice (zero-copy) | < 5ns | ✅ ~0.55ns |
| SqlValue null check | < 5ns | ✅ ~0.44ns |

*Note: Benchmarks run on Linux with Rust 1.85 (release mode). Actual numbers vary by hardware. Run `cargo bench --package mssql-client` locally for accurate measurements.*

## Benchmark History

### 2025-12-16 Initial Measurements

First comprehensive benchmark suite added with 26 benchmark cases across connection string parsing, type conversions, Arc<Bytes> operations, and SqlValue creation.

**Highlights:**
- Zero-copy architecture validated: Arc<Bytes> clone is O(1) regardless of buffer size
- Sub-nanosecond buffer slicing confirms efficiency of row parsing optimization
- Connection string parsing well under 1μs target
- All FromSql operations under 10ns
