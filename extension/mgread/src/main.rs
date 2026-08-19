use serde::{Deserialize, Serialize};

macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        eprintln!("[mgread] {}", msg);
    }};
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub cover: String,
    pub status: String,
    pub chapter: Option<String>,
    pub chapter_time: Option<String>,
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
    pub views: String,
    pub rating: String,
    pub chapter_count: usize,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopRankingItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub cover: String,
    pub views: String,
    pub rank: usize,
}

pub struct Mgread;

impl Mgread {
    async fn fetch_html(url: &str, user_agent: &str) -> Result<String, String> {
        log_debug!("fetch_html: url={}", url);
        
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                log_debug!("Client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| {
                log_debug!("Request error: {}", e);
                e.to_string()
            })?;

        log_debug!("fetch_html: status={}", response.status());

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        let text = response.text()
            .await
            .map_err(|e| {
                log_debug!("Read text error: {}", e);
                e.to_string()
            })?;
        
        log_debug!("fetch_html success: {} bytes", text.len());
        Ok(text)
    }

    async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str, user_agent: &str) -> Result<T, String> {
        log_debug!("fetch_json: url={}", url);
        
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                log_debug!("Client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| {
                log_debug!("Request error: {}", e);
                e.to_string()
            })?;

        log_debug!("fetch_json: status={}", response.status());

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        let json = response.json::<T>()
            .await
            .map_err(|e| {
                log_debug!("JSON parse error: {}", e);
                e.to_string()
            })?;
        
        log_debug!("fetch_json success");
        Ok(json)
    }

    fn extract_items(html: &str) -> Vec<SearchResult> {
        log_debug!("extract_items: parsing {} bytes", html.len());
        let mut results = Vec::new();
        
        let re = regex::Regex::new(
            r#"<div class="bs(?: styletere 5)?"[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>"#
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

            let result = SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
                chapter: None,
                chapter_time: None,
            };
            
            log_debug!("extract_items: found item id={}, title={}", result.id, result.title);
            results.push(result);
        }

        log_debug!("extract_items: found {} items", results.len());
        results
    }

    fn extract_latest_items(html: &str) -> Vec<SearchResult> {
        log_debug!("extract_latest_items: parsing {} bytes", html.len());
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

            let result = SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
                chapter: None,
                chapter_time: None,
            };
            
            log_debug!("extract_latest_items: found item id={}, title={}", result.id, result.title);
            results.push(result);
        }

        log_debug!("extract_latest_items: found {} items", results.len());
        results
    }

    fn extract_manga_info(html: &str, identifier: &str) -> MangaInfo {
        log_debug!("extract_manga_info: identifier={}, html size={}", identifier, html.len());
        
        let title_re = regex::Regex::new(r#"<h1[^>]*>([^<]+)</h1>"#).unwrap();
        let title = title_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_else(|| {
                log_debug!("extract_manga_info: no title found, using identifier");
                identifier.to_string()
            });

        let cover_re = regex::Regex::new(r#"<img[^>]*class="[^"]*image-3-4[^"]*"[^>]*src="([^"]+)"[^>]*>"#).unwrap();
        let cover = cover_re.captures(html)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();

        let desc_re = regex::Regex::new(r#"<div id="manga-description"[^>]*>[\s\S]*?<p>([\s\S]*?)</p>"#).unwrap();
        let description = desc_re.captures(html)
            .map(|cap| {
                let desc = cap[1].trim().to_string();
                regex::Regex::new(r"<[^>]*>").unwrap().replace_all(&desc, "").to_string()
            })
            .unwrap_or_default();

        let status_re = regex::Regex::new(r#"id="manga-status"[^>]*>([^<]+)</span>"#).unwrap();
        let status = status_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let views_re = regex::Regex::new(r#"<span[^>]*data-view="[^"]*"[^>]*data-id="[^"]*"[^>]*>([^<]+)</span>"#).unwrap();
        let views = views_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let rating_re = regex::Regex::new(r#"<strong>([\d.]+)</strong><sub>/5</sub>"#).unwrap();
        let rating = rating_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let chapter_count_re = regex::Regex::new(r#"<span[^>]*uk-icon="icon: file-text"[^>]*></span>[\s]*(\d+)[\s]*<span[^>]*>Chapters</span>"#).unwrap();
        let chapter_count = chapter_count_re.captures(html)
            .map(|cap| cap[1].parse::<usize>().unwrap_or(0))
            .unwrap_or(0);

        let alt_re = regex::Regex::new(r#"Alternate Title: <span[^>]*id="comic-othername"[^>]*>([^<]+)</span>"#).unwrap();
        let alt_title = alt_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let mut genres = Vec::new();
        let genre_re = regex::Regex::new(r#"href="https://mgread.io/genre/([^/]+)/"[^>]*>[\s\S]*?<span uk-icon="icon: hashtag"></span>([^<]+)</a>"#).unwrap();
        for cap in genre_re.captures_iter(html) {
            genres.push(cap[2].trim().to_string());
        }

        let mut chapters = Vec::new();
        let chapter_re = regex::Regex::new(
            r#"<a[^>]*href="([^"]+)"[^>]*>[\s\S]*?<h3[^>]*>([^<]+)</h3>[\s\S]*?<time[^>]*datetime="([^"]*)"[^>]*>([^<]+)</time>"#
        ).unwrap();

        for cap in chapter_re.captures_iter(html) {
            let chapter_url = cap[1].to_string();
            let chapter_title = cap[2].trim().to_string();
            let chapter_date = cap[4].trim().to_string();

            let chapter_number = if let Some(cap) = regex::Regex::new(r"Chapter\s*(\d+)").unwrap().captures(&chapter_title) {
                cap[1].parse::<usize>().unwrap_or(0)
            } else {
                0
            };

            let chapter_slug = if let Some(cap) = regex::Regex::new(r"/chapter-(\d+)/").unwrap().captures(&chapter_url) {
                format!("chapter-{}", &cap[1])
            } else {
                format!("chapter-{}", chapter_number)
            };

            chapters.push(ChapterInfo {
                number: chapter_number,
                slug: chapter_slug,
                title: chapter_title,
                date: chapter_date,
                url: chapter_url,
            });
        }

        chapters.sort_by(|a, b| a.number.cmp(&b.number));

        log_debug!("extract_manga_info: title={}, chapters={}", title, chapters.len());

        MangaInfo {
            id: identifier.to_string(),
            slug: identifier.to_string(),
            title,
            alt_title,
            description,
            cover,
            author: String::new(),
            status,
            r#type: String::new(),
            genres,
            chapters,
            views,
            rating,
            chapter_count,
        }
    }

    fn extract_chapter_images(html: &str) -> Vec<String> {
        log_debug!("extract_chapter_images: parsing {} bytes", html.len());
        let mut images = Vec::new();
        let img_re = regex::Regex::new(r#"<img[^>]*src="([^"]+)"[^>]*alt="Chapter[^"]*"[^>]*>"#).unwrap();

        for cap in img_re.captures_iter(html) {
            let src = cap[1].to_string();
            if src.contains("mg.mgread.io") && !src.contains("avatar") && !src.contains("logo") {
                log_debug!("extract_chapter_images: found image {}", src);
                images.push(src);
            }
        }

        log_debug!("extract_chapter_images: found {} images", images.len());
        images
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        log_debug!("search: query={}", query);
        let url = format!("https://mgread.io/?s={}", urlencoding::encode(query));
        
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let results = Self::extract_items(&html);
                let total = results.len();
                log_debug!("search: found {} results", total);
                Ok(SearchResponse {
                    data: results,
                    total,
                    page: 1,
                    per_page: total,
                    has_more: false,
                })
            }
            Err(e) => {
                log_debug!("search error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_latest(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        log_debug!("get_latest: page={}", page);
        let url = if page == 1 {
            "https://mgread.io/recently-updated/".to_string()
        } else {
            format!("https://mgread.io/recently-updated/page/{}/", page)
        };

        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let items = Self::extract_latest_items(&html);
                let total = items.len();
                let has_more = items.len() == 12;
                log_debug!("get_latest: found {} items, has_more={}", total, has_more);
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => {
                log_debug!("get_latest error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_filtered(user_agent: &str, filter: &str, page: usize) -> Result<SearchResponse, String> {
        log_debug!("get_filtered: filter={}, page={}", filter, page);
        let url = if page == 1 {
            format!("https://mgread.io/advanced-filter/?{}", filter)
        } else {
            format!("https://mgread.io/advanced-filter/page/{}/?{}", page, filter)
        };

        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let items = Self::extract_items(&html);
                let total = items.len();
                let has_more = items.len() == 12;
                log_debug!("get_filtered: found {} items", total);
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => {
                log_debug!("get_filtered error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_by_genre(user_agent: &str, genre: &str, page: usize) -> Result<SearchResponse, String> {
        log_debug!("get_by_genre: genre={}, page={}", genre, page);
        let url = if page == 1 {
            format!("https://mgread.io/genre/{}/", genre)
        } else {
            format!("https://mgread.io/genre/{}/page/{}/", genre, page)
        };

        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let items = Self::extract_items(&html);
                let total = items.len();
                let has_more = items.len() == 12;
                log_debug!("get_by_genre: found {} items", total);
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => {
                log_debug!("get_by_genre error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_popular(user_agent: &str, range: &str) -> Result<Vec<TopRankingItem>, String> {
        log_debug!("get_popular: range={}", range);
        let url = format!("https://mgread.io/wp-json/initmanga/v1/top-ranking?range={}", range);
        
        #[derive(Debug, Deserialize)]
        struct TopResponse {
            success: bool,
            posts: Vec<TopPost>,
        }

        #[derive(Debug, Deserialize)]
        struct TopPost {
            id: String,
            html: String,
        }

        let response: TopResponse = Self::fetch_json(&url, user_agent).await?;
        
        if !response.success {
            log_debug!("get_popular: API returned success=false");
            return Err("Failed to fetch top ranking".to_string());
        }

        let mut results = Vec::new();
        let re = regex::Regex::new(
            r#"<a href="https://mgread.io/manga/([^/]+)/"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<a[^>]*href="https://mgread.io/manga/[^/]+/"[^>]*>([^<]+)</a>"#
        ).unwrap();

        for (index, post) in response.posts.iter().enumerate() {
            if let Some(cap) = re.captures(&post.html) {
                let slug = cap[1].to_string();
                let cover = cap[2].to_string();
                let title = cap[3].trim().to_string();

                let views_re = regex::Regex::new(r#"<span[^>]*>([\d.]+)\s*K?</span>"#).unwrap();
                let views = if let Some(views_cap) = views_re.captures(&post.html) {
                    views_cap[1].to_string()
                } else {
                    "0".to_string()
                };

                results.push(TopRankingItem {
                    id: post.id.clone(),
                    slug,
                    title,
                    cover,
                    views,
                    rank: index + 1,
                });
            }
        }

        log_debug!("get_popular: found {} items", results.len());
        Ok(results)
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        log_debug!("manga_info: identifier={}", identifier);
        let url = format!("https://mgread.io/manga/{}/", identifier);
        
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let info = Self::extract_manga_info(&html, identifier);
                log_debug!("manga_info: success for {}", identifier);
                Ok(info)
            }
            Err(e) => {
                log_debug!("manga_info error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_chapter_images(
        book_id: &str,
        chapter: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
        log_debug!("get_chapter_images: book_id={}, chapter={}, page={}, per_page={}", book_id, chapter, page, per_page);
        let url = format!("https://mgread.io/manga/{}/chapter-{}/", book_id, chapter);
        
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let all_images = Self::extract_chapter_images(&html);
                let total = all_images.len();
                let start = (page - 1) * per_page;
                let end = std::cmp::min(start + per_page, total);
                let paginated = if start < total {
                    all_images[start..end].to_vec()
                } else {
                    Vec::new()
                };

                log_debug!("get_chapter_images: total={}, returning {} images", total, paginated.len());

                Ok(ChapterImages {
                    images: paginated,
                    total,
                    page,
                    per_page,
                    has_more: end < total,
                })
            }
            Err(e) => {
                log_debug!("get_chapter_images error: {}", e);
                Err(e)
            }
        }
    }

    pub fn extension_info() -> ExtensionInfo {
        log_debug!("extension_info called");
        ExtensionInfo {
            id: "mgread".to_string(),
            name: "Mgread.io".to_string(),
            version: "1.0.0".to_string(),
            description: "Read Free Manga – Manhwa – Manhua – Anime Online".to_string(),
            author: "wzread".to_string(),
            cover: "./extension_cover.png".to_string(),
            icon: "./extension_icon.png".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    log_debug!("main: args={:?}", args);
    
    if args.len() < 2 {
        eprintln!("Usage: mgread <method> [args]");
        log_debug!("main: no method provided");
        return;
    }

    let method = &args[1];
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string();
        log_debug!("main: using default user agent");
        ua
    });
    
    log_debug!("main: method={}", method);

    match method.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: mgread search <query>");
                log_debug!("main: search missing query");
                std::process::exit(1);
            }
            let query = &args[2];
            log_debug!("main: search query={}", query);
            match Mgread::search(query, &user_agent).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: search success, items={}", result.data.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: search error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getLatest" => {
            let page: usize = args.get(2).map(|p| p.parse().unwrap_or(1)).unwrap_or(1);
            log_debug!("main: getLatest page={}", page);
            match Mgread::get_latest(&user_agent, page).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: getLatest success, items={}", result.data.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: getLatest error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getFiltered" => {
            if args.len() < 4 {
                eprintln!("Usage: mgread getFiltered <filter_params> <page>");
                log_debug!("main: getFiltered missing params");
                std::process::exit(1);
            }
            let filter = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            log_debug!("main: getFiltered filter={}, page={}", filter, page);
            match Mgread::get_filtered(&user_agent, filter, page).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: getFiltered success, items={}", result.data.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: getFiltered error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getByGenre" => {
            if args.len() < 4 {
                eprintln!("Usage: mgread getByGenre <genre> <page>");
                log_debug!("main: getByGenre missing params");
                std::process::exit(1);
            }
            let genre = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            log_debug!("main: getByGenre genre={}, page={}", genre, page);
            match Mgread::get_by_genre(&user_agent, genre, page).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: getByGenre success, items={}", result.data.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: getByGenre error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getPopular" => {
            let range = args.get(2).map(|r| r.as_str()).unwrap_or("day");
            log_debug!("main: getPopular range={}", range);
            match Mgread::get_popular(&user_agent, range).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: getPopular success, items={}", result.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: getPopular error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "manga_info" => {
            if args.len() < 3 {
                eprintln!("Usage: mgread manga_info <identifier>");
                log_debug!("main: manga_info missing identifier");
                std::process::exit(1);
            }
            let identifier = &args[2];
            log_debug!("main: manga_info identifier={}", identifier);
            match Mgread::manga_info(identifier, &user_agent).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: manga_info success, title={}", result.title);
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: manga_info error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "get_chapter_images" => {
            if args.len() < 6 {
                eprintln!("Usage: mgread get_chapter_images <book_id> <chapter> <page> <per_page>");
                log_debug!("main: get_chapter_images missing params");
                std::process::exit(1);
            }
            let book_id = &args[2];
            let chapter = &args[3];
            let page: usize = args[4].parse().unwrap_or(1);
            let per_page: usize = args[5].parse().unwrap_or(5);
            log_debug!("main: get_chapter_images book_id={}, chapter={}, page={}, per_page={}", book_id, chapter, page, per_page);
            match Mgread::get_chapter_images(book_id, chapter, &user_agent, page, per_page).await {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap();
                    log_debug!("main: get_chapter_images success, images={}", result.images.len());
                    println!("{}", json);
                }
                Err(e) => {
                    log_debug!("main: get_chapter_images error={}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "extension_info" => {
            log_debug!("main: extension_info");
            let info = Mgread::extension_info();
            let json = serde_json::to_string(&info).unwrap();
            log_debug!("main: extension_info success");
            println!("{}", json);
        }
        _ => {
            log_debug!("main: unknown method={}", method);
            eprintln!("Unknown method: {}", method);
            std::process::exit(1);
        }
    }
}