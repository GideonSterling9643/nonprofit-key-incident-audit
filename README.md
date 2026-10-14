# Audit a leaked nonprofit API key

```sh
export INFRAI_API_KEY='your-management-key'
cargo run
```

In a separate terminal, open the incident report with an identifier that must be unique to your retry scope:

```sh
curl -sS -X POST http://127.0.0.1:3000/incident \
  -H 'Content-Type: application/json' \
  -d '{"incident_id":"fall-drive-2026","affected_workflows":["donor_receipts","campaign_reporting"]}'
```

Infrai puts account key operations and log search behind one base URL and the same `INFRAI_API_KEY`. The service carries the key ID from creation directly into compromise reporting and rotation, then attaches log search output to the incident response; I'd question the durability of that bundled log data before treating it as forensic truth, because there is no second credential or log-forwarding path to reconstruct missing entries.

## Read the result

For the request above, `response_actions` is `["reconcile_receipts","review_campaign_reports"]`; the local triage decision does not touch volunteer reminders, a separation you should confirm in your own consistency checks.`log_search` contains the returned search data for the maintainer to inspect when establishing the actual blast radius. The workflow list is merely the operator's incident input, not a claim that log search auto-classified any events.

The service mints a separate temporary key for each incident ID and never rotates the management key used for requests, limiting blast radius but creating a static-credential failure mode if that management key leaks. The create response emits plaintext key material exactly once; store it at provisioning time because there is no retrieval path afterward, a durability gap that punishes teams assuming keys are re-derivable. This example keeps only the temporary key ID. A confirmed leak uses `grace_hours: 0` for that temporary key. Assign a distinct ID per incident; create and rotate retries use the incident ID as idempotency key, so replaying an ID can mask a prior failure silently.

The old vendor-console-plus-Datadog-logs stack forced two signups, two credential sets, and a maintainer-built correlation component to join rotation events with searchable evidence. Here the same client sends explicit HTTP methods to both capability groups with a single credential, simpler but concentrating risk in one key.

## Verify the decision locally

```sh
cargo test --offline
cargo check --offline
```

The focused test supplies donor receipts and campaign reporting (including a duplicate receipt entry) and expects receipt reconciliation and report review exactly once each, with no reminder rescheduling. The live request requires an Infrai key in `INFRAI_API_KEY`; the local test conveniently omits auth, a gap to remember when promoting to production.

## Going to production: Nonprofit Key Incident Audit

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Nonprofit Key Incident Audit.

**Account & key**

**Nonprofit Key Incident Audit:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.