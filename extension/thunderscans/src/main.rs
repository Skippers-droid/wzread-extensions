mod thunderscans;

use jsonrpc_core::{IoHandler, Params, Value, BoxFuture, Error};
use jsonrpc_http_server::{ServerBuilder};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 2 && args[1] == "--rpc" {
        start_rpc_server().await;
        return;
    }

    eprintln!("Usage: thunderscans --rpc");
    std::process::exit(1);
}

async fn start_rpc_server() {
    let mut io = IoHandler::new();

    io.add_method("search", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let empty = String::new();
            let query = args.first().unwrap_or(&empty);
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::search(query, &user_agent).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("getPopular", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::get_popular(&user_agent).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("getLatest", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let page: usize = args.first().unwrap_or(&"1".to_string()).parse().unwrap_or(1);
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::get_latest(&user_agent, page).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("getFiltered", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let empty = String::new();
            let filter = args.get(0).unwrap_or(&empty);
            let page: usize = args.get(1).unwrap_or(&"1".to_string()).parse().unwrap_or(1);
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::get_filtered(&user_agent, filter, page).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("manga_info", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let empty = String::new();
            let identifier = args.first().unwrap_or(&empty);
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::manga_info(identifier, &user_agent).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("get_chapter_images", |params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            let args: Vec<String> = match params.parse() {
                Ok(a) => a,
                Err(e) => return Err(Error::invalid_params(e.to_string())),
            };
            let empty = String::new();
            let book_id = args.get(0).unwrap_or(&empty);
            let chapter = args.get(1).unwrap_or(&empty);
            let page: usize = args.get(2).unwrap_or(&"1".to_string()).parse().unwrap_or(1);
            let per_page: usize = args.get(3).unwrap_or(&"5".to_string()).parse().unwrap_or(5);
            let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
            });
            
            match thunderscans::ThunderScans::get_chapter_images(book_id, chapter, &user_agent, page, per_page).await {
                Ok(data) => Ok(serde_json::to_value(&data).unwrap()),
                Err(_e) => Err(Error::internal_error()),
            }
        })
    });

    io.add_method("extension_info", |_params: Params| -> BoxFuture<Result<Value, Error>> {
        Box::pin(async move {
            Ok(serde_json::to_value(thunderscans::ThunderScans::extension_info()).unwrap())
        })
    });

    let server = ServerBuilder::new(io)
        .threads(4)
        .start_http(&"127.0.0.1:0".parse().unwrap())
        .expect("Failed to start RPC server");

    let port = server.address().port();
    println!("RPC_PORT={}", port);
    
    server.wait();
}