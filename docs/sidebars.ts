import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    {
      type: 'category',
      label: 'Start',
      collapsed: false,
      items: [
        'intro',
        'installation',
        'first-scene',
        'mental-model',
        'package-structure',
        'common-first-mistakes',
      ],
    },
    {
      type: 'category',
      label: 'Write scenes',
      collapsed: false,
      items: [
        'visual-foundations',
        'coordinate-system',
        'video-formats',
        'tattvas/index',
        'animations',
        'timelines',
        'camera',
        'scene-views',
        'export-and-capture',
        '3d-prop-assets',
      ],
    },
    {
      type: 'category',
      label: 'Teaching views',
      collapsed: false,
      items: [
        'murali-kit',
        'ai-visualization/index',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsed: false,
      items: [
        'python-bindings',
        'which-api-should-i-use',
        'roadmap',
      ],
    },
    {
      type: 'category',
      label: 'Internals',
      collapsed: true,
      items: [
        'rust-first-scene',
        'scene-and-app',
        'updaters',
        {
          type: 'category',
          label: 'Tattva details',
          link: {type: 'doc', id: 'tattvas/properties'},
          items: [
            'tattvas/primitives',
            'tattvas/text',
            'tattvas/tables',
            'tattvas/composite',
            'tattvas/opening',
            'tattvas/graphs',
            'tattvas/math',
            'tattvas/layout',
            'tattvas/storytelling',
            'tattvas/ai',
            'tattvas/utility',
          ],
        },
        {
          type: 'category',
          label: '3D',
          items: [
            'examples/three_d',
          ],
        },
        {
          type: 'category',
          label: 'Rust examples',
          link: {type: 'doc', id: 'examples/index'},
          items: [
            'examples/showcase',
            'examples/reference-videos',
            'examples/basics',
            'examples/text_and_math',
            'examples/animation',
            'examples/graphs_and_fields',
            'examples/dynamics',
          ],
        },
        {
          type: 'category',
          label: 'Beta & Experimental',
          link: {type: 'doc', id: 'beta/index'},
          items: [
            'beta/experimental-features',
            'beta/chat-input-box',
          ],
        },
        'useful-features',
        {
          type: 'category',
          label: 'Architecture',
          items: [
            'architecture/overview',
            'architecture/architecture',
            'architecture/scene-timeline',
            'architecture/tattva',
            'architecture/dirty-flags',
            'architecture/projection',
            'architecture/ecs',
            'architecture/renderer',
            'architecture/text-and-latex',
            'architecture/end-to-end-flow',
          ],
        },
        {
          type: 'category',
          label: 'Feature Internals',
          items: [
            'feature-internals/overview',
            'feature-internals/stepwise',
            'feature-internals/neural-network',
          ],
        },
      ],
    },
  ],
};

export default sidebars;
