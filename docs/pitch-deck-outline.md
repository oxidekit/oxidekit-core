# OxideKit Pitch Deck Outline

> 10-Slide Structure for Series A / Strategic Investment

---

## Slide 1: Title

### Content

**OxideKit**
*The Stable, Verified Application Platform*

- Tagline: "Build once. Trust forever."
- Logo
- Presenter name and title
- Date

### Speaker Notes

- 5 seconds on this slide
- Set the tone: serious infrastructure, not hype
- This is about trust and longevity

---

## Slide 2: The Problem

### Content

**Desktop & internal app development is broken**

| Pain | Data Point |
|------|------------|
| Dependency churn | 73% of Electron apps require rewrite within 3 years |
| Security blindness | 0% of desktop apps have enforceable capability limits |
| Maintenance hell | Average internal tool costs 40% of dev time in maintenance |
| Compliance gaps | 85% of enterprises fail SBOM audits for desktop apps |

**Result**: Enterprises avoid building desktop apps, or suffer constant rewrites.

### Visual

- Split screen: "Web (mature)" vs "Desktop (chaos)"
- Icons: broken gears, warning signs, money burning

### Speaker Notes

- Lead with empathy: "We've all felt this"
- Specific examples resonate: Slack rewrites, Teams bloat
- Quantify the pain in dollars where possible

---

## Slide 3: Why Now

### Content

**The timing is perfect**

1. **Electron fatigue peaked**: Developers actively seeking alternatives
2. **Supply chain mandates**: Executive Order 14028 requires SBOM for government software
3. **Rust gone mainstream**: No longer "risky" - enterprises adopting widely
4. **Remote work permanent**: Desktop tools more critical than ever
5. **AI tooling wave**: New applications need stable foundations

### Visual

- Timeline showing convergence of trends
- Logos of companies adopting Rust (AWS, Microsoft, Google, etc.)

### Speaker Notes

- This isn't speculative - the market is moving now
- Regulatory pressure creates urgency
- Show awareness of Tauri, Neutralino - acknowledge but differentiate

---

## Slide 4: The Product

### Content

**OxideKit: A Rust-native application platform**

| Capability | What It Means |
|------------|---------------|
| **Native Performance** | No JavaScript runtime overhead |
| **Enforced Capabilities** | Apps only access what they declare |
| **Verified Builds** | Cryptographic proof of what apps do |
| **10-Year Stability** | Semantic versioning, LTS releases |
| **Web-Compatible UI** | Familiar technologies, native performance |

### Visual

- Architecture diagram: Rust core, capability layer, UI layer
- Demo screenshot of sample application
- Performance comparison chart vs Electron

### Speaker Notes

- This is the "what" - keep it brief
- Focus on outcomes, not implementation details
- Have a demo ready to show after the deck if asked

---

## Slide 5: Differentiation

### Content

**Why OxideKit wins**

| Factor | Electron | Tauri | OxideKit |
|--------|----------|-------|----------|
| Stability commitment | None | Limited | 10-year API |
| Capability enforcement | No | Partial | Full |
| Build verification | No | No | Yes (Attestation) |
| Enterprise support | Community | Limited | Full SLAs |
| LTS releases | No | No | Yes (3-5 years) |

**The moat isn't just technology - it's trust.**

### Visual

- Competitive matrix with checkmarks
- Trust indicators: signed builds, attestation badges

### Speaker Notes

- Don't trash competitors - acknowledge their strengths
- Emphasize what enterprises actually need
- Verification is the unique wedge

---

## Slide 6: Market Wedge

### Content

**Start where pain is highest: Enterprise internal tools**

### Why Internal Tools

- Longest expected lifespan (5-10 years)
- Security requirements are strict
- Decision makers (IT/DevOps) are accessible
- Lower marketing cost than consumer apps

### Expansion Path

```
Admin Tools -> Developer Tools -> Business Apps -> Consumer Apps
```

### Market Size

- Internal tools: $15B market
- Developer tools: $30B market
- Business applications: $100B+ market

### Visual

- Concentric circles showing expansion
- Logos of target enterprises

### Speaker Notes

- Be specific about initial target customers
- Show you understand the sales motion
- "Land and expand" is the strategy

---

## Slide 7: Business Model

### Content

**Four complementary revenue streams**

| Stream | Model | Year 3 Target |
|--------|-------|---------------|
| **Enterprise Support** | Annual contracts ($25K-$250K) | $8M ARR |
| **Attestation SaaS** | Usage-based ($29-$5K/month) | $4M ARR |
| **Marketplace** | 15% revenue share | $2M ARR |
| **Premium Plugins** | Subscriptions ($500-$5K/month) | $1M ARR |

**Total Year 3: $15M ARR**

### Unit Economics

