# Code Signing & Notarization

This document covers signing OxideKit applications for distribution.

## macOS

### Developer ID Certificate

1. Enroll in the Apple Developer Program
2. Create a "Developer ID Application" certificate in Xcode or Apple Developer portal
3. Export and install the certificate

### Signing

```bash
# Build release
oxide build --release --target macos

# Sign the app bundle
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --options runtime \
  target/release/bundle/macos/YourApp.app
```

### Notarization

```bash
# Create ZIP for notarization
ditto -c -k --keepParent \
  target/release/bundle/macos/YourApp.app \
  YourApp.zip

# Submit for notarization
xcrun notarytool submit YourApp.zip \
  --apple-id "your@email.com" \
  --team-id "TEAMID" \
  --password "@keychain:AC_PASSWORD" \
  --wait

# Staple the ticket
xcrun stapler staple target/release/bundle/macos/YourApp.app
```

### Hardened Runtime

OxideKit apps use hardened runtime by default. If you need specific entitlements:

```xml
<!-- Entitlements.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <false/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <false/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <false/>
</dict>
</plist>
```

## Windows

### Code Signing Certificate

Obtain an EV (Extended Validation) certificate from a trusted CA:
- DigiCert
- Sectigo
- GlobalSign

### Signing with SignTool

```powershell
# Build release
oxide build --release --target windows

# Sign the executable
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /n "Your Company Name" ^
  target\release\YourApp.exe
```

### Windows Installer (MSI)

Use WiX Toolset to create signed MSI installers:

```xml
<!-- Product.wxs -->
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="YourApp" Version="1.0.0"
           Manufacturer="Your Company" Language="1033">
    <Package InstallerVersion="200" Compressed="yes"/>
    <Media Id="1" Cabinet="app.cab" EmbedCab="yes"/>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLDIR" Name="YourApp">
          <Component Id="MainExecutable" Guid="*">
            <File Id="AppExe" Source="target\release\YourApp.exe" KeyPath="yes"/>
          </Component>
        </Directory>
      </Directory>
    </Directory>
    <Feature Id="Complete" Level="1">
      <ComponentRef Id="MainExecutable"/>
    </Feature>
  </Product>
</Wix>
```

## Linux

### GPG Signing

```bash
# Generate a GPG key
gpg --full-generate-key

# Sign the binary
gpg --armor --detach-sign target/release/your-app

# Verify signature
gpg --verify target/release/your-app.asc target/release/your-app
```

### AppImage Signing

```bash
# Build AppImage
oxide build --release --target appimage

# Sign with GPG
gpg --armor --detach-sign YourApp.AppImage
```

### Debian/RPM Packages

For distribution via apt/dnf, sign packages:

```bash
# Debian
dpkg-sig --sign builder your-app_1.0.0_amd64.deb

# RPM
rpmsign --addsign your-app-1.0.0.x86_64.rpm
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/release.yml
- name: Sign macOS
  if: matrix.os == 'macos-latest'
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    echo $APPLE_CERTIFICATE | base64 --decode > certificate.p12
    security create-keychain -p "" build.keychain
    security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD"
    security set-keychain-settings build.keychain
    security unlock-keychain -p "" build.keychain
    codesign --deep --force --sign "Developer ID Application" target/release/YourApp
```

## SBOM Generation

OxideKit can generate Software Bill of Materials:

```bash
# Generate SBOM
oxide legal sbom --format spdx > sbom.spdx.json

# Or CycloneDX format
oxide legal sbom --format cyclonedx > sbom.cdx.json
```

Include SBOM with releases for supply chain transparency.

## Build Attestation

```bash
# Generate attestation report
oxide attest report target/release/your-app

# Verify a build
oxide attest verify target/release/your-app
```

See [Attestation](./attestation.md) for details on the attestation service.
