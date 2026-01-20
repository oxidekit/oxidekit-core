//! Software Bill of Materials (SBOM) generation
//!
//! Generates SBOM documents in industry-standard formats:
//! - SPDX (ISO/IEC 5962:2021)
//! - CycloneDX
//! - Custom JSON format

use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::LegalResult;
use crate::scanner::{LicenseScanner, ScanResult, DependencyLicense};

/// SBOM format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    /// SPDX 2.3 JSON format
    SpdxJson,
    /// SPDX 2.3 tag-value format
    SpdxTagValue,
    /// CycloneDX 1.5 JSON format
    CycloneDxJson,
    /// CycloneDX 1.5 XML format
    CycloneDxXml,
    /// Custom OxideKit format
    OxideKit,
}

impl SbomFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            SbomFormat::SpdxJson => "spdx.json",
            SbomFormat::SpdxTagValue => "spdx",
            SbomFormat::CycloneDxJson => "cdx.json",
            SbomFormat::CycloneDxXml => "cdx.xml",
            SbomFormat::OxideKit => "sbom.json",
        }
    }
}

/// SBOM generator
#[derive(Debug)]
pub struct SbomGenerator {
    /// Scanner to use
    scanner: LicenseScanner,
    /// Creator tool name
    tool_name: String,
    /// Creator tool version
    tool_version: String,
    /// Document namespace
    namespace: String,
}

impl SbomGenerator {
    /// Create a new SBOM generator
    pub fn new(manifest_path: impl AsRef<Path>) -> Self {
        Self {
            scanner: LicenseScanner::new(manifest_path),
            tool_name: "oxide-legal".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            namespace: "https://oxidekit.com/sbom".to_string(),
        }
    }

    /// Set the creator tool information
    pub fn with_tool(mut self, name: &str, version: &str) -> Self {
        self.tool_name = name.to_string();
        self.tool_version = version.to_string();
        self
    }

    /// Set the document namespace
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = namespace.to_string();
        self
    }

    /// Generate SBOM in the specified format
    pub fn generate(&mut self, format: SbomFormat) -> LegalResult<String> {
        let scan = self.scanner.scan()?;
        let sbom = Sbom::from_scan(&scan, &self.tool_name, &self.tool_version, &self.namespace);

        match format {
            SbomFormat::SpdxJson => sbom.to_spdx_json(),
            SbomFormat::SpdxTagValue => sbom.to_spdx_tag_value(),
            SbomFormat::CycloneDxJson => sbom.to_cyclonedx_json(),
            SbomFormat::CycloneDxXml => sbom.to_cyclonedx_xml(),
            SbomFormat::OxideKit => sbom.to_oxidekit_json(),
        }
    }

    /// Generate and save SBOM to file
    pub fn generate_to_file(&mut self, format: SbomFormat, path: impl AsRef<Path>) -> LegalResult<()> {
        let content = self.generate(format)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Software Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// Document identifier
    pub id: String,
    /// Document name
    pub name: String,
    /// SBOM version
    pub version: String,
    /// Creation timestamp
    pub created: DateTime<Utc>,
    /// Creator information
    pub creators: Vec<SbomCreator>,
    /// Document namespace
    pub namespace: String,
    /// Primary component (the project itself)
    pub primary_component: SbomComponent,
    /// All components (dependencies)
    pub components: Vec<SbomComponent>,
    /// Relationships between components
    pub relationships: Vec<SbomRelationship>,
}

