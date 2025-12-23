# Unsupported Features

This document lists features that are explicitly **not supported** by rust-mssql-driver, along with the rationale and recommended alternatives.

## Explicit Non-Goals

The following features are intentionally not planned for implementation:

### 1. MARS (Multiple Active Result Sets)

**Status:** Not Planned

**Rationale:** MARS adds significant protocol complexity for limited benefit. It allows multiple active statements on a single connection, but:
- Increases connection state complexity
- Makes cancellation semantics ambiguous
- Most applications work fine with connection pooling instead

**Alternative:** Use connection pooling (`mssql-driver-pool`) to run concurrent queries on separate connections. This is simpler, more predictable, and often performs better.

### 2. Runtime Agnosticism

**Status:** Not Planned

**Rationale:** This driver is Tokio-native by design. Supporting multiple async runtimes (async-std, smol) would:
- Increase maintenance burden
- Prevent optimizations specific to Tokio
- Add conditional compilation complexity

**Alternative:** Use Tokio. It's the dominant async runtime in the Rust ecosystem.

### 3. SQL Server 2008 / 2012 Support

**Status:** Not Planned

**Rationale:** These versions are out of extended support from Microsoft. Supporting them would:
- Require TDS 7.1/7.2 protocol variants
- Prevent use of modern TDS features
- Increase testing matrix significantly

**Alternative:** Upgrade to SQL Server 2014+ (TDS 7.4) or SQL Server 2022 (TDS 8.0).

### 4. Windows SSPI Authentication (Native)

**Status:** Not Planned for Cross-Platform

**Rationale:** Native SSPI is Windows-only. On Windows, consider:
- Using SQL Server authentication with secure credential storage
- Using Azure AD authentication for cloud deployments

**What IS Supported:** Kerberos/GSSAPI authentication on Linux/macOS via the `integrated-auth` feature, which is wire-compatible with SSPI.

### 5. Named Pipes Transport

**Status:** Not Planned

**Rationale:** Named pipes are:
- Windows-only
- Rarely used in modern deployments
- TCP/IP is universally supported and performs similarly

**Alternative:** Use TCP/IP connections (the default).

### 6. Shared Memory Transport

**Status:** Not Planned

**Rationale:** Shared memory transport is:
- Windows-only (localhost connections)
- Undocumented in the public TDS specification
- Limited to same-machine scenarios where TCP/IP localhost works equally well

**Alternative:** Use TCP/IP connections to `localhost` or `127.0.0.1`.

### 7. Linked Servers / Distributed Queries

**Status:** Not Planned (Driver-Level)

**Rationale:** Linked server queries work at the SQL Server level, not the driver level. The driver executes whatever SQL you send.

**Alternative:** Your queries can use linked servers - the driver doesn't need special support.

---

## Unsupported Data Types

The following SQL Server data types are not currently supported:

### Spatial Types

| Type | Status | Rationale |
|------|--------|-----------|
| `GEOMETRY` | Not Supported | Complex binary format, requires geometric computation library |
| `GEOGRAPHY` | Not Supported | Complex binary format, requires geodetic computation library |

**Workaround:** Convert to WKT (Well-Known Text) or GeoJSON in SQL:

```sql
-- Return as WKT string
SELECT Location.STAsText() AS LocationWkt FROM Places;

-- Return as GeoJSON (SQL Server 2016+)
SELECT Location.STAsGeoJSON() AS LocationJson FROM Places;
```

### Hierarchical Types

| Type | Status | Rationale |
|------|--------|-----------|
| `HIERARCHYID` | Not Supported | Proprietary binary format with complex path operations |

**Workaround:** Convert to string representation in SQL:

```sql
SELECT OrgNode.ToString() AS OrgPath FROM OrgChart;
```

### User-Defined Types (UDT)

| Type | Status | Rationale |
|------|--------|-----------|
| CLR UDTs | Not Supported | Requires .NET CLR integration and type metadata |
| Alias Types | Supported | Treated as their underlying base type |

**Workaround:** Convert UDTs to standard types in your queries.

### Sparse Columns

**Status:** Partial Support

Sparse columns are returned with their base data type. The `COLUMN_SET` XML representation is not automatically generated.

**Workaround:** Query the column set explicitly if needed:

```sql
SELECT *, SparseColumnSet FROM TableWithSparseColumns;
```

---

## Platform Limitations

### SQL Server Express LocalDB

**Status:** Not Tested

LocalDB uses a different connection mechanism (automatic instance management via Windows named pipes). While TCP/IP connections to LocalDB may work, this configuration is not tested.

**Alternative:** Use a full SQL Server Express instance with TCP/IP enabled, or use Docker containers for local development.

### 32-bit Platforms

**Status:** Not Supported

The driver is only tested on 64-bit platforms (x86_64, aarch64). 32-bit builds may work but are not part of the CI matrix.

**Rationale:** 32-bit systems are increasingly rare in production environments, and testing resources are limited.

---

## Connection Pool Limitations

### Thread Sharing

**Status:** Not Supported

`Client<S>` is single-owner by design. Connections cannot be shared between tasks without explicit synchronization.

**Rationale:** This simplifies the state machine and prevents race conditions in query execution.

**Alternative:** Use `Pool` to manage multiple connections, acquiring one per task as needed.

### Pool Metrics/Instrumentation

**Status:** Basic Only

The pool exposes basic metrics (size, available connections) but not detailed instrumentation.

