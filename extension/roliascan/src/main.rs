mod roliascan;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: roliascan <method> [args]");
        std::process::exit(1);
    }

    let method = &args[1];
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
    });

    match method.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Error: search requires a query parameter");
                std::process::exit(1);
            }
            let query = &args[2];
            match roliascan::Roliascan::search(query, &user_agent).await {
                Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getLatest" => {
            let page: usize = args.get(2).map(|p| p.parse().unwrap_or(1)).unwrap_or(1);
            match roliascan::Roliascan::get_latest(&user_agent, page).await {
                Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getPopular" => {
            let page: usize = args.get(2).map(|p| p.parse().unwrap_or(1)).unwrap_or(1);
            match roliascan::Roliascan::get_popular(&user_agent, page).await {
                Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "manga_info" => {
            if args.len() < 3 {
                eprintln!("Error: manga_info requires an identifier (slug)");
                std::process::exit(1);
            }
            let identifier = &args[2];
            match roliascan::Roliascan::manga_info(identifier, &user_agent).await {
                Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "get_chapter_images" => {
            if args.len() < 4 {
                eprintln!("Error: get_chapter_images requires chapter_id, page, and per_page");
                std::process::exit(1);
            }
            let chapter_id = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            let per_page: usize = args[4].parse().unwrap_or(5);
            match roliascan::Roliascan::get_chapter_images(chapter_id, &user_agent, page, per_page).await {
                Ok(data) => println!("{}", serde_json::to_string(&data).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "extension_info" => {
            let info = roliascan::Roliascan::extension_info();
            println!("{}", serde_json::to_string(&info).unwrap());
        }
        _ => {
            eprintln!("Unknown method: {}", method);
            std::process::exit(1);
        }
    }
}