const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://manhwaread.com/?s=${searchQuery}`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const html = await response.text();

      const results = [];

      const itemRegex = /<div class="manga-item loop-item[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<a href="([^"]+)"[^>]*>[\s\S]*?<h3[^>]*>[\s\S]*?<a[^>]*>([^<]+)<\/a>[\s\S]*?<span class="manga-status"[^>]*data-status="([^"]+)"[^>]*>[\s\S]*?<span class="manga-status__label">([^<]+)<\/span>[\s\S]*?<span class="chapter-name[^>]*>([^<]+)<\/span>/g;

      let match;
      while ((match = itemRegex.exec(html)) !== null) {
        let cover = match[1];
        const url = match[2];
        const title = match[3].trim();
        const status = match[4];
        const statusText = match[5].trim();
        const latestChapter = match[6].trim();

        const idMatch = url.match(/\/manhwa\/([^\/]+)\/?/);
        const id = idMatch ? idMatch[1] : '';

        if (cover && cover.startsWith('/')) {
          cover = `https://manhwaread.com${cover}`;
        }

        results.push({
          id: id,
          slug: id,
          title: title,
          cover: cover,
          status: status || statusText,
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
      const url = `https://manhwaread.com/manhwa/${identifier}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const html = await response.text();

      const titleMatch = html.match(/<h1[^>]*>([^<]+)<\/h1>/);
      const title = titleMatch ? titleMatch[1].trim() : identifier;

      const altMatch = html.match(/<h2[^>]*>([^<]+)<\/h2>/);
      const altTitle = altMatch ? altMatch[1].trim() : '';

      let cover = '';
      const coverMatch = html.match(/<img[^>]*src="([^"]+)"[^>]*>/);
      if (coverMatch) {
        cover = coverMatch[1];
        if (cover && cover.startsWith('/')) {
          cover = `https://manhwaread.com${cover}`;
        }
      }

      const descMatch = html.match(/<div id="mangaDesc"[^>]*>[\s\S]*?<div[^>]*>([\s\S]*?)<\/div>[\s\S]*?<button[^>]*>/);
      let description = descMatch ? descMatch[1].trim() : '';
      description = description.replace(/<[^>]*>/g, '').trim();

      const statusMatch = html.match(/<span class="manga-status"[^>]*data-status="([^"]+)"[^>]*>/);
      const status = statusMatch ? statusMatch[1].trim() : '';

      const genres = [];
      const genreRegex = /<a[^>]*href="\/genre\/[^"]+"[^>]*>([^<]+)<\/a>/g;
      let genreMatch;
      while ((genreMatch = genreRegex.exec(html)) !== null) {
        genres.push(genreMatch[1].trim());
      }

      const chapters = [];
      const chapterRegex = /<a[^>]*href="([^"]+)"[^>]*data-id="([^"]+)"[^>]*>[\s\S]*?<span[^>]*>([^<]+)<\/span>[\s\S]*?<span[^>]*>([^<]+)<\/span>/g;

      let chapterMatch;
      while ((chapterMatch = chapterRegex.exec(html)) !== null) {
        const chapterUrl = chapterMatch[1];
        const chapterId = chapterMatch[2];
        const chapterName = chapterMatch[3].trim();
        const chapterDate = chapterMatch[4].trim();

        const numberMatch = chapterName.match(/\d+/);
        const chapterNumber = numberMatch ? parseInt(numberMatch[0], 10) : 0;

        const slugMatch = chapterUrl.match(/\/chapter-(\d+)\/?/);
        const slug = slugMatch ? `chapter-${slugMatch[1]}` : `chapter-${chapterNumber}`;

        chapters.push({
          number: chapterNumber,
          slug: slug,
          title: chapterName,
          date: chapterDate,
          url: chapterUrl,
          id: chapterId,
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
        status: status,
        genres: genres,
        chapters: chapters,
      };

    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'ManhwaRead',
    version: '1.0.0',
    description: 'ManhwaRead extension',
    author: 'wzread',
    cover: './extension_cover.png',
    type: 'adult'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const paddedChapter = String(chapterNumber).padStart(2, '0');
      const url = `https://manhwaread.com/manhwa/${bookId}/chapter-${paddedChapter}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const html = await response.text();

      const images = [];

      const imageRegex = /"src":"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g;
      let imageMatch;
      while ((imageMatch = imageRegex.exec(html)) !== null) {
        const src = imageMatch[1].replace(/\\/g, '');
        images.push(src);
      }

      if (images.length === 0) {
        const altImageRegex = /<img[^>]*src="([^"]+\.(?:jpg|jpeg|png|webp|gif))"[^>]*>/g;
        let imgMatch;
        while ((imgMatch = altImageRegex.exec(html)) !== null) {
          const src = imgMatch[1];
          if (src.includes('manread.xyz') || src.includes('manhwaread.com')) {
            images.push(src);
          }
        }
      }

      return {
        number: chapterNumber,
        totalImages: images.length,
        images: images.map((url) => ({ url })),
      };

    } catch (error) {
      throw error;
    }
  },

  getChapterImages: async (bookId, chapter, page = 1, perPage = 5) => {
    try {
      const paddedChapter = String(chapter).padStart(2, '0');
      const url = `https://manhwaread.com/manhwa/${bookId}/chapter-${paddedChapter}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const html = await response.text();

      let images = [];

      const imageRegex = /"src":"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g;
      let imageMatch;
      while ((imageMatch = imageRegex.exec(html)) !== null) {
        const src = imageMatch[1].replace(/\\/g, '');
        images.push(src);
      }

      if (images.length === 0) {
        const altImageRegex = /<img[^>]*src="([^"]+\.(?:jpg|jpeg|png|webp|gif))"[^>]*>/g;
        let imgMatch;
        while ((imgMatch = altImageRegex.exec(html)) !== null) {
          const src = imgMatch[1];
          if (src.includes('manread.xyz') || src.includes('manhwaread.com')) {
            images.push(src);
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