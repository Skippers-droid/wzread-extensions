const extension = {
  search: async (query) => {
    try {
      const searchQuery = encodeURIComponent(query);
      const response = await fetch(`https://api.qimanga.com/api/v1/series/search?q=${searchQuery}&perPage=20`, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      const data = await response.json();
      
      if (!data.data || data.data.length === 0) {
        return [];
      }
      
      const results = data.data.map(item => ({
        id: item.id,
        slug: item.slug,
        title: item.title,
        description: item.alternativeTitles || '',
        cover: item.cover
      }));
      
      return results;
      
    } catch (error) {
      throw error;
    }
  },

  manga_info: async (identifier) => {
    console.log('[Qiscan] Fetching manga info for:', identifier);
    
    const seriesResponse = await fetch(`https://api.qimanga.com/api/v1/series/${identifier}`, {
      headers: {
        'User-Agent': '{user-agent}'
      }
    });
    const seriesData = await seriesResponse.json();
    console.log('[Qiscan] Series data received:', seriesData.title);
    
    let chapters = [];
    try {
      const chaptersResponse = await fetch(
        `https://api.qimanga.com/api/v1/series/${identifier}/chapters?page=1&perPage=100&sort=desc`,
        {
          headers: {
            'User-Agent': '{user-agent}'
          }
        }
      );
      const chaptersData = await chaptersResponse.json();
      console.log('[Qiscan] Chapters count:', chaptersData.data?.length || 0);
      
      if (chaptersData.data && Array.isArray(chaptersData.data)) {
        chapters = chaptersData.data.map(ch => ({
          number: ch.number,
          slug: ch.slug,
          title: ch.title || `Chapter ${ch.number}`,
          id: ch.id,
          isFree: ch.isFree,
          requiresPurchase: ch.requiresPurchase
        }));
        chapters.sort((a, b) => a.number - b.number);
      }
    } catch (error) {
      console.error('[Qiscan] Failed to fetch chapters:', error);
    }
    
    return {
      id: seriesData.id,
      slug: seriesData.slug,
      title: seriesData.title,
      description: seriesData.description ? seriesData.description.replace(/<[^>]*>/g, '') : '',
      cover: seriesData.cover,
      author: seriesData.author || '',
      status: seriesData.status,
      genres: seriesData.genres?.map(g => g.name) || [],
      chapters: chapters,
      type: seriesData.type,
      alternativeTitles: seriesData.alternativeTitles
    };
  },

  extension_info: () => ({
    name: 'Qiscan',
    version: '1.0.0',
    description: '',
    author: 'wzread',
    cover: './extension_cover.png'
  }),

  chapter: async (bookId, chapterNumber) => {
    console.log('[Qiscan] Fetching chapter:', { bookId, chapterNumber });
    
    const slug = typeof chapterNumber === 'string' ? chapterNumber : `chapter-${chapterNumber}`;
    const url = `https://api.qimanga.com/api/v1/series/${bookId}/chapters/${slug}`;
    console.log('[Qiscan] Chapter URL:', url);
    
    const response = await fetch(url, {
      headers: {
        'User-Agent': '{user-agent}'
      }
    });
    const data = await response.json();
    console.log('[Qiscan] Chapter data received:', {
      number: data.number,
      totalImages: data.totalImages || 0,
      imagesCount: data.images?.length || 0
    });
    
    return {
      number: data.number,
      totalImages: data.totalImages || 0,
      images: data.images?.sort((a, b) => a.order - b.order).map(img => img.url) || []
    };
  },

  getChapterImages: async (bookId, chapter) => {
    console.log('[Qiscan] getChapterImages called with:', { bookId, chapter });
    
    try {
      let slug = chapter;
      if (typeof chapter === 'number' || !isNaN(parseInt(chapter))) {
        slug = `chapter-${chapter}`;
      }
      
      const url = `https://api.qimanga.com/api/v1/series/${bookId}/chapters/${slug}`;
      console.log('[Qiscan] getChapterImages URL:', url);
      
      const response = await fetch(url, {
        headers: {
          'User-Agent': '{user-agent}'
        }
      });
      
      if (!response.ok) {
        console.error('[Qiscan] Chapter fetch failed:', response.status, response.statusText);
        throw new Error(`Failed to fetch chapter: ${response.status} ${response.statusText}`);
      }
      
      const data = await response.json();
      console.log('[Qiscan] Chapter data received:', {
        id: data.id,
        number: data.number,
        totalImages: data.totalImages || 0,
        imagesCount: data.images?.length || 0,
        hasImages: !!data.images,
        isArray: Array.isArray(data.images),
        firstImageUrl: data.images && data.images.length > 0 ? data.images[0].url : 'No images'
      });
      
      if (!data.images || !Array.isArray(data.images)) {
        console.error('[Qiscan] No images array in response:', data);
        return [];
      }
      
      const images = data.images
        .sort((a, b) => a.order - b.order)
        .map(img => img.url)
        .filter(url => url && url.length > 0);
      
      console.log('[Qiscan] Extracted images:', {
        count: images.length,
        first: images[0] || 'No images',
        last: images[images.length - 1] || 'No images'
      });
      
      return images;
      
    } catch (error) {
      console.error('[Qiscan] Failed to get chapter images:', {
        error: error.message,
        bookId,
        chapter,
        stack: error.stack
      });
      throw error;
    }
  }
};

export default extension;