# OxideKit Marketplace Monetization

> Last Updated: January 2026

This document defines the monetization rules for the OxideKit Marketplace, including revenue sharing, verification fees, and governance policies.

---

## Principles

The OxideKit Marketplace is designed around these principles:

1. **Fair compensation**: Creators should earn sustainable income
2. **User trust**: Verification and transparency over pay-to-win
3. **Ecosystem health**: Sustainable take rates that don't discourage participation
4. **Quality over quantity**: Verified, safe assets get prominence
5. **No paywalling basics**: Core functionality remains free

---

## Revenue Share Model

### Standard Revenue Share

| Asset Type | OxideKit Share | Creator Share |
|------------|----------------|---------------|
| Paid plugins | 15% | 85% |
| Paid themes | 15% | 85% |
| Premium extensions | 15% | 85% |
| Template packs | 15% | 85% |
| Asset bundles | 15% | 85% |

### Why 15%

- **Lower than app stores**: Apple/Google take 30%, we take 15%
- **Sustainable for OxideKit**: Covers infrastructure, verification, support
- **Sustainable for creators**: 85% allows viable businesses
- **Competitive with peers**: Matches or beats similar platforms

### Volume Discounts

High-volume creators receive reduced rates:

| Annual Revenue | OxideKit Share | Creator Share |
|----------------|----------------|---------------|
| < $10,000 | 15% | 85% |
| $10,000 - $100,000 | 12% | 88% |
| $100,000 - $500,000 | 10% | 90% |
| > $500,000 | 8% | 92% |

Volume discounts are calculated annually and applied retroactively.

### Free Assets

Free assets have no revenue share (obviously), but:
- Must still pass safety scanning
- Encouraged to apply for verification
- May include optional donation links

---

## Verification Tiers

### Trust Levels

| Tier | Badge | Requirements | Benefits |
|------|-------|--------------|----------|
| **Unverified** | None | Passed safety scan | Basic listing |
| **Verified** | Verified | Passed attestation | Featured placement |
| **Trusted Publisher** | Trusted Publisher | Track record + verification | Premium placement, reduced fees |

### Unverified

**Requirements**:
- Pass automated malware/security scan
- Provide basic metadata
- Accept marketplace terms

**Listing**:
- Appears in search results
- Shows "Unverified" indicator
- Lower default ranking

### Verified

**Requirements**:
- Pass OxideKit Attestation Service scan
- Valid SBOM provided
- Capability declarations match actual behavior
- Source code available for review (not necessarily open source)
- Respond to security reports within 7 days

**Benefits**:
- "Verified" badge displayed
- Higher search ranking
- Featured in category pages
- Eligible for promotional features

### Trusted Publisher

**Requirements**:
- 6+ months of Verified status
- 90%+ positive ratings
- No security incidents
- Active maintenance (updates within 90 days)
- Response time < 48 hours for issues

**Benefits**:
- "Trusted Publisher" badge
- Top search ranking
- Featured on homepage
- Reduced revenue share (12% instead of 15%)
- Early access to marketplace features
- Direct support channel

---

## Verification Fees

### Publisher Account Fees

| Account Type | Monthly Fee | Annual Fee | Included Attestations |
|--------------|-------------|------------|----------------------|
| **Free** | $0 | $0 | 0 |
| **Indie** | $19 | $199 | 10/month |
| **Pro** | $99 | $999 | 100/month |
| **Business** | $299 | $2,999 | Unlimited |

### Per-Asset Attestation Fees

For publishers not on a plan or exceeding plan limits:

| Asset Type | Initial Attestation | Re-attestation (update) |
|------------|---------------------|------------------------|
| Plugin | $25 | $10 |
| Theme | $15 | $5 |
| Extension | $25 | $10 |
| Template | $10 | $5 |

### Free Tier

Free publisher accounts can:
- List unlimited free assets
- Use automated safety scanning
- Receive user ratings and reviews

Cannot:
- List paid assets
- Receive "Verified" badge
- Access premium placement

---

## Pricing Guidelines

### Minimum Prices

| Asset Type | Minimum Price |
|------------|---------------|
| Plugin | $1.99 |
| Theme | $0.99 |
| Extension | $1.99 |
| Template | $0.99 |
| Bundle | $4.99 |

### Subscription Pricing

Assets may be offered as subscriptions:

| Billing Cycle | Minimum | Revenue Share |
|---------------|---------|---------------|
| Monthly | $0.99/month | 15% |
| Annual | $9.99/year | 15% |

### One-Time Purchase

Standard model for most assets. Includes updates for the major version.

### Freemium

Allowed with conditions:
- Free version must be genuinely useful
- Clear disclosure of paid features
- No dark patterns for upselling

---

## Discovery and Ranking

### Ranking Factors

| Factor | Weight | Notes |
|--------|--------|-------|
| Verification status | High | Verified assets ranked higher |
| User ratings | High | Quality signal |
| Recent updates | Medium | Active maintenance preferred |
| Download count | Medium | Popularity signal |
| Response time | Medium | Publisher engagement |
| Compatibility | Medium | Works with current OxideKit |

### What Does NOT Affect Ranking

