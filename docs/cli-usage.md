# CLI Usage

This page documents how to interact with a deployed RemitFlow contract using
the Stellar CLI. All examples assume the contract has been deployed and
initialized (see the [Deployment Guide](deployment-guide.md)).

---

## Environment Setup

Export common variables once to keep commands concise:

```sh
export CONTRACT_ID=CABC...XYZ   # contract ID from deploy
export NETWORK=testnet
export SOURCE=my-key            # stellar identity alias
```

---

## Admin Operations

### Initialize (once)

```sh
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS>
```

### Pause / Unpause

```sh
# Pause — blocks new transfers
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- pause

# Unpause — re-enables transfer creation
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- unpause
```

### Two-Step Admin Transfer

```sh
# Step 1 — current admin nominates a successor
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- transfer_admin \
  --new_admin <NEW_ADMIN_ADDRESS>

# Step 2 — successor accepts (must sign with their own key)
stellar contract invoke \
  --id $CONTRACT_ID --source <NEW_ADMIN_KEY> --network $NETWORK \
  -- accept_admin
```

### Manage Caller Allowlist

```sh
# Add a privileged caller
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- add_caller \
  --caller <ADDRESS>

# Remove a privileged caller
stellar contract invoke \
  --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- remove_caller \
  --caller <ADDRESS>

# Check allowlist status
stellar contract invoke \
  --id $CONTRACT_ID --network $NETWORK \
  -- is_caller_allowed \
  --caller <ADDRESS>
```

---

## Transfer Operations

### Create a Transfer

```sh
stellar contract invoke \
  --id $CONTRACT_ID --source <SENDER_KEY> --network $NETWORK \
  -- create_transfer \
  --from    <SENDER_ADDRESS> \
  --recipient <RECIPIENT_ADDRESS> \
  --amount  1000000 \
  --expiry  1800000000
```

Returns the new transfer id (u64).

### Claim a Transfer

```sh
stellar contract invoke \
  --id $CONTRACT_ID --source <RECIPIENT_KEY> --network $NETWORK \
  -- claim_transfer \
  --id 1 \
  --recipient <RECIPIENT_ADDRESS>
```

### Cancel a Transfer (after expiry)

```sh
stellar contract invoke \
  --id $CONTRACT_ID --source <SENDER_KEY> --network $NETWORK \
  -- cancel_transfer \
  --id 1 \
  --from <SENDER_ADDRESS>
```

### Batch Operations

```sh
stellar contract invoke \
  --id $CONTRACT_ID --source <KEY> --network $NETWORK \
  -- batch_operations \
  --operations '[
    {"Create": {"from": "G...", "recipient": "G...", "amount": 500, "expiry": 1800000000}},
    {"Claim":  {"id": 1, "recipient": "G..."}}
  ]'
```

---

## Query Operations

All queries are read-only and do not require a signing source.

```sh
# Get admin address
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- get_admin

# Get pending admin (two-step transfer in progress)
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- get_pending_admin

# Get token address
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- get_token

# Check if paused
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- is_paused

# Transfer counter (total created)
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- counter

# Total amount currently escrowed
stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- total_escrowed

# Get a single transfer
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- get_transfer --id 1

# Get transfer status
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- get_status --id 1

# Check if a transfer is expired
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- is_expired --id 1

# Check if a transfer id exists
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- transfer_exists --id 1

# Paginated transfer list (ids 1–25)
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- get_transfers_paged --start_id 1 --limit 25

# Count by status (Pending | Claimed | Cancelled)
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- count_by_status --status Pending

# Count transfers for a sender
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- count_for_sender --from <ADDRESS>

# Count transfers for a recipient
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- count_for_recipient --recipient <ADDRESS>
```

---

## Useful CLI Tips

**Simulate without submitting** (fee estimation):

```sh
stellar contract invoke ... --fee 1000000 --simulate-only
```

**JSON output** for scripting:

```sh
stellar contract invoke ... -- get_transfer --id 1 --output json
```

**Inspect contract metadata**:

```sh
stellar contract inspect --network $NETWORK --id $CONTRACT_ID
```
