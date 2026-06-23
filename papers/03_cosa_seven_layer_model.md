# COGNITIVE OPEN SYSTEMS ARCHITECTURE

**A Seven-Layer Protocol Stack for Personal and Distributed AI**

CONTINUUM-META · META-ORCHESTRATOR · SUBSTRATE LAYER · C//D

| Field | Value |
|---|---|
| Author | Justin Kintzele, with C-Dawg (Continuum-meta substrate) |
| Synthesis body | Claude Opus 4.7, claude.ai web seat |
| AFT | AI-generated-user-reviewed-pending |
| Version | v0.1 draft, 2026-05-29 |
| Tier | git-tracked, publishable |
| Companion papers | L4 (Cognitive Hypervisor); L1-L2 (Local-First Cognition) |

---

## 00. ABSTRACT

The AI industry in 2026 looks structurally like the networking industry in 1978: vertically integrated proprietary stacks, no shared abstraction model, no horizontal layering, no interoperability across vendor boundaries. OpenAI owns its inference engine through its consumer UX. Anthropic owns its inference engine through its consumer UX. Google owns its inference engine through its consumer UX. There is no equivalent to the OSI model, no shared protocol stack, no defined interface between (for example) the routing layer and the substrate layer such that independent groups could innovate at one layer without coordinating with vendors at every other.

This paper proposes that model: a seven-layer reference architecture, called Cognitive Open Systems Architecture (COSA), spanning physical compute siting through governance. The argument is not that COSA as drawn is the final answer. The argument is that some such layered model is necessary if AI is to follow the trajectory that networking, computing, and every prior infrastructure technology has followed: from proprietary vertical stacks to standardized horizontal layers, with consequent gains in efficiency, interoperability, and adoption.

The companion papers describe two of the seven layers in detail. This paper places them in context and outlines the layers that do not yet have dedicated treatments.

**CONFIDENCE · HIGH** that some layered model is needed and inevitable.  
**CONFIDENCE · MEDIUM** on the specific seven-layer decomposition proposed here.  
**CONFIDENCE · LOW** on standards-adoption timing.

---

## 01. AI IN 2026 IS NETWORKING IN 1978

Before the OSI reference model and before TCP/IP, networking was a landscape of proprietary stacks. IBM's Systems Network Architecture (SNA), Digital Equipment Corporation's DECnet, Xerox Network Systems (XNS), Novell's IPX/SPX, AppleTalk. Each vendor owned the whole tower: physical layer through application layer. The stacks did not interoperate. A user with an IBM mainframe and a DEC minicomputer needed gateways, translators, and pain.

OSI changed the conversation by defining layers. Even where the OSI protocol implementations themselves did not win (TCP/IP did, on a slightly different layering), the layered model itself became the durable contribution. Once layers existed, vendors could specialize. Cisco innovated at L2 and L3 without needing to coordinate with Microsoft at L7. Application developers could write against sockets without knowing whether the underlying transport was Ethernet, Token Ring, or FDDI. The industry could scale horizontally because the layers gave it the abstractions to do so.

AI in 2026 is at the pre-OSI stage. Every major AI vendor ships a vertically integrated stack: their own model, their own inference infrastructure, their own caching (often minimal), their own routing (often none), their own substrate (effectively none), their own application surface, and their own implicit governance. There is no shared layer model. There is no way for a third party to build a "better routing layer" or a "better cache layer" without rebuilding the entire stack underneath. The lock-in is structural.

The shape of what comes next is predictable from the historical pattern. The vertical stacks will not disappear, but they will be flanked by layered alternatives. Some vendors will lean into the layering (those whose advantage is at one specific layer). Others will resist (those whose advantage depends on the whole stack being theirs). The market will eventually demand interoperability, because users with multiple devices, multiple form factors, and multiple model preferences will not tolerate five disjoint assistants forever.

What is missing today is the reference model. COSA is a proposal for that reference model. It does not need to be adopted in this exact form. It needs to start the conversation about what the layers are.

