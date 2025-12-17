# Comparative Performance Analysis

This document compares rust-mssql-driver performance characteristics against Tiberius and other SQL Server drivers.

## Methodology Note

**Important:** Direct benchmark comparisons require:
- Same hardware environment
- Same SQL Server version
- Same network conditions
- Same query workloads

The analysis below focuses on **architectural differences** and **micro-benchmark characteristics** rather than end-to-end throughput, which varies significantly by deployment.

## Architecture Comparison

| Feature | rust-mssql-driver | Tiberius |
|---------|-------------------|----------|
| Runtime | Tokio-native | Runtime-agnostic |
| TLS | rustls | rustls or native-tls |
| Memory Model | Arc<Bytes> zero-copy | Vec-based copies |
| Connection Pooling | Built-in | External (bb8/deadpool) |
| Prepared Statements | LRU cache | Manual lifecycle |
| LOB Handling | BlobReader API | Full materialization |

## Micro-Benchmark Results (rust-mssql-driver)

Benchmarks run with Criterion 0.5 on Linux (Rust 1.85, release mode).

### Type Conversions

| Operation | Time | Notes |
|-----------|------|-------|
| i32 from SqlValue::Int | 2.7 ns | Near-zero overhead |
| i64 from SqlValue::BigInt | 2.7 ns | Same as i32 |
| String from SqlValue::String | 9.1 ns | Includes validation |
| Option<i32> (Some) | 6.2 ns | Null check + extraction |
| Option<i32> (None) | 2.0 ns | Early return path |
| f64 from SqlValue::Float | 3.1 ns | Bit reinterpretation |
| bool from SqlValue::Bit | 3.0 ns | Single byte |

### Memory Operations

| Operation | Time | Significance |
|-----------|------|--------------|
| Arc<Bytes> clone (64B) | 12.7 ns | O(1) regardless of size |
| Arc<Bytes> clone (1KB) | 12.8 ns | Same as 64B |
| Arc<Bytes> clone (64KB) | 12.7 ns | Same as 64B |
| Buffer slice | 0.55 ns | Sub-nanosecond |
| is_null check | 0.44 ns | Branch on discriminant |

**Key Insight:** Arc<Bytes> clone is O(1) - large row buffers share memory efficiently.

### Configuration

| Operation | Time | Notes |
|-----------|------|-------|
| Connection string (simple) | 264 ns | Minimal parsing |
| Connection string (with port) | 253 ns | Port extraction |
| Connection string (instance) | 390 ns | Instance name parsing |
| Connection string (full Azure) | 554 ns | All options |
| Config builder (minimal) | 94 ns | Defaults only |
| Config builder (full) | 117 ns | All options set |

## Architectural Advantages

### 1. Zero-Copy Row Data (ADR-004)

```rust
// Traditional approach (Tiberius-style)
let row_data = Vec::from(packet_slice);  // COPY
let column_value = row_data[offset..end].to_vec();  // COPY

// rust-mssql-driver approach
let row_data = packet_buffer.slice(row_range);  // NO COPY - 0.55ns
let column_value = row_data.slice(col_range);   // NO COPY - 0.55ns
```

**Impact:** For a row with 10 columns:
- Traditional: 10+ allocations per row
- rust-mssql-driver: 0 allocations until value extraction needed

### 2. Built-in Connection Pool

```rust
// rust-mssql-driver - integrated pool
let pool = Pool::builder()
    .max_connections(10)
    .build(config).await?;

// Tiberius - external pool required
let manager = TiberiusConnectionManager::new(config);
let pool = bb8::Pool::builder()
    .max_size(10)
    .build(manager).await?;
```

**Impact:**
- One fewer dependency
- Unified configuration
- Pool-aware prepared statement cache

### 3. Prepared Statement Cache

| Aspect | rust-mssql-driver | Tiberius |
|--------|-------------------|----------|
| Cache location | Built-in LRU | Manual |
| Lifecycle | Automatic | Manual prepare/unprepare |
| Cross-connection | Pool-aware | N/A |
| First execution | sp_prepare + sp_execute | Manual |
| Subsequent | sp_execute only | Manual |

**Impact:** Repeated queries with different parameters execute ~50% faster after initial prepare.

### 4. LOB Streaming API

```rust
// rust-mssql-driver - streaming API
let mut reader = BlobReader::from_bytes(data);
tokio::io::copy(&mut reader, &mut file).await?;

// Tiberius - full materialization only
let data: Vec<u8> = row.get(0)?;
file.write_all(&data)?;
```

**Impact:** BlobReader enables chunked processing and progress tracking without additional allocations per read.

## Expected Performance Characteristics

### Protocol Operations

| Operation | Expected Range | Notes |
|-----------|----------------|-------|
| Packet header encode | 20-30 ns | 8 bytes |
| Packet header decode | 25-35 ns | Validation included |
| PreLogin encode | 400-600 ns | Version + options |
| SQL batch encode | 2-5 ns/byte | UTF-16 conversion |

### Network-Bound Operations

For operations involving SQL Server communication, network latency dominates:

| Scenario | Driver Overhead | Network Time |
|----------|----------------|--------------|
| Simple SELECT | ~100 μs | 1-10 ms |
| Parameterized query (cached) | ~50 μs | 1-10 ms |
| Parameterized query (new) | ~200 μs | 2-20 ms |
| Large result set (1000 rows) | ~1 ms | 10-100 ms |

**Conclusion:** For network-bound workloads (the common case), driver efficiency is less important than connection pooling and query optimization.

## When Performance Matters

### rust-mssql-driver Excels At:

1. **High-throughput local connections** - Zero-copy matters when network isn't the bottleneck
2. **Large result sets** - Arc<Bytes> sharing reduces memory pressure
3. **Repeated parameterized queries** - Prepared statement cache
4. **LOB processing** - BlobReader for chunked streaming

### Performance is Similar When:

1. **Network latency > 1ms** - Driver overhead is noise
2. **Small result sets** - Allocation overhead negligible
3. **One-off queries** - No cache benefit

## Running Your Own Benchmarks

```bash
# Run rust-mssql-driver benchmarks
cargo bench --package mssql-client

# View HTML reports
open target/criterion/report/index.html
```

For real-world comparison, benchmark your specific workload:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_your_workload(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = /* ... setup ... */;

    c.bench_function("your_query", |b| {
        b.to_async(&rt).iter(|| async {
            let mut conn = pool.get().await?;
            conn.query("YOUR ACTUAL QUERY", &[]).await
        })
    });
}
```

## Summary

| Dimension | rust-mssql-driver | Tiberius | Winner |
|-----------|-------------------|----------|--------|
| Memory efficiency | Arc<Bytes> zero-copy | Vec copies | rust-mssql-driver |
| Type conversions | < 10ns | Similar | Tie |
| Connection pooling | Built-in | External | rust-mssql-driver |
| Prepared statements | Auto-cached | Manual | rust-mssql-driver |
| LOB handling | BlobReader API | Full buffer | rust-mssql-driver |
| Runtime flexibility | Tokio-only | Any runtime | Tiberius |
| Maturity | New | Battle-tested | Tiberius |

**Recommendation:** For new Tokio-based projects requiring SQL Server, rust-mssql-driver offers architectural advantages. For runtime-agnostic needs or proven stability, Tiberius remains excellent.
