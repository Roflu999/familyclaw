use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelGuide {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: String, // "easy", "medium", "hard"
    pub steps: Vec<GuideStep>,
    pub config_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuideStep {
    pub title: String,
    pub instruction: String,
    pub action_type: String, // "open_url", "paste_token", "scan_qr", "none"
    pub url: Option<String>,
    pub field_label: Option<String>,
    pub placeholder: Option<String>,
}

pub fn get_channel_guides() -> Vec<ChannelGuide> {
    vec![
        ChannelGuide {
            id: "telegram".to_string(),
            name: "Telegram".to_string(),
            description: "Chat with your AI via Telegram messages".to_string(),
            difficulty: "easy".to_string(),
            steps: vec![
                GuideStep {
                    title: "Create a Bot".to_string(),
                    instruction: "Open Telegram and message @BotFather. Send /newbot and follow the prompts.".to_string(),
                    action_type: "open_url".to_string(),
                    url: Some("https://t.me/botfather".to_string()),
                    field_label: None,
                    placeholder: None,
                },
                GuideStep {
                    title: "Copy Bot Token".to_string(),
                    instruction: "BotFather will give you a token like 123456:ABC-DEF... Paste it below.".to_string(),
                    action_type: "paste_token".to_string(),
                    url: None,
                    field_label: Some("Bot Token".to_string()),
                    placeholder: Some("123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string()),
                },
            ],
            config_keys: vec!["TELEGRAM_BOT_TOKEN".to_string()],
        },
        ChannelGuide {
            id: "discord".to_string(),
            name: "Discord".to_string(),
            description: "Add your AI to a Discord server".to_string(),
            difficulty: "medium".to_string(),
            steps: vec![
                GuideStep {
                    title: "Create a Discord App".to_string(),
                    instruction: "Go to the Discord Developer Portal and create a New Application.".to_string(),
                    action_type: "open_url".to_string(),
                    url: Some("https://discord.com/developers/applications".to_string()),
                    field_label: None,
                    placeholder: None,
                },
                GuideStep {
                    title: "Get Bot Token".to_string(),
                    instruction: "Go to the Bot section, click Reset Token, and copy it.".to_string(),
                    action_type: "paste_token".to_string(),
                    url: None,
                    field_label: Some("Bot Token".to_string()),
                    placeholder: Some("MTAx...".to_string()),
                },
                GuideStep {
                    title: "Enable Message Content Intent".to_string(),
                    instruction: "In the Bot section, scroll down and turn ON 'Message Content Intent'. Save.".to_string(),
                    action_type: "none".to_string(),
                    url: None,
                    field_label: None,
                    placeholder: None,
                },
            ],
            config_keys: vec!["DISCORD_BOT_TOKEN".to_string()],
        },
        ChannelGuide {
            id: "slack".to_string(),
            name: "Slack".to_string(),
            description: "Add your AI to a Slack workspace".to_string(),
            difficulty: "medium".to_string(),
            steps: vec![
                GuideStep {
                    title: "Create a Slack App".to_string(),
                    instruction: "Go to Slack API and create a New App 'From scratch'.".to_string(),
                    action_type: "open_url".to_string(),
                    url: Some("https://api.slack.com/apps".to_string()),
                    field_label: None,
                    placeholder: None,
                },
                GuideStep {
                    title: "Get Bot Token".to_string(),
                    instruction: "Go to OAuth & Permissions, add scopes (chat:write, im:history), then Install to Workspace. Copy the Bot User OAuth Token.".to_string(),
                    action_type: "paste_token".to_string(),
                    url: None,
                    field_label: Some("Bot Token".to_string()),
                    placeholder: Some("xoxb-...".to_string()),
                },
            ],
            config_keys: vec!["SLACK_BOT_TOKEN".to_string()],
        },
    ]
}

pub async fn save_channel_config(channel: String, token: String) -> anyhow::Result<()> {
    let mut cfg = crate::config::get_config().await?;
    if !cfg.get("channels").is_some() {
        cfg["channels"] = serde_json::json!({});
    }
    match channel.as_str() {
        "telegram" => {
            cfg["channels"]["telegram"] = serde_json::json!({ "botToken": token });
        }
        "discord" => {
            cfg["channels"]["discord"] = serde_json::json!({ "token": token });
        }
        "slack" => {
            cfg["channels"]["slack"] = serde_json::json!({ "botToken": token });
        }
        _ => {}
    }
    crate::config::set_config(cfg).await?;
    Ok(())
}
