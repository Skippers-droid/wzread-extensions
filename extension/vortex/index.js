const extension = {
  search: async (query) => {
    try {
      const searchTerm = encodeURIComponent(query);
      const url = `https://api.vortexscans.org/api/query?searchTerm=${searchTerm}&perPage=12`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data.posts || !Array.isArray(data.posts)) {
        return [];
      }

      return data.posts.map((item) => ({
        id: item.id,
        slug: item.slug,
        title: item.postTitle,
        cover: item.featuredImage || '',
        status: item.seriesStatus,
        type: item.seriesType,
        latestChapter: item.chapters?.[0]?.number || null,
        synopsis: item.postContent || '',
        genres: item.genres?.map((g) => g.name) || [],
      }));
    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    try {
      const url = `https://api.vortexscans.org/api/series/${identifier}`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data || !data.post) {
        throw new Error('Series not found');
      }

      const post = data.post;

      const chapters = (post.chapters || []).map((ch) => ({
        number: ch.number,
        slug: ch.slug,
        title: ch.title || `Chapter ${ch.number}`,
        id: ch.id,
      }));

      chapters.sort((a, b) => a.number - b.number);

      const genres = post.genres?.map((g) => g.name) || [];

      return {
        id: post.id,
        slug: post.slug,
        title: post.postTitle,
        altTitle: post.alternativeTitles || '',
        description: post.postContent || '',
        cover: post.featuredImage || '',
        author: post.author || '',
        artist: post.artist || '',
        status: post.seriesStatus,
        type: post.seriesType,
        genres: genres,
        chapters: chapters,
        totalViews: post.totalViews || 0,
        averageRating: post.averageRating || 0,
        totalRatings: post.totalRatings || 0,
        bookmarksCount: post._count?.bookmarks || 0,
        isPinned: post.isPinned || false,
        isHot: post.hot || false,
        isNew: post.isNew || false,
        releaseDate: post.releaseDate || null,
        updatedAt: post.updatedAt,
        createdAt: post.createdAt,
      };
    } catch (error) {
      throw error;
    }
  },

  extension_info: () => ({
    name: 'Vortex Scans',
    version: '1.0.0',
    description: 'Vortex Scans extension for reading comics, manga, manhua, and manhwa.',
    author: 'wzread',
    cover: './extension_cover.png'
  }),

  chapter: async (bookId, chapterNumber) => {
    try {
      const seriesUrl = `https://api.vortexscans.org/api/series/${bookId}`;

      const seriesResponse = await fetch(seriesUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const seriesData = await seriesResponse.json();

      if (!seriesData || !seriesData.post) {
        throw new Error('Series not found');
      }

      const chapters = seriesData.post.chapters || [];

      const chapter = chapters.find((ch) => ch.number === parseInt(chapterNumber, 10));

      if (!chapter) {
        throw new Error(`Chapter ${chapterNumber} not found`);
      }

      const chapterUrl = `https://api.vortexscans.org/api/series/${bookId}/chapter/${chapter.slug}`;

      const chapterResponse = await fetch(chapterUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const chapterData = await chapterResponse.json();

      if (!chapterData || !chapterData.images || !Array.isArray(chapterData.images)) {
        return {
          number: chapterNumber,
          totalImages: 0,
          images: [],
        };
      }

      const images = chapterData.images.map((img) => ({
        url: img.url,
        width: img.width || null,
        height: img.height || null,
      }));

      return {
        number: chapterNumber,
        totalImages: images.length,
        images: images,
        title: chapter.title || `Chapter ${chapterNumber}`,
        createdAt: chapter.createdAt || null,
        updatedAt: chapter.updatedAt || null,
        isLocked: chapter.isLocked || false,
        isAccessible: chapter.isAccessible || false,
        isPurchased: chapter.hasPurchased || false,
        price: chapter.price || null,
        finalPrice: chapter.finalPrice || chapter.price || null,
        likesCount: chapter.likesCount || 0,
        commentsCount: chapter._count?.comments || 0,
      };
    } catch (error) {
      throw error;
    }
  },

  getChapterImages: async (bookId, chapter, page = 1, perPage = 10) => {
    try {
      const seriesUrl = `https://api.vortexscans.org/api/series/${bookId}`;

      const seriesResponse = await fetch(seriesUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const seriesData = await seriesResponse.json();

      if (!seriesData || !seriesData.post) {
        throw new Error('Series not found');
      }

      const chapters = seriesData.post.chapters || [];

      const chapterObj = chapters.find((ch) => ch.number === parseInt(chapter, 10));

      if (!chapterObj) {
        throw new Error(`Chapter ${chapter} not found`);
      }

      const chapterUrl = `https://api.vortexscans.org/api/series/${bookId}/chapter/${chapterObj.slug}`;

      const chapterResponse = await fetch(chapterUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const chapterData = await chapterResponse.json();

      if (!chapterData || !chapterData.images || !Array.isArray(chapterData.images)) {
        return {
          images: [],
          total: 0,
          page: page,
          perPage: perPage,
          hasMore: false,
        };
      }

      const images = chapterData.images.map((img) => img.url);
      const total = images.length;
      const start = (page - 1) * perPage;
      const end = start + perPage;
      const paginatedImages = images.slice(start, end);

      return {
        images: paginatedImages,
        total: total,
        page: page,
        perPage: perPage,
        hasMore: end < total,
      };
    } catch (error) {
      throw error;
    }
  },

  getPopular: async () => {
    try {
      const url = `https://api.vortexscans.org/api/query?popular=true&perPage=20`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data.posts || !Array.isArray(data.posts)) {
        return [];
      }

      return data.posts.map((item) => ({
        id: item.id,
        slug: item.slug,
        title: item.postTitle,
        cover: item.featuredImage || '',
        status: item.seriesStatus,
        type: item.seriesType,
        latestChapter: item.chapters?.[0]?.number || null,
        synopsis: item.postContent || '',
        genres: item.genres?.map((g) => g.name) || [],
        averageRating: item.averageRating || 0,
      }));
    } catch (error) {
      throw error;
    }
  },

  getLatest: async () => {
    try {
      const url = `https://api.vortexscans.org/api/query?latest=true&perPage=20`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data.posts || !Array.isArray(data.posts)) {
        return [];
      }

      return data.posts.map((item) => ({
        id: item.id,
        slug: item.slug,
        title: item.postTitle,
        cover: item.featuredImage || '',
        status: item.seriesStatus,
        type: item.seriesType,
        latestChapter: item.chapters?.[0]?.number || null,
        synopsis: item.postContent || '',
        genres: item.genres?.map((g) => g.name) || [],
        averageRating: item.averageRating || 0,
        updatedAt: item.updatedAt,
      }));
    } catch (error) {
      throw error;
    }
  },

  getFiltered: async (filter = {}) => {
    try {
      let url = `https://api.vortexscans.org/api/query?perPage=24`;

      const params = new URLSearchParams();

      if (filter.status) {
        params.append('status', filter.status.toLowerCase());
      }

      if (filter.type) {
        params.append('type', filter.type.toLowerCase());
      }

      if (filter.order) {
        switch (filter.order) {
          case 'asc':
            params.append('order', 'asc');
            break;
          case 'desc':
            params.append('order', 'desc');
            break;
          case 'title':
            params.append('sortBy', 'title');
            break;
          case 'popular':
            params.append('sortBy', 'popular');
            break;
          default:
            params.append('order', 'desc');
        }
      } else {
        params.append('order', 'desc');
      }

      if (filter.genre) {
        params.append('genres[]', filter.genre);
      }

      if (filter.search) {
        params.append('searchTerm', filter.search);
      }

      const queryString = params.toString();
      if (queryString) {
        url += `&${queryString}`;
      }

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data.posts || !Array.isArray(data.posts)) {
        return [];
      }

      return data.posts.map((item) => ({
        id: item.id,
        slug: item.slug,
        title: item.postTitle,
        cover: item.featuredImage || '',
        status: item.seriesStatus,
        type: item.seriesType,
        latestChapter: item.chapters?.[0]?.number || null,
        synopsis: item.postContent || '',
        genres: item.genres?.map((g) => g.name) || [],
        averageRating: item.averageRating || 0,
      }));
    } catch (error) {
      throw error;
    }
  },

  getChapterPages: async (bookId, chapterNumber) => {
    try {
      const seriesUrl = `https://api.vortexscans.org/api/series/${bookId}`;

      const seriesResponse = await fetch(seriesUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const seriesData = await seriesResponse.json();

      if (!seriesData || !seriesData.post) {
        throw new Error('Series not found');
      }

      const chapters = seriesData.post.chapters || [];

      const chapter = chapters.find((ch) => ch.number === parseInt(chapterNumber, 10));

      if (!chapter) {
        throw new Error(`Chapter ${chapterNumber} not found`);
      }

      const chapterUrl = `https://api.vortexscans.org/api/series/${bookId}/chapter/${chapter.slug}`;

      const chapterResponse = await fetch(chapterUrl, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const chapterData = await chapterResponse.json();

      if (!chapterData || !chapterData.images || !Array.isArray(chapterData.images)) {
        return {
          chapterNumber: parseInt(chapterNumber, 10),
          title: chapter.title || '',
          images: [],
          hasNext: false,
          hasPrev: false,
        };
      }

      const images = chapterData.images.map((img) => ({
        url: img.url,
        width: img.width || 0,
        height: img.height || 0,
        index: img.index || 0,
      }));

      const chapterIndex = chapters.findIndex((ch) => ch.number === parseInt(chapterNumber, 10));

      return {
        chapterNumber: parseInt(chapterNumber, 10),
        title: chapter.title || `Chapter ${chapterNumber}`,
        images: images,
        hasNext: chapterIndex > 0,
        hasPrev: chapterIndex < chapters.length - 1,
        nextChapter: chapterIndex > 0 ? chapters[chapterIndex - 1].number : null,
        prevChapter: chapterIndex < chapters.length - 1 ? chapters[chapterIndex + 1].number : null,
      };
    } catch (error) {
      throw error;
    }
  },

  getSeriesChapters: async (bookId, limit = 50, offset = 0) => {
    try {
      const url = `https://api.vortexscans.org/api/series/${bookId}`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data || !data.post) {
        throw new Error('Series not found');
      }

      const chapters = data.post.chapters || [];

      const sortedChapters = chapters.sort((a, b) => b.number - a.number);

      const total = sortedChapters.length;
      const start = offset;
      const end = Math.min(start + limit, total);
      const paginatedChapters = sortedChapters.slice(start, end);

      return {
        chapters: paginatedChapters.map((ch) => ({
          number: ch.number,
          slug: ch.slug,
          title: ch.title || `Chapter ${ch.number}`,
          id: ch.id,
          isLocked: ch.isLocked || false,
          isAccessible: ch.isAccessible || false,
          isPurchased: ch.hasPurchased || false,
          price: ch.price || null,
          finalPrice: ch.finalPrice || ch.price || null,
          likesCount: ch.likesCount || 0,
          commentsCount: ch._count?.comments || 0,
          createdAt: ch.createdAt,
          updatedAt: ch.updatedAt,
        })),
        total: total,
        hasMore: end < total,
      };
    } catch (error) {
      throw error;
    }
  },

  getSeriesGenres: async () => {
    try {
      const url = `https://api.vortexscans.org/api/genres`;

      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });

      const data = await response.json();

      if (!data || !Array.isArray(data)) {
        return [];
      }

      return data.map((genre) => ({
        id: genre.id,
        name: genre.name,
        color: genre.color || null,
      }));
    } catch (error) {
      throw error;
    }
  },
};

export default extension;