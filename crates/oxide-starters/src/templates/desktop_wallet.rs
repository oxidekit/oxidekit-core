//! Desktop Wallet Starter
//!
//! A secure, professional desktop wallet UI shell for crypto/financial applications.

use crate::{
    StarterSpec, StarterMetadata, StarterCategory, StarterTarget,
    PluginRequirement, PermissionPreset, NetworkPermissions, SystemPermissions,
    GeneratedFile, PostInitStep, MessageLevel,
};

/// Create the desktop-wallet starter spec
pub fn create_spec() -> StarterSpec {
    StarterSpec {
        id: "desktop-wallet".to_string(),
        name: "Desktop Wallet".to_string(),
        description: "Secure desktop wallet UI with sidebar navigation and crypto-focused components".to_string(),
        long_description: Some(
            "A production-ready wallet starter that includes:\n\
            - Sidebar + topbar navigation\n\
            - Dashboard with portfolio overview\n\
            - Assets list with detailed views\n\
            - Send/Receive transaction forms\n\
            - Transaction history\n\
            - Settings with security options\n\
            - Keychain integration for secure storage\n\
            - Network connectivity for blockchain APIs\n\n\
            Built for crypto wallets, payment apps, and financial tools."
                .to_string(),
        ),
        version: "0.1.0".to_string(),
        min_core_version: Some("0.1.0".to_string()),
        metadata: StarterMetadata {
            category: StarterCategory::Wallet,
            tags: vec![
                "wallet".to_string(),
                "crypto".to_string(),
                "finance".to_string(),
                "desktop".to_string(),
                "secure".to_string(),
            ],
            author: Some("OxideKit Team".to_string()),
            homepage: Some("https://oxidekit.com/starters/desktop-wallet".to_string()),
            screenshots: vec![
                "https://oxidekit.com/screenshots/wallet-dashboard.png".to_string(),
                "https://oxidekit.com/screenshots/wallet-send.png".to_string(),
            ],
            official: true,
            featured: true,
        },
        targets: vec![StarterTarget::Desktop],
        plugins: vec![
            PluginRequirement {
                id: "ui.core".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.data".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.forms".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.navigation".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.charts".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "design.wallet.modern".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "theme.wallet.dark".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "i18n.core".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "native.keychain".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "native.network".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "native.diagnostics".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
        ],
        permissions: PermissionPreset {
            network: NetworkPermissions {
                hosts: vec!["*".to_string()],
                allow_all: true,
            },
            system: SystemPermissions {
                keychain: true,
                notifications: true,
                clipboard: true,
                tray: true,
            },
            ..Default::default()
        },
        files: vec![
            GeneratedFile {
                path: "ui/layouts/wallet_shell.oui".to_string(),
                template: "content:// Wallet shell layout\n\nWalletShell {\n    Sidebar {\n        NavItem { icon: \"home\" label: \"Dashboard\" href: \"/\" }\n        NavItem { icon: \"wallet\" label: \"Assets\" href: \"/assets\" }\n        NavItem { icon: \"send\" label: \"Send\" href: \"/send\" }\n        NavItem { icon: \"receive\" label: \"Receive\" href: \"/receive\" }\n        NavItem { icon: \"history\" label: \"History\" href: \"/history\" }\n        NavItem { icon: \"settings\" label: \"Settings\" href: \"/settings\" }\n    }\n    TopBar {\n        NetworkIndicator { }\n        WalletSelector { }\n    }\n    Content {\n        slot: \"main\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/dashboard.oui".to_string(),
                template: "content:// Wallet Dashboard\n\nPage {\n    layout: \"wallet_shell\"\n\n    Column {\n        gap: 24\n\n        PortfolioCard {\n            total: \"$12,345.67\"\n            change: \"+2.4%\"\n        }\n\n        Row {\n            gap: 16\n            QuickAction { icon: \"send\" label: \"Send\" }\n            QuickAction { icon: \"receive\" label: \"Receive\" }\n            QuickAction { icon: \"swap\" label: \"Swap\" }\n        }\n\n        Card {\n            title: \"Assets\"\n            AssetList { limit: 5 }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/assets.oui".to_string(),
                template: "content:// Assets List\n\nPage {\n    layout: \"wallet_shell\"\n\n    Column {\n        gap: 16\n\n        Text { content: \"Assets\" role: \"heading\" }\n\n        AssetList {\n            showAll: true\n            sortable: true\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/send.oui".to_string(),
                template: "content:// Send Transaction\n\nPage {\n    layout: \"wallet_shell\"\n\n    Column {\n        gap: 24\n        maxWidth: 480\n\n        Text { content: \"Send\" role: \"heading\" }\n\n        Card {\n            Form {\n                AssetSelector { label: \"Asset\" name: \"asset\" }\n                AddressInput { label: \"Recipient\" name: \"recipient\" }\n                AmountInput { label: \"Amount\" name: \"amount\" }\n                Button { text: \"Review\" variant: \"primary\" type: \"submit\" }\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/receive.oui".to_string(),
                template: "content:// Receive\n\nPage {\n    layout: \"wallet_shell\"\n\n    Column {\n        gap: 24\n        maxWidth: 480\n        align: \"center\"\n\n        Text { content: \"Receive\" role: \"heading\" }\n\n        Card {\n            Column {\n                gap: 16\n                align: \"center\"\n\n                QRCode { value: \"{{address}}\" size: 200 }\n                AddressDisplay { address: \"{{address}}\" copyable: true }\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/settings.oui".to_string(),
                template: "content:// Settings\n\nPage {\n    layout: \"wallet_shell\"\n\n    Column {\n        gap: 24\n\n        Text { content: \"Settings\" role: \"heading\" }\n\n        Card {\n            title: \"General\"\n            Form {\n                Select { label: \"Language\" name: \"language\" options: [\"en\", \"es\", \"fr\"] }\n                Select { label: \"Currency\" name: \"currency\" options: [\"USD\", \"EUR\", \"GBP\"] }\n                Toggle { label: \"Dark Mode\" name: \"dark_mode\" }\n            }\n        }\n\n        Card {\n            title: \"Security\"\n            Form {\n                Toggle { label: \"Require PIN\" name: \"require_pin\" }\n                Toggle { label: \"Auto-lock\" name: \"auto_lock\" }\n                Button { text: \"Export Diagnostics\" variant: \"secondary\" }\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "i18n/en.toml".to_string(),
                template: "content:[dashboard]\ntitle = \"Dashboard\"\nportfolio = \"Portfolio\"\n\n[assets]\ntitle = \"Assets\"\n\n[send]\ntitle = \"Send\"\nrecipient = \"Recipient Address\"\namount = \"Amount\"\nreview = \"Review\"\n\n[receive]\ntitle = \"Receive\"\ncopy_address = \"Copy Address\"\n\n[settings]\ntitle = \"Settings\"\nlanguage = \"Language\"\ncurrency = \"Currency\"\ndark_mode = \"Dark Mode\"".to_string(),
                condition: None,
            },
        ],
        post_init: vec![
            PostInitStep::Message {
                text: "Desktop wallet created successfully!".to_string(),
                level: MessageLevel::Success,
            },
            PostInitStep::Message {
                text: "Security Note: This starter includes keychain access for secure key storage.".to_string(),
                level: MessageLevel::Warning,
            },
            PostInitStep::Command {
                command: "cd {{project_name}} && oxide dev".to_string(),
                description: Some("Start the development server".to_string()),
            },
        ],
        variables: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_wallet_spec() {
        let spec = create_spec();

        assert_eq!(spec.id, "desktop-wallet");
        assert!(spec.metadata.official);
        assert!(spec.targets.contains(&StarterTarget::Desktop));
        assert!(spec.permissions.system.keychain);
    }
}
