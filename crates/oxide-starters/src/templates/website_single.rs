//! Website Single Starter
//!
//! A single-page marketing website starter.

use crate::{
    StarterSpec, StarterMetadata, StarterCategory, StarterTarget,
    PluginRequirement, PermissionPreset,
    GeneratedFile, PostInitStep, MessageLevel,
};

/// Create the website-single starter spec
pub fn create_spec() -> StarterSpec {
    StarterSpec {
        id: "website-single".to_string(),
        name: "Marketing Website".to_string(),
        description: "Single marketing website with hero, features, pricing, and contact sections".to_string(),
        long_description: Some(
            "A production-ready marketing website starter that includes:\n\
            - Hero section with CTA\n\
            - Features grid\n\
            - Pricing table\n\
            - FAQ accordion\n\
            - Contact form\n\
            - Footer with links\n\
            - Mobile-responsive design\n\n\
            Perfect for landing pages, product sites, and company websites."
                .to_string(),
        ),
        version: "0.1.0".to_string(),
        min_core_version: Some("0.1.0".to_string()),
        metadata: StarterMetadata {
            category: StarterCategory::Website,
            tags: vec![
                "website".to_string(),
                "landing".to_string(),
                "marketing".to_string(),
                "static".to_string(),
            ],
            author: Some("OxideKit Team".to_string()),
            homepage: Some("https://oxidekit.com/starters/website-single".to_string()),
            screenshots: vec![
                "https://oxidekit.com/screenshots/website-hero.png".to_string(),
            ],
            official: true,
            featured: false,
        },
        targets: vec![StarterTarget::Static],
        plugins: vec![
            PluginRequirement {
                id: "ui.core".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.forms".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "design.website.clean".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "theme.website.light".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
        ],
        permissions: PermissionPreset::default(),
        files: vec![
            GeneratedFile {
                path: "ui/pages/index.oui".to_string(),
                template: "content:// Homepage\n\nPage {\n    Column {\n        HeroSection { }\n        FeaturesSection { }\n        PricingSection { }\n        FAQSection { }\n        ContactSection { }\n        Footer { }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/hero.oui".to_string(),
                template: "content:// Hero section\n\nHeroSection {\n    Column {\n        gap: 24\n        align: \"center\"\n        padding: 64\n        background: \"gradient\"\n\n        Text { content: \"{{project_name}}\" role: \"display\" }\n        Text { content: \"Build amazing things with OxideKit\" role: \"body-large\" color: \"muted\" }\n\n        Row {\n            gap: 16\n            Button { text: \"Get Started\" variant: \"primary\" size: \"large\" }\n            Button { text: \"Learn More\" variant: \"secondary\" size: \"large\" }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/features.oui".to_string(),
                template: "content:// Features section\n\nFeaturesSection {\n    Column {\n        gap: 32\n        padding: 64\n        align: \"center\"\n\n        Text { content: \"Features\" role: \"heading\" }\n\n        Row {\n            gap: 24\n            wrap: true\n            justify: \"center\"\n\n            FeatureCard {\n                icon: \"zap\"\n                title: \"Fast\"\n                description: \"Lightning fast performance\"\n            }\n            FeatureCard {\n                icon: \"shield\"\n                title: \"Secure\"\n                description: \"Built with security in mind\"\n            }\n            FeatureCard {\n                icon: \"code\"\n                title: \"Developer Friendly\"\n                description: \"Great developer experience\"\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/pricing.oui".to_string(),
                template: "content:// Pricing section\n\nPricingSection {\n    Column {\n        gap: 32\n        padding: 64\n        align: \"center\"\n        background: \"surface\"\n\n        Text { content: \"Pricing\" role: \"heading\" }\n\n        Row {\n            gap: 24\n\n            PricingCard {\n                name: \"Free\"\n                price: \"$0\"\n                features: [\"Basic features\", \"Community support\"]\n            }\n            PricingCard {\n                name: \"Pro\"\n                price: \"$29\"\n                features: [\"All features\", \"Priority support\", \"Advanced analytics\"]\n                highlighted: true\n            }\n            PricingCard {\n                name: \"Enterprise\"\n                price: \"Custom\"\n                features: [\"Everything in Pro\", \"Dedicated support\", \"Custom integrations\"]\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/faq.oui".to_string(),
                template: "content:// FAQ section\n\nFAQSection {\n    Column {\n        gap: 24\n        padding: 64\n        maxWidth: 800\n\n        Text { content: \"FAQ\" role: \"heading\" align: \"center\" }\n\n        Accordion {\n            AccordionItem {\n                title: \"What is {{project_name}}?\"\n                content: \"{{project_name}} is built with OxideKit, a modern application platform.\"\n            }\n            AccordionItem {\n                title: \"How do I get started?\"\n                content: \"Click the Get Started button above to begin.\"\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
        ],
        post_init: vec![
            PostInitStep::Message {
                text: "Marketing website created successfully!".to_string(),
                level: MessageLevel::Success,
            },
            PostInitStep::Command {
                command: "cd {{project_name}} && oxide dev".to_string(),
                description: Some("Start the development server".to_string()),
            },
            PostInitStep::Command {
                command: "oxide build --target static".to_string(),
                description: Some("Build for production".to_string()),
            },
        ],
        variables: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_single_spec() {
        let spec = create_spec();

        assert_eq!(spec.id, "website-single");
        assert!(spec.targets.contains(&StarterTarget::Static));
    }
}