| Metric | Status |
|--------|--------|
| Pool size | Available |
| Available connections | Available |
| Wait queue depth | Not Exposed |
| Connection acquisition time histogram | Not Exposed |
| Connection lifetime histogram | Not Exposed |

**Planned:** OpenTelemetry metrics integration is planned for v0.2.0.

### TTL-Based Connection Expiration

**Status:** Not Supported

Connections are evicted based on LRU (Least Recently Used) policy, not time-based expiration.

**Rationale:** LRU provides a good balance between connection reuse and resource cleanup without the complexity of background timers.

**Alternative:** If you need to force connection refresh, reduce the pool size or periodically recycle the pool.

### Custom Health Checks

**Status:** Not Supported

The pool uses a hardcoded `SELECT 1` query for health checks. Custom health check logic is not supported.

**Rationale:** `SELECT 1` is sufficient for connection liveness and doesn't require database-specific knowledge.

---

## Statement Cache Limitations

### Cross-Connection Statement Sharing

**Status:** Not Supported (By Design)

Prepared statement handles are connection-specific in SQL Server. Each connection maintains its own LRU statement cache.

**Rationale:** Sharing would require complex coordination and could lead to race conditions.

### TTL-Based Statement Expiration

**Status:** Not Supported

Cached statements are evicted based on LRU policy, not time-based expiration.

**Rationale:** LRU naturally evicts stale statements as new ones are prepared. Time-based expiration adds complexity without significant benefit.

---

## Administrative Features

The following administrative and diagnostic features are not directly exposed:

### Extended Events Integration

**Status:** Not Supported

There is no API for programmatic Extended Events session management.

**Workaround:** Use SQL commands to manage Extended Events sessions:

```sql
CREATE EVENT SESSION [MySession] ON SERVER ...
```

### Query Plan Retrieval

**Status:** Not Exposed

There is no direct API to retrieve query plans.

**Workaround:** Use SHOWPLAN options:

```sql
SET SHOWPLAN_XML ON;
-- Your query here
SET SHOWPLAN_XML OFF;
```

Or query the plan cache:

```sql
SELECT query_plan FROM sys.dm_exec_query_plan(plan_handle);
```

### Login Retry/Backoff Configuration

**Status:** Basic Only

Connection retry uses a simple fixed policy. Exponential backoff with jitter is not configurable.

**Planned:** Consider using a retry middleware or implementing custom retry logic in your application.

### Circuit Breaker Pattern

**Status:** Not Implemented

There is no built-in circuit breaker for failing connections.

**Alternative:** Implement circuit breaker logic in your application using crates like `failsafe` or `backoff`.

---

## Features Planned for Future Releases

The following features are planned but not yet implemented:

### v0.2.0 Targets

| Feature | Description | Status |
|---------|-------------|--------|
| Table-Valued Parameters (TVP) | Pass structured data as parameters | API defined, encoding pending |
| Always Encrypted | Client-side encryption for sensitive columns | Planned |
| Change Tracking Integration | Built-in change tracking query support | Planned |
| OpenTelemetry Metrics | Counter/histogram metrics (tracing already works) | Planned |

### Workarounds

#### Table-Valued Parameters

Until TVP is fully implemented:

```sql
-- Option 1: Temporary table
CREATE TABLE #UserIds (UserId INT);
INSERT INTO #UserIds VALUES (1), (2), (3);
SELECT * FROM Users WHERE UserId IN (SELECT UserId FROM #UserIds);

-- Option 2: JSON (SQL Server 2016+)
SELECT * FROM Users WHERE UserId IN (SELECT value FROM OPENJSON(@json));

-- Option 3: XML
SELECT * FROM Users WHERE UserId IN (
  SELECT x.value('.', 'INT') FROM @xml.nodes('/ids/id') AS T(x)
);
```

#### Always Encrypted

Until Always Encrypted is implemented:
- Use application-layer encryption before sending data to SQL Server
- Use Transparent Data Encryption (TDE) for data-at-rest protection
- **Do NOT use `ENCRYPTBYKEY`** as a workaround - it does not provide the same security guarantees (keys are accessible to DBAs)

---

## Protocol Limitations

### Large Object (LOB) Streaming

**Current Status:** Buffered

LOBs (VARBINARY(MAX), NVARCHAR(MAX), XML) are currently buffered in memory before being returned. The `BlobReader` API provides chunked reading from this buffer.

**For LOBs over 100MB:** Consider chunking via SQL:

```sql
-- Read in chunks
SELECT SUBSTRING(Data, @offset, @length) FROM Documents WHERE Id = @id;
```

### Cursor Support

**Status:** Not Implemented

Server-side cursors are not directly supported. However:
- Result set streaming is supported and efficient
- For large datasets, consider pagination with `OFFSET`/`FETCH`

---

## Why Not Support X?

If you're wondering why a specific feature isn't supported, the general principles are:

1. **Complexity vs. Value:** Features that add significant complexity for limited benefit are deprioritized
2. **Modern Practices:** Features obsoleted by modern alternatives are not implemented
3. **Cross-Platform:** Windows-only features are generally not supported
4. **Security:** Features with security implications receive extra scrutiny

## Feature Requests

If you need a feature not listed here:

1. Check if it's already tracked in [GitHub Issues](https://github.com/praxiomlabs/rust-mssql-driver/issues)
2. Open an issue with your use case
3. Consider whether a workaround exists

We prioritize features based on community need and alignment with the driver's goals.
