# Build Attestation

OxideKit provides build attestation to verify that applications match their declared permissions and dependencies.

## Local Attestation

### Generate Report

```bash
# Generate attestation report for a binary
oxide attest report target/release/my-app

# Output: attestation.json + attestation.md
```

### Report Contents

The attestation report includes:

- **Build metadata**: Version, timestamp, compiler version
- **Declared permissions**: From oxide.toml
- **Network allowlist**: Declared hosts
- **Filesystem access**: Declared paths
- **Dependencies**: Full SBOM
- **Extension versions**: Declared extensions and versions

### Verify Locally

```bash
# Verify a build against its attestation
oxide attest verify target/release/my-app

# Output:
# Checking permissions... OK
# Checking network allowlist... OK
# Checking dependencies... OK
# Build attestation: VALID
```

## Attestation Service (Future)

The OxideKit Attestation Service provides third-party verification:

### How It Works

1. **Upload**: Developer submits binary + attestation
2. **Verify**: Service verifies signature and permissions
3. **Scan**: Service checks for known vulnerabilities
4. **Attest**: Service issues signed attestation
5. **Publish**: Attestation published to registry

### Verification Levels

| Level | Checks | Badge |
|-------|--------|-------|
| **Basic** | Signature valid, permissions declared | Verified |
| **Standard** | + Dependency audit, no known CVEs | Audited |
| **Enterprise** | + Manual review, reproducible build | Certified |

### User Experience

When users install an app:

```
Installing my-app v1.0.0...

Permissions:
  Network: api.example.com, cdn.example.com
  Filesystem: read ./config, write ./data

Attestation: Verified by OxideKit
  Level: Standard (audited)
  Last checked: 2026-01-15

Continue installation? [Y/n]
```

## attestation.json Schema

```json
{
  "version": "1.0",
  "app": {
    "id": "com.example.myapp",
    "name": "My App",
    "version": "1.0.0"
  },
  "build": {
    "timestamp": "2026-01-15T10:30:00Z",
    "compiler": "rustc 1.75.0",
    "target": "aarch64-apple-darwin",
    "profile": "release"
  },
  "permissions": {
    "network": {
      "allowed_hosts": ["api.example.com", "cdn.example.com"],
      "protocols": ["https"]
    },
    "filesystem": {
      "read": ["./config"],
      "write": ["./data"]
    }
  },
  "dependencies": {
    "sbom_url": "./sbom.spdx.json",
    "hash": "sha256:abc123..."
  },
  "extensions": [
    {"id": "ui.charts", "version": "0.2.0"},
    {"id": "native.keychain", "version": "0.1.0"}
  ],
  "signature": {
    "algorithm": "ed25519",
    "public_key": "...",
    "value": "..."
  }
}
```

## Human-Readable Report

The `attestation.md` file provides a human-readable summary:

```markdown
# Build Attestation Report

**Application**: My App (com.example.myapp)
**Version**: 1.0.0
**Built**: January 15, 2026

## Permissions

### Network Access
This application can connect to:
- api.example.com (HTTPS only)
- cdn.example.com (HTTPS only)

### Filesystem Access
- Read: ./config
- Write: ./data

## Dependencies
This application includes 47 dependencies.
See sbom.spdx.json for full details.

No known vulnerabilities detected.

## Extensions
- ui.charts v0.2.0
- native.keychain v0.1.0

## Verification
Signature: Valid
Hash: sha256:abc123...
```

## CLI Commands

```bash
# Generate attestation
oxide attest report <binary>

# Verify attestation
oxide attest verify <binary>

# Upload to service (future)
oxide attest publish <binary>

# Check attestation status (future)
oxide attest status com.example.myapp
```

## Integration with CI/CD

```yaml
# .github/workflows/release.yml
- name: Generate Attestation
  run: |
    oxide attest report target/release/my-app

- name: Upload Attestation
  uses: actions/upload-artifact@v4
  with:
    name: attestation
    path: |
      attestation.json
      attestation.md
      sbom.spdx.json
```

## Best Practices

1. **Include attestation in releases**: Upload alongside binaries
2. **Pin dependency versions**: Use Cargo.lock for reproducibility
3. **Declare all permissions**: Be explicit about what your app needs
4. **Review before signing**: Verify attestation matches intent
5. **Keep keys secure**: Protect signing keys appropriately
