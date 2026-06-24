const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const url = `https://comix.to/api/v1/manga?keyword=${searchQuery}&limit=10&content_rating%5B%5D=safe&content_rating%5B%5D=suggestive`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (data.status !== 'ok' || !data.result || !data.result.items) {
        return [];
      }

      const results = data.result.items.map((item) => ({
        id: item.hid,
        slug: item.hid,
        title: item.title,
        cover: item.poster?.large || item.poster?.medium || '',
        status: item.status,
        type: item.type,
        latestChapter: item.latestChapter,
        synopsis: item.synopsis || '',
      }));

      return results;

    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const detailUrl = `https://comix.to/api/v1/manga?hid=${identifier}`;

      const detailResponse = await fetch(detailUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const detailData = await detailResponse.json();

      if (detailData.status !== 'ok' || !detailData.result || !detailData.result.items || detailData.result.items.length === 0) {
        throw new Error('Manga not found');
      }

      const manga = detailData.result.items[0];

      const pageUrl = `https://comix.to/title/${identifier}`;

      const pageResponse = await fetch(pageUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const html = await pageResponse.text();

      const initialDataMatch = html.match(/<script type="application\/json" id="initial-data">([\s\S]*?)<\/script>/);
      let chapters = [];

      if (initialDataMatch) {
        try {
          const initialData = JSON.parse(initialDataMatch[1]);
          
          for (const [key, value] of Object.entries(initialData.queries || {})) {
            if (key.includes('chapters') && value && Array.isArray(value)) {
              chapters = value.map((ch) => ({
                number: ch.number || ch.chapterNumber || 0,
                slug: ch.slug || `chapter-${ch.number}`,
                title: ch.title || `Chapter ${ch.number}`,
                id: ch.id,
                url: ch.url || `/title/${identifier}/${ch.id}-chapter-${ch.number}`,
              }));
              break;
            }
          }
        } catch (e) {}
      }

      if (chapters.length === 0) {
        try {
          const chaptersUrl = `https://comix.to/api/v1/manga/${manga.id}/chapters?limit=100&order=asc`;

          const chaptersResponse = await fetch(chaptersUrl, {
            headers: {
              'User-Agent': '{user-agent}'
            }
          });

          const chaptersData = await chaptersResponse.json();

          if (chaptersData.status === 'ok' && chaptersData.result && Array.isArray(chaptersData.result.items)) {
            chapters = chaptersData.result.items.map((ch) => ({
              number: ch.number || 0,
              slug: ch.slug || `chapter-${ch.number}`,
              title: ch.title || `Chapter ${ch.number}`,
              id: ch.id,
              url: `/title/${identifier}/${ch.id}-chapter-${ch.number}`,
            }));
          }
        } catch (e) {}
      }

      chapters.sort((a, b) => a.number - b.number);

      const genres = manga.genres?.map((g) => g.title) || [];
      const demographics = manga.demographics?.map((d) => d.title) || [];

      return {
        id: manga.hid,
        slug: manga.hid,
        title: manga.title,
        altTitle: manga.altTitles?.join(', ') || '',
        description: manga.synopsis || '',
        cover: manga.poster?.large || manga.poster?.medium || '',
        author: manga.authors?.map((a) => a.title).join(', ') || '',
        artist: manga.artists?.map((a) => a.title).join(', ') || '',
        status: manga.status,
        type: manga.type,
        genres: [...genres, ...demographics],
        chapters: chapters,
      };

    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'Comix',
    version: '1.0.0',
    description: 'Comix.to extension',
    author: 'wzread',
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const detailUrl = `https://comix.to/api/v1/manga?hid=${bookId}`;

      const detailResponse = await fetch(detailUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const detailData = await detailResponse.json();

      if (detailData.status !== 'ok' || !detailData.result || !detailData.result.items || detailData.result.items.length === 0) {
        throw new Error('Manga not found');
      }

      const manga = detailData.result.items[0];
      const mangaId = manga.id;

      const chaptersUrl = `https://comix.to/api/v1/manga/${mangaId}/chapters?limit=200&order=asc`;

      const chaptersResponse = await fetch(chaptersUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const chaptersData = await chaptersResponse.json();

      if (chaptersData.status !== 'ok' || !chaptersData.result || !Array.isArray(chaptersData.result.items)) {
        throw new Error('Chapters not found');
      }

      const chapter = chaptersData.result.items.find((ch) => ch.number === parseInt(chapterNumber, 10));

      if (!chapter) {
        throw new Error(`Chapter ${chapterNumber} not found`);
      }

      const imageUrl = `https://comix.to/api/v1/chapter/${chapter.id}/images`;

      const imageResponse = await fetch(imageUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const imageData = await imageResponse.json();

      let images = [];

      if (imageData.status === 'ok' && imageData.result && Array.isArray(imageData.result.items)) {
        images = imageData.result.items.map((img) => img.url || img);
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
      const detailUrl = `https://comix.to/api/v1/manga?hid=${bookId}`;

      const detailResponse = await fetch(detailUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const detailData = await detailResponse.json();

      if (detailData.status !== 'ok' || !detailData.result || !detailData.result.items || detailData.result.items.length === 0) {
        throw new Error('Manga not found');
      }

      const manga = detailData.result.items[0];
      const mangaId = manga.id;

      const chaptersUrl = `https://comix.to/api/v1/manga/${mangaId}/chapters?limit=200&order=asc`;

      const chaptersResponse = await fetch(chaptersUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const chaptersData = await chaptersResponse.json();

      if (chaptersData.status !== 'ok' || !chaptersData.result || !Array.isArray(chaptersData.result.items)) {
        throw new Error('Chapters not found');
      }

      const chapterObj = chaptersData.result.items.find((ch) => ch.number === parseInt(chapter, 10));

      if (!chapterObj) {
        throw new Error(`Chapter ${chapter} not found`);
      }

      const imageUrl = `https://comix.to/api/v1/chapter/${chapterObj.id}/images`;

      const imageResponse = await fetch(imageUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const imageData = await imageResponse.json();

      let images = [];

      if (imageData.status === 'ok' && imageData.result && Array.isArray(imageData.result.items)) {
        images = imageData.result.items.map((img) => img.url || img);
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