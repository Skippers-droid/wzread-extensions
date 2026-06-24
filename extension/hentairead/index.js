const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://hentairead.com/?s=${searchQuery}`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const results = [];

      const itemRegex = /<div class="manga-item loop-item group\/manga-item">[\s\S]*?<a href="([^"]+)"[^>]*>[\s\S]*?<img[^>]*src="([^"]+)"[^>]*>[\s\S]*?<h3[^>]*>[\s\S]*?<a href="[^"]*"[^>]*>([^<]+)<\/a>[\s\S]*?<span class="text-sm pt-0\.5">([\d.]+)<\/span>[\s\S]*?<span class="text-sm pt-0\.5">([\d.]+)<\/span>/g;

      let match;
      while ((match = itemRegex.exec(html)) !== null) {
        const url = match[1];
        const cover = match[2];
        const title = match[3].trim();
        const rating = match[4];
        const pages = match[5];

        const idMatch = url.match(/\/hentai\/([^\/]+)\/?/);
        const id = idMatch ? idMatch[1] : '';

        results.push({
          id: id,
          slug: id,
          title: title,
          cover: cover,
          rating: rating,
          pages: pages,
        });
      }

      return results;

    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const url = `https://hentairead.com/hentai/${identifier}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const titleMatch = html.match(/<h1[^>]*>([^<]+)<\/h1>/);
      const title = titleMatch ? titleMatch[1].trim() : identifier;

      const altMatch = html.match(/<h2[^>]*class="[^"]*text-gray-200[^"]*"[^>]*>([^<]+)<\/h2>/);
      const altTitle = altMatch ? altMatch[1].trim() : '';

      const coverMatch = html.match(/<img[^>]*fetchpriority="high"[^>]*src="([^"]+)"[^>]*>/);
      const cover = coverMatch ? coverMatch[1] : '';

      const descMatch = html.match(/<div class="entry-content entry-content-single"[^>]*>[\s\S]*?<p>([\s\S]*?)<\/p>/);
      let description = descMatch ? descMatch[1].trim() : '';
      description = description.replace(/<[^>]*>/g, '').trim();

      const typeMatch = html.match(/<div class="text-primary font-medium text-md">Category:<\/div>[\s\S]*?<a[^>]*>[\s\S]*?<span[^>]*>([^<]+)<\/span>/);
      const type = typeMatch ? typeMatch[1].trim() : '';

      const artistMatch = html.match(/<div class="text-primary font-medium text-md">Artist:<\/div>[\s\S]*?<a[^>]*>[\s\S]*?<span[^>]*>([^<]+)<\/span>/);
      const author = artistMatch ? artistMatch[1].trim() : '';

      const tags = [];
      const tagRegex = /<div class="text-primary font-medium text-md">Tags:<\/div>[\s\S]*?<a[^>]*href="[^"]*"[^>]*>[\s\S]*?<span[^>]*>([^<]+)<\/span>/g;
      let tagMatch;
      while ((tagMatch = tagRegex.exec(html)) !== null) {
        tags.push(tagMatch[1].trim());
      }

      const pagesMatch = html.match(/<div class="text-primary font-medium text-md">Pages:<\/div>[\s\S]*?<span[^>]*>([\d.]+)<\/span>/);
      const pages = pagesMatch ? parseInt(pagesMatch[1]) : 0;

      const chapters = [];
      const chapterRegex = /<li class="chapter-image-item[^"]*" data-page="(\d+)">[\s\S]*?<a href="([^"]+)"[^>]*>/g;
      let chapterMatch;
      while ((chapterMatch = chapterRegex.exec(html)) !== null) {
        const pageNumber = parseInt(chapterMatch[1]);
        const pageUrl = chapterMatch[2];
        
        chapters.push({
          number: pageNumber,
          slug: `p/${pageNumber}`,
          title: `Page ${pageNumber}`,
          url: pageUrl,
        });
      }

      return {
        id: identifier,
        slug: identifier,
        title: title,
        altTitle: altTitle,
        description: description,
        cover: cover,
        author: author,
        type: type,
        tags: tags,
        chapters: chapters,
        pages: pages,
      };

    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'HentaiRead',
    version: '1.0.0',
    description: 'HentaiRead extension',
    author: 'wzread',
    cover: './extension_cover.png',
    type: 'adult'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const url = `https://hentairead.com/hentai/${bookId}/english/p/${chapterNumber}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      const images = [];

      const imageRegex = /"src":"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g;
      let imageMatch;
      while ((imageMatch = imageRegex.exec(html)) !== null) {
        const imgUrl = imageMatch[1];
        if (imgUrl && !imgUrl.includes('cover')) {
          images.push(imgUrl);
        }
      }

      if (images.length === 0) {
        const altImageRegex = /https:\/\/henread\.xyz\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
        const altMatches = html.match(altImageRegex);
        if (altMatches) {
          images.push(...altMatches);
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
      const url = `https://hentairead.com/hentai/${bookId}/english/p/${chapter}/`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await response.text();

      let images = [];

      const imageRegex = /"src":"([^"]+\.(?:jpg|jpeg|png|webp|gif))"/g;
      let imageMatch;
      while ((imageMatch = imageRegex.exec(html)) !== null) {
        const imgUrl = imageMatch[1];
        if (imgUrl && !imgUrl.includes('cover')) {
          images.push(imgUrl);
        }
      }

      if (images.length === 0) {
        const altImageRegex = /https:\/\/henread\.xyz\/[^"]+\.(?:jpg|jpeg|png|webp|gif)/g;
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