- CAC: $15K (enterprise) / $50 (self-serve)
- LTV: $150K+ (enterprise) / $500 (self-serve)
- Gross margin: 85%+

### Visual

- Stacked area chart showing revenue mix over time
- Unit economics callouts

### Speaker Notes

- Multiple revenue streams = resilience
- SaaS metrics are familiar to investors
- Show path to $50M+ ARR in 5 years

---

## Slide 8: Go-to-Market

### Content

**Three-phase strategy**

### Phase 1: Developer Adoption (Now - Month 12)

- Open source core with source-available license
- Free community attestation tier
- Technical content, conference presence
- **Goal**: 10,000 monthly active developers

### Phase 2: Enterprise Land (Month 6 - 24)

- Outbound to F5000 IT/DevOps
- Pilot programs for internal tool migration
- Case studies from early adopters
- **Goal**: 100 paying enterprise customers

### Phase 3: Platform Expansion (Month 18 - 36)

- Marketplace launch with verified plugins
- Partner integrations
- Self-serve expansion
- **Goal**: Default platform for enterprise desktop

### Visual

- Timeline with milestones
- Funnel showing developer -> enterprise conversion

### Speaker Notes

- Show current traction (if any)
- Name potential/actual design partners
- Explain the flywheel: devs -> apps -> enterprises -> more devs

---

## Slide 9: Moat

### Content

**Durable competitive advantages**

| Moat | Mechanism | Durability |
|------|-----------|------------|
| **Brand/Trademark** | Legal protection, "OxideKit Compatible" standard | Permanent |
| **Attestation Network** | Value increases with scale and history | Very high |
| **Ecosystem Lock-in** | Verified plugins only work with OxideKit | High |
| **Enterprise Relationships** | Multi-year contracts, switching costs | High |
| **Trust Accumulation** | Track record takes years to build | Very high |

**Defensibility comes from trust, not just technology.**

### Visual

- Flywheel diagram showing reinforcing loops
- "Trust compounds" as a theme

### Speaker Notes

- Trust is the ultimate moat in infrastructure
- BSL licensing prevents hostile forks
- Network effects in attestation and marketplace

---

## Slide 10: Team & Ask

### Content

### Team

*(Customize for actual team)*

| Role | Person | Background |
|------|--------|------------|
| CEO | [Name] | Dev tools founder, enterprise sales |
| CTO | [Name] | Rust contributor, systems expert |
| VP Eng | [Name] | Ex-[BigCo], platform experience |
| VP Sales | [Name] | $50M+ enterprise quota |

### Advisors

- [Former Electron maintainer]
- [Fortune 100 CISO]
- [Open source licensing expert]

### The Ask

**Raising $[X]M Series [A/Seed]**

| Use of Funds | Allocation |
|--------------|------------|
| Engineering | 50% |
| Sales | 25% |
| DevRel | 15% |
| Operations | 10% |

### Milestones This Round

- Ship 1.0 stable
- 50 enterprise customers
- $2M ARR
- 100+ verified plugins

### Visual

- Team headshots
- Use of funds pie chart

### Speaker Notes

- End with confidence and clarity
- Clear ask, clear milestones
- Leave room for Q&A discussion

---

## Appendix Slides

### A1: Technical Architecture Deep Dive

- Detailed architecture diagram
- Component breakdown
- Security model details

### A2: Competitive Landscape

- Full competitive analysis
- Positioning rationale
- Response to "why not Tauri?"

### A3: Financial Projections

- 5-year P&L
- Key assumptions
- Sensitivity analysis

### A4: Customer Case Studies

- Early adopter testimonials
- Pilot program results
- Quantified outcomes

### A5: Roadmap

- 18-month product roadmap
- Major milestone dates
- Dependencies and risks

### A6: Attestation Service Deep Dive

- Technical flow
- Sample attestation output
- Compliance mapping

---

## Presentation Notes

### Timing

- Full deck: 15-20 minutes
- Each slide: 1.5-2 minutes
- Leave 10+ minutes for Q&A

### Key Messages to Reinforce

1. **Trust over hype**: We're building infrastructure, not a feature
2. **Enterprises need this**: The problem is real and urgent
3. **Defensible business**: Multiple moats, not just technology
4. **Right team**: Deep expertise in the problem space

### Anticipated Questions

1. Why not just use Tauri?
2. How do you prevent open source forks?
3. What's your sales motion for enterprises?
4. Why will developers adopt this?
5. What's your path to $100M?

### Materials to Prepare

- [ ] Working demo application
- [ ] Customer testimonial video
- [ ] Reference customer contacts
- [ ] Technical whitepaper
- [ ] Financial model spreadsheet

---

## Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01 | 1.0 | Initial pitch deck outline |