impl Sbom {
    /// Create SBOM from scan results
    pub fn from_scan(scan: &ScanResult, tool_name: &str, tool_version: &str, namespace: &str) -> Self {
        let id = format!("SPDXRef-DOCUMENT-{}", Uuid::new_v4());
        let created = Utc::now();

        // Primary component
        let primary = SbomComponent {
            id: format!("SPDXRef-Package-{}", scan.project_name),
            name: scan.project_name.clone(),
            version: scan.project_version.clone(),
            purl: Some(format!(
                "pkg:cargo/{}@{}",
                scan.project_name, scan.project_version
            )),
            license: scan.project_license.clone().unwrap_or_else(|| "NOASSERTION".to_string()),
            supplier: None,
            download_location: None,
            homepage: None,
            description: None,
            checksums: vec![],
            external_refs: vec![],
        };

        // Convert dependencies to components
        let components: Vec<SbomComponent> = scan
            .dependencies
            .iter()
            .map(SbomComponent::from_dependency)
            .collect();

        // Create relationships
        let mut relationships = vec![
            SbomRelationship {
                element: id.clone(),
                related: primary.id.clone(),
                relationship_type: "DESCRIBES".to_string(),
            },
        ];

        // All components are dependencies of the primary
        for component in &components {
            relationships.push(SbomRelationship {
                element: primary.id.clone(),
                related: component.id.clone(),
                relationship_type: "DEPENDS_ON".to_string(),
            });
        }

        Self {
            id,
            name: format!("{}-sbom", scan.project_name),
            version: "1.0".to_string(),
            created,
            creators: vec![
                SbomCreator {
                    creator_type: "Tool".to_string(),
                    name: format!("{}-{}", tool_name, tool_version),
                },
            ],
            namespace: format!("{}/{}/{}", namespace, scan.project_name, scan.project_version),
            primary_component: primary,
            components,
            relationships,
        }
    }

    /// Export to SPDX 2.3 JSON format
    pub fn to_spdx_json(&self) -> LegalResult<String> {
        let spdx = SpdxDocument::from_sbom(self);
        Ok(serde_json::to_string_pretty(&spdx)?)
    }

    /// Export to SPDX tag-value format
    pub fn to_spdx_tag_value(&self) -> LegalResult<String> {
        let mut output = String::new();

        // Document header
        output.push_str(&format!("SPDXVersion: SPDX-2.3\n"));
        output.push_str(&format!("DataLicense: CC0-1.0\n"));
        output.push_str(&format!("SPDXID: {}\n", self.id));
        output.push_str(&format!("DocumentName: {}\n", self.name));
        output.push_str(&format!("DocumentNamespace: {}\n", self.namespace));

        for creator in &self.creators {
            output.push_str(&format!("Creator: {}: {}\n", creator.creator_type, creator.name));
        }
        output.push_str(&format!("Created: {}\n\n", self.created.format("%Y-%m-%dT%H:%M:%SZ")));

        // Primary package
        output.push_str(&self.primary_component.to_spdx_tag_value());
        output.push('\n');

        // Dependencies
        for component in &self.components {
            output.push_str(&component.to_spdx_tag_value());
            output.push('\n');
        }

        // Relationships
        for rel in &self.relationships {
            output.push_str(&format!(
                "Relationship: {} {} {}\n",
                rel.element, rel.relationship_type, rel.related
            ));
        }

        Ok(output)
    }

    /// Export to CycloneDX 1.5 JSON format
    pub fn to_cyclonedx_json(&self) -> LegalResult<String> {
        let cdx = CycloneDxBom::from_sbom(self);
        Ok(serde_json::to_string_pretty(&cdx)?)
    }

    /// Export to CycloneDX 1.5 XML format
    pub fn to_cyclonedx_xml(&self) -> LegalResult<String> {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<bom xmlns=\"http://cyclonedx.org/schema/bom/1.5\" version=\"1\">\n");

        // Metadata
        xml.push_str("  <metadata>\n");
        xml.push_str(&format!("    <timestamp>{}</timestamp>\n", self.created.format("%Y-%m-%dT%H:%M:%SZ")));
        xml.push_str("    <tools>\n");
        for creator in &self.creators {
            xml.push_str("      <tool>\n");
            xml.push_str(&format!("        <name>{}</name>\n", creator.name));
            xml.push_str("      </tool>\n");
        }
        xml.push_str("    </tools>\n");

