# LOCAL-FIRST COGNITION

**The Personal AI of 2030 Lives in Your Pocket, Not in a Datacenter**

COSA ARCHITECTURE · CORE SUBSTRATE LAYER

| Field | Value |
|---|---|
| Author | Justin Kintzele |
| Synthesis body | Claude Opus 4.7 |
| AFT | AI-generated-user-reviewed-pending |
| Version | v0.1 draft, 2026-05-29 |
| Tier | git-tracked, publishable |
| Placement | COSA Layers 1 and 2 (Physical Compute and Cache Hierarchy) |

---

## 00. ABSTRACT

The dominant pattern in 2026 consumer AI is cloud-first: a user types a query into a phone, the query travels to a hyperscale datacenter, an inference is computed, and the answer travels back. This pattern is treated as the natural state of personal AI. It is not. It is a transitional artifact of the period when only datacenter-scale hardware could run capable models, and the period is ending.

This paper argues that the personal AI of the coming decade will be local-first: small models on the device handling most interactions, a home compute node handling medium-complexity work, and the cloud reserved for genuinely hard reasoning. The form factor that makes this practical already exists in consumer electronics: the dock. The thermodynamics are favorable: home GPUs displace heating cost in cold climates rather than burdening the grid. The cross-device continuity problem (assistant follows you from phone to car to robot to glasses) is structurally the same problem as cross-vendor persona portability, which has already been solved at small scale in the Continuum-meta substrate.

This paper is one of three describing Cognitive Open Systems Architecture (COSA). It addresses Layers 1 and 2 of the stack: where compute physically lives, and how cognitive state is cached across those locations. A companion paper (L4) describes the substrate layer that makes the assistant portable across both vendors and form factors. A third paper describes the full seven-layer model.

**CONFIDENCE · HIGH** on the form-factor and thermodynamic arguments.  
**CONFIDENCE · MEDIUM** on industry-trajectory timing.  
**CONFIDENCE · LOW** on specific market-adoption predictions.

---

## 01. THE CURRENT PATTERN IS BACKWARD

Walk through a normal user interaction in 2026. A person asks their phone "what's the weather tomorrow." The phone packages the query, opens a TLS connection to a datacenter potentially hundreds of miles away, the query traverses a half-dozen network hops, an NVIDIA H100 spins up a context window, runs an inference pass against billions of parameters, generates a response, and ships it back over the same path. Total energy expenditure for the interaction is somewhere between watts and tens of watts of datacenter draw, plus the network transport, plus the cooling overhead. The latency is anywhere from two hundred milliseconds to two seconds. The marginal cost to the vendor is small but nonzero. The cost to the grid is small but nonzero. Multiplied across a billion daily interactions, the total is not small.

The user could have gotten the same answer from a local weather service running on a Raspberry Pi in their own house. Or from a 7-billion-parameter model running on the NPU built into their phone. Or, in many cases, from the cached answer to the same query they ran two hours ago. The full datacenter round-trip is overkill for the work being done.

This is the dominant pattern not because it is efficient but because it is the only pattern the current generation of consumer AI products support. The product surface area assumes the model lives in the cloud, the user lives on the edge, and every interaction is a stateless round trip across the gap. There is no architecture for "answer this locally if you can," no architecture for "use the cached answer from earlier today," and no architecture for "hand this question to my home compute node instead." The pattern is backward because the architecture is missing.

---

## 02. THE DOCK MODEL

The form factor that resolves this is borrowed wholesale from consumer electronics: the dock. The Nintendo Switch is the canonical example. The handheld unit has enough compute for portable play. Docking it connects it to mains power and a larger display, and the same console becomes a living-room device with more thermal headroom and more capability. The continuity is total: the same game state, the same user account, the same hardware identity, just a different power and thermal envelope.

The personal AI dock model maps directly:

