use poem::{listener::TcpListener, Route, endpoint::StaticFilesEndpoint};
use poem_openapi::{param::Query, payload::{PlainText, Html}, OpenApi, OpenApiService};
use kv_log_macro::*;

const IP: &str = "0.0.0.0";
const PORT: &str = "3000";

struct Api;

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

        Html("<h1>Film list</h1>".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(femme::LevelFilter::Info);

    let listen_address = format!("{IP}:{PORT}");

    info!("Listening info: http://{listen_address}");

    let api_service =
        OpenApiService::new(Api, "Rust & HTMX", "1.0").server("http://localhost:3000/");

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
