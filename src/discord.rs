//! Utility wrapper. Stolen with modifications from https://github.com/EmbarkStudios/discord-sdk/blob/main/examples-shared/src/lib.rs

/// syscord APPID
pub const APP_ID: discord_sdk::AppId = 868125538273878066;

pub struct Client {
    pub discord: discord_sdk::Discord,
    pub user: discord_sdk::user::User,
    pub wheel: discord_sdk::wheel::Wheel,
}

pub async fn make_client(subs: discord_sdk::Subscriptions) -> Client {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
        tracing::error!(error = ?err, "encountered an error");
    }));

    let mut user = wheel.user();

    let discord = discord_sdk::Discord::new(
        discord_sdk::DiscordApp::PlainId(APP_ID),
        subs,
        Box::new(handler),
    )
    .expect("unable to create discord client");

    tracing::info!("waiting for handshake...");
    user.0.changed().await.unwrap();

    let user = match &*user.0.borrow() {
        discord_sdk::wheel::UserState::Connected(user) => user.clone(),
        discord_sdk::wheel::UserState::Disconnected(err) => {
            panic!("failed to connect to Discord: {}", err)
        }
    };

    tracing::info!("connected to Discord, local user is {:#?}", user);

    Client {
        discord,
        user,
        wheel,
    }
}
