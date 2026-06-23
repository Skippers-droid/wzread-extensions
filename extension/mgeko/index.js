const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://www.mgeko.cc/autocomplete?term=${searchQuery}`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const results = [];
      const ulMatch = html.match(/<ul[^>]*class="novel-list[^>]*>([\s\S]*?)<\/ul>/i);
      
      if (ulMatch) {
        const ulContent = ulMatch[1];
        const liRegex = /<li[^>]*class="novel-item"[^>]*>([\s\S]*?)<\/li>/gi;
        let match;
        
        while ((match = liRegex.exec(ulContent)) !== null) {
          const liContent = match[1];
          
          const titleMatch = liContent.match(/<h4[^>]*class="novel-title[^>]*>([\s\S]*?)<\/h4>/i);
          const title = titleMatch ? titleMatch[1].replace(/<mark>/g, '').replace(/<\/mark>/g, '').trim() : '';
          
          const hrefMatch = liContent.match(/<a[^>]*href="([^"]*)"[^>]*>/i);
          const href = hrefMatch ? hrefMatch[1] : '';
          const slug = href.replace('/manga/', '').replace('/', '');
          
          const coverMatch = liContent.match(/<img[^>]*src="([^"]*)"[^>]*>/i);
          const cover = coverMatch ? coverMatch[1] : '';
          
          const statsMatch = liContent.match(/<strong>([^<]*)<\/strong>/i);
          const chapter = statsMatch ? statsMatch[1] : '';
          
          if (title && slug) {
            results.push({
              id: slug,
              slug: slug,
              title: title,
              cover: cover,
              chapter: chapter
            });
          }
        }
      }
      
      return results;
      
    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const url = `https://www.mgeko.cc/manga/${identifier}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const titleMatch = html.match(/<h1[^>]*itemprop="name"[^>]*>([\s\S]*?)<\/h1>/i);
      const title = titleMatch ? titleMatch[1].trim() : '';
      
      const altTitleMatch = html.match(/<h2[^>]*class="alternative-title"[^>]*>([\s\S]*?)<\/h2>/i);
      const altTitle = altTitleMatch ? altTitleMatch[1].trim() : '';
      
      const authorMatch = html.match(/<span[^>]*itemprop="author"[^>]*>([\s\S]*?)<\/span>/i);
      const author = authorMatch ? authorMatch[1].trim() : '';
      
      const coverMatch = html.match(/<img[^>]*class="lazy"[^>]*data-src="([^"]*)"[^>]*>/i);
      const cover = coverMatch ? coverMatch[1] : '';
      
      const statusMatch = html.match(/<strong[^>]*class="(ongoing|completed)"[^>]*>/i);
      const status = statusMatch ? statusMatch[1].toUpperCase() : 'UNKNOWN';
      
      const genres = [];
      const genreRegex = /<li[^>]*>[\s]*<a[^>]*href="[^"]*genre_included=[^"]*"[^>]*>([\s\S]*?)<\/a>[\s]*<\/li>/gi;
      let genreMatch;
      while ((genreMatch = genreRegex.exec(html)) !== null) {
        const genre = genreMatch[1].trim();
        if (genre) genres.push(genre);
      }
      
      const descriptionMatch = html.match(/<p[^>]*class="description"[^>]*>([\s\S]*?)<\/p>/i);
      let description = descriptionMatch ? descriptionMatch[1].trim() : '';
      description = description.replace(/<br\s*\/?>/gi, '\n').replace(/<[^>]*>/g, '').trim();
      
      const chapters = [];
      const chapterRegex = /<li[^>]*class="chapter-list-item"[^>]*>[\s\S]*?<a[^>]*href="([^"]*)"[^>]*>[\s\S]*?<div[^>]*class="chapter-number"[^>]*>([\s\S]*?)<span[^>]*class="chapter-stats"[^>]*>([\s\S]*?)<\/span>[\s\S]*?<\/div>[\s\S]*?<\/a>[\s\S]*?<\/li>/gi;
      let chapterMatch;
      
      while ((chapterMatch = chapterRegex.exec(html)) !== null) {
        const href = chapterMatch[1];
        const numberMatch = href.match(/chapter-(\d+)/i);
        const number = numberMatch ? parseInt(numberMatch[1]) : 0;
        const title = chapterMatch[2].trim();
        const date = chapterMatch[3].trim();
        
        if (number > 0) {
          chapters.push({
            number: number,
            slug: href.replace('/reader/en/', '').replace('/', ''),
            title: title,
            date: date
          });
        }
      }
      
      chapters.sort((a, b) => a.number - b.number);
      
      return {
        id: identifier,
        slug: identifier,
        title: title,
        altTitle: altTitle,
        author: author,
        cover: cover,
        status: status,
        genres: genres,
        description: description,
        chapters: chapters
      };
      
    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'Mgeko',
    version: '1.0.0',
    description: 'Mgeko manga reader extension',
    author: 'wzread',
    cover: './extension_cover.png'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const slug = typeof chapterNumber === 'string' ? chapterNumber : `${bookId}-chapter-${chapterNumber}-eng-li`;
      const url = `https://www.mgeko.cc/reader/en/${slug}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const images = [];
      const imgRegex = /<img[^>]*src="([^"]*)"[^>]*id="image-\d+"[^>]*>/gi;
      let imgMatch;
      
      while ((imgMatch = imgRegex.exec(html)) !== null) {
        const src = imgMatch[1];
        if (src && !src.includes('credits-mgeko')) {
          images.push(src);
        }
      }
      
      return {
        number: chapterNumber,
        totalImages: images.length,
        images: images
      };
      
    } catch (error) {
      throw error;
    }
  },

  getChapterImages: async (bookId, chapter) => {
    try {
      const slug = typeof chapter === 'string' && chapter.includes('chapter-') 
        ? chapter 
        : `${bookId}-chapter-${chapter}-eng-li`;
      const url = `https://www.mgeko.cc/reader/en/${slug}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const images = [];
      const imgRegex = /<img[^>]*src="([^"]*)"[^>]*id="image-\d+"[^>]*>/gi;
      let imgMatch;
      
      while ((imgMatch = imgRegex.exec(html)) !== null) {
        const src = imgMatch[1];
        if (src && !src.includes('credits-mgeko')) {
          images.push(src);
        }
      }
      
      return images;
      
    } catch (error) {
      throw error;
    }
  }
};

export default extension;