// TODO inspired (i.e. copy-pasted) by prover/prover_fri_gateway/src/api_data_fetcher.rs

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{sync::watch, time::sleep};

use crate::metrics::METRICS;

/// The path to the TEE API endpoint that returns the next proof generation data
pub(crate) const PROOF_GENERATION_DATA_ENDPOINT: &str = "/tee/proof_inputs";

/// The path to the API endpoint that submits the proof
pub(crate) const SUBMIT_PROOF_ENDPOINT: &str = "/tee/submit_proofs";

/// The path to the API endpoint that registers the attestation
pub(crate) const REGISTER_ATTESTATION_ENDPOINT: &str = "/tee/register_attestation";

pub(crate) struct PeriodicApiStruct {
    pub(crate) api_url: String,
    pub(crate) poll_duration: Duration,
    pub(crate) client: Client,
}

// TODO copy-paste
impl PeriodicApiStruct {
    pub(crate) async fn send_http_request<Req, Resp>(
        &self,
        request: Req,
        endpoint: &str,
    ) -> Result<Resp, reqwest::Error>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        tracing::info!("Sending request to {}", endpoint);

        self.client
            .post(endpoint)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<Resp>()
            .await
    }

    pub(crate) async fn run<Req>(
        self,
        mut stop_receiver: watch::Receiver<bool>,
    ) -> anyhow::Result<()>
    where
        Req: Send,
        Self: PeriodicApi<Req>,
    {
        tracing::info!(
            "Starting periodic job: {} with frequency: {:?}",
            Self::SERVICE_NAME,
            self.poll_duration
        );

        loop {
            if *stop_receiver.borrow() {
                tracing::warn!("Stop signal received, shutting down {}", Self::SERVICE_NAME);
                return Ok(());
            }

            if let Some((job_id, request)) = self.get_next_request().await {
                match self.send_request(job_id, request).await {
                    Ok(response) => {
                        self.handle_response(job_id, response).await;
                    }
                    Err(err) => {
                        METRICS.http_error[&Self::SERVICE_NAME].inc();
                        tracing::error!("HTTP request failed due to error: {}", err);
                    }
                }
            }
            tokio::select! {
                _ = stop_receiver.changed() => {
                    tracing::warn!("Stop signal received, shutting down {}", Self::SERVICE_NAME);
                    return Ok(());
                }
                _ = sleep(self.poll_duration) => {}
            }
        }
    }
}

/// Trait for fetching data from an API periodically.
#[async_trait]
pub(crate) trait PeriodicApi<Req: Send>: Sync + Send {
    type JobId: Send + Copy;
    type Response: Send;

    const SERVICE_NAME: &'static str;

    /// Returns the next request to be sent to the API and the endpoint to send it to.
    async fn get_next_request(&self) -> Option<(Self::JobId, Req)>;

    /// Handles the response from the API.
    async fn send_request(
        &self,
        job_id: Self::JobId,
        request: Req,
    ) -> reqwest::Result<Self::Response>;

    async fn handle_response(&self, job_id: Self::JobId, response: Self::Response);
}
