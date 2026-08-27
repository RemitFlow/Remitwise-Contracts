# Governed wasm upgrades

This contract treats an upgrade as a security-sensitive state transition. The
`upgrade` entrypoint does not accept a human-readable release name, a mutable
URL, or an arbitrary identifier. It accepts the exact 32-byte wasm artifact
hash that the Soroban deployer will resolve. The hash is the identity of the
artifact; a release number is the ordering identity.

## Why the baseline is explicit

Soroban's contract runtime can update the current contract wasm, but the
application contract does not receive a portable “currently installed wasm
hash” value that can safely be compared in business logic. The one-time
`set_upgrade_baseline` operation makes the deployment process declare that
value explicitly. It is admin-authorized and emits an audit event.

The deployment runbook must obtain the hash from the same artifact that is
installed at the contract address. It must not use a source commit hash in
place of the wasm hash, and it must not register a hash obtained from an
untrusted URL. Once set, the baseline cannot be overwritten through the
contract API.

## Upgrade invariants

Every accepted replacement satisfies all of these conditions:

1. The contract is initialized.
2. The current admin authenticates the call.
3. The caller-supplied expected hash equals the recorded active hash.
4. The replacement hash differs from the active hash.
5. The release number is exactly the active release number plus one.
6. The deployer host accepts the replacement hash as an uploaded artifact.
7. The storage record and audit event describe the same artifact and version.

Checks one through five execute before the contract changes its recorded
active artifact. The host update is the final operation. A host rejection
aborts the transaction, which rolls back the storage and event changes made by
the contract. This is important: a failed upgrade must not leave the metadata
claiming that an artifact is active when the runtime still executes the old
artifact.

## Mismatch handling

The expected hash is an optimistic concurrency guard. A deployment tool reads
the active hash, prepares a replacement, and submits the exact value it read.
If another release wins the race first, the stale deployment fails with an
artifact mismatch rather than overwriting the newer release. The operator must
reload state and decide whether the replacement is still appropriate.

An upgrade with a matching expected hash but a non-sequential version fails as
well. Version gaps make incident reconstruction ambiguous and permit an
operator to accidentally skip a required migration. The release number is
therefore monotonic and contiguous, beginning with version one after the
baseline version zero.

## Authorization model

Only the current admin can register the baseline or execute an upgrade. The
existing two-step admin transfer remains the source of truth for who holds
that role. A former admin cannot update the wasm after accepting a successor,
and a pending nominee cannot update it before accepting the role.

The privileged-call cooldown also applies to replacement execution. This is a
deliberate operational brake on repeated administrative changes. The cooldown
does not replace authentication, artifact verification, or version checks; it
only limits the frequency of otherwise valid privileged operations.

## Event and indexer contract

`upgrade_baseline_set` records the admin and baseline artifact. Every accepted
replacement emits `upgrade_applied` with the admin, sequential version, and
replacement artifact. Indexers should use the artifact bytes and version from
the event, not data reconstructed from a release dashboard.

An indexer should alert when it observes a version gap, a duplicate version,
an event whose artifact differs from the contract's current getter, or an
upgrade event without the expected preceding state transition. It should
retain the ledger sequence, transaction hash, contract address, admin, and
artifact hash for every event.

Events are evidence of the contract transition, not evidence that a human
review occurred. The deployment pipeline should attach the review record,
source revision, reproducible build inputs, wasm hash, and approval identities
to the release record outside the contract.

## Deployment procedure

Before the first replacement:

- build the release in a reproducible environment;
- calculate the SHA-256 wasm hash using the exact upload bytes;
- verify the artifact against the review record;
- verify the contract address and current admin;
- set the baseline once, if the contract has not recorded one;
- confirm the emitted baseline event in the target network.

For each later release:

- read the active artifact and version;
- build and independently verify the new wasm hash;
- submit the expected active hash, replacement hash, and next version;
- wait for finality and verify the upgrade event;
- call a read-only health method from the new ABI;
- archive the transaction and artifact metadata.

Never “fix” a mismatch by blindly retrying the old transaction. A mismatch
means the expected state changed. A failed artifact lookup is also not a
reason to use a source hash or a shortened digest. Stop the deployment and
reconcile the network state first.

## Rollback guarantees

Rollback is a new governed upgrade to the previously known artifact. It is
not a direct storage edit and it must use the next release number. For example,
version three can restore the bytes used by version one, but it is still
recorded as version four and must expect version three's hash. This preserves
the complete sequence and prevents the indexer from seeing a version move
backward.

If the host rejects an artifact because it was never uploaded or is malformed,
the current artifact metadata remains unchanged. Operators may upload and
verify the intended artifact, then retry using a freshly read expected hash.
No contract-side recovery path bypasses the admin check or version guard.

## Failure matrix

| Condition | Result | State change |
| --- | --- | --- |
| caller is not current admin | authorization failure | none |
| baseline is missing | artifact mismatch | none |
| expected hash is stale | artifact mismatch | none |
| replacement equals current hash | unchanged-artifact failure | none |
| version skips a release | version failure | none |
| replacement is not uploaded | host failure | transaction rollback |
| replacement is valid | upgrade event and runtime update | one new release |

The “none” outcomes are intentional. In particular, rejected requests must
not consume the next release number, update the cooldown marker, overwrite the
active hash, or emit a success event.

## Test expectations

The unit test suite covers baseline initialization, duplicate baseline
registration, missing baseline, stale expected hash, identical replacement,
version gaps, and uninitialized reads. A network integration suite should add
an uploaded wasm artifact and verify that a successful host update changes
behavior while retaining prior instance and persistent state.

