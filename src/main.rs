use std::env;

use image::Luma;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{SqliteStorage, serializer::Bincode},
    prelude::*,
    types::InputFile,
    utils::command::BotCommands,
};

use crate::brcode::BrCode;
mod brcode;

type MyDialogue = Dialogue<State, SqliteStorage<Bincode>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveCity {
        full_name: String,
    },
    ReceiveKey {
        full_name: String,
        city: String,
    },
    Configured {
        full_name: String,
        city: String,
        key: String,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Comandos suportados:")]
enum Command {
    #[command(description = "Mostra esse texto!")]
    Help,
    #[command(description = "Troca o nome do recebedor")]
    SetName(String),
    #[command(description = "Troca a cidade do recebedor")]
    SetCity(String),
    #[command(description = "Troca a chave pix do recebedor")]
    SetKey(String),
    #[command(description = "Gera o QR Code de pagamento", parse_with = "split")]
    Code { val: f64 },
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();
    let dbfile = match env::var("DATABASE_FILE") {
        Ok(file) => file,
        Err(_) => "bot_state.sqlite".into(),
    };
    let storage = SqliteStorage::open(&dbfile, Bincode)
        .await
        .expect("Failed to open SQLite storage");

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, SqliteStorage<Bincode>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::case![State::ReceiveCity { full_name }].endpoint(receive_city))
            .branch(dptree::case![State::ReceiveKey { full_name, city }].endpoint(receive_key))
            .branch(
                dptree::case![State::Configured {
                    full_name,
                    city,
                    key
                }]
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(handle_command),
                )
                .endpoint(handle_text),
            ),
    )
    .dependencies(dptree::deps![storage])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Para começar digite o seu nome? Ele sera visivel ao pagador",
    )
    .await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(
                msg.chat.id,
                "Me envie a cidade que está cadastrada sua conta bancaria",
            )
            .await?;
            dialogue
                .update(State::ReceiveCity {
                    full_name: text.into(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Me diga seu nome?").await?;
        }
    }

    Ok(())
}

async fn receive_city(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(city) => {
            bot.send_message(msg.chat.id, "Qual a sua chave pix?")
                .await?;
            dialogue
                .update(State::ReceiveKey {
                    full_name,
                    city: city.into(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Qual a sua cidade?").await?;
        }
    }

    Ok(())
}

async fn receive_key(
    bot: Bot,
    dialogue: MyDialogue,
    (full_name, city): (String, String),
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(key) => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
            dialogue
                .update(State::Configured {
                    full_name,
                    city,
                    key: key.into(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Qual a sua chave pix?")
                .await?;
        }
    }

    Ok(())
}

async fn handle_text(bot: Bot, _: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn handle_command(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    cmd: Command,
    (full_name, city, key): (String, String, String),
) -> HandlerResult {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::SetName(name) => {
            dialogue
                .update(State::Configured {
                    full_name: name.into(),
                    city,
                    key,
                })
                .await?;
        }
        Command::SetCity(city) => {
            dialogue
                .update(State::Configured {
                    full_name,
                    city: city.into(),
                    key,
                })
                .await?;
        }
        Command::SetKey(key) => {
            dialogue
                .update(State::Configured {
                    full_name,
                    city,
                    key: key.into(),
                })
                .await?;
        }
        Command::Code { val } => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Gerando QR code de pagamento no valor {:.2}, para '{} - {}'",
                    val, full_name, city
                ),
            )
            .await?;
            let code = BrCode::new(key, val, full_name, city);
            let payload = code.to_emv_code().to_string();
            let qr = QrCode::new(payload.as_bytes())?;
            let data = qr.render::<Luma<u8>>().build();
            let name = format!("/tmp/code-{}.png", msg.chat.id);
            data.save(&name)?;
            let photo = InputFile::file(&name);
            bot.send_photo(msg.chat.id, photo).await?;
            bot.send_message(msg.chat.id, payload).await?;
        }
    };
    Ok(())
}