        // Primary component
        xml.push_str("    <component type=\"application\">\n");
        xml.push_str(&format!("      <name>{}</name>\n", self.primary_component.name));
        xml.push_str(&format!("      <version>{}</version>\n", self.primary_component.version));
        if let Some(ref purl) = self.primary_component.purl {
            xml.push_str(&format!("      <purl>{}</purl>\n", purl));
        }
        xml.push_str("    </component>\n");
        xml.push_str("  </metadata>\n");

        // Components
        xml.push_str("  <components>\n");
        for component in &self.components {
            xml.push_str("    <component type=\"library\">\n");
            xml.push_str(&format!("      <name>{}</name>\n", component.name));
            xml.push_str(&format!("      <version>{}</version>\n", component.version));
            if let Some(ref purl) = component.purl {
                xml.push_str(&format!("      <purl>{}</purl>\n", purl));
            }
            xml.push_str("      <licenses>\n");
            xml.push_str("        <license>\n");
            xml.push_str(&format!("          <id>{}</id>\n", component.license));
            xml.push_str("        </license>\n");
            xml.push_str("      </licenses>\n");
            xml.push_str("    </component>\n");
        }
        xml.push_str("  </components>\n");

        // Dependencies
        xml.push_str("  <dependencies>\n");
        xml.push_str(&format!("    <dependency ref=\"{}\">\n", self.primary_component.purl.as_deref().unwrap_or("")));
        for component in &self.components {
            if let Some(ref purl) = component.purl {
                xml.push_str(&format!("      <dependency ref=\"{}\"/>\n", purl));
            }
        }
        xml.push_str("    </dependency>\n");
        xml.push_str("  </dependencies>\n");

        xml.push_str("</bom>");

        Ok(xml)
    }

    /// Export to OxideKit JSON format
    pub fn to_oxidekit_json(&self) -> LegalResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Get total component count (excluding primary)
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Get unique license count
    pub fn unique_license_count(&self) -> usize {
        let mut licenses: std::collections::HashSet<&str> = std::collections::HashSet::new();
        licenses.insert(&self.primary_component.license);
        for c in &self.components {
            licenses.insert(&c.license);
        }
        licenses.len()
    }
}

/// SBOM component (package/library)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomComponent {
    /// Component identifier (SPDX ref)
    pub id: String,
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package URL (PURL)
    pub purl: Option<String>,
    /// License (SPDX expression)
    pub license: String,
    /// Supplier/vendor
    pub supplier: Option<String>,
    /// Download location
    pub download_location: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Description
    pub description: Option<String>,
    /// File checksums
    pub checksums: Vec<SbomChecksum>,
    /// External references
    pub external_refs: Vec<SbomExternalRef>,
}

impl SbomComponent {
    /// Create component from dependency license info
    pub fn from_dependency(dep: &DependencyLicense) -> Self {
        Self {
            id: format!("SPDXRef-Package-{}-{}", dep.name, dep.version.replace('.', "-")),
            name: dep.name.clone(),
            version: dep.version.clone(),
            purl: Some(format!("pkg:cargo/{}@{}", dep.name, dep.version)),
            license: dep.license.spdx_id.clone(),
            supplier: if !dep.authors.is_empty() {
                Some(dep.authors.join(", "))
            } else {
                None
            },
            download_location: dep.repository.clone(),
            homepage: dep.repository.clone(),
            description: dep.description.clone(),
            checksums: vec![],
            external_refs: vec![],
        }
    }

    /// Convert to SPDX tag-value format
    fn to_spdx_tag_value(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("PackageName: {}\n", self.name));
        output.push_str(&format!("SPDXID: {}\n", self.id));
        output.push_str(&format!("PackageVersion: {}\n", self.version));

        if let Some(ref supplier) = self.supplier {
            output.push_str(&format!("PackageSupplier: Person: {}\n", supplier));
        }

