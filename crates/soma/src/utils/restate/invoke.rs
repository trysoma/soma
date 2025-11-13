use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use tracing::info;

#[derive(Clone)]
pub struct RestateIngressClient {
    client: Client,
    restate_base: String,
}

impl RestateIngressClient {
    pub fn new(restate_base: String) -> Self {
        Self {
            client: Client::new(),
            restate_base: restate_base.trim_end_matches('/').to_string(),
        }
    }

    pub async fn invoke_virtual_object_handler(
        &self,
        service: &str,
        key: &str,
        handler: &str,
        body: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/{}/{}/{}",
            self.restate_base,
            urlencoding::encode(service),
            urlencoding::encode(key),
            urlencoding::encode(handler)
        );
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn resolve_awakeable_generic(
        &self,
        awakeable_id: &str,
        body: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/restate/awakeables/{}/resolve",
            self.restate_base,
            urlencoding::encode(awakeable_id)
        );
        info!("Resolving awakeable: {}", url);
        let res = self.client.post(&url).json(&body).send().await?;
        let status = res.status();
        let text = res.text().await?;
        info!("Response from {} ({})", url, status);
        info!("  {}", text);

        if !status.is_success() {
            return Err(anyhow::anyhow!("Failed to resolve awakeable: {text}"));
        }
        Ok(())
    }

    pub async fn resolve_awakeable<T>(&self, awakeable_id: &str, body: &T) -> Result<()>
    where
        T: Serialize,
    {
        let url = format!(
            "{}/restate/awakeables/{}/resolve",
            self.restate_base,
            urlencoding::encode(awakeable_id)
        );
        info!("Resolving awakeable: {}", url);
        let res = self.client.post(&url).json(body).send().await?;
        let status = res.status();
        let text = res.text().await?;
        info!("Response from {} ({})", url, status);
        info!("  {}", text);

        if !status.is_success() {
            return Err(anyhow::anyhow!("Failed to resolve awakeable: {text}"));
        }
        Ok(())
    }

    pub async fn reject_awakeable<T: Serialize>(&self, awakeable_id: &str, body: &T) -> Result<()> {
        let url = format!(
            "{}/restate/awakeables/{}/reject",
            self.restate_base,
            urlencoding::encode(awakeable_id)
        );
        let res = self.client.post(&url).json(body).send().await?;
        let status = res.status();
        let text = res.text().await?;
        info!("Response from {} ({})", url, status);
        info!("  {}", text);

        if !status.is_success() {
            return Err(anyhow::anyhow!("Failed to reject awakeable: {text}"));
        }
        Ok(())
    }
}

pub fn construct_initial_object_id(task_id: &str) -> String {
    format!("soma:v1:task:{task_id}")
}

pub fn construct_cancel_awakeable_id(task_id: &str) -> String {
    format!("soma:v1:task:{task_id}:cancel")
}

pub fn construct_invocation_object_id(task_id: &str) -> String {
    format!("soma:v1:task:{task_id}:invocation")
}

// const constructCancelId = (taskId: string) => `soma:v1:cancel:${taskId}`;
// const constructInvocationKey = (taskId: string) => `soma:v1:invocation:${taskId}`;
