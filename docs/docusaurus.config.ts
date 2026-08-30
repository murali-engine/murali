import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Murali',
  tagline: 'A Python animation engine for math, AI, and teaching visuals',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://muraliengine.com',
  baseUrl: '/',

  organizationName: 'murali-engine',
  projectName: 'murali',

  onBrokenLinks: 'throw',
  markdown: {
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
          lastVersion: 'current',
          versions: {
            current: {
              label: '0.3.0 🚧',
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
    announcementBar: {
      id: 'python-authoring-unstable-0-5',
      content:
        'Authoring moved to Python; Rust stays the engine. That change followed public demand. These APIs are <strong>unstable until 0.5.0</strong>. For Rust scene authoring, use the <a href="/docs/0.2.4/intro">0.2.4 docs</a> and the <a href="https://crates.io/crates/murali/0.2.4">murali 0.2.4</a> crate.',
      backgroundColor: '#3d2b00',
      textColor: '#ffe7a3',
      isCloseable: true,
    },
    navbar: {
      title: 'Murali',
      logo: {
        alt: 'Murali Logo',
        src: 'img/murali_logo_light.png',
        srcDark: 'img/murali_logo_dark.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Docs',
        },
        {to: '/blog', label: 'Blog', position: 'left'},
        {href: 'https://github.com/murali-engine/murali-kit/tree/main/examples', label: 'Examples', position: 'left'},
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
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['python', 'rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
