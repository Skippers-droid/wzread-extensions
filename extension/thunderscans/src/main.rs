mod thunderscans;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: thunderscan <method> [args]");
        std::process::exit(1);
    }

    let method = &args[1];
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
    });

    match method.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: thunderscan search <query>");
                std::process::exit(1);
            }
            let query = &args[2];
            match thunderscans::ThunderScans::search(query, &user_agent).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "manga_info" => {
            if args.len() < 3 {
                eprintln!("Usage: thunderscan manga_info <identifier>");
                std::process::exit(1);
            }
            let identifier = &args[2];
            match thunderscans::ThunderScans::manga_info(identifier, &user_agent).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "get_chapter_images" => {
            if args.len() < 6 {
                eprintln!("Usage: thunderscan get_chapter_images <book_id> <chapter> <page> <per_page>");
                std::process::exit(1);
            }
            let book_id = &args[2];
            let chapter = &args[3];
            let page: usize = args[4].parse().unwrap_or(1);
            let per_page: usize = args[5].parse().unwrap_or(5);
            match thunderscans::ThunderScans::get_chapter_images(book_id, chapter, &user_agent, page, per_page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getPopular" => {
            match thunderscans::ThunderScans::get_popular(&user_agent).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getLatest" => {
            if args.len() < 3 {
                eprintln!("Usage: thunderscan getLatest <page>");
                std::process::exit(1);
            }
            let page: usize = args[2].parse().unwrap_or(1);
            match thunderscans::ThunderScans::get_latest(&user_agent, page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getFiltered" => {
            if args.len() < 4 {
                eprintln!("Usage: thunderscan getFiltered <filter_params> <page>");
                std::process::exit(1);
            }
            let filter = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            match thunderscans::ThunderScans::get_filtered(&user_agent, filter, page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "extension_info" => {
            let info = thunderscans::ThunderScans::extension_info();
            println!("{}", serde_json::to_string(&info).unwrap());
        }
        _ => {
            eprintln!("Unknown method: {}", method);
            std::process::exit(1);
        }
    }
}