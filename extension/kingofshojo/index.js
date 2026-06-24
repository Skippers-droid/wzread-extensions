const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://kingofshojo.com/?s=${searchQuery}`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const results = [];

      const itemRegex = /<div class="bs">[\s\S]*?<a href="([^"]+)" title="([^"]+)">[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<div class="tt">([^<]+)<\/div>[\s\S]*?<div class="epxs">([^<]+)<\/div>/g;

      let match;
      while ((match = itemRegex.exec(html)) !== null) {
        const url = match[1];
        const title = match[2];
        const cover = match[3];
        const titleText = match[4].trim();
        const latestChapter = match[5].trim();

        const idMatch = url.match(/\/manga\/([^\/]+)\/?/);
        const id = idMatch ? idMatch[1] : '';

        let cleanCover = cover;
        if (cleanCover) {
          cleanCover = cleanCover.replace(/\/\//g, '/');
          cleanCover = cleanCover.replace(/^http:\//, 'http://');
          cleanCover = cleanCover.replace(/^https:\//, 'https://');
        }

        results.push({
          id: id,
          slug: id,
          title: titleText || title,
          cover: cleanCover,
          latestChapter: latestChapter,
        });
      }

      return results;

    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const url = `https://kingofshojo.com/manga/${identifier}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const titleMatch = html.match(/<h1 class="entry-title"[^>]*>([^<]+)<\/h1>/);
      const title = titleMatch ? titleMatch[1].trim() : identifier;

      const altMatch = html.match(/<tr>[\s\S]*?<td>Alternative<\/td>[\s\S]*?<td>([^<]+)<\/td>/);
      const altTitle = altMatch ? altMatch[1].trim() : '';

      const coverMatch = html.match(/<div class="thumb"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>/);
      let cover = coverMatch ? coverMatch[1] : '';
      if (cover) {
        cover = cover.replace(/\/\//g, '/');
        cover = cover.replace(/^http:\//, 'http://');
        cover = cover.replace(/^https:\//, 'https://');
      }

      const descMatch = html.match(/<div class="entry-content entry-content-single"[^>]*>[\s\S]*?<p>([\s\S]*?)<\/p>/);
      let description = descMatch ? descMatch[1].trim() : '';
      description = description.replace(/<[^>]*>/g, '').trim();

      const typeMatch = html.match(/<tr>[\s\S]*?<td>Type<\/td>[\s\S]*?<td>([^<]+)<\/td>/);
      const type = typeMatch ? typeMatch[1].trim() : '';

      const statusMatch = html.match(/<tr>[\s\S]*?<td>Status<\/td>[\s\S]*?<td>([^<]+)<\/td>/);
      const status = statusMatch ? statusMatch[1].trim() : '';

      const authorMatch = html.match(/<tr>[\s\S]*?<td>Author<\/td>[\s\S]*?<td>([^<]+)<\/td>/);
      const author = authorMatch ? authorMatch[1].trim() : '';

      const genres = [];
      const genreRegex = /<div class="seriestugenre">[\s\S]*?<a href="[^"]*"[^>]*>([^<]+)<\/a>/g;
      let genreMatch;
      while ((genreMatch = genreRegex.exec(html)) !== null) {
        genres.push(genreMatch[1].trim());
      }

      const chapters = [];
      const chapterRegex = /<li data-num="(\d+)">[\s\S]*?<a href="([^"]+)"[^>]*>[\s\S]*?<span class="chapternum">Chapter\s*(\d+)<\/span>[\s\S]*?<span class="chapterdate">([^<]+)<\/span>/g;

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
    name: 'Kingofshojo',
    version: '1.0.0',
    description: 'Kingofshojo extension',
    author: 'wzread',
    cover: './extension_cover.png',
    type: 'adult'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const url = `https://kingofshojo.com/${bookId}-chapter-${chapterNumber}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const images = [];

      const imageRegex = /"images"\s*:\s*\[([\s\S]*?)\]/;
      const imageMatch = html.match(imageRegex);

      if (imageMatch) {
        const imageArray = imageMatch[1].match(/"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g);
        if (imageArray) {
          images.push(...imageArray.map(img => {
            let clean = img.replace(/"/g, '');
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            return clean;
          }));
        }
      }

      if (images.length === 0) {
        const altImageRegex = /https:\/\/cdn\.kingofshojo\.com\/king-bucket\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
        const altMatches = html.match(altImageRegex);
        if (altMatches) {
          images.push(...altMatches.map(url => {
            let clean = url;
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            return clean;
          }));
        }
      }

      if (images.length === 0) {
        const imgRegex = /<img[^>]*src="([^"]+\.(?:jpg|jpeg|png|webp|gif))"[^>]*>/g;
        let imgMatch;
        while ((imgMatch = imgRegex.exec(html)) !== null) {
          const src = imgMatch[1];
          if (src.includes('king-bucket')) {
            let clean = src;
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            images.push(clean);
          }
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
      const url = `https://kingofshojo.com/${bookId}-chapter-${chapter}/`;

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
          images = imageArray.map(img => {
            let clean = img.replace(/"/g, '');
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            return clean;
          });
        }
      }

      if (images.length === 0) {
        const altImageRegex = /https:\/\/cdn\.kingofshojo\.com\/king-bucket\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
        const altMatches = html.match(altImageRegex);
        if (altMatches) {
          images = altMatches.map(url => {
            let clean = url;
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            return clean;
          });
        }
      }

      if (images.length === 0) {
        const imgRegex = /<img[^>]*src="([^"]+\.(?:jpg|jpeg|png|webp|gif))"[^>]*>/g;
        let imgMatch;
        while ((imgMatch = imgRegex.exec(html)) !== null) {
          const src = imgMatch[1];
          if (src.includes('king-bucket')) {
            let clean = src;
            clean = clean.replace(/\/\//g, '/');
            clean = clean.replace(/^http:\//, 'http://');
            clean = clean.replace(/^https:\//, 'https://');
            images.push(clean);
          }
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