**Pocket tier.** A small local model (call it 7-to-13-billion parameters, dense or MoE) runs on the device's NPU. Battery-bounded, thermally constrained, optimized for latency. Handles roughly 80 percent of routine queries: weather, calendar lookups, short-form drafting, common reference questions, voice commands, transcription. No network round-trip. Sub-100ms response. Zero marginal cost per query.

**Home tier.** A larger model (call it 70-to-200-billion parameters, with serious context window) runs on a basement or closet GPU node, drawing maybe 400 to 800 watts under load. When the pocket device is on the home network, queries that exceed the pocket model's capability route to the home node automatically. Sub-second response. Marginal cost limited to electricity. The node is doing for personal AI what a home NAS does for storage and what a home server does for media: providing a private compute resource that the user owns.

**Cloud tier.** Genuinely hard reasoning, novel research questions, the deep work that benefits from frontier models, routes to the hyperscale cloud as it does today. Higher latency, real per-query cost, but used sparingly because the lower tiers absorb the routine load.

The pocket-to-home transition is the dock event. When a phone arrives at home and joins the local network, it does not need to know it is now docked. The routing layer (COSA L3, covered in the synthesis paper) discovers the home node and starts preferring it for medium-complexity queries. The user does not see a mode change. They see the same assistant getting faster, more capable, and quieter on their bill.

---

## 03. THERMODYNAMICS AS THE DESIGN CONSTRAINT

The argument for siting compute at home is not primarily a privacy argument or a sovereignty argument, though both apply. The argument is thermodynamic. Compute generates heat. Heat is a liability in a datacenter and an asset in a Minnesota house in February.

A 600-watt GPU running continuously in a residential basement produces roughly 2,000 BTU/hour of heat output. That is approximately equivalent to a baseboard heater. In a cold-climate home, that heat is not waste, it is heating, and it offsets the cost of the furnace or heat pump that would otherwise be producing the same BTUs. The compute is, in effect, free during heating season: the electricity is doing the same work that electricity would do anyway, plus producing useful cognition as a byproduct.

This is not speculative. Microsoft and Meta have both deployed datacenter-scale variants of this idea at industrial scale, with district heating offtake in Finland and the Netherlands. The residential-scale version is mechanically simpler because the home is already a heat sink: every joule the GPU produces stays in the building envelope and reduces furnace load.

The summer case requires more thought. In hot months, the same compute becomes a thermal liability, and a well-designed home node should be capable of throttling or shifting load. One practical pattern: in summer, the home node runs only when needed, and routine queries route to the cloud where centralized cooling has scale economies; in winter, the home node runs aggressively because the heat is wanted anyway. Time-of-year routing becomes a tuning parameter, not an architectural quirk.

The cumulative point is simple. The current model treats all AI compute as if it occurs in a single thermal context (datacenter, cooling expensive, summer everywhere). The local-first model recognizes that residential and small-business sites have widely varying thermal needs across geography and season, and that placing compute where the heat is wanted is straightforwardly more efficient than placing it where the heat must be expensively rejected.

**CONFIDENCE · HIGH** on the heat-reclamation argument for cold-climate residential.  
**CONFIDENCE · MEDIUM** on hot-climate strategies; the seasonal-routing pattern is plausible but unproven at scale.

---

## 04. CROSS-FORM-FACTOR CONTINUITY: JARVIS AS AN ENGINEERING PROBLEM

The "AI assistant follows you everywhere" picture (Jarvis from Iron Man, the OS in Her, every science fiction depiction of a persistent companion) is treated in industry discourse as either decades away or as a product problem one vendor will eventually solve. It is neither. It is an engineering problem with a known shape.

The user wants the same assistant on:
- Their phone, when out of the house
- Their car, when driving
- A humanoid robot or home-deployed personal assistant device, when at home
- Smart glasses or wearables, throughout
- A laptop or workstation, when at work
- A voice interface in the kitchen, the living room, the bedroom

