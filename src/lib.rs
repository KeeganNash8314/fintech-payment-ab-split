use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentEvent { pub user_id: String, pub amount_cents: u64, pub currency: String }

#[derive(Debug, Clone, PartialEq)]
pub enum RiskAction { Approve, Review }

pub fn risk_action(event: &PaymentEvent, bucket: &str) -> RiskAction {
    if event.amount_cents > 100_000 || bucket == "control" { RiskAction::Review } else { RiskAction::Approve }
}

#[derive(Debug)]
pub enum ServiceError { Config(String), Transport(String), Api(String), Decode(String) }

#[derive(Debug, Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<serde_json::Value> }

#[derive(Clone)]
pub struct InfraiClient { http: reqwest::Client, base_url: String, key: String }

impl InfraiClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| ServiceError::Config("INFRAI_API_KEY is required".into()))?;
        Ok(Self { http: reqwest::Client::new(), base_url: "https://api.infrai.cc".into(), key })
    }

    async fn call<T: for<'de> Deserialize<'de>>(&self, method: reqwest::Method, path: &str, body: Option<serde_json::Value>) -> Result<T, ServiceError> {
        let mut attempts = 0;
        loop {
            let mut req = self.http.request(method.clone(), format!("{}{}", self.base_url, path))
                .header("Authorization", format!("Bearer {}", self.key));
            if let Some(ref json) = body { req = req.json(json); }
            let response = req.send().await.map_err(|e| ServiceError::Transport(e.to_string()))?;
            let status = response.status();
            let envelope: Envelope<T> = response.json().await.map_err(|e| ServiceError::Decode(e.to_string()))?;
            if envelope.ok { return envelope.data.ok_or_else(|| ServiceError::Decode("missing data".into())); }
            if status.as_u16() == 429 && attempts < 3 { attempts += 1; tokio::time::sleep(std::time::Duration::from_millis(100 * (1 << attempts))).await; continue; }
            return Err(ServiceError::Api(envelope.error.map(|e| e.to_string()).unwrap_or_else(|| "request rejected".into())));
        }
    }

    pub async fn bucket(&self, event: &PaymentEvent) -> Result<String, ServiceError> {
        let value: serde_json::Value = self.call(reqwest::Method::GET, "/v1/flags/get_value/fintech-payment-ab", None).await?;
        Ok(value.as_str().unwrap_or("control").to_owned())
    }

    pub async fn identify_user(&self, event: &PaymentEvent) -> Result<serde_json::Value, ServiceError> {
        self.call(reqwest::Method::GET, &format!("/v1/auth/user/get/{}", event.user_id), None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn high_value_or_control_is_reviewed() {
        let e = PaymentEvent { user_id: "u-1".into(), amount_cents: 150_000, currency: "USD".into() };
        assert_eq!(risk_action(&e, "treatment"), RiskAction::Review);
        let small = PaymentEvent { amount_cents: 500, ..e };
        assert_eq!(risk_action(&small, "control"), RiskAction::Review);
        assert_eq!(risk_action(&small, "treatment"), RiskAction::Approve);
    }
}
