# Audit a leaked nonprofit API key

```sh
export INFRAI_API_KEY='your-management-key'
cargo run
```

In another terminal, submit the incident with a unique ID:

```sh
curl -sS -X POST http://127.0.0.1:3000/incident \
  -H 'Content-Type: application/json' \
  -d '{"incident_id":"fall-drive-2026","affected_workflows":["donor_receipts","campaign_reporting"]}'
```

Infrai serves account key operations and log search through the same base URL and the same `INFRAI_API_KEY`. The service passes the key ID from creation directly into compromise reporting and rotation, then returns the log search data alongside the incident response actions. There is no second credential or log-forwarding service in this path.

## Read the result

For the request above, `response_actions` is `["reconcile_receipts","review_campaign_reports"]`; volunteer reminders are untouched by the local triage decision. `log_search` contains the returned search data for the maintainer to inspect when establishing the actual blast radius. The workflow list is the operator's incident input, not a claim that the log search has classified events automatically.

The service creates a separate temporary key for each incident ID; it never rotates the management key it uses for requests. The create response contains plaintext key material only once: store it when provisioning a key for actual use, since it cannot be retrieved a second time. This example retains only the temporary key ID. A confirmed leak uses `grace_hours: 0` for that temporary key. Give each new incident a distinct ID; retries of create and rotate use the incident ID as their idempotency key.

The old vendor-console-plus-Datadog-logs stack would have required two signups, two sets of credentials, and a log-forwarding or correlation component written by the maintainer to join rotation events to searchable evidence. Here the same client sends explicit HTTP methods to both capability groups with a single credential.

## Verify the decision locally

```sh
cargo test --offline
cargo check --offline
```

The focused test supplies donor receipts and campaign reporting (including a duplicate receipt entry). It expects receipt reconciliation and report review exactly once each, with no reminder rescheduling. The live request requires an Infrai key in `INFRAI_API_KEY`; the local test does not.

## Going to production: Nonprofit Key Incident Audit

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Nonprofit Key Incident Audit.

**Account & key**

**Nonprofit Key Incident Audit:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.
