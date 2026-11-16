import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({
      fallback: undefined,
      precompress: false,
      strict: true
    }),
    prerender: {
      entries: ['*'],
      handleMissingId: 'ignore'
    }
  }
};
