use reqwest::Client;
use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{query, ConnectOptions};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

type BaseUrl = str;
type Url = String;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
struct Data {
    state: State,
    //bets: Option<Bets>,
}

impl Data {
    const STATE: &BaseUrl = "https://www.saltybet.com/state.json?t=";
    const BETS: &BaseUrl = "https://www.saltybet.com/zdata.json?t=";
    fn get_full(partial: &BaseUrl, time: &str) -> String {
        let mut full = String::from(partial);
        full.push_str(time);
        return full;
    }
    fn state(time: &str) -> Url {
        Self::get_full(Self::STATE, time)
    }
    fn bets(time: &str) -> Url {
        Self::get_full(Self::BETS, time)
    }
    async fn from(time: SystemTime, client: &Client) -> Result<Self, Error> {
        let stamp = time
            .duration_since(UNIX_EPOCH)
            .expect("System needs a time for this program to function")
            .as_secs()
            .to_string();
        let urls = [Self::state(&stamp), Self::bets(&stamp)];
        let (state, _bets) = (
            client.get(&urls[0]).send().await?,
            client.get(&urls[1]).send().await?,
        );
        Ok(Self {
            state: state.json::<State>().await?,
            //bets: None, TODO bets dont serialize yet
        })
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct State {
    #[serde(rename = "p1name")]
    player1: String,
    #[serde(rename = "p2name")]
    player2: String,
    p1total: String,
    p2total: String,
    alert: String,
    status: MatchStatus,
}

impl State {
    fn finished(&self) -> bool {
        self.status == MatchStatus::PlayerOneWin || self.status == MatchStatus::PlayerTwoWin
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy, Hash)]
#[serde(rename_all = "lowercase")]
enum MatchStatus {
    OPEN,
    LOCKED,
    #[serde(rename = "1")]
    PlayerOneWin,
    #[serde(rename = "2")]
    PlayerTwoWin,
}

impl Into<i16> for MatchStatus {
    fn into(self) -> i16 {
        match self {
            MatchStatus::OPEN => 0,
            MatchStatus::LOCKED => -1,
            MatchStatus::PlayerOneWin => 1,
            MatchStatus::PlayerTwoWin => 2,
        }
    }
}

pub struct Logger {
    client: Client,
    pool: PgPool,
}

impl Logger {
    pub async fn from(db: &str, pwd: &str) -> Result<Self, Error> {
        let client = Client::new();
        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            "recorder", pwd, db, 5432, "salty"
        );
        dbg!(&url);
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .idle_timeout(Duration::from_millis(10))
            .connect(&url)
            .await?;
        let logger = Self { client, pool };
        Ok(logger)
    }

    async fn poll_api(&mut self) -> Result<Data, Error> {
        Data::from(SystemTime::now(), &self.client).await
    }
    pub async fn record(&mut self) {
        // TODO - this should control the timing.
        // website uses websockets (socket.io kind) for synchronisation
        // rust_socketio kinda blows, tungstenite-tokio or actix-web-actors
        // seem to be better but i would have to implement socketio myself using them
        // server sends a message type packet, which (i think) indicates either
        // start and end of a match, or start and end of a transition (since
        // there's always a couple of them (usually 4, sometimes 3?) at once
        let mut last_match = None;
        loop {
            let data = self.poll_api().await;
            if let Ok(data) = data {
                let state = data.state;
                dbg!(&state);
                if state.finished()
                    && (last_match.is_none()
                        || (last_match.is_some() && last_match.as_ref() != Some(&state)))
                {
                    if state.player1 != "Team A" && state.player2 != "Team B" {
                        println!(
                            "Writing match {} vs {} -> {:#?}",
                            state.player1, state.player2, state.status
                        );
                        //update win/loss counts
                        let (winner, loser) = match state.status {
                            MatchStatus::PlayerOneWin => (&state.player1, &state.player2),
                            MatchStatus::PlayerTwoWin => (&state.player2, &state.player1),
                            _ => panic!("wrong state"),
                        };
                        query!("INSERT INTO players (name, wins) VALUES ($1, 1) ON CONFLICT (name) DO UPDATE SET wins = players.wins + 1",winner).execute(&self.pool).await.unwrap();
                        query!("INSERT INTO players (name, losses) VALUES ($1, 1) ON CONFLICT (name) DO UPDATE SET losses = players.losses + 1",loser).execute(&self.pool).await.unwrap();
                    }
                    last_match = Some(state);
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
pub mod api_test {
    use super::*;
    fn time() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    }
    #[tokio::test]
    async fn state() {
        let state_api = Data::state(&time());
        let response = reqwest::get(state_api).await.unwrap();
        let state = response.json::<State>().await.unwrap();
        panic!("{:#?}", state);
    }
}
