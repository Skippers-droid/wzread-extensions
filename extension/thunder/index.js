const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://en-thunderscans.com/?s=${searchQuery}`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const results = [];
      
      const itemRegex = /<div class="bs">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<span class="status-dot ([^"]+)"><\/span>[\s\S]*?<i>([^<]+)<\/i>/g;
      
      let match;
      while ((match = itemRegex.exec(html)) !== null) {
        const url = match[1];
        const title = match[2];
        const cover = match[3];
        const titleText = match[4].trim();
        const status = match[5];
        const statusText = match[6].trim();
        
        const idMatch = url.match(/\/comics\/([^\/]+)\/?/);
        const id = idMatch ? idMatch[1] : '';
        
        results.push({
          id: id,
          slug: id,
          title: titleText || title,
          cover: cover,
          status: statusText || status,
        });
      }
      
      return results;
      
    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const url = `https://en-thunderscans.com/comics/${identifier}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const titleMatch = html.match(/<h1 class="entry-title"[^>]*>([^<]+)<\/h1>/);
      const title = titleMatch ? titleMatch[1].trim() : identifier;
      
      const altMatch = html.match(/<div class="alternative">[\s\S]*?<div class="desktop-titles">([^<]+)<\/div>/);
      const altTitle = altMatch ? altMatch[1].trim() : '';
      
      const coverMatch = html.match(/<div class="thumb"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>/);
      const cover = coverMatch ? coverMatch[1] : '';
      
      const descMatch = html.match(/<div class="entry-content entry-content-single"[^>]*>[\s\S]*?<p>([\s\S]*?)<\/p>/);
      let description = descMatch ? descMatch[1].trim() : '';
      description = description.replace(/<[^>]*>/g, '').trim();
      
      const typeMatch = html.match(/<div class="imptdt">[\s\S]*?<h1>\s*Type\s*<\/h1>[\s\S]*?<i>([^<]+)<\/i>/);
      const type = typeMatch ? typeMatch[1].trim() : '';
      
      const statusMatch = html.match(/<div class="imptdt">[\s\S]*?<h1>\s*Status\s*<\/h1>[\s\S]*?<i>([^<]+)<\/i>/);
      const status = statusMatch ? statusMatch[1].trim() : '';
      
      const authorMatch = html.match(/<div class="imptdt">[\s\S]*?<h1>\s*Author\s*<\/h1>[\s\S]*?<i>([^<]+)<\/i>/);
      const author = authorMatch ? authorMatch[1].trim() : '';
      
      const genres = [];
      const genreRegex = /<span class="mgen">[\s\S]*?<a href="[^"]*"[^>]*>([^<]+)<\/a>/g;
      let genreMatch;
      while ((genreMatch = genreRegex.exec(html)) !== null) {
        genres.push(genreMatch[1].trim());
      }
      
      const chapters = [];
      const chapterRegex = /<li data-num="(\d+)">[\s\S]*?<a[^>]*href="([^"]+)"[^>]*>[\s\S]*?<span class="chapternum">[\s\S]*?Chapter\s*(\d+)<\/span>[\s\S]*?<span class="chapterdate">([^<]+)<\/span>/g;
      
      let chapterMatch;
      while ((chapterMatch = chapterRegex.exec(html)) !== null) {
        const chapterUrl = chapterMatch[2];
        const chapterNumber = parseInt(chapterMatch[3], 10);
        const chapterDate = chapterMatch[4].trim();
        
        const chapterSlugMatch = chapterUrl.match(/\/[^\/]+-chapter-(\d+)\/?/);
        const chapterSlug = chapterSlugMatch ? `chapter-${chapterSlugMatch[1]}` : `chapter-${chapterNumber}`;
        
        chapters.push({
          number: chapterNumber,
          slug: chapterSlug,
          title: `Chapter ${chapterNumber}`,
          date: chapterDate,
          url: chapterUrl,
        });
      }
      
      chapters.sort((a, b) => a.number - b.number);
      
      return {
        id: identifier,
        slug: identifier,
        title: title,
        altTitle: altTitle,
        description: description,
        cover: cover,
        author: author,
        status: status,
        type: type,
        genres: genres,
        chapters: chapters,
      };
      
    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'ThunderScans',
    version: '1.0.0',
    description: 'ThunderScans EN extension',
    author: 'wzread',
    cover: './extension_cover.png'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const url = `https://en-thunderscans.com/${bookId}-chapter-${chapterNumber}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      const imageRegex = /"images"\s*:\s*\[([\s\S]*?)\]/;
      const imageMatch = html.match(imageRegex);
      
      let images = [];
      if (imageMatch) {
        const imageArray = imageMatch[1].match(/"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g);
        if (imageArray) {
          images = imageArray.map(img => img.replace(/"/g, ''));
        }
      }
      
      if (images.length === 0) {
        const altImageRegex = /https:\/\/en-thunderscans\.com\/wp-content\/uploads\/manga\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
        const altMatches = html.match(altImageRegex);
        if (altMatches) {
          images = altMatches;
        }
      }
      
      return {
        number: chapterNumber,
        totalImages: images.length,
        images: images.map(url => ({ url })),
      };
      
    } catch (error) {
      throw error;
    }
  },

  getChapterImages: async (bookId, chapter, page = 1, perPage = 5) => {
    try {
      const url = `https://en-thunderscans.com/${bookId}-chapter-${chapter}/`;
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const html = await response.text();
      
      let images = [];
      
      const imageRegex = /"images"\s*:\s*\[([\s\S]*?)\]/;
      const imageMatch = html.match(imageRegex);
      
      if (imageMatch) {
        const imageArray = imageMatch[1].match(/"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g);
        if (imageArray) {
          images = imageArray.map(img => img.replace(/"/g, ''));
        }
      }
      
      if (images.length === 0) {
        const altImageRegex = /https:\/\/en-thunderscans\.com\/wp-content\/uploads\/manga\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
        const altMatches = html.match(altImageRegex);
        if (altMatches) {
          images = altMatches;
        }
      }
      
      const total = images.length;
      const start = (page - 1) * perPage;
      const end = start + perPage;
      const paginatedImages = images.slice(start, end);
      
      return {
        images: paginatedImages,
        total: total,
        page: page,
        perPage: perPage,
        hasMore: end < total
      };
      
    } catch (error) {
      throw error;
    }
  }
};

export default extension;