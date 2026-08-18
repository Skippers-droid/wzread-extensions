// extension/thunderscans/src/main.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub cover: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub data: Vec<SearchResult>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterInfo {
    pub number: usize,
    pub slug: String,
    pub title: String,
    pub date: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MangaInfo {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub alt_title: String,
    pub description: String,
    pub cover: String,
    pub author: String,
    pub status: String,
    pub r#type: String,
    pub genres: Vec<String>,
    pub chapters: Vec<ChapterInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterImages {
    pub images: Vec<String>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub cover: String,
    pub icon: String,
}

pub struct ThunderScans;

impl ThunderScans {
    async fn fetch_html(url: &str, user_agent: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        response.text()
            .await
            .map_err(|e| e.to_string())
    }

    async fn fetch_ajax(url: &str, user_agent: &str, page: usize) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let page_str = page.to_string();
        let mut params = HashMap::new();
        params.insert("action", "load_more_manga_posts");
        params.insert("page", &page_str);

        let response = client.post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        response.text()
            .await
            .map_err(|e| e.to_string())
    }

    fn extract_search_results(html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let re = regex::Regex::new(
            r#"<div class="bs">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>"#
        ).unwrap();

        for cap in re.captures_iter(html) {
            let url = cap[1].to_string();
            let title = cap[2].to_string();
            let cover = cap[3].to_string();
            let title_text = cap[4].trim().to_string();
            let status_text = cap[6].trim().to_string();

            let id = if let Some(cap) = regex::Regex::new(r"/comics/([^/]+)/?").unwrap().captures(&url) {
                cap[1].to_string()
            } else {
                String::new()
            };

            results.push(SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
            });
        }

        results
    }

    fn extract_latest_items(html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let re = regex::Regex::new(
            r#"<div class="bs styletere 5">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>"#
        ).unwrap();

        for cap in re.captures_iter(html) {
            let url = cap[1].to_string();
            let title = cap[2].to_string();
            let cover = cap[3].to_string();
            let title_text = cap[4].trim().to_string();
            let status_text = cap[6].trim().to_string();

            let id = if let Some(cap) = regex::Regex::new(r"/comics/([^/]+)/?").unwrap().captures(&url) {
                cap[1].to_string()
            } else {
                String::new()
            };

            results.push(SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
            });
        }

        results
    }

    fn extract_manga_info(html: &str, identifier: &str) -> MangaInfo {
        let title_re = regex::Regex::new(r#"<h1 class="entry-title"[^>]*>([^<]+)</h1>"#).unwrap();
        let title = title_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_else(|| identifier.to_string());

        let alt_re = regex::Regex::new(r#"<div class="alternative">[\s\S]*?<div class="desktop-titles">([^<]+)</div>"#).unwrap();
        let alt_title = alt_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let cover_re = regex::Regex::new(r#"<div class="thumb"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>"#).unwrap();
        let cover = cover_re.captures(html)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();

        let desc_re = regex::Regex::new(r#"<div class="entry-content entry-content-single"[^>]*>[\s\S]*?<p>([\s\S]*?)</p>"#).unwrap();
        let description = desc_re.captures(html)
            .map(|cap| {
                let desc = cap[1].trim().to_string();
                regex::Regex::new(r"<[^>]*>").unwrap().replace_all(&desc, "").to_string()
            })
            .unwrap_or_default();

        let type_re = regex::Regex::new(r#"<div class="imptdt">[\s\S]*?<h1>\s*Type\s*</h1>[\s\S]*?<i>([^<]+)</i>"#).unwrap();
        let r#type = type_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let status_re = regex::Regex::new(r#"<div class="imptdt">[\s\S]*?<h1>\s*Status\s*</h1>[\s\S]*?<i>([^<]+)</i>"#).unwrap();
        let status = status_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let author_re = regex::Regex::new(r#"<div class="imptdt">[\s\S]*?<h1>\s*Author\s*</h1>[\s\S]*?<i>([^<]+)</i>"#).unwrap();
        let author = author_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let mut genres = Vec::new();
        let genre_re = regex::Regex::new(r#"<span class="mgen">[\s\S]*?<a href="[^"]*"[^>]*>([^<]+)</a>"#).unwrap();
        for cap in genre_re.captures_iter(html) {
            genres.push(cap[1].trim().to_string());
        }

        let mut chapters = Vec::new();
        let chapter_re = regex::Regex::new(
            r#"<li data-num="(\d+)">[\s\S]*?<a[^>]*href="([^"]+)"[^>]*>[\s\S]*?<span class="chapternum">[\s\S]*?Chapter\s*(\d+)</span>[\s\S]*?<span class="chapterdate">([^<]+)</span>"#
        ).unwrap();

        for cap in chapter_re.captures_iter(html) {
            let chapter_url = cap[2].to_string();
            let chapter_number: usize = cap[3].parse().unwrap_or(0);
            let chapter_date = cap[4].trim().to_string();

            let chapter_slug = if let Some(cap) = regex::Regex::new(r"/[^/]+-chapter-(\d+)/?").unwrap().captures(&chapter_url) {
                format!("chapter-{}", &cap[1])
            } else {
                format!("chapter-{}", chapter_number)
            };

            chapters.push(ChapterInfo {
                number: chapter_number,
                slug: chapter_slug,
                title: format!("Chapter {}", chapter_number),
                date: chapter_date,
                url: chapter_url,
            });
        }

        chapters.sort_by(|a, b| a.number.cmp(&b.number));

        MangaInfo {
            id: identifier.to_string(),
            slug: identifier.to_string(),
            title,
            alt_title,
            description,
            cover,
            author,
            status,
            r#type,
            genres,
            chapters,
        }
    }

    fn extract_images(html: &str) -> Vec<String> {
        let mut images = Vec::new();

        let image_re = regex::Regex::new(r#""images"\s*:\s*\[([\s\S]*?)\]"#).unwrap();
        if let Some(cap) = image_re.captures(html) {
            let image_array = &cap[1];
            let img_re = regex::Regex::new(r#""([^"]+\.(?:jpg|jpeg|png|webp|gif))""#).unwrap();
            for img_cap in img_re.captures_iter(image_array) {
                images.push(img_cap[1].to_string());
            }
        }

        if images.is_empty() {
            let alt_re = regex::Regex::new(r#"https://en-thunderscans\.com/wp-content/uploads/manga/[^"]+\.(?:jpg|jpeg|png|webp|gif)"#).unwrap();
            for cap in alt_re.find_iter(html) {
                images.push(cap.as_str().to_string());
            }
        }

        images
    }

    fn extract_popular_items(html: &str) -> Vec<SearchResult> {
        let mut items = Vec::new();
        
        let re = regex::Regex::new(
            r#"<div class="bs">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>"#
        ).unwrap();

        for cap in re.captures_iter(html) {
            let url = cap[1].to_string();
            let title = cap[2].to_string();
            let cover = cap[3].to_string();
            let title_text = cap[4].trim().to_string();
            let status_text = cap[6].trim().to_string();

            let id = if let Some(cap) = regex::Regex::new(r"/comics/([^/]+)/?").unwrap().captures(&url) {
                cap[1].to_string()
            } else {
                String::new()
            };

            items.push(SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
            });
        }

        items
    }

    fn extract_filtered_items(html: &str) -> Vec<SearchResult> {
        let mut items = Vec::new();
        
        let re = regex::Regex::new(
            r#"<div class="bs">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>"#
        ).unwrap();

        for cap in re.captures_iter(html) {
            let url = cap[1].to_string();
            let title = cap[2].to_string();
            let cover = cap[3].to_string();
            let title_text = cap[4].trim().to_string();
            let status_text = cap[6].trim().to_string();

            let id = if let Some(cap) = regex::Regex::new(r"/comics/([^/]+)/?").unwrap().captures(&url) {
                cap[1].to_string()
            } else {
                String::new()
            };

            items.push(SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
            });
        }

        items
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let search_query = urlencoding::encode(query);
            let url = format!("https://en-thunderscans.com/?s={}", search_query);

            match Self::fetch_html(&url, user_agent).await {
                Ok(html) => {
                    let results = Self::extract_search_results(&html);
                    let total = results.len();
                    return Ok(SearchResponse {
                        data: results,
                        total,
                        page: 1,
                        per_page: total,
                        has_more: false,
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let url = format!("https://en-thunderscans.com/comics/{}/", identifier);

            match Self::fetch_html(&url, user_agent).await {
                Ok(html) => {
                    return Ok(Self::extract_manga_info(&html, identifier));
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub async fn get_chapter_images(
        book_id: &str,
        chapter: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let url = format!("https://en-thunderscans.com/{}-chapter-{}/", book_id, chapter);

            match Self::fetch_html(&url, user_agent).await {
                Ok(html) => {
                    let all_images = Self::extract_images(&html);
                    let total = all_images.len();
                    let start = (page - 1) * per_page;
                    let end = std::cmp::min(start + per_page, total);
                    let paginated = if start < total {
                        all_images[start..end].to_vec()
                    } else {
                        Vec::new()
                    };

                    return Ok(ChapterImages {
                        images: paginated,
                        total,
                        page,
                        per_page,
                        has_more: end < total,
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub async fn get_popular(user_agent: &str) -> Result<SearchResponse, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let url = "https://en-thunderscans.com/";

            match Self::fetch_html(&url, user_agent).await {
                Ok(html) => {
                    let items = Self::extract_popular_items(&html);
                    let total = items.len();
                    return Ok(SearchResponse {
                        data: items,
                        total,
                        page: 1,
                        per_page: total,
                        has_more: false,
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub async fn get_latest(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let url = "https://en-thunderscans.com/wp-admin/admin-ajax.php";

            match Self::fetch_ajax(url, user_agent, page).await {
                Ok(html) => {
                    let items = Self::extract_latest_items(&html);
                    let total = items.len();
                    let has_more = items.len() == 12;
                    
                    return Ok(SearchResponse {
                        data: items,
                        total,
                        page,
                        per_page: 12,
                        has_more,
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub async fn get_filtered(user_agent: &str, filter: &str, page: usize) -> Result<SearchResponse, String> {
        let mut attempts = 0;
        while attempts < 3 {
            let url = format!("https://en-thunderscans.com/comics/page/{}/?{}", page, filter);

            match Self::fetch_html(&url, user_agent).await {
                Ok(html) => {
                    let items = Self::extract_filtered_items(&html);
                    let total = items.len();
                    let has_more = items.len() == 12;
                    
                    return Ok(SearchResponse {
                        data: items,
                        total,
                        page,
                        per_page: 12,
                        has_more,
                    });
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempts)).await;
                }
            }
        }
        Err("Max retries exceeded".to_string())
    }

    pub fn extension_info() -> ExtensionInfo {
        ExtensionInfo {
            id: "thunderscans".to_string(),
            name: "ThunderScans".to_string(),
            version: "1.0.0".to_string(),
            description: "ThunderScans EN extension - Read comics from ThunderScans".to_string(),
            author: "wzread".to_string(),
            cover: "./extension_cover.png".to_string(),
            icon: "./extension_icon.png".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: thunderscan <method> [args]");
        return;
    }

    let method = &args[1];
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());

    match method.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: thunderscan search <query>");
                std::process::exit(1);
            }
            let query = &args[2];
            match ThunderScans::search(query, &user_agent).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
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
            match ThunderScans::manga_info(identifier, &user_agent).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
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
            match ThunderScans::get_chapter_images(book_id, chapter, &user_agent, page, per_page).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getPopular" => {
            match ThunderScans::get_popular(&user_agent).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
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
            match ThunderScans::get_latest(&user_agent, page).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
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
            match ThunderScans::get_filtered(&user_agent, filter, page).await {
                Ok(result) => {
                    println!("{}", serde_json::to_string(&result).unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "extension_info" => {
            let info = ThunderScans::extension_info();
            println!("{}", serde_json::to_string(&info).unwrap());
        }
        _ => {
            eprintln!("Unknown method: {}", method);
            std::process::exit(1);
        }
    }
}