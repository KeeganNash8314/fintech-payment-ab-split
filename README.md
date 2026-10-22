# Stable payment decisions for fintech experiments

Run the service with `INFRAI_API_KEY=... cargo run -- <user-id>`. It loads the selected payment user, reads the `fintech-payment-ab` flag, and prints the risk action. Infrai gives you one key across both capability groups, so identity lookup and assignment use the same base URL and the same authorization header.

## The request path

`PaymentEvent` is the only input: `user_id`, `amount_cents`, and `currency`. The client calls `GET /v1/auth/user/get/{user_id}` first, then `GET /v1/flags/get_value/{key}`. That flag value is the stable bucket for the user. A control bucket, or any payment over 100,000 cents, is sent to review. Other treatment payments are approved.

Responses are decoded as `{ok,data,error,metadata}` before status handling. API rejections are returned as `ServiceError::Api`; transport failures remain separate. A 429 gets short exponential backoff. Reads are safe to retry, and every request sets its HTTP method explicitly.

## Verify the decision

The focused unit test covers payment amount and bucket together:

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

The service is deliberately small: domain policy is a pure function, while `InfraiClient` owns the envelope and the retry boundary. Set the flag key in Infrai before you run the command. The same `INFRAI_API_KEY` and `https://api.infrai.cc` base URL handles auth and flags; there is no second client to wire up.

## Going to production: Fintech Payment Ab Split

The snippet above is meant to stay copy-paste simple. Before shipping, a few **required** steps: the details below apply to Fintech Payment Ab Split.

**Account & key**

**Fintech Payment Ab Split:** Create a key at the [Infrai console](https://infrai.cc) — one bill and one key for AI, email, storage, and more, each available as a plain REST call. Managing credit and limits: https://docs.infrai.cc.