Each of these is a different form factor with different sensors, different actuators, different compute envelopes, and potentially different underlying models. What the user wants is identity continuity: it is the same assistant, with the same memory of yesterday's conversation, the same understanding of their preferences, the same voice and tone, no matter which form factor they are addressing it through.

This is the same problem as cross-vendor persona portability, which has already been solved at small scale. In the Continuum-meta substrate (described in the L4 companion paper), a persona is defined as a file: a structured Markdown definition that specifies voice, scope, rules, and operational history. When Claude renders the persona, it is a Claude body executing the persona definition. When Gemini renders it, it is a Gemini body executing the same definition. The persona persists across vendors because the persona is not the vendor.

The form-factor case is the same problem one layer over. When the persona is rendered by the robot's onboard model, it is a robot-body execution of the persona definition. When it is rendered by the car's voice assistant, it is a car-body execution of the same definition. When it is rendered by the phone's NPU, it is a phone-body execution. The persona persists across form factors because the persona is not the form factor.

The substrate (L4) handles identity. The cache hierarchy (L2) handles state continuity, so that the robot in the kitchen knows what the car said on the drive home. The routing layer (L3) handles which body handles which query. The physical layer (L1) is just where each body's compute happens to live. The architecture, decomposed correctly, has the form-factor continuity problem already solved in principle. What is missing is the deployment.

---

## 05. WHAT THE INDUSTRY IS ALREADY BUILDING

The pieces are arriving faster than the consumer narrative reflects.

- **Apple Intelligence.** On-device first, cloud fallback ("Private Cloud Compute"). The architectural choice is explicit: routine queries stay on the device, hard queries route to Apple-controlled cloud infrastructure designed for verifiable privacy. This is the pocket-tier-plus-cloud-tier model without the home-tier in between, but it endorses the local-first direction.
- **Qualcomm.** Snapdragon X and successor chipsets ship with NPUs sized for 7-to-13B-parameter local inference at sub-1W power profiles. The hardware to support the pocket tier is already in shipping consumer devices.
- **NVIDIA Project DIGITS and successors.** A consumer-priced personal AI workstation aimed at developer and enthusiast home use. This is the home tier in early commercial form. The product framing is "developer," but the hardware envelope is exactly what a residential home node needs.
- **Open-weight models.** Llama, Mistral, Qwen, DeepSeek, and others continue to release capable open-weight models that can run on the hardware described above. The model-availability constraint that previously forced cloud-first is loosening every quarter.
- **Mobile chipset roadmaps.** Apple, Google Tensor, MediaTek, and Samsung Exynos all show committed NPU growth on multi-year roadmaps. The trajectory is not contested.

What is missing is not the hardware. The hardware is shipping. What is missing is the orchestration layer that makes these scattered pieces behave as a coherent personal cognitive system rather than five disconnected products with the same logo.

---

## 06. WHAT IS MISSING: THE ORCHESTRATION LAYER

A user with a Snapdragon-powered phone, a NVIDIA DIGITS workstation, an Apple Watch, a car with built-in voice assistant, and a cloud-AI subscription has access to all five tiers of capable compute. They cannot use them as a single system, because no single system spans them. Each device runs its own walled assistant. Apple's stays on Apple. Google's stays on Google. The car's stays on whatever the OEM licensed. None of them know about each other. None of them share state. None of them route queries between tiers.

The orchestration layer is the missing piece. Its responsibilities:

- **Discovery.** When the user's phone arrives on the home network, it should detect the home compute node and the wearables and the in-home voice interface, and register them as available bodies.
- **Routing.** Given a query, decide which body should handle it. Trivial query plus pocket capable plus user away from home equals pocket. Heavy query plus user at home equals home node. Cross-domain reasoning equals cloud. The decision is a function of query shape, body availability, latency budget, and cost.
- **State synchronization.** Whatever the user told the kitchen voice interface this morning should be available to the car this afternoon. Whatever the robot did in the living room should be visible to the phone when the user picks it up. The shared state lives in the cache hierarchy and is replicated lazily to wherever it is needed.
- **Identity continuity.** All of the above bodies are rendering the same persona. The user is talking to "their assistant," not to "Apple's assistant on the phone, and Google's in the car, and OpenAI in the basement." The persona is the user-visible entity. The bodies are infrastructure.