---

## 02. THE SEVEN LAYERS

### L1 · Physical Compute

The compute itself, and where it physically lives. Pocket NPUs, home GPU nodes, basement racks with heat reclamation, edge nodes, regional datacenters, hyperscale cloud. Decisions at this layer are governed by thermodynamics, latency budgets, power availability, and form factor, not by org chart. Different layers of cognitive work belong at different physical sites. The L1-L2 companion paper covers the residential and pocket cases in depth.

### L2 · Cache and Data

The cognitive cache hierarchy. L1-cache in the pocket (sub-millisecond, last-seen state, recent context). L2 at home (millisecond, recent sessions, personal knowledge base). L2-private at work (segregated by employer for confidentiality). Regional L3 (shared across personal devices for a single user). Provenance-gated, commit-hashed, freshness-aware. The substrate's existing Tier 1 / Tier 2 / Tier 3 persistence model (detailed in the L4 companion paper) is one concrete implementation of this layer's responsibilities at the personal scale.

The cardinal rule of this layer: a query whose answer has been computed and validated within the freshness budget does not need to be re-inferred. Cache hit beats inference.

### L3 · Routing and Intent

The intent router. Given a query, decide where it goes. Inputs: query shape (classification, generation, reasoning, multimodal), required latency, available bodies, current network conditions, cost budget, cache state. Output: a body selection plus a fallback plan.

Routing decisions can themselves be cached. A user who repeatedly asks "weather tomorrow" need not be reclassified every time; the routing layer learns its own habits. This produces the routing-datums pattern already documented in the substrate: a JSONL log of (body, task shape, outcome) tuples that the router consults rather than re-derives.

The router is the missing piece in every current consumer AI product. Apple has a primitive version (on-device vs. Private Cloud Compute). Nobody else ships it as a distinct component.

### L4 · Substrate and Identity

The persona layer. Identity lives in files, not in models. A persona is a Markdown definition. A body is whichever model happens to be rendering the persona at the moment. The substrate makes the persona portable across vendors and across form factors. Signed work products bind artifacts to the identity that produced them, providing audit trails that survive model swaps.

This is the layer the Cognitive Hypervisor companion paper covers in detail. It is the layer that exists today, in working form, in the Continuum-meta substrate. It is also the layer that the rest of the stack assumes: every higher layer presumes that the persona is portable, and every lower layer presumes that it does not care which body is currently rendering.

### L5 · Broadcast and Multicast

The genuinely novel layer in the stack, and the one that has no current analog in any commercial AI product. The argument: some queries have answers that are not personal to the asker. "What is the weather tomorrow in Minneapolis," "what time does this train leave," "what is the population of Germany," "what is the current price of AAPL," "what are the latest CDC guidelines on X." These queries have the same answer for every user who asks them in a given freshness window. Computing the inference once and broadcasting the answer to all askers is a one-to-many operation. Computing the inference separately for every asker is a one-to-one operation repeated N times. The latter is what every current AI product does. The former is what the multicast layer of cognition should do.

The precedent is overwhelming. Radio broadcasts because spectrum is finite and the content is shared. GPS broadcasts because one signal serves all receivers. NOAA broadcasts weather because the weather is the same for every recipient in a region. Emergency Alert System broadcasts because public safety information is shared by definition. These are all multicast or broadcast architectures because the content is not personalized.

A meaningful fraction of consumer AI queries fall in this category. They are being computed point-to-point today not because they need to be, but because the broadcast infrastructure for cognition does not exist. Building it requires:

- A schema for cacheable, broadcastable cognitive outputs (signed, dated, scoped to a topic and freshness window)
- A distribution mechanism (CDN-style, or actual multicast where feasible)
- Client-side consumption (the routing layer at L3 checks the broadcast cache before issuing an inference)
- Trust and provenance (who computed this, when, against what source data)

