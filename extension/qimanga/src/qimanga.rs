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
struct SeriesListResponse {
    data: Vec<SeriesItem>,
    #[serde(rename = "totalItems")]
    total_items: usize,
    current: usize,
    next: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SeriesItem {
    id: u32,
    slug: String,
    title: String,
    cover: String,
    r#type: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponseRaw {
    data: Vec<SeriesItem>,
    #[serde(rename = "totalItems")]
    total_items: usize,
    current: usize,
    next: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ChapterListResponse {
    data: Vec<ChapterItemRaw>,
    #[serde(rename = "totalItems")]
    total_items: usize,
    current: usize,
    next: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ChapterItemRaw {
    number: f64,
    slug: String,
    title: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ChapterDetailResponse {
    images: Vec<ChapterImage>,
}

#[derive(Debug, Deserialize)]
struct ChapterImage {
    url: String,
}

pub struct Qimanga;

impl Qimanga {
    const BASE_URL: &'static str = "https://api.qimanga.com/api/v1";
    
    async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str, user_agent: &str) -> Result<T, String> {
        eprintln!("[qimanga] fetch_json: url={}", url);
        
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                eprintln!("[qimanga] client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("[qimanga] request error: {}", e);
                e.to_string()
            })?;

        eprintln!("[qimanga] fetch_json: status={}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[qimanga] error response body: {}", text);
            return Err(format!("HTTP error! status: {} - {}", status, text));
        }

        let text = response.text().await.map_err(|e| {
            eprintln!("[qimanga] read text error: {}", e);
            e.to_string()
        })?;
        
        eprintln!("[qimanga] response body (first 500 chars): {}", &text[..std::cmp::min(text.len(), 500)]);
        
        serde_json::from_str(&text).map_err(|e| {
            eprintln!("[qimanga] json parse error: {} - body: {}", e, text);
            e.to_string()
        })
    }

    async fn fetch_all_chapters(identifier: &str, user_agent: &str) -> Result<Vec<ChapterInfo>, String> {
        eprintln!("[qimanga] fetch_all_chapters: identifier={}", identifier);
        let mut all_chapters = Vec::new();
        let mut page = 1;
        let per_page = 100;
        
        loop {
            let url = format!(
                "{}/series/{}/chapters?page={}&perPage={}&sort=asc",
                Self::BASE_URL, identifier, page, per_page
            );
            
            eprintln!("[qimanga] fetching chapters page {}", page);
            let response: ChapterListResponse = Self::fetch_json(&url, user_agent).await?;
            
            eprintln!("[qimanga] got {} chapters on page {}", response.data.len(), page);
            
            for item in response.data {
                let slug = item.slug.clone();
                all_chapters.push(ChapterInfo {
                    number: item.number,
                    slug: slug.clone(),
                    title: item.title.unwrap_or_else(|| format!("Chapter {}", item.number)),
                    date: item.created_at,
                    url: format!("/{}/{}", identifier, slug),
                });
            }
            
            if response.next.is_none() {
                eprintln!("[qimanga] no more pages, total chapters: {}", all_chapters.len());
                break;
            }
            page += 1;
        }
        
        Ok(all_chapters)
    }

    pub async fn search(query: &str, user_agent: &str) -> Result<SearchResponse, String> {
        eprintln!("[qimanga] search: query={}", query);
        let url = format!(
            "{}/series/search?q={}&perPage=20",
            Self::BASE_URL,
            urlencoding::encode(query)
        );
        
        let response: SearchResponseRaw = Self::fetch_json(&url, user_agent).await?;
        
        let results: Vec<SearchResult> = response.data.into_iter().map(|item| {
            SearchResult {
                id: item.id.to_string(),
                slug: item.slug,
                title: item.title,
                cover: item.cover,
                status: item.status,
                r#type: item.r#type,
            }
        }).collect();

        let total = response.total_items;
        let has_more = response.next.is_some();
        
        eprintln!("[qimanga] search: found {} results", total);
        Ok(SearchResponse {
            data: results,
            total,
            page: response.current,
            per_page: 20,
            has_more,
        })
    }

    pub async fn get_latest(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        eprintln!("[qimanga] get_latest: page={}", page);
        let url = format!(
            "{}/series?page={}&perPage=20&sort=newest&type=MANHWA",
            Self::BASE_URL, page
        );
        
        let response: SeriesListResponse = Self::fetch_json(&url, user_agent).await?;
        
        let results: Vec<SearchResult> = response.data.into_iter().map(|item| {
            SearchResult {
                id: item.id.to_string(),
                slug: item.slug,
                title: item.title,
                cover: item.cover,
                status: item.status,
                r#type: item.r#type,
            }
        }).collect();

        let total = response.total_items;
        let has_more = response.next.is_some();
        
        eprintln!("[qimanga] get_latest: found {} results, has_more={}", total, has_more);
        Ok(SearchResponse {
            data: results,
            total,
            page: response.current,
            per_page: 20,
            has_more,
        })
    }

    pub async fn get_popular(user_agent: &str, page: usize) -> Result<SearchResponse, String> {
        eprintln!("[qimanga] get_popular: page={}", page);
        let url = format!(
            "{}/series?page={}&perPage=20&sort=popular",
            Self::BASE_URL, page
        );
        
        let response: SeriesListResponse = Self::fetch_json(&url, user_agent).await?;
        
        let results: Vec<SearchResult> = response.data.into_iter().map(|item| {
            SearchResult {
                id: item.id.to_string(),
                slug: item.slug,
                title: item.title,
                cover: item.cover,
                status: item.status,
                r#type: item.r#type,
            }
        }).collect();

        let total = response.total_items;
        let has_more = response.next.is_some();
        
        eprintln!("[qimanga] get_popular: found {} results, has_more={}", total, has_more);
        Ok(SearchResponse {
            data: results,
            total,
            page: response.current,
            per_page: 20,
            has_more,
        })
    }

    pub async fn manga_info(identifier: &str, user_agent: &str) -> Result<MangaInfo, String> {
        eprintln!("[qimanga] manga_info: identifier={}", identifier);
        let url = format!("{}/series/{}", Self::BASE_URL, identifier);
        
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                eprintln!("[qimanga] client build error: {}", e);
                e.to_string()
            })?;

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("[qimanga] request error: {}", e);
                e.to_string()
            })?;

        eprintln!("[qimanga] manga_info: status={}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eprintln!("[qimanga] error response body: {}", text);
            return Err(format!("HTTP error! status: {} - {}", status, text));
        }

        let text = response.text().await.map_err(|e| {
            eprintln!("[qimanga] read text error: {}", e);
            e.to_string()
        })?;
        
        eprintln!("[qimanga] manga_info response (first 500 chars): {}", &text[..std::cmp::min(text.len(), 500)]);
        
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            eprintln!("[qimanga] json parse error: {} - body: {}", e, text);
            e.to_string()
        })?;
        
        let genres: Vec<String> = json["genres"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|g| g["name"].as_str().map(String::from))
            .collect();

        let chapters = Self::fetch_all_chapters(identifier, user_agent).await?;

        eprintln!("[qimanga] manga_info: found {} chapters", chapters.len());

        Ok(MangaInfo {
            id: json["id"].to_string(),
            slug: json["slug"].as_str().unwrap_or("").to_string(),
            title: json["title"].as_str().unwrap_or("").to_string(),
            alt_title: json["alternativeTitles"].as_str().unwrap_or("").to_string(),
            description: json["description"].as_str().unwrap_or("").to_string(),
            cover: json["cover"].as_str().unwrap_or("").to_string(),
            author: json["author"].as_str().unwrap_or("").to_string(),
            artist: json["artist"].as_str().unwrap_or("").to_string(),
            status: json["status"].as_str().unwrap_or("").to_string(),
            r#type: json["type"].as_str().unwrap_or("").to_string(),
            genres,
            chapters,
        })
    }

    pub async fn get_chapter_images(
        series_slug: &str,
        chapter_slug: &str,
        user_agent: &str,
        page: usize,
        per_page: usize,
    ) -> Result<ChapterImages, String> {
        eprintln!("[qimanga] get_chapter_images: series={}, chapter={}, page={}, per_page={}", 
            series_slug, chapter_slug, page, per_page);
        let url = format!("{}/series/{}/chapters/{}", Self::BASE_URL, series_slug, chapter_slug);
        let response: ChapterDetailResponse = Self::fetch_json(&url, user_agent).await?;

        let all_images: Vec<String> = response.images.into_iter()
            .map(|img| img.url)
            .collect();

        let total = all_images.len();
        let start = (page - 1) * per_page;
        let end = std::cmp::min(start + per_page, total);
        let paginated = if start < total {
            all_images[start..end].to_vec()
        } else {
            Vec::new()
        };

        eprintln!("[qimanga] get_chapter_images: total={}, returning {} images", total, paginated.len());

        Ok(ChapterImages {
            images: paginated,
            total,
            page,
            per_page,
            has_more: end < total,
        })
    }

    pub fn extension_info() -> ExtensionInfo {
        eprintln!("[qimanga] extension_info called");
        ExtensionInfo {
            id: "qimanga".to_string(),
            name: "Qimanga".to_string(),
            version: "1.0.0".to_string(),
            description: "Qimanga extension - Read comics from Qimanga".to_string(),
            author: "wzread".to_string(),
            cover: "./extension_cover.png".to_string(),
            icon: "./extension_icon.png".to_string(),
        }
    }
}