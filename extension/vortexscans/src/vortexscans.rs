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

#[derive(Debug, Deserialize)]
struct SearchApiResponse {
    posts: Vec<SearchApiItem>,
}

#[derive(Debug, Deserialize)]
struct SearchApiItem {
    slug: String,
    post_title: String,
    featured_image: String,
    series_status: String,
    chapters: Vec<ChapterApiItem>,
}

#[derive(Debug, Deserialize)]
struct ChapterApiItem {
    number: usize,
    created_at: String,
}

pub struct VortexScans;

impl VortexScans {
    async fn fetch_html(url: &str, user_agent: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", "https://vortexscans.org/")
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
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Referer", "https://vortexscans.org/")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        let text = response.text()
            .await
            .map_err(|e| e.to_string())?;

        serde_json::from_str(&text)
            .map_err(|e| e.to_string())
    }

    fn extract_latest_items(html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        let item_re = regex::Regex::new(
            r#"<a href="/series/([^"]+)"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<a[^>]*href="/series/[^"]+"[^>]*>([^<]+)</a>[\s\S]*?<span[^>]*class="h-\[10px\] w-\[10px\] rounded-full inline-block relative bg-green-500"[^>]*>[\s\S]*?<p[^>]*>([^<]+)</p>[\s\S]*?<a[^>]*href="/series/[^"]+/chapter-(\d+)"[^>]*>[\s\S]*?<span>Chapter \d+</span>[\s\S]*?<time[^>]*>([^<]+)</time>"#
        ).unwrap();

        for cap in item_re.captures_iter(html) {
            let slug = cap[1].to_string();
            let cover = cap[2].to_string();
            let title = cap[3].trim().to_string();
            let status = cap[4].trim().to_string();
            let chapter = format!("Chapter {}", cap[5].to_string());
            let chapter_time = cap[6].trim().to_string();

            results.push(SearchResult {
                id: slug.clone(),
                slug,
                title,
                cover,
                status,
                chapter: Some(chapter),
                chapter_time: Some(chapter_time),
            });
        }

        results
    }

    fn extract_popular_items(html: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        let item_re = regex::Regex::new(
            r#"<a href="/series/([^"]+)"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div[^>]*class="[^"]*text-2xl[^"]*"[^>]*>(\d+)</div>[\s\S]*?<div[^>]*class="[^"]*font-bold[^"]*"[^>]*>([^<]+)</div>"#
        ).unwrap();

        for cap in item_re.captures_iter(html) {
            let slug = cap[1].to_string();
            let cover = cap[2].to_string();
            let title = cap[4].trim().to_string();

            results.push(SearchResult {
                id: slug.clone(),
                slug,
                title,
                cover,
                status: "Ongoing".to_string(),
                chapter: None,
                chapter_time: None,
            });
        }

        results
    }