        output.push_str(&format!(
            "PackageDownloadLocation: {}\n",
            self.download_location.as_deref().unwrap_or("NOASSERTION")
        ));

        output.push_str("FilesAnalyzed: false\n");
        output.push_str(&format!("PackageLicenseConcluded: {}\n", self.license));
        output.push_str(&format!("PackageLicenseDeclared: {}\n", self.license));
        output.push_str("PackageCopyrightText: NOASSERTION\n");

        if let Some(ref desc) = self.description {
            output.push_str(&format!("PackageSummary: {}\n", desc));
        }

        for checksum in &self.checksums {
            output.push_str(&format!("PackageChecksum: {}: {}\n", checksum.algorithm, checksum.value));
        }

        output
    }
}

/// SBOM creator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomCreator {
    /// Creator type (Tool, Person, Organization)
    pub creator_type: String,
    /// Creator name/identifier
    pub name: String,
}

/// SBOM relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomRelationship {
    /// Source element ID
    pub element: String,
    /// Target element ID
    pub related: String,
    /// Relationship type
    pub relationship_type: String,
}

/// File checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomChecksum {
    /// Algorithm (SHA256, SHA1, MD5)
    pub algorithm: String,
    /// Checksum value
    pub value: String,
}

/// External reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomExternalRef {
    /// Reference type
    pub ref_type: String,
    /// Reference locator
    pub locator: String,
}

/// SPDX 2.3 document structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpdxDocument {
    spdx_version: String,
    data_license: String,
    #[serde(rename = "SPDXID")]
    spdx_id: String,
    name: String,
    document_namespace: String,
    creation_info: SpdxCreationInfo,
    packages: Vec<SpdxPackage>,
    relationships: Vec<SpdxRelationship>,
}