- **Payment for placement**: No pay-to-win
- **Price**: Expensive assets not ranked differently
- **Revenue share tier**: Volume discounts don't affect visibility
- **First-party vs third-party**: Equal treatment

### Promotional Features

OxideKit may feature assets editorially:
- "Staff Picks" curated by team
- "New and Notable" for recent releases
- Category spotlights

Criteria for promotion:
- Quality and usefulness
- User satisfaction
- Ecosystem value
- NOT payment

---

## Prohibited Practices

### Prohibited Content

- Malware, spyware, or tracking software
- Assets that circumvent OxideKit security
- Stolen or plagiarized content
- Misleading descriptions
- Undisclosed affiliate/tracking
- Cryptocurrency miners
- Assets requiring unnecessary permissions

### Prohibited Business Practices

- **Fake reviews**: Buying or manipulating ratings
- **Bait and switch**: Changing functionality after purchase
- **Forced subscriptions**: Requiring subscription for basic use
- **Hidden fees**: Undisclosed costs after purchase
- **Data harvesting**: Collecting user data beyond necessity
- **Anti-competitive behavior**: Blocking competing assets

### Enforcement

Violations result in:

| Severity | Action |
|----------|--------|
| Minor | Warning, 30 days to correct |
| Moderate | Temporary delisting, remediation required |
| Severe | Permanent ban, revenue clawback |
| Critical (malware) | Immediate removal, legal action |

---

## Payout Terms

### Payment Schedule

- **Standard**: Monthly, 30 days after month end
- **Fast payout**: Weekly (2.5% fee)

### Minimum Payout

- Standard: $50
- Accumulated balance carries forward

### Payment Methods

- Bank transfer (ACH/SEPA)
- PayPal
- Wise

### Currency

- All prices in USD
- Payouts in local currency at market rate

### Tax Requirements

Publishers must provide:
- W-9 (US) or W-8BEN (non-US)
- VAT number (EU publishers over threshold)

OxideKit handles VAT/GST collection for applicable jurisdictions.

---

## Refund Policy

### User Refunds

| Timeframe | Refund Type | Creator Impact |
|-----------|-------------|----------------|
| < 48 hours | Full refund, no questions | Revenue clawed back |
| 48h - 14 days | Conditional refund | Case-by-case |
| > 14 days | No refund | N/A |

### Refund Reasons

Automatic approval:
- Asset doesn't work as described
- Compatibility issues
- Duplicate purchase

Requires review:
- "Changed my mind"
- Buyer's remorse

### Abuse Prevention

Users with excessive refund rates may:
- Require manual review for refunds
- Be restricted from purchasing

---

## Disputes

### Creator-User Disputes

1. User reports issue
2. Creator has 7 days to respond
3. If unresolved, OxideKit mediates
4. Final decision by OxideKit

### Creator Appeals

Creators may appeal:
- Delisting decisions
- Review removals
- Account restrictions

Appeal process:
1. Submit appeal with evidence
2. Review within 14 business days
3. Decision is final

---

## Anti-Malware Policy

### Scanning Requirements

All assets undergo:
1. **Automated scanning**: Virus/malware detection
2. **Static analysis**: Code pattern detection
3. **Behavior analysis**: Runtime permission checking
4. **Dependency audit**: Known vulnerable dependencies

### Capability Verification

Assets must:
- Declare all capabilities in manifest
- Actually require all declared capabilities
- Not request more than necessary

Mismatches result in:
- Listing blocked until resolved
- Verification revoked if already verified

### Incident Response

If malware discovered post-listing:
1. Immediate delisting
2. User notification
3. Automatic uninstall pushed (if critical)
4. Publisher account review
5. Public disclosure if significant

---

## Analytics for Publishers

Publishers have access to:

### Free Metrics

- Total downloads
- Active installs
- Rating distribution
- Refund rate

### Pro Metrics (Indie+ plans)

- Conversion funnel
- Geographic distribution
- Version adoption
- Competitive benchmarks
- Revenue trends

### Data Privacy

OxideKit does NOT provide:
- Individual user information
- Personal data of users
- Usage patterns per user

---

## Ecosystem Health Metrics

OxideKit publishes quarterly:
- Total marketplace volume
- Creator payout total
- Average creator revenue
- Top categories
- Verification rates
- Trust score distribution

---

## Changes to This Policy

### Notification

Changes to monetization policy:
- 90 days notice for fee increases
- 30 days notice for other changes
- Published in changelog and email

### Grandfathering

Existing arrangements:
- Volume discount tiers locked for 12 months
- Price changes don't affect existing subscriptions
- Verification status maintained through transitions

---

## FAQ

### How do I become a Trusted Publisher?
Maintain Verified status for 6 months with 90%+ ratings and active maintenance.

### Can I sell the same asset on other platforms?
Yes, exclusivity is not required.

### What if my asset is copied?
Report to marketplace@oxidekit.com. We enforce copyright.

### Can I change my pricing?
Yes. Existing subscribers keep their rate. New purchases use new price.

### How do bundle discounts work?
Creators set bundle prices. Revenue share applies to bundle price.

---

## Contact

- **Publisher Support**: publishers@oxidekit.com
- **Abuse Reports**: abuse@oxidekit.com
- **Partnership Inquiries**: partnerships@oxidekit.com

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial marketplace monetization policy |