The integration suite should also attempt an unauthorized update, an update
with the wrong expected hash, an unavailable artifact, a stale version, and a
rollback. After every failure it should query the active artifact and version
and compare them with the pre-call values. It should also verify that no
success event was emitted for the failed transaction.

## Reviewer checklist

- [ ] The baseline was obtained from installed wasm bytes.
- [ ] The baseline transaction was signed by the current admin.
- [ ] The replacement hash is independently reproducible.
- [ ] The expected hash was read immediately before submission.
- [ ] The version is the next contiguous release number.
- [ ] The artifact was uploaded before the update call.
- [ ] The upgrade event was observed and archived.
- [ ] The post-upgrade health call used the new interface.
- [ ] Existing state was checked after the replacement.
- [ ] A rollback plan uses a new version number.

This process keeps authorization, artifact identity, ordering, auditability,
and runtime installation in one atomic contract operation while leaving human
approval and reproducible-build evidence in the deployment system where they
can be independently reviewed.

## Threat scenarios

### Stolen deployment credentials

A stolen non-admin deployment credential cannot invoke the entrypoint because
the current admin's authorization is required. A stolen admin credential is a
governance incident; the deployment system should revoke it and complete an
admin transfer before any planned release. The contract's version and hash
guards still prevent an operator from silently replacing a release with the
same artifact or skipping an expected release.

### Stale automation

Two deployment jobs may read the same active artifact. Only the first valid
job can advance the release. The second job presents a stale expected hash and
fails without changing metadata. Automation should treat this as a state
conflict requiring review, not as a transient network error to retry forever.

### Wrong network or contract

The artifact hash is not enough to identify a deployment target. The pipeline
must verify the network, contract address, current admin, active version, and
active hash before signing. A correct artifact sent to the wrong contract is
still an operational failure, so target verification belongs in the pipeline
and the release record.

### Malicious artifact upload

Uploading a wasm artifact does not authorize it. The contract only accepts the
hash named by an authenticated admin and the caller must supply the expected
active hash. Review tooling should compare the uploaded bytes, locally built
bytes, and the hash passed to `upgrade` before the transaction is signed.

### Event ingestion delay

An indexer may observe the transaction later than the deployment tool. The
tool should wait for finality and retain the transaction hash rather than
assuming that an absent event means failure. Indexers should be replay-safe:
the same ledger event must not create two release records.

## State preservation

The wasm replacement changes code, not the contract's instance or persistent
storage. The migration review must list every storage key read by the new
artifact and confirm that its encoding remains compatible. Adding a new key is
safe when it has a distinct `InstanceKey` or `PersistentKey` variant; changing
the meaning of an existing key requires a versioned migration.

After a successful upgrade, the verification suite should read the admin,
token, transfer counter, paused state, and representative transfer records.
It should compare those values with a pre-upgrade snapshot. The active
artifact and release getters should then report the new hash and next version.

A release that cannot read prior state safely must not be installed merely
because its wasm hash is valid. Artifact authenticity and state compatibility
are separate review gates.

## Operational response

When an upgrade fails, preserve the failed transaction result and the exact
arguments. Read the active artifact and version from the contract, inspect the
latest upgrade event, and compare them with the deployment manifest. Do not
modify the baseline to make an old manifest pass. If a rollback is necessary,
open a new release record and use the normal sequential upgrade path.

When an unexpected upgrade succeeds, stop further privileged calls, preserve
the event and transaction data, and transfer or rotate administrative control
according to the incident plan. The contract intentionally does not include a
special emergency bypass because bypasses make the audit trail and atomicity
guarantees weaker under pressure.

## Manifest format

Each release manifest should contain the target network, contract address,
active version, expected active hash, replacement version, replacement hash,
source revision, reproducible-build environment, reviewer identities, and
transaction hash after finality. Keeping both old and new values in one
manifest makes stale-state detection straightforward during an incident.

The manifest should be immutable after submission. If a build is recreated,
create a new manifest and compare its bytes and hash with the original rather
than editing the old record. This preserves the distinction between a planned
release and the artifact that was actually installed.

## Compatibility promise

The upgrade guard is intentionally small and independent of transfer business
logic. Future releases may add policy around approvals or timelocks, but they
must retain the expected-artifact check, contiguous version rule, authenticated
admin, and atomic runtime update. Removing one of these checks requires a
separate security review and a migration plan.

The release owner is responsible for checking that the submitted version is
unused, the expected hash is current, and the replacement hash is the one
approved in the manifest. The contract records the result; it cannot judge
whether the surrounding approval process was followed. This separation keeps
the on-chain rule deterministic while making the off-chain evidence complete.

When a release is abandoned, do not consume a version by submitting an
unrelated artifact. A version is consumed only by a successful runtime update.
Abandoned manifests should be marked superseded or cancelled in the release
system and retained for audit, while the contract's next-version requirement
remains unchanged.

The final deployment report should include the before-and-after getter values,
the emitted event topics, and the host transaction result. These records let a
reviewer distinguish a rejected proposal, a rolled-back host update, and a
completed release without relying on mutable dashboard state.

This evidence should be retained with the source review and build logs for the
full lifetime of the contract, including after later replacements.

Retention is part of the security boundary: without the original manifest and
receipt, a later audit cannot prove which bytes were authorized or installed.

The same retention rule applies to failed attempts, because failures document
which concurrency and artifact guards protected the live contract.
