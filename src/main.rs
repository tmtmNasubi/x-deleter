use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

const API_URI: &str = "https://api.twitter.com/2/tweets";

#[derive(Serialize, Deserialize, Debug)]
struct TweetContainer {
    tweet: Tweet,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tweet {
    id_str: String,
}

/// JSON部分を抽出してVec<TweetContainer>に変換する
fn part_json(content: &str) -> Vec<TweetContainer> {
    let tweets_list = if let Some(pos) = content.find("[") {
        let json_part = &content[pos..];
        let tweets_list: Vec<TweetContainer> = serde_json::from_str(json_part).unwrap();
        println!("{} tweets found", tweets_list.len());
        tweets_list
    } else {
        Vec::new()
    };
    tweets_list
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("API_KEY")?;
    let api_secret = env::var("API_KEY_SECRET")?;
    let access_token = env::var("ACCESS_TOKEN")?;
    let access_token_secret = env::var("ACCESS_TOKEN_SECRET")?;

    let content = std::fs::read_to_string("src/data/tweets.js")?;

    let tweets_list = part_json(&content);

    let client = reqwest::Client::new();
    let token = oauth::Token::from_parts(api_key, api_secret, access_token, access_token_secret);

    for tweet in tweets_list {
        let id = &tweet.tweet.id_str;
        let endpoint = format!("{}/{}", API_URI, id);

        let authorization_header = oauth::delete(&endpoint, &(), &token, oauth::HMAC_SHA1);

        let response = client
            .delete(&endpoint)
            .header("Authorization", authorization_header)
            .send()
            .await?;

        let status = response.status();

        match status {
            reqwest::StatusCode::OK => println!("✔︎Tweet deleted successfully: {}", id),
            _ => {
                println!("×Failed to delete tweet: \n id: {}\n status: {} \n error_body: {}", id, status, response.text().await?);
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }

    Ok(())
}
