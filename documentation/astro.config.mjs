import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://ubihome.github.io',
  outDir: './site',
  integrations: [
    starlight({
      title: 'UbiHome',
      description:
        'UbiHome is a single executable that allows you to integrate any device running an OS into your smart home.',
      favicon: '/assets/favicon.png',
      logo: {
        src: './src/content/docs/assets/logo.png',
        alt: 'UbiHome'
      },
      editLink: {
        baseUrl: 'https://github.com/UbiHome/UbiHome/edit/main/documentation/src/content/docs/'
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/UbiHome/UbiHome'
        }
      ],
      sidebar: [
        { label: 'Home', link: '/' },
        { label: 'Getting started', link: '/getting_started/' },
        {
          label: 'Features',
          items: [
            { label: 'Overview', link: '/features/' },
            { label: 'Connectivity', autogenerate: { directory: 'features/connectivity' } },
            { label: 'Platforms', autogenerate: { directory: 'features/platforms' } },
            { label: 'Components', autogenerate: { directory: 'features/components' } },
            { label: 'Utilities', autogenerate: { directory: 'features/utilities' } }
          ]
        },
        {
          label: 'Examples',
          autogenerate: { directory: 'examples' }
        },
        { label: 'Help', link: '/help/' }
      ]
    }),
    mdx()
  ]
});
