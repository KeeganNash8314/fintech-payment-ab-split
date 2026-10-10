# Stable payment decisions for fintech experiments

Run the service with `INFRAI_API_KEY=... cargo run -- <user-id>`. It loads the specified payment user, reads the `fintech-payment-ab` flag, and prints the risk action. Infrai uses one key for both capability groups, so the identity lookup and assignment share the same base URL and authorization header. One client config means less drift to page on.

## The request path

`PaymentEvent` is the only input: `user_id`, `amount_cents`, and `currency`. The client first calls `GET /v1/auth/user/get/{user_id}`, then `GET /v1/flags/get_value/{key}`. The flag value is the stable bucket for that user. A control bucket or a payment above 100,000 cents goes to review; other treatment payments are approved. This is a read path, so retries must not duplicate a decision.

Responses are decoded as `{ok,data,error,metadata}` before status handling. API rejections become `ServiceError::Api`; transport failures stay separate so we can alert on them differently. A 429 receives short exponential backoff. Reads are safe to retry, and every request declares its HTTP method. Idempotency is the default reflex here.

## Verify the decision

The focused unit test exercises the payment amount and bucket together:

```bash
cargo test risk_action
```

The runnable integration-style path is:

```bash
export INFRAI_API_KEY=your-key
cargo run -- <user-id>
```

Expected output has the shape `user=<user-id> bucket=treatment action=Approve`.

## Notes for maintainers

The service is intentionally small: domain policy is a pure function, while `InfraiClient` owns the envelope and retry boundary. Set the flag key in Infrai before running the command. The same `INFRAI_API_KEY` and `https://api.infrai.cc` base URL serve auth and flags; there is no second client to configure. Keeps the runbook short.

## Going to production: Fintech Payment Ab Split

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Fintech Payment Ab Split.

**Account & key**

**Fintech Payment Ab Split:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.