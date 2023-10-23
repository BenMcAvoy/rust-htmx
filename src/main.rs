use poem::{listener::TcpListener, Route, endpoint::StaticFilesEndpoint};
use poem_openapi::{param::Query, payload::{PlainText, Html}, OpenApi, OpenApiService};

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/api/hello", method = "get")]
    async fn hello(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }

    #[oai(path = "/", method = "get")]
    async fn index(&self) -> Html<String> {
        Html("<h1>Film list</h1>".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service =
        OpenApiService::new(Api, "Rust & HTMX", "1.0").server("http://localhost:3000/");

    let ui = api_service.swagger_ui();

    let static_endpoint = StaticFilesEndpoint::new("src/static/");

    let app = Route::new()
        .nest("/static", static_endpoint)
        .nest("/", api_service)
        .nest("/swagger", ui);

    poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
