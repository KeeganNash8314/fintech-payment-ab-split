use fintech_ab_split::{risk_action, InfraiClient, PaymentEvent};

#[tokio::main]
async fn main() {
    let user_id = match std::env::args().nth(1) {
        Some(user_id) => user_id,
        None => {
            eprintln!("usage: fintech-ab-split <user-id>");
            return;
        }
    };
    let event = PaymentEvent { user_id, amount_cents: 4200, currency: "USD".into() };
    let client = match InfraiClient::from_env() { Ok(c) => c, Err(e) => { eprintln!("configuration error: {:?}", e); return; } };
    match (client.identify_user(&event).await, client.bucket(&event).await) {
        (Ok(_user), Ok(bucket)) => println!("user={} bucket={} action={:?}", event.user_id, bucket, risk_action(&event, &bucket)),
        (Err(e), _) | (_, Err(e)) => eprintln!("request error: {:?}", e),
    }
}
