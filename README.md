# hodeauxledger

HodeauxLedger is the current implementation of the ledger powering Temporal Cryptophysics. It provides an append-only, cryptographically signed record model (R⬢) anchored to Genesis Time (GT), with scope-bound policy, role grants, and usher K-of-N quorum. CBOR on the wire, auditable by design—scope or it didn’t happen.

# HodeauxLedger 🕰️⬢

_“Scope or it didn’t happen.”_

HodeauxLedger is the core implementation of the **temporal lattice** powering **Trust Architecture / Temporal Cryptophysics**.  
It provides an **append-only**, **cryptographically signed**, and **time-anchored** record model (`R⬢`) bound to **Genesis Time (GT)**.

Records are serialized as **CBOR**, scoped by cryptographic namespaces, governed by **policy**, and propagated through **usher quorum**.  
No proof-of-work. No burning the planet. Just verifiable, ordered truth.

---

## ✨ Key Features

-   🧭 **Temporal Anchoring** — Every record is pinned to sidereal-based Genesis Time (GT), forming a universal chronological substrate.
-   🔐 **Cryptographically Signed R⬢ Records** — Immutable, append-only ledger entries with Ed25519 signatures and hash chaining.
-   🧰 **Scopes & Policy** — Hierarchical cryptographic namespaces with local policy, role grants, and authority rules.
-   🧑‍⚖️ **K-of-N Usher Quorum** — Distributed validation without proof-of-work; trust is enforced through quorum signatures.
-   🧾 **CBOR Serialization** — Compact binary records for efficient on-the-wire transport and auditability.
-   🪄 **Auditable by Design** — Every interaction is scoped, timestamped, and signed. Scope or it didn’t happen.

---

## 🧪 Quickstart — “Hello, Temporal World”

The fastest way to see HodeauxLedger in action is to cast a **dummy record** into a test scope.

```bash
# Clone the repo
git clone https://github.com/spark-86/hodeauxledger
cd hodeauxledger

# Build the CLI
cargo build --release

# Tool commands to come

```
