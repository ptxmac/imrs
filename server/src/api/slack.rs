use crate::SharedState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct Slack {
    text: String,
    response_url: String,
}

#[derive(Serialize)]
struct SlackResponse {
    response_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct SlackMessageAttachment {
    image_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct SlackMessage {
    text: String,
    response_type: String,
    attachments: Vec<SlackMessageAttachment>,
}

pub async fn slack(
    Query(query): Query<Slack>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("Slack request, {:?}", query);
    info!(" state: {:?}", state);
    let opt = {
        let state = state.read().await;
        state.opt.clone()
    };
    info!(" opts: {:?}", opt);
    let prefix = opt.url_prefix;

    tokio::spawn(async move {
        let ident = {
            let mut state = state.write().await;
            state.get_id_and_title(&query.text).await
        }
        .unwrap();

        info!("id: {:?}", ident);
        {
            let mut state = state.write().await;
            let _entry = match state.check(&ident) {
                Some(entry) => entry,
                None => state.update(&ident).await.unwrap(),
            };
        }

        // send to slack
        let name = urlencoding::encode(&query.text);
        info!("encoded: {}", name);

        let client = reqwest::Client::new();

        let m = SlackMessage {
            response_type: "in_channel".to_string(),
            text: ident.title,
            attachments: vec![SlackMessageAttachment {
                image_url: Some(format!("{}/api/image?name={}", prefix, name)),
            }],
        };

        info!("slack response: {:?}", m);
        let resp = client.post(&query.response_url).json(&m).send().await;
        if let Err(e) = resp {
            error!("Slack error: {}", e);
        }
    });

    Json(SlackResponse {
        response_type: "in_channel".to_string(),
        text: "Loading...".to_string(),
    })
}