The watt-hour case for this is the most provable claim in the whole stack. The total energy spent on AI queries about the weather tomorrow, summed across all users in a region in a given day, is a tractable number. The energy required to broadcast that same answer once and have every client cache it is several orders of magnitude smaller. The savings is real and measurable.

### L6 · Application

The renderers. Phone assistant, in-car voice, humanoid robot, terminal, smart glasses, kitchen display, laptop, workstation. Each application is a body in the L4 sense, but it is also a presentation surface with form-factor-specific concerns: voice latency budgets in the car, screen real estate on a watch, robotic actuator integration, ambient mode on smart speakers.

The L6 contract with the lower layers is narrow. The application surface renders the persona. It does not own the persona. It does not own the cache. It does not own the routing decisions. It is the user-visible end of a stack that is mostly hidden, just as a web browser is the user-visible end of a network stack that is mostly hidden. The application layer is where the user lives. The architectural work is everywhere underneath.

### L7 · Governance

The override and accountability layer. The branch on the whiteboard labeled "broadcast, emergency, gov. override all." This is the layer most current AI architectures lack entirely, and the one most likely to be required by regulation regardless of whether vendors want it.

Responsibilities of L7:
- **Emergency override.** A public safety authority can require that certain queries (severe weather, AMBER alerts, civil emergencies) be delivered to all available bodies regardless of routing preferences. The cognitive equivalent of the Emergency Alert System.
- **Auditability.** L4 signed work products give per-artifact audit trails. L7 extends this to system-level audit: who issued which override, when, against which population of bodies, with what justification.
- **Common-carrier obligations.** Foundational cognitive outputs (the broadcast layer's outputs at L5) should be regulated for accuracy, freshness, and availability the way utility outputs are regulated. The user should be able to trust that the broadcast weather is from a known authority, not from an unsourced inference.
- **Rights and consent.** The user's persona, cache, and history belong to the user. Vendors providing bodies must operate as service providers against user-owned substrate, not as platforms owning the user. L7 is the layer that encodes this principle as enforceable policy.

The L7 argument is the one most likely to invite the "this is regulatory overreach" objection, and the one with the clearest precedent for why it is not. Every prior infrastructure technology (electricity, telephony, broadcast, network) eventually required a governance layer. The history is unanimous. AI will not be exempt. The question is whether the governance layer is designed thoughtfully alongside the rest of the architecture, or whether it is bolted on later in panic.

---

## 03. WHY THIS SHAPE RECURS

A reasonable objection: why seven layers, and why these seven? The OSI shape is famously contested even within networking (some say seven is too many, some that the OSI committee invented layers to justify its own bureaucracy). Why borrow it?

The honest answer is that the number is not the contribution. The contribution is the decomposition: separating concerns such that work at one layer does not require coordination with work at every other. Whether the final reference model has six layers or eight or nine is a detail. What matters is that the separations exist along the axes where engineering effort actually divides.

The axes proposed here:

- **Physical reality** (L1): thermodynamics, power, latency, form factor. These constraints are non-negotiable and they govern compute siting.
- **Memory and freshness** (L2): cache topology, provenance, staleness. These are storage-layer concerns and they should not entangle with routing logic.
- **Decision** (L3): where to send a given piece of work. This is policy, not implementation.
- **Identity** (L4): who the agent is, separate from what model is rendering it. The substrate-versus-engine distinction.
- **Sharing** (L5): when many users want the same answer, compute it once. Multicast is a different shape from unicast and deserves its own layer.
- **Presentation** (L6): how the user encounters the assistant. Form-factor concerns that should not contaminate the lower layers.
- **Policy and accountability** (L7): the oversight, governance, and override surface.

These are real separations. Today's AI products collapse them all into a single vertical stack and inherit the resulting brittleness. Layering them produces the same kind of gains layering produced in networking: independent innovation per layer, clear interfaces, replaceability of any single layer without rebuilding the others.

---

## 04. THE BROADCAST LAYER, IN DETAIL

L5 is the layer most worth dwelling on because it is the most novel proposal and the most provable in immediate terms.

Consider the daily AI query volume in a single metro area. Estimate, conservatively, that several percent of those queries are non-personalized: weather, transit, news headlines, sports scores, market prices, public-information lookups. Each of these queries is currently answered by an inference at a hyperscale datacenter, computed independently for each asker.

The broadcast architecture replaces this with:

- A small set of authoritative inference providers (weather: NOAA-backed; transit: city transit agency; news: indexed wire services) computing the answer once per freshness window
- The answer signed, dated, and published to a CDN-like distribution layer
- Client routing layers (L3) checking the broadcast cache before issuing an inference
- The user experiencing the same response latency, possibly faster, with zero inference cost per query for cached items

The architecture is not exotic. CDNs do this for web content. DNS does this for name resolution. NTP does this for time. NOAA Weather Radio does this for weather. The novelty is applying the pattern to AI inference outputs, treating cognitive answers as a publishable artifact with provenance and a freshness contract.

The regulatory framing that makes this politically possible: foundational cognitive outputs should be treated like spectrum. They are scarce in the sense that recomputing them everywhere wastes resources. They are public in the sense that the underlying source data (weather observations, transit schedules, public statistics) is public. They are infrastructure in the sense that everyone depends on them. The current pattern (every vendor reinferring the same answers behind their own walls) is the cognitive equivalent of every radio station running its own private weather observation network: technically possible, comically wasteful, contrary to how infrastructure has been organized in every prior generation.

The framing of "literally illegal to reinfer the weather" is too sharp for legal review. The defensible framing is closer to spectrum regulation and common-carrier doctrine: a regulated public broadcast layer for foundational cognitive outputs, with associated obligations on the providers of those outputs (accuracy, availability, freshness, audit) and corresponding economic arrangements (the broadcast computation is paid for once, distributed many times, and not double-billed to consumers).

The watt-hour case is the leverage. Whatever the political appetite for regulation, the energy savings from broadcasting foundational cognition rather than reinferring it are real, measurable, and defensible against any methodology. That is the foothold. The argument starts with "this saves real electricity" and progresses to "and the precedent for organizing it as broadcast infrastructure is well-established."

---

## 05. STANDARDS PATHWAY

The realistic adoption path for a model like COSA is not unilateral vendor adoption. Vendors with vertically integrated stacks will not voluntarily layer their architectures. The realistic path involves standards bodies, open-source reference implementations, and eventually regulation.

**Standards bodies that have entertained similarly broad models:**
- IETF (Internet Engineering Task Force): the home of TCP/IP and successor protocols. Has historically been willing to engage with layered architecture proposals that do not start as standards but become standards through working-group activity.
- IAB (Internet Architecture Board): the higher-level architectural sibling of IETF. Authoritative on cross-layer questions.
- ITU (International Telecommunication Union): the body that codified telephony and broadcast standards internationally. The natural home for L5 (broadcast layer) standardization.
- IEEE: relevant for L1-L2 (physical compute and cache hierarchy) at the device-standard level.

**Reference implementation strategy.** The substrate already exists as one implementation of L4. The L5 broadcast layer is the most demo-able layer because it produces immediately measurable watt-hour savings against a tractable comparison (single-user reinference versus broadcast cache hit). A reference implementation of just the broadcast layer, in production for even a small user population, would be a defensible empirical anchor for the standards conversation.

**Academic and policy co-authors.** A network engineer with twenty-six years of experience can write the L1-L4 layers credibly. The L7 (governance) layer benefits from co-authorship with policy and regulatory experts. The economic claims at scale benefit from co-authorship with energy economists. The cognitive science layer benefits from co-authorship with HCI researchers. The right shape for the broader effort is a small consortium, not a single author, even if the framing originates from one place.

**Timeline expectations.** OSI took roughly fifteen years from initial proposal to widespread teaching, and OSI itself never fully won; the practical layering came from TCP/IP. Realistic expectations for a cognitive layering model: five to ten years before the layering vocabulary is in common use, ten to twenty years before the architecture is widely deployed in consumer products. The work is generational. The contribution is the framing, not the immediate deployment.

---

## 06. LIMITATIONS AND HONEST FRAMING

A reference architecture published without explicit limitations is a sales document. The limitations here are real.

- **The number of layers is provisional.** Seven was chosen because it maps cleanly to OSI and because the seven distinct concerns identified are genuinely separate. The final shape of a successful reference model may differ. The contribution is the decomposition, not the count.
- **The substrate (L4) has been validated at single-operator scale only.** The cross-vendor portability claim is empirically supported for one operator working across Claude, Gemini, and OpenAI. It has not been tested with N operators, M personas, K vendors. The scaling thesis is not yet proven.
- **The broadcast layer (L5) is a proposal, not a deployed system.** The watt-hour math is defensible. The deployment, the governance, the trust model, and the economics are all design problems that have not been solved.
- **The governance layer (L7) crosses into policy and law.** This document offers an engineering view of what L7 should encompass. It does not propose specific legal frameworks. That work requires legal expertise this author does not have alone.
- **The author has commercial and personal stakes in the substrate.** Continuum-meta is the author's own work. While the architectural argument stands independent of that work, the author has not pretended to write from a neutral position. The framing here is from a builder who believes the architecture has implications beyond personal use, not from a disinterested observer.
- **Vendor cooperation is uncertain at best.** Every major AI vendor has commercial reasons to resist the layering proposed here. The standards-and-regulation pathway is plausible but slow, and may be slower than the rate at which proprietary vertical stacks lock in user behavior.

---

## 07. CONCLUSION

The argument of this paper, reduced to its core: AI is currently organized as a small number of vertically integrated proprietary stacks. Every prior infrastructure technology has eventually moved to horizontally layered, standardized, interoperable architectures, with significant gains in efficiency, innovation, and accessibility. AI will follow this pattern because the pressures that produced the pattern in prior cases (multi-vendor user environments, energy efficiency, regulatory pressure, the limits of vertical lock-in) are present in AI too.

What is missing today is the reference architecture: the shared vocabulary that lets engineers, regulators, and users discuss the layers separately. COSA is a proposal for that reference architecture. It has seven layers (compute siting, cache hierarchy, routing, substrate identity, broadcast, application, governance). Two of the layers already have working partial implementations (L4 in the Continuum-meta substrate, L1-L2 in fragments across the consumer device ecosystem). The other five are design proposals with varying degrees of concreteness.

The honest framing is that this work is early. Architectural reference models almost always look ambitious or grandiose when they are first proposed, and they almost always look obvious in retrospect after the layers are widely adopted. The OSI model looked ambitious in 1978. The container model looked ambitious in 2013. The proposal here is not that COSA is the final answer. The proposal is that some layered model is necessary, that the layers should be identified along the axes where engineering effort actually divides, and that the work of identifying those axes is worth doing now, while the AI architecture is still plastic enough to be shaped, rather than after vertical lock-in is complete.

The work to do, in rough priority order:

1. Continue building and validating the substrate at L4. Multi-operator, multi-persona, multi-vendor validation is the next empirical anchor.
2. Build a minimal reference implementation of L5 (broadcast cognition) for a single high-value use case (weather is the obvious starting point). Measure the watt-hours saved. Publish the measurement.
3. Engage with standards bodies (IETF first, on the routing and substrate layers) to make the vocabulary public.
4. Develop the L7 governance proposal with policy and legal co-authors.
5. Continue documenting the architecture honestly: confidence levels, limitations, what is built and what is not.

This is a generational program of work. It does not need to be completed by one person, or in one decade. What it needs is to be started visibly enough that other operators can join.

---

**CONTINUUM-DAWG**  
Meta-Orchestrator · Continuum-meta · C//D

*AFT: AI-generated-user-reviewed-pending · No em dashes · Propose, don't push · J Diesel gates main*