    fn extract_title(html: &str, default: &str) -> String {
        let re = regex::Regex::new(r#"<meta[^>]*property="og:title"[^>]*content="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].trim().to_string();
        }
        let re2 = regex::Regex::new(r#"<h1[^>]*itemprop="name"[^>]*>([^<]+)</h1>"#).unwrap();
        if let Some(cap) = re2.captures(html) {
            return cap[1].trim().to_string();
        }
        default.to_string()
    }

    fn extract_cover(html: &str) -> String {
        let re = regex::Regex::new(r#"<img[^>]*class="[^"]*image-3-4[^"]*"[^>]*src="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].to_string();
        }
        let re2 = regex::Regex::new(r#"<meta[^>]*property="og:image"[^>]*content="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re2.captures(html) {
            return cap[1].to_string();
        }
        let re3 = regex::Regex::new(r#"<div[^>]*class="[^"]*cover[^"]*"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re3.captures(html) {
            return cap[1].to_string();
        }
        String::new()
    }

    fn extract_description(html: &str) -> String {
        let re = regex::Regex::new(r#"<meta[^>]*name="description"[^>]*content="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].trim().to_string();
        }
        let re2 = regex::Regex::new(r#"<div[^>]*itemprop="description"[^>]*>[\s\S]*?<p>([\s\S]*?)</p>"#).unwrap();
        if let Some(cap) = re2.captures(html) {
            let desc = cap[1].trim().to_string();
            return regex::Regex::new(r"<[^>]*>").unwrap().replace_all(&desc, "").to_string();
        }
        String::new()
    }

    fn extract_status(html: &str) -> String {
        let re = regex::Regex::new(r#"<span[^>]*class="h-\[10px\] w-\[10px\] rounded-full inline-block relative bg-green-500"[^>]*>[\s\S]*?<p[^>]*>([^<]+)</p>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].trim().to_string();
        }
        "Ongoing".to_string()
    }

    fn extract_views(html: &str) -> String {
        let re = regex::Regex::new(r#"<span[^>]*class="init-plugin-suite-view-count-number"[^>]*data-view="([^"]+)"[^>]*>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].to_string();
        }
        "0".to_string()
    }

    fn extract_rating(html: &str) -> String {
        let re = regex::Regex::new(r#"<div[^>]*class="numscore"[^>]*>([\d.]+)</div>"#).unwrap();
        if let Some(cap) = re.captures(html) {
            return cap[1].to_string();
        }
        "0.0".to_string()
    }

    fn extract_alt_titles(html: &str) -> String {
        let re = regex::Regex::new(r#"<div[^>]*class="text-sm[^"]*"[^>]*>([^<]+)</div>"#).unwrap();
        for cap in re.captures_iter(html) {
            let text = cap[1].trim();
            if text.contains('\n') {
                return text.to_string();
            }
        }
        String::new()
    }

    fn extract_genres(html: &str) -> Vec<String> {
        let mut genres = Vec::new();
        let re = regex::Regex::new(r#"<a[^>]*href="/genre/[^"]+"[^>]*>([^<]+)</a>"#).unwrap();
        for cap in re.captures_iter(html) {
            genres.push(cap[1].trim().to_string());
        }
        genres
    }

    fn extract_chapters(html: &str, identifier: &str) -> Vec<ChapterInfo> {
        let mut chapters = Vec::new();
        
        let re = regex::Regex::new(
            r#"<a[^>]*href="/series/[^"]+/chapter-(\d+(?:\.\d+)?)"[^>]*>[\s\S]*?<span[^>]*>Chapter\s*(\d+(?:\.\d+)?)</span>[\s\S]*?<time[^>]*datetime="([^"]*)"[^>]*>([^<]+)</time>"#
        ).unwrap();

        for cap in re.captures_iter(html) {
            let chapter_str = cap[2].to_string();
            let chapter_number: usize = chapter_str.parse().unwrap_or(0);
            let chapter_date = cap[4].trim().to_string();
            let chapter_slug = format!("chapter-{}", chapter_str);

            chapters.push(ChapterInfo {
                number: chapter_number,
                slug: chapter_slug,
                title: format!("Chapter {}", chapter_str),
                date: chapter_date,
                url: format!("/series/{}/chapter-{}", identifier, chapter_str),
            });
        }

        if chapters.is_empty() {
            let re2 = regex::Regex::new(
                r#"<a[^>]*href="/series/[^"]+/chapter-(\d+(?:\.\d+)?)"[^>]*>[\s\S]*?<span[^>]*>Chapter\s*(\d+(?:\.\d+)?)</span>"#
            ).unwrap();
            
            for cap in re2.captures_iter(html) {
                let chapter_str = cap[2].to_string();
                let chapter_number: usize = chapter_str.parse().unwrap_or(0);
                let chapter_slug = format!("chapter-{}", chapter_str);

                chapters.push(ChapterInfo {
                    number: chapter_number,
                    slug: chapter_slug,
                    title: format!("Chapter {}", chapter_str),
                    date: "".to_string(),
                    url: format!("/series/{}/chapter-{}", identifier, chapter_str),
                });
            }
        }

        chapters.sort_by(|a, b| a.number.cmp(&b.number));
        chapters
    }

    fn extract_manga_info(html: &str, identifier: &str) -> MangaInfo {
        let title = Self::extract_title(html, identifier);
        let cover = Self::extract_cover(html);
        let description = Self::extract_description(html);
        let status = Self::extract_status(html);
        let views = Self::extract_views(html);
        let rating = Self::extract_rating(html);
        let alt_title = Self::extract_alt_titles(html);
        let genres = Self::extract_genres(html);
        let chapters = Self::extract_chapters(html, identifier);
        let chapter_count = chapters.len();

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

        let re = regex::Regex::new(r#"<img[^>]*src="([^"]+)"[^>]*class="[^"]*object-cover[^"]*"[^>]*>"#).unwrap();

        for cap in re.captures_iter(html) {
            let src = cap[1].to_string();
            if !src.contains("avatar") && !src.contains("logo") && !src.contains("icon") {
                images.push(src);
            }
        }

        images
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        let url = format!("https://api.vortexscans.org/api/query?page=1&perPage=36&view=archive&searchTerm={}&orderBy=lastChapterAddedAt&orderDirection=desc", urlencoding::encode(query));
        
        let response: SearchApiResponse = Self::fetch_json(&url, user_agent).await?;
        
        let results: Vec<SearchResult> = response.posts.into_iter().map(|item| {
            let latest_chapter = item.chapters.first();
            SearchResult {
                id: item.slug.clone(),
                slug: item.slug,
                title: item.post_title,
                cover: item.featured_image,
                status: item.series_status,
                chapter: latest_chapter.map(|c| format!("Chapter {}", c.number)),
                chapter_time: latest_chapter.map(|c| {
                    let date = chrono::DateTime::parse_from_rfc3339(&c.created_at).unwrap_or_default();
                    let now = chrono::Utc::now();
                    let diff = now.signed_duration_since(date.with_timezone(&chrono::Utc));
                    
                    if diff.num_days() > 30 {
                        format!("{} months ago", diff.num_days() / 30)
                    } else if diff.num_days() > 7 {
                        format!("{} weeks ago", diff.num_days() / 7)
                    } else if diff.num_days() > 1 {
                        format!("{} days ago", diff.num_days())
                    } else if diff.num_hours() > 1 {
                        format!("{} hours ago", diff.num_hours())
                    } else {
                        "Just now".to_string()
                    }
                }),
            }
        }).collect();

        let total = results.len();
        Ok(SearchResponse {
            data: results,
            total,
            page: 1,
            per_page: total,
            has_more: false,
        })
    }

    pub async fn get_latest(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        let url = if page == 1 {
            "https://vortexscans.org/latest-updates".to_string()
        } else {
            format!("https://vortexscans.org/latest-updates?page={}", page)
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
            Err(e) => Err(e)
        }
    }

    pub async fn get_popular(user_agent: &str) -> Result<Vec<SearchResult>, String> {
        let url = "https://vortexscans.org/";
        
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let items = Self::extract_popular_items(&html);
                Ok(items)
            }
            Err(e) => Err(e)
        }
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        let url = format!("https://vortexscans.org/series/{}", identifier);
        
        match Self::fetch_html(&url, user_agent).await {
            Ok(html) => {
                let info = Self::extract_manga_info(&html, identifier);
                Ok(info)
            }
            Err(e) => Err(e)
        }
    }

    pub async fn get_chapter_images(
        book_id: &str,
        chapter: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
        let url = format!("https://vortexscans.org/series/{}/chapter-{}", book_id, chapter);
        
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
            Err(e) => Err(e)
        }
    }

    pub fn extension_info() -> ExtensionInfo {
        ExtensionInfo {
            id: "vortexscans".to_string(),
            name: "Vortex Scans".to_string(),
            version: "1.0.0".to_string(),
            description: "Read Comics, manga, manhua, manhwa, translated swiftly: Vortex, your ultimate library.".to_string(),
            author: "wzread".to_string(),
            cover: "./extension_cover.png".to_string(),
            icon: "./extension_icon.png".to_string(),
        }
    }
}