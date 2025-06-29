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
use log::error;
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

        // println!("Request: {:?}", request.clone());

        let response = self.client
            .chat()
            .create(request)
            .await
            .context("OpenAI chat completion request failed")?;


        Ok(response)
    }

    async fn execute_requests(
        &self,
        requests: Vec<HashMap<String, serde_json::Value>>,
    ) -> Vec<CreateChatCompletionResponse> {
        let arc_self = Arc::new(self.clone_inner());
        let mut futs = FuturesUnordered::new();

        for req in requests {
            let cloned = Arc::clone(&arc_self);
            futs.push(tokio::spawn(async move { cloned.send(req).await }));
        }

        let mut results = Vec::new();
        while let Some(res) = futs.next().await {
            match res {
                Ok(Ok(resp)) => results.push(resp),
                Ok(Err(e)) => error!("OpenAI request error: {:?}", e),
                Err(join_err) => error!("Task join error: {:?}", join_err),
            }
        }

        results
    }

    pub async fn send_multiple(
        &self,
        requests: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<PredictResponse>> {
        if requests.is_empty() {
            bail!("No data to send");
        }
        let responses = self.execute_requests(requests).await;
        log::info!("Collected {} responses for send_multiple", responses.len());

        // println!("Responses: \n{:#?}", responses.clone());

        let mut result = Vec::with_capacity(responses.len());
        for response in responses {
            for choise in response.choices {
                let json = match &choise.message.content {
                    Some(content) => content,
                    None => {
                        log::error!("Empty content from model");
                        continue;
                    }
                };

                let content = match serde_json::from_str::<PredictResponse>(json) {
                    Ok(mut val) => {
                        val.sort_predictions();
                        val
                    },
                    Err(err) => {
                        log::error!("{err}");
                        continue;
                    }
                };

                result.push(content);
            }
        }

        Ok(result)
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

        let responses = self.execute_requests(requests_info.requests.clone()).await;
        log::debug!("Collected {} responses for test", responses.len());

        let col = database.collection(DOG_INFO_COLLECTION);
        let repo = MongoDogInfoRepo::new(col);

        let (meta, races) = process_test_results(
            responses, 
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
