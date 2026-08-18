use serde::{Deserialize, Serialize};

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

    async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str, user_agent: &str) -> Result<T, String> {
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

        response.json::<T>()
            .await
            .map_err(|e| e.to_string())
    }

    fn extract_items(html: &str) -> Vec<SearchResult> {
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

            results.push(SearchResult {
                id: id.clone(),
                slug: id,
                title: if title_text.is_empty() { title } else { title_text },
                cover,
                status: status_text,
                chapter: None,
                chapter_time: None,
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
                chapter: None,
                chapter_time: None,
            });
        }

        results
    }

    fn extract_manga_info(html: &str, identifier: &str) -> MangaInfo {
        let title_re = regex::Regex::new(r#"<h1[^>]*>([^<]+)</h1>"#).unwrap();
        let title = title_re.captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_else(|| identifier.to_string());

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
        let mut images = Vec::new();
        let img_re = regex::Regex::new(r#"<img[^>]*src="([^"]+)"[^>]*alt="Chapter[^"]*"[^>]*>"#).unwrap();

        for cap in img_re.captures_iter(html) {
            let src = cap[1].to_string();
            if src.contains("mg.mgread.io") && !src.contains("avatar") && !src.contains("logo") {
                images.push(src);
            }
        }

        images
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        let url = format!("https://mgread.io/?s={}", urlencoding::encode(query));
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let results = Self::extract_items(&html);
                let total = results.len();
                Ok(SearchResponse {
                    data: results,
                    total,
                    page: 1,
                    per_page: total,
                    has_more: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_latest(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
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
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_filtered(user_agent: &str, filter: &str, page: usize) -> Result<SearchResponse, String> {
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
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_by_genre(user_agent: &str, genre: &str, page: usize) -> Result<SearchResponse, String> {
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
                Ok(SearchResponse {
                    data: items,
                    total,
                    page,
                    per_page: 12,
                    has_more,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_popular(user_agent: &str, range: &str) -> Result<Vec<TopRankingItem>, String> {
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

        Ok(results)
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        let url = format!("https://mgread.io/manga/{}/", identifier);
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => Ok(Self::extract_manga_info(&html, identifier)),
            Err(e) => Err(e),
        }
    }

    pub async fn get_chapter_images(
        book_id: &str,
        chapter: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
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

                Ok(ChapterImages {
                    images: paginated,
                    total,
                    page,
                    per_page,
                    has_more: end < total,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn extension_info() -> ExtensionInfo {
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
    if args.len() < 2 {
        println!("Usage: mgread <method> [args]");
        return;
    }

    let method = &args[1];
    let user_agent = std::env::var("USER_AGENT").unwrap_or_else(|_| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());

    match method.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: mgread search <query>");
                std::process::exit(1);
            }
            let query = &args[2];
            match Mgread::search(query, &user_agent).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getLatest" => {
            let page: usize = args.get(2).map(|p| p.parse().unwrap_or(1)).unwrap_or(1);
            match Mgread::get_latest(&user_agent, page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getFiltered" => {
            if args.len() < 4 {
                eprintln!("Usage: mgread getFiltered <filter_params> <page>");
                std::process::exit(1);
            }
            let filter = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            match Mgread::get_filtered(&user_agent, filter, page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getByGenre" => {
            if args.len() < 4 {
                eprintln!("Usage: mgread getByGenre <genre> <page>");
                std::process::exit(1);
            }
            let genre = &args[2];
            let page: usize = args[3].parse().unwrap_or(1);
            match Mgread::get_by_genre(&user_agent, genre, page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "getPopular" => {
            let range = args.get(2).map(|r| r.as_str()).unwrap_or("day");
            match Mgread::get_popular(&user_agent, range).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "manga_info" => {
            if args.len() < 3 {
                eprintln!("Usage: mgread manga_info <identifier>");
                std::process::exit(1);
            }
            let identifier = &args[2];
            match Mgread::manga_info(identifier, &user_agent).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "get_chapter_images" => {
            if args.len() < 6 {
                eprintln!("Usage: mgread get_chapter_images <book_id> <chapter> <page> <per_page>");
                std::process::exit(1);
            }
            let book_id = &args[2];
            let chapter = &args[3];
            let page: usize = args[4].parse().unwrap_or(1);
            let per_page: usize = args[5].parse().unwrap_or(5);
            match Mgread::get_chapter_images(book_id, chapter, &user_agent, page, per_page).await {
                Ok(result) => println!("{}", serde_json::to_string(&result).unwrap()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "extension_info" => {
            let info = Mgread::extension_info();
            println!("{}", serde_json::to_string(&info).unwrap());
        }
        _ => {
            eprintln!("Unknown method: {}", method);
            std::process::exit(1);
        }
    }
}