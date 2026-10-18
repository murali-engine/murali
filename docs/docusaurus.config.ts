import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Murali',
  tagline: 'A Rust-powered math animation engine',
  favicon: 'img/favicon.ico',
  headTags: [
    {
      tagName: 'link',
      attributes: {
        rel: 'icon',
        type: 'image/png',
        sizes: '32x32',
        href: '/img/favicon-32x32.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'icon',
        type: 'image/png',
        sizes: '16x16',
        href: '/img/favicon-16x16.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'apple-touch-icon',
        sizes: '180x180',
        href: '/img/apple-touch-icon.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'manifest',
        href: '/img/site.webmanifest',
      },
    },
  ],

  future: {
    v4: true,
  },

  url: 'https://muraliengine.com',
  baseUrl: '/',

  organizationName: 'murali-engine',
  projectName: 'murali',

  onBrokenLinks: 'throw',
  markdown: {
    // The docs still use the legacy ":::note Title" form. v4 turns that off.
    mdx1Compat: {
      admonitions: true,
    },
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/murali-engine/murali/tree/main/docs/',
          lastVersion: '0.2.4',
          versions: {
            current: {
              label: 'Next 🚧',
              path: 'next',
            },
          },
        },
        blog: {
          showReadingTime: true,
          feedOptions: {
            type: ['rss', 'atom'],
            xslt: true,
          },
          editUrl: 'https://github.com/murali-engine/murali/tree/main/docs/',
          onInlineTags: 'warn',
          onInlineAuthors: 'warn',
          onUntruncatedBlogPosts: 'warn',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      defaultMode: 'dark',
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Murali',
      logo: {
        alt: 'Murali Logo',
        src: 'img/murali_mark.svg',
        srcDark: 'img/murali_mark.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Docs',
        },
        {to: '/blog', label: 'Blog', position: 'left'},
        {href: 'https://github.com/murali-engine/murali/blob/main/examples/README.md', label: 'Examples', position: 'left'},
        {href: 'https://www.youtube.com/@muraliengine', label: 'Showcase', position: 'left'},
        {
          type: 'docsVersionDropdown',
          position: 'right',
        },
        {
          href: 'https://github.com/murali-engine/murali',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Getting Started', to: '/docs/intro'},
          ],
        },
        {
          title: 'More',
          items: [
            {label: 'Blog', to: '/blog'},
            {label: 'Design Guidelines', to: '/design-guidelines'},
            {label: 'Kavriq', href: 'https://kavriq.com/'},
            {label: 'GitHub', href: 'https://github.com/murali-engine/murali'},
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Murali. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.oneDark,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