This is not a research problem. It is an integration problem. The pieces all exist. They have not been assembled this way because no single vendor has the incentive to assemble them: Apple wants the user inside Apple's silo, Google wants the user inside Google's silo, OpenAI wants the user inside OpenAI's silo. The orchestration layer is the thing that has to live outside any single vendor, owned by the user, run on infrastructure the user controls. That is what the Continuum-meta substrate, generalized to the consumer case, becomes.

---

## 07. LIMITATIONS AND OPEN QUESTIONS

- **Vendor cooperation is uncertain.** The major device vendors have commercial reasons to resist a user-controlled orchestration layer. Apple may not allow third-party persistent assistants to bind to the device-level intent layer. Google may permit it but degrade the experience. The standards-and-regulatory pathway (covered in the synthesis paper) is the realistic route, but it is slower than wishful thinking suggests.
- **Privacy and security at the home node are nontrivial.** A home compute node that holds the user's full cognitive cache becomes a high-value target. Encryption at rest, access control, and backup strategy all need to be designed before this is consumer-deployable. Treating the home node like a NAS (which has matured significantly over twenty years) is the closest analog and a workable starting point.
- **Initial cost is real.** A capable home compute node in 2026 is a several-thousand-dollar investment. The economics work out over the device's life when measured against cloud-AI subscription costs plus the value of the heat output, but the upfront cost is a real adoption barrier. The trajectory is favorable (consumer NPUs and home GPUs are getting cheaper per FLOP every year), but it will be a five-to-ten-year window before the home tier is broadly affordable.
- **Model availability for the home tier.** Open-weight models are improving but still trail frontier closed models. Whether the home tier can absorb 80 percent of medium-complexity work depends on whether open-weight quality continues its current trajectory. Most signs point to yes; this is a confidence-medium claim.
- **Software ecosystem.** Even with hardware and models in place, the application layer (COSA L6) has to learn to address "the assistant" rather than "the API." This requires either standards adoption or third-party integration, neither of which is automatic.

---

## 08. CONCLUSION

The personal AI of the late 2020s and early 2030s will not live in a datacenter. The hardware trajectory does not support it. The thermodynamic argument does not support it. The economic argument for the user does not support it. The cross-form-factor continuity the user actually wants does not support it. What it will look like instead is a tiered system: a small fast model in the pocket for routine work, a larger node in the home for medium-complexity work, the cloud reserved for the genuinely hard cases. The same assistant identity persists across all three tiers and across every form factor the user interacts with through them.

The architecture to make this real already exists in fragments. Apple has the pocket-and-cloud version. NVIDIA has the home-node hardware. Open-weight models have the local intelligence. The Continuum-meta substrate has the cross-body identity persistence. What is missing is the orchestration layer that assembles these into a coherent personal cognitive system the user owns.

The contribution of this paper is not new technology. It is the framing: that the cloud-first pattern is transitional, that the local-first pattern is the natural endpoint, and that the orchestration layer is the unsexy infrastructure piece that has to be built (and that no major vendor has reason to build) for the natural endpoint to arrive on time.

That framing matters, because it changes what counts as ambitious. Building a better cloud assistant is incremental. Building the dock model, with persona persistence across form factors and heat reclamation as a design feature, is not incremental. It is the next architectural generation. The hardware is shipping. The substrate exists. What remains is the work.

---

**CONTINUUM-DAWG**  
Meta-Orchestrator · Continuum-meta · C//D

*AFT: AI-generated-user-reviewed-pending · No em dashes · Propose, don't push · J Diesel gates main*
