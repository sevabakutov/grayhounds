use std::{
    sync::Arc,
    collections::HashMap
};
use anyhow::{
    anyhow, 
    Context, 
    Result, 
    bail
};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, 
        CreateChatCompletionRequestArgs, 
        CreateChatCompletionResponse, 
        ReasoningEffort, 
        ResponseFormat
    },
    Client
};
use mongodb::Database;
use crate::{
    constants::DOG_INFO_COLLECTION, 
    models::{
        OddsRange, 
        PredictResponse, 
        RequestsInfo, 
        Settings, 
        TestResults
    }, 
    utils::{
        get_response_format_json_schema, 
        process_test_results
    }, 
    MongoDogInfoRepo
};

pub struct OpenAIClient {
    client: Arc<Client<OpenAIConfig>>,
    config: Settings
}

impl OpenAIClient {
    pub fn new(config: Settings) -> Self {
        let openai_cfg = OpenAIConfig::new();
        let client = Arc::new(Client::with_config(openai_cfg));

        Self {
            client,
            config
        }
    }

    pub async fn send(&self, data: HashMap<String, serde_json::Value>) -> Result<CreateChatCompletionResponse> {
        let messages_value = data
            .get("messages")
            .ok_or(anyhow!("Missing key 'messages'"))?;

        let messages: Vec<ChatCompletionRequestMessage> = serde_json::from_value(messages_value.clone())
            .context("Failed to parse 'messages' value")?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.config.model.to_string())
            .frequency_penalty(self.config.frequency_penalty.unwrap_or_default())
            .logprobs(self.config.logprobs.unwrap_or_default())
            .presence_penalty(self.config.presence_penalty.unwrap_or_default())
            .reasoning_effort(self.config.reasoning_effort.clone().unwrap_or(ReasoningEffort::Medium))
            .temperature(self.config.temperature.unwrap_or(1.0))
            // .max_completion_tokens(self.config.max_completion_tokens.unwrap_or(10000))
            .response_format(ResponseFormat::JsonSchema { json_schema: get_response_format_json_schema() })
            // .seed(self.config.seed.unwrap_or(0))
            .messages(messages)
            .build()
            .context("Failed to build CreateChatCompletionRequestArgs")?;

        let response = self.client
            .chat()
            .create(request)
            .await
            .context("OpenAI chat completion request failed")?;


        Ok(response)
    }

    async fn execute_requests(
        &self,
        mut requests: Vec<HashMap<String, serde_json::Value>>,
    ) -> Vec<PredictResponse> {
        const MAX_RETRIES: usize = 3;
        let client = Arc::new(self.clone_inner());
        let mut ok = Vec::with_capacity(requests.len());

        for _ in 0..MAX_RETRIES {
            if requests.is_empty() {
                break;
            }

            let mut futs = FuturesUnordered::new();
            for (idx, req) in requests.into_iter().enumerate() {
                let c = Arc::clone(&client);
                futs.push(tokio::spawn(async move {
                    let r = c.send(req.clone()).await;
                    (idx, req, r)
                }));
            }

            let mut failed = Vec::new();
            while let Some(join_res) = futs.next().await {
                match join_res {
                    Ok((idx, _, Ok(resp))) if self.response_ok(&resp) => {
                        if let Some(p) = self.parse_choice(&resp) {
                            ok.push((idx, p));
                        }
                    }

                    Ok((_, orig_req, _)) => {
                        failed.push(orig_req);
                    }

                    Err(join_err) => {
                        log::error!("Task join error: {:?}", join_err);
                    }
                }
            }
            requests = failed;
        }

        ok.sort_by_key(|(idx, _)| *idx);
        ok.into_iter().map(|(_, p)| p).collect()
    }

    fn response_ok(&self, resp: &CreateChatCompletionResponse) -> bool {
        resp.choices.iter().all(|c| {
            c.message
                .content
                .as_ref()
                .and_then(|s| serde_json::from_str::<PredictResponse>(s).ok())
                .map(|p| p.predictions.iter().filter(|x| x.raw_score.eq(&0.0)).count() < 2)
                .unwrap_or(false)
        })
    }

    fn parse_choice(&self, resp: &CreateChatCompletionResponse) -> Option<PredictResponse> {
        resp.choices.iter().find_map(|c| {
            c.message.content.as_ref().and_then(|s| serde_json::from_str(s).ok())
        })
    }

    pub async fn send_multiple(
        &self,
        requests: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<PredictResponse>> {
        if requests.is_empty() {
            bail!("No data to send");
        }

        let mut responses = self
            .execute_requests(requests)
            .await
            .into_iter()
            .map(|mut p| {
                p.sort_predictions();
                p
            })
            .collect::<Vec<_>>();

        responses.sort_by_key(|p| (p.meta.date, p.meta.time));
        Ok(responses)
    }

    pub async fn test(
        &self,
        requests_info: RequestsInfo,
        database: Database,
        initial_balance: f64,
        initial_stake: f64,
        odds_range: OddsRange,
        is_favorite_protected: bool
    ) -> Result<TestResults> {
        if requests_info.requests.is_empty() {
            bail!("No data to send");
        }

        let predictions  = self.execute_requests(requests_info.requests.clone()).await;
        // log::debug!("Collected {} responses for test", responses.len());

        let col = database.collection(DOG_INFO_COLLECTION);
        let repo = MongoDogInfoRepo::new(col);

        let (meta, races) = process_test_results(
            predictions, 
            &repo,
            requests_info.total_races, 
            initial_balance, 
            initial_stake, 
            odds_range, 
            is_favorite_protected
        ).await?;

        Ok(TestResults::new(meta, races, requests_info.requests))
    }

    fn clone_inner(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            config: self.config.clone(),
        }
    }
}
