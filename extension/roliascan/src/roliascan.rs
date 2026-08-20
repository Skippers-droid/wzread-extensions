use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub cover: String,
    pub status: String,
    pub r#type: String,
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
    pub number: f64,
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
    pub artist: String,
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

#[derive(Debug, Deserialize)]
struct SeriesItem {
    id: String,
    title: String,
    url: String,
    cover: String,
    r#type: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct ChaptersResponse {
    success: bool,
    chapters: Vec<ChapterItem>,
    total: usize,
}

#[derive(Debug, Deserialize)]
struct ChapterItem {
    id: String,
    chapter: String,
    title: String,
    date: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct ChapterContentResponse {
    images: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResponseRaw {
    results: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: u32,
    title: String,
    slug: String,
    thumbnail: String,
    r#type: String,
    status: String,
}

pub struct Roliascan;

impl Roliascan {
    const BASE_URL: &'static str = "https://roliascan.com";
    const API_URL: &'static str = "https://roliascan.com/wp-json/manga/v1";
    const AUTH_URL: &'static str = "https://roliascan.com/auth";
    const TOKEN_T: &'static str = "52e1c3e5cc3f1d07";
    const TOKEN_TS: &'static str = "1787211165";

    async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str, user_agent: &str) -> Result<T, String> {
        eprintln!("[roliascan] fetch_json: url={}", url);

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                eprintln!("[roliascan] client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("[roliascan] request error: {}", e);
                e.to_string()
            })?;

        eprintln!("[roliascan] fetch_json: status={}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[roliascan] error response body: {}", text);
            return Err(format!("HTTP error! status: {} - {}", status, text));
        }

        let text = response.text().await.map_err(|e| {
            eprintln!("[roliascan] read text error: {}", e);
            e.to_string()
        })?;

        eprintln!("[roliascan] response body (first 500 chars): {}", &text[..std::cmp::min(text.len(), 500)]);

