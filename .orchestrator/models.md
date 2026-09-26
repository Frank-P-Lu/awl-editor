# Model routing

Read before dispatch. [Orchestration guide](README.md).

- **Production (default)** — current Sonnet at medium, or `gpt-5.6-terra`
  at `medium`: implementation, structured research, routine diagnosis, merges
  and audits. Routine orchestration — status reads, dispatching agreed briefs
  and board updates — also uses this tier.
- **Difficult implementation** — current Opus, or `gpt-5.6-sol` at `high`:
  complex refactors and debugging with a concrete objective and understood scope.
- **Architecture and hard diagnosis** — current Opus, or `gpt-6-astra` at
  `high`: ambiguous ownership/state design, interacting failures, consequential
  design decisions and adversarial analysis that needs this tier's judgment.
  Prefer a bounded analysis with a concrete handoff to Production or Difficult
  implementation once the uncertainty is resolved; do not require a second agent
  when that would merely repeat the work.
- **Repeatable** — `gpt-5.6-luna` at `medium`: bounded extraction,
  classification, mechanical edits, contact-detail changes and deterministic
  inventories. Do not use this tier for ownership decisions or production audits.
- **Visual judge** — Fable, or `gpt-5.6-sol` at `xhigh`: receives real
  captures and returns a verdict only; the implementer applies it and owns the
  gates. Escalate an unresolved, consequential design question to the Architecture
  tier rather than routinely using Astra for every screenshot review.

Use full model IDs and an explicit effort in every dispatch. `high` is the starting
point for difficult implementation and architecture; use `xhigh` only when several
plausible candidates must be eliminated. `xhigh` remains the worker ceiling; do not
dispatch workers at `max` or above. Astra's availability does not promote ordinary
workers or routine orchestration to Astra, and this policy does not change the model
of an already-running conversation.

Choose by failure cost and uncertainty, not task size. Keep briefs/context focused,
avoid duplicate investigation and speculative delegation, and follow
`docs/verification.md`: targeted worker checks, then one full gate on the integrated
candidate. When weekly usage runs well ahead of the reset cycle, cut concurrency
and optional dispatch before cutting the tier where failure is expensive.

This routing is a workload recommendation, not an awl-specific model benchmark or a
promise of lower allowance consumption. Official guidance describes Astra's stronger
multistep capabilities and possible task-level API savings, while Codex allowances
also depend on context, reasoning, tools and caching. Reassess from actual outcomes
and usage rather than assuming that per-token price or message estimates alone
predict cost per completed task.

References: [Astra guidance](https://developers.openai.com/api/docs/guides/latest-model),
[Codex usage guidance](https://learn.chatgpt.com/docs/pricing#what-are-the-usage-limits-for-my-plan).
