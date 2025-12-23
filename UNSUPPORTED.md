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

### 6. Linked Servers / Distributed Queries

**Status:** Not Planned (Driver-Level)

**Rationale:** Linked server queries work at the SQL Server level, not the driver level. The driver executes whatever SQL you send.

**Alternative:** Your queries can use linked servers - the driver doesn't need special support.

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