        serde_json::from_str(&text).map_err(|e| {
            eprintln!("[roliascan] json parse error: {} - body: {}", e, text);
            e.to_string()
        })
    }

    async fn fetch_html(url: &str, user_agent: &str) -> Result<String, String> {
        eprintln!("[roliascan] fetch_html: url={}", url);

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                eprintln!("[roliascan] client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("[roliascan] request error: {}", e);
                e.to_string()
            })?;

        eprintln!("[roliascan] fetch_html: status={}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[roliascan] error response body: {}", text);
            return Err(format!("HTTP error! status: {} - {}", status, text));
        }

        response.text().await.map_err(|e| {
            eprintln!("[roliascan] read text error: {}", e);
            e.to_string()
        })
    }

    fn extract_manga_id_from_html(html: &str) -> String {
        let re = regex::Regex::new(r#"<meta[^>]*name=["']twitter:image["'][^>]*content=["']https://roliascan\.com/content/media/manga-(\d+)-[^"']+["'][^>]*>"#)
            .unwrap();
        
        if let Some(cap) = re.captures(html) {
            return cap[1].to_string();
        }

        let re2 = regex::Regex::new(r#"<meta[^>]*property=["']og:image["'][^>]*content=["']https://roliascan\.com/content/media/manga-(\d+)-[^"']+["'][^>]*>"#)
            .unwrap();
        
        if let Some(cap) = re2.captures(html) {
            return cap[1].to_string();
        }

        let re3 = regex::Regex::new(r#"data-manga-id=["'](\d+)["']"#)
            .unwrap();
        
        if let Some(cap) = re3.captures(html) {
            return cap[1].to_string();
        }

        "0".to_string()
    }

    fn extract_manga_info_from_html(html: &str, slug: &str) -> MangaInfo {
        eprintln!("[roliascan] extract_manga_info_from_html: parsing");

        let title = regex::Regex::new(r#"<h1[^>]*>([^<]+)</h1>"#)
            .unwrap()
            .captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_else(|| slug.to_string());

        let alt_titles = regex::Regex::new(r#"<p[^>]*class="[^"]*alt-titles[^"]*"[^>]*>([^<]+)</p>"#)
            .unwrap()
            .captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let description = regex::Regex::new(r#"<div[^>]*id="description-content-tab"[^>]*>([\s\S]*?)</div>"#)
            .unwrap()
            .captures(html)
            .map(|cap| {
                let text = cap[1].trim();
                regex::Regex::new(r"<[^>]*>").unwrap().replace_all(text, "").to_string()
            })
            .unwrap_or_default();

        let cover = regex::Regex::new(r#"<img[^>]*src="([^"]+)"[^>]*alt="[^"]*cover[^"]*"[^>]*>"#)
            .unwrap()
            .captures(html)
            .map(|cap| cap[1].to_string())
            .unwrap_or_default();

        let author = regex::Regex::new(r#"<div[^>]*class="[^"]*text-neutral-200[^"]*"[^>]*>([^<]+)</div>"#)
            .unwrap()
            .captures(html)
            .map(|cap| cap[1].trim().to_string())
            .unwrap_or_default();

        let r#type = regex::Regex::new(r#"Manhwa|Manga|Manhua"#)
            .unwrap()
            .find(html)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let status = regex::Regex::new(r#"Ongoing|Completed|Hiatus|Cancelled"#)
            .unwrap()
            .find(html)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut genres = Vec::new();
        let genre_re = regex::Regex::new(r#"<a[^>]*href="[^"]*tag/[^"]+"[^>]*>([^<]+)</a>"#).unwrap();
        for cap in genre_re.captures_iter(html) {
            let genre = cap[1].trim().to_string();
            if !genre.is_empty() {
                genres.push(genre);
            }
        }

        let manga_id = Self::extract_manga_id_from_html(html);

        MangaInfo {
            id: manga_id,
            slug: slug.to_string(),
            title,
            alt_title: alt_titles,
            description,
            cover,
            author,
            artist: String::new(),
            status,
            r#type,
            genres,
            chapters: Vec::new(),
        }
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        eprintln!("[roliascan] search: query={}", query);

        let url = format!("{}/auth/search", Self::AUTH_URL);
        let body = serde_json::json!({
            "query": query,
            "limit": 20
        });

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP error! status: {}", response.status()));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;
        let json: SearchResponseRaw = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        let results: Vec<SearchResult> = json.results.into_iter().map(|item| {
            SearchResult {
                id: item.id.to_string(),
                slug: item.slug,
                title: item.title,
                cover: item.thumbnail,
                status: item.status,
                r#type: item.r#type,
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
        eprintln!("[roliascan] get_latest: page={}", page);

        let url = format!("{}/load", Self::API_URL);
        let body = serde_json::json!({
            "page": page,
            "search": "",
            "years": "[]",
            "genres": "[]",
            "types": "[]",
            "statuses": "[]",
            "sort": "post_desc",
            "genreMatchMode": "any"
        });

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[roliascan] error response: {}", text);
            return Err(format!("HTTP error! status: {}", status));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;
        
        let items: Vec<SeriesItem> = serde_json::from_str(&text).map_err(|e| {
            eprintln!("[roliascan] json parse error: {}", e);
            e.to_string()
        })?;

        let results: Vec<SearchResult> = items.into_iter().map(|item| {
            let slug = item.url.trim_end_matches('/').split('/').last().unwrap_or("").to_string();
            
            SearchResult {
                id: item.id,
                slug,
                title: item.title,
                cover: item.cover,
                status: item.status,
                r#type: item.r#type,
            }
        }).collect();

        let total = results.len();
        let has_more = results.len() == 20;

        Ok(SearchResponse {
            data: results,
            total,
            page,
            per_page: 20,
            has_more,
        })
    }

    pub async fn get_popular(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        eprintln!("[roliascan] get_popular: page={}", page);

        let url = format!("{}/load", Self::API_URL);
        let body = serde_json::json!({
            "page": page,
            "search": "",
            "years": "[]",
            "genres": "[]",
            "types": "[]",
            "statuses": "[]",
            "sort": "popular_desc",
            "genreMatchMode": "any"
        });

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[roliascan] error response: {}", text);
            return Err(format!("HTTP error! status: {}", status));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;
        
        let items: Vec<SeriesItem> = serde_json::from_str(&text).map_err(|e| {
            eprintln!("[roliascan] json parse error: {}", e);
            e.to_string()
        })?;

        let results: Vec<SearchResult> = items.into_iter().map(|item| {
            let slug = item.url.trim_end_matches('/').split('/').last().unwrap_or("").to_string();
            
            SearchResult {
                id: item.id,
                slug,
                title: item.title,
                cover: item.cover,
                status: item.status,
                r#type: item.r#type,
            }
        }).collect();

        let total = results.len();
        let has_more = results.len() == 20;

        Ok(SearchResponse {
            data: results,
            total,
            page,
            per_page: 20,
            has_more,
        })
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        eprintln!("[roliascan] manga_info: identifier={}", identifier);

        let url = format!("{}/manga/{}/", Self::BASE_URL, identifier);
        let html = Self::fetch_html(&url, user_agent).await?;

        let mut info = Self::extract_manga_info_from_html(&html, identifier);

        if info.id != "0" {
            let chapters_url = format!(
                "{}/manga-chapters?manga_id={}&offset=0&limit=500&order=DESC&_t={}&_ts={}",
                Self::AUTH_URL, info.id, Self::TOKEN_T, Self::TOKEN_TS
            );

            let chapters_response_result: Result<ChaptersResponse, String> = Self::fetch_json(&chapters_url, user_agent).await;

            if let Ok(chapters_response) = chapters_response_result {
                if chapters_response.success {
                    let mut chapters = Vec::new();
                    for item in chapters_response.chapters {
                        let chapter_num: f64 = item.chapter.parse().unwrap_or(0.0);
                        let chapter_id = regex::Regex::new(r"/ch(\d+)-|/ch(\d+)/")
                            .unwrap()
                            .captures(&item.url)
                            .map(|c| c[1].to_string())
                            .unwrap_or_else(|| "0".to_string());

                        chapters.push(ChapterInfo {
                            number: chapter_num,
                            slug: chapter_id,
                            title: if item.title != "N/A" { item.title } else { format!("Chapter {}", chapter_num) },
                            date: item.date,
                            url: item.url,
                        });
                    }
                    info.chapters = chapters;
                }
            } else {
                eprintln!("[roliascan] API failed, extracting chapters from HTML");
                let chapters = Self::extract_chapters_from_html(&html);
                info.chapters = chapters;
            }
        } else {
            eprintln!("[roliascan] No manga ID found, extracting chapters from HTML");
            let chapters = Self::extract_chapters_from_html(&html);
            info.chapters = chapters;
        }

        Ok(info)
    }

    fn extract_chapters_from_html(html: &str) -> Vec<ChapterInfo> {
        eprintln!("[roliascan] extract_chapters_from_html: parsing");
        let mut chapters = Vec::new();

        let chapter_re = regex::Regex::new(
            r#"<a[^>]*href="([^"]+)"[^>]*>[\s\S]*?Chapter\s*([\d.]+)[\s\S]*?<span[^>]*class="[^"]*date[^"]*"[^>]*>([^<]+)</span>"#
        ).unwrap();

        for cap in chapter_re.captures_iter(html) {
            let url = cap[1].to_string();
            let chapter_num: f64 = cap[2].parse().unwrap_or(0.0);
            let date = cap[3].trim().to_string();
            
            let id = regex::Regex::new(r"/ch(\d+)-|/ch(\d+)/")
                .unwrap()
                .captures(&url)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "0".to_string());

            chapters.push(ChapterInfo {
                number: chapter_num,
                slug: id,
                title: format!("Chapter {}", chapter_num),
                date,
                url,
            });
        }

        eprintln!("[roliascan] extract_chapters_from_html: found {} chapters", chapters.len());
        chapters
    }

    pub async fn get_chapter_images(
        chapter_id: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
        eprintln!("[roliascan] get_chapter_images: chapter_id={}, page={}, per_page={}", chapter_id, page, per_page);

        let url = format!("{}/chapter-content?chapter_id={}", Self::AUTH_URL, chapter_id);
        let response: ChapterContentResponse = Self::fetch_json(&url, user_agent).await?;

        let all_images = response.images;
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

    pub fn extension_info() -> ExtensionInfo {
        ExtensionInfo {
            id: "roliascan".to_string(),
            name: "Roliascan".to_string(),
            version: "1.0.0".to_string(),
            description: "Roliascan extension - Read comics from Roliascan".to_string(),
            author: "wzread".to_string(),
            cover: "./extension_cover.png".to_string(),
            icon: "./extension_icon.png".to_string(),
        }
    }
}