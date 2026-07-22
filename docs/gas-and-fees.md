# Gas and Fees — Instruction Budget Considerations

This page documents the **instruction budget** profile of `remitflow-contract`
and how each entrypoint behaves under Soroban's metering model.

Soroban charges for every operation — storage reads/writes, host function
calls, arithmetic, and memory allocation. Each contract invocation has a
**CPU instruction budget** (~1 million) and a **memory budget** (~128 KiB).
The fee in network tokens (XLM) is proportional to the resources consumed,
so more-expensive entrypoints cost more per call.

When an entrypoint exceeds the instruction budget the invocation is **reverted**
with a `SorobanError: ResourceExhausted` — no partial state changes survive.

---

## Entrypoint Cost Classification

| Category | Entrypoints | Complexity | Notes |
|---|---|---|---|
| **Constant-time** | `initialize`, `get_admin`, `get_token`, `counter` | O(1) | 1–2 storage reads, no loops |
| **Admin actions** | `pause`, `unpause`, `add_caller`, `remove_caller`, `transfer_admin`, `accept_admin`, `rotate_admin` | O(1) | Auth check + 1 storage write + event |
| **Single-read** | `get_transfer`, `transfer_exists`, `get_status`, `is_expired` | O(1) | 1 storage read by id |
| **Single-mutate** | `claim_transfer`, `cancel_transfer` | O(1) | 1 storage read, auth check, token transfer, 1 storage write |
| **Create** | `create_transfer` | **O(n)** incurs internal call to `total_escrowed()` which scans all transfers |
| **Paged read** | `get_transfers_paged` | O(n) capped at `MAX_PAGE_SIZE` (100) | Worst case: scan 100-ish ids, return up to 100 records |
| **Full-scan** | `total_escrowed`, `count_for_sender`, `count_for_recipient`, `count_by_status` | **O(n) unbounded** — scan every recorded id from 1 to counter |
| **Batch** | `batch_operations` | O(k × cost(op)) where `k` = operation count | Each individual operation has its own cost; internal calls (e.g. `Create` → `total_escrowed`) compound |

---

## O(1) Entrypoints — Budget Safe

These entrypoints execute a deterministic, small number of operations and will
**never** exhaust the instruction budget on their own:

- `get_admin`, `get_token`, `counter`, `is_paused`
- `get_transfer(id)`, `transfer_exists(id)`, `get_status(id)`, `is_expired(id)`
- `claim_transfer(id, recipient)`, `cancel_transfer(id, from)`
- `pause()`, `unpause()`, `add_caller(caller)`, `remove_caller(caller)`
- `is_caller_allowed(caller)`

### Single-mutate implementation (`claim_transfer`, `cancel_transfer`)

```rust
// Reading a key-value pair from storage by id — O(1).
storage::get_transfer(&env, id);

// Token transfer — 5 Soroban host-function calls (constant).
token::Client::new(&env, &token).transfer(&src, &dst, &amount);

// Writing a single key — O(1).
storage::set_transfer(&env, &transfer);
```

No loops, no unbounded iteration.

---

## O(n) Unbounded Entrypoints — Budget Risk

These entrypoints iterate from id `1` to `counter`, making them linearly more
expensive as the number of transfers grows.

### `total_escrowed()`

```rust
let last = storage::get_counter(&env);    // O(1)
let mut id = 1u64;
while id <= last {
    // One storage read per transfer id — O(n)
    if let Some(transfer) = storage::get_transfer(&env, id) {
        if transfer.status == Status::Pending {
            total = saturating_add_amount(total, transfer.amount);
        }
    }
    id += 1;
}
```

Each iteration performs a **key-value storage read** (`Env::storage().get()`),
a pattern comparison, and conditional arithmetic. At scale, the accumulated
host-function calls exhaust the instruction budget.

### `count_for_sender(from)`, `count_for_recipient(recipient)`, `count_by_status(status)`

Identical scan pattern — same loop structure, same budget profile.

### Estimated budget exhaustion threshold

Based on Soroban's default ~1M CPU instruction limit and the host-function
cost per storage read (~3,000–4,000 instructions on testnet), a single
invocation of `total_escrowed` is expected to exhaust the budget somewhere
between **150–300 transfers**. This varies with:
- Storage key length (longer addresses cost slightly more)
- Network congestion (base fee adjustments)
- Soroban version / protocol updates

> ⚠️ **Practical rule**: Do not call `total_escrowed`, `count_for_sender`,
> `count_for_recipient`, or `count_by_status` on a contract that has more than
> **~100 transfers**. Beyond that, the call will likely revert.

### Indirect cost — `create_transfer` calls `total_escrowed`

Every call to `create_transfer` invokes `Self::total_escrowed(env.clone())`
to enforce the `MAX_TOTAL_ESCROWED` cap (lines 175–185 of `lib.rs`). This means
**creating a transfer is also an O(n) operation** with the same budget
implications. As transfer volume grows, `create_transfer` becomes more
expensive and eventually reverts.

---

## Bounded O(n) — `get_transfers_paged(start_id, limit)`

```rust
let page_size = limit.min(MAX_PAGE_SIZE);   // at most 100
while id <= last && page.len() < page_size {
    if let Some(transfer) = storage::get_transfer(&env, id) {
        page.push_back(transfer);
    }
    id += 1;
}
```

The loop is bounded by `MAX_PAGE_SIZE` (100), so the worst case is 100
storage reads. This is **budget-safe** even at high transfer counts — the
caller controls the limit and cannot exceed 100 records per page.

However, when `start_id` is far below `counter` and there are many empty
gaps (cancelled/consumed ids), the iteration will skip many ids before
collecting a full page — the actual work is still bounded by the loop
termination condition, not by the counter.

---

## `batch_operations` — Composite Cost

```rust
for operation in operations.iter() {
    let result = match operation {
        BatchOperation::Create(params) => Self::create_transfer(...),
        BatchOperation::Claim(params)  => Self::claim_transfer(...),
        BatchOperation::Cancel(params) => Self::cancel_transfer(...),
    };
}
```

The batch cost is **k × cost(op)**, where `k` is the number of operations.
Since every `Create` in the batch calls `total_escrowed()`, the batch
compounds the O(n) scan cost **k times** in a single invocation.

Budget recommendations for batches:

| Batch size | Safe? | Notes |
|---|---|---|
| 1–3 ops | ✅ Yes | If contract has < ~100 transfers |
| 5+ ops | ⚠️ Risky | `Create` ops each O(n), risk of budget exhaustion |
| 10+ ops | ❌ Avoid | Almost certainly reverts at moderate transfer counts |

---

## Fee Calculation — `calculate_fee(basis_points)`

The fee function in `src/math.rs` is O(1) — it performs three arithmetic
operations (division, multiplication, addition) with no loops or storage
access. Fee computation contributes negligible instruction cost.

---

## Recommendations

1. **Replace `total_escrowed()` storage scans with an accumulator.** Keep a
   running `total_escrowed` counter in storage, add on `create_transfer`,
   subtract on `claim_transfer` / `cancel_transfer`. This makes
   `create_transfer` O(1) and `total_escrowed` a single read.

2. **Replace count scans with indexed tallies.** Maintain increment-only
   counters per sender, per recipient, and per status. Update them atomically
   alongside each transfer state change.

3. **For now, avoid calling scan entrypoints past ~100 transfers.** As a
   short-term workaround until the accumulator/index changes are implemented,
   integrators should gate calls to `create_transfer`, `total_escrowed`, and the
   `count_*` functions by the counter value.

4. **Keep batch sizes small.** Limit `batch_operations` to 3–5 operations per
   call if any of them are `Create`.
