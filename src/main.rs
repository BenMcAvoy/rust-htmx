use std::{path::PathBuf, process::exit, time::Duration};

use poem::{endpoint::StaticFilesEndpoint, listener::TcpListener, Route};
use poem_openapi::{
    param::Query,
    payload::{Html, PlainText},
    OpenApi, OpenApiService,
};

use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use handlebars::Handlebars;
use serde_json::json;

use serde::{Deserialize, Serialize};

use kv_log_macro::*;

const IP: &str = "0.0.0.0";
const PORT: &str = "3000";

const HTML: &str = include_str!("static/index.html");

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    username: String,
    password: String,
    port: u16,
    host: String,
    name: String,
}

struct Api {
    handlebars: Handlebars<'static>,
    pool: Pool<MySql>,
}

#[derive(Serialize, Deserialize)]
struct Film {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Data {
    films: Vec<Film>,
}

#[OpenApi]
impl Api {
    #[oai(path = "/api/hello", method = "get")]
    async fn hello(&self, name: Query<Option<String>>) -> PlainText<String> {
        info!("Request at \"/api/hello\"");

        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }

    #[oai(path = "/", method = "get")]
    async fn index(&self) -> Html<String> {
        info!("Request at \"/\"");

        // let json = serde_json::to_string(data).unwrap();
        let json = json!({
            "films": [
                "Film1",
                "Film2",
                "Film3",
            ]
        });

        let response = self.handlebars.render_template(HTML, &json);

        match response {
            Ok(v) => Html(v),
            Err(e) => Html(e.to_string()),
        }
    }

    pub async fn new() -> Self {
        let cfg: Config = confy::load_path(PathBuf::from("./config.toml")).unwrap();

        info!("Creating DB pool");

        let address = format!(
            "mysql://{}:{}@{}/{}",
            cfg.username, cfg.password, cfg.host, cfg.name
        );

        if cfg.username.is_empty()
            || cfg.password.is_empty()
            || cfg.host.is_empty()
            || cfg.name.is_empty()
        {
            error!("Not all values in `config.toml` are filled in!");
            exit(-1);
        }

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::new(5, 0))
            .connect(&address)
            .await;

        let pool = match pool {
            Ok(v) => v,
            Err(e) => {
                error!("Pool error: {e}");
                exit(-1);
            }
        };

        Self {
            handlebars: Handlebars::default(),
            pool,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(femme::LevelFilter::Info);

    let listen_address = format!("{IP}:{PORT}");

    let api_service = OpenApiService::new(Api::new().await, "Rust & HTMX", "1.0")
        .server("http://localhost:3000/");

    let ui = api_service.swagger_ui();

    let static_endpoint = StaticFilesEndpoint::new("src/static/");

    let app = Route::new()
        .nest("/static", static_endpoint)
        .nest("/", api_service)
        .nest("/swagger", ui);

    poem::Server::new(TcpListener::bind(listen_address))
        .run(app)
        .await
}