impl SpdxDocument {
    fn from_sbom(sbom: &Sbom) -> Self {
        let mut packages = vec![SpdxPackage::from_component(&sbom.primary_component)];
        packages.extend(sbom.components.iter().map(SpdxPackage::from_component));

        Self {
            spdx_version: "SPDX-2.3".to_string(),
            data_license: "CC0-1.0".to_string(),
            spdx_id: sbom.id.clone(),
            name: sbom.name.clone(),
            document_namespace: sbom.namespace.clone(),
            creation_info: SpdxCreationInfo {
                created: sbom.created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                creators: sbom.creators.iter().map(|c| format!("{}: {}", c.creator_type, c.name)).collect(),
            },
            packages,
            relationships: sbom.relationships.iter().map(|r| SpdxRelationship {
                spdx_element_id: r.element.clone(),
                relationship_type: r.relationship_type.clone(),
                related_spdx_element: r.related.clone(),
            }).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpdxCreationInfo {
    created: String,
    creators: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpdxPackage {
    #[serde(rename = "SPDXID")]
    spdx_id: String,
    name: String,
    version_info: String,
    download_location: String,
    files_analyzed: bool,
    license_concluded: String,
    license_declared: String,
    copyright_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    supplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    external_refs: Vec<SpdxExternalRef>,
}

impl SpdxPackage {
    fn from_component(c: &SbomComponent) -> Self {
        let mut external_refs = vec![];
        if let Some(ref purl) = c.purl {
            external_refs.push(SpdxExternalRef {
                reference_category: "PACKAGE-MANAGER".to_string(),
                reference_type: "purl".to_string(),
                reference_locator: purl.clone(),
            });
        }

        Self {
            spdx_id: c.id.clone(),
            name: c.name.clone(),
            version_info: c.version.clone(),
            download_location: c.download_location.clone().unwrap_or_else(|| "NOASSERTION".to_string()),
            files_analyzed: false,
            license_concluded: c.license.clone(),
            license_declared: c.license.clone(),
            copyright_text: "NOASSERTION".to_string(),
            supplier: c.supplier.as_ref().map(|s| format!("Person: {}", s)),
            homepage: c.homepage.clone(),
            description: c.description.clone(),
            external_refs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpdxRelationship {
    spdx_element_id: String,
    relationship_type: String,
    related_spdx_element: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpdxExternalRef {
    reference_category: String,
    reference_type: String,
    reference_locator: String,
}

/// CycloneDX 1.5 BOM structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxBom {
    bom_format: String,
    spec_version: String,
    version: i32,
    serial_number: String,
    metadata: CycloneDxMetadata,
    components: Vec<CycloneDxComponent>,
    dependencies: Vec<CycloneDxDependency>,
}

impl CycloneDxBom {
    fn from_sbom(sbom: &Sbom) -> Self {
        Self {
            bom_format: "CycloneDX".to_string(),
            spec_version: "1.5".to_string(),
            version: 1,
            serial_number: format!("urn:uuid:{}", Uuid::new_v4()),
            metadata: CycloneDxMetadata {
                timestamp: sbom.created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                tools: sbom.creators.iter().map(|c| CycloneDxTool {
                    name: c.name.clone(),
                }).collect(),
                component: Some(CycloneDxComponent::from_component(&sbom.primary_component, "application")),
            },
            components: sbom.components.iter().map(|c| CycloneDxComponent::from_component(c, "library")).collect(),
            dependencies: vec![CycloneDxDependency {
                ref_field: sbom.primary_component.purl.clone().unwrap_or_default(),
                depends_on: sbom.components.iter()
                    .filter_map(|c| c.purl.clone())
                    .collect(),
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxMetadata {
    timestamp: String,
    tools: Vec<CycloneDxTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<CycloneDxComponent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CycloneDxTool {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxComponent {
    #[serde(rename = "type")]
    component_type: String,
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    purl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    licenses: Vec<CycloneDxLicense>,
}

impl CycloneDxComponent {
    fn from_component(c: &SbomComponent, component_type: &str) -> Self {
        Self {
            component_type: component_type.to_string(),
            name: c.name.clone(),
            version: c.version.clone(),
            purl: c.purl.clone(),
            description: c.description.clone(),
            licenses: vec![CycloneDxLicense {
                license: CycloneDxLicenseInfo {
                    id: Some(c.license.clone()),
                    name: None,
                },
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CycloneDxLicense {
    license: CycloneDxLicenseInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct CycloneDxLicenseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxDependency {
    #[serde(rename = "ref")]
    ref_field: String,
    depends_on: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScanSummary;

    fn create_test_scan() -> ScanResult {
        ScanResult {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            project_license: Some("MIT".to_string()),
            dependencies: vec![],
            summary: ScanSummary::default(),
        }
    }

    #[test]
    fn test_sbom_from_scan() {
        let scan = create_test_scan();
        let sbom = Sbom::from_scan(&scan, "test-tool", "1.0.0", "https://example.com");

        assert_eq!(sbom.primary_component.name, "test-project");
        assert_eq!(sbom.primary_component.version, "1.0.0");
    }

    #[test]
    fn test_sbom_to_spdx_json() {
        let scan = create_test_scan();
        let sbom = Sbom::from_scan(&scan, "test-tool", "1.0.0", "https://example.com");

        let json = sbom.to_spdx_json().unwrap();
        assert!(json.contains("SPDX-2.3"));
        assert!(json.contains("test-project"));
    }

    #[test]
    fn test_sbom_to_cyclonedx_json() {
        let scan = create_test_scan();
        let sbom = Sbom::from_scan(&scan, "test-tool", "1.0.0", "https://example.com");

        let json = sbom.to_cyclonedx_json().unwrap();
        assert!(json.contains("CycloneDX"));
        assert!(json.contains("1.5"));
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(SbomFormat::SpdxJson.extension(), "spdx.json");
        assert_eq!(SbomFormat::CycloneDxJson.extension(), "cdx.json");
    }
}
