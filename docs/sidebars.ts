import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    {
      type: 'category',
      label: 'Murali',
      collapsed: false,
      items: [
        'intro',
        'installation',
        'package-structure',
        'roadmap',
      ],
    },
    {
      type: 'category',
      label: 'Murali Engine',
      collapsed: false,
      items: [
        'visual-foundations',
        'first-scene',
        'mental-model',
        'which-api-should-i-use',
        'python-bindings',
        'common-first-mistakes',
        'coordinate-system',
        'video-formats',
      ],
    },
    {
      type: 'category',
      label: 'Murali Kit',
      collapsed: false,
      items: [
        'murali-kit',
      ],
    },
    {
      type: 'category',
      label: 'Future Packages',
      collapsed: true,
      items: [
        'core-and-addons',
      ],
    },
    'ai-visualization/index',
    {
      type: 'category',
      label: '3D',
      collapsed: false,
      items: [
        'examples/three_d',
        '3d-prop-assets',
      ],
    },
    {
      type: 'category',
      label: 'Tattvas',
      link: {type: 'doc', id: 'tattvas/index'},
      items: [
        'tattvas/properties',
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
    'animations',
    'timelines',
    'scene-views',
    'scene-and-app',
    'export-and-capture',
    'camera',
    'updaters',
    {
      type: 'category',
      label: 'Examples',
      link: {type: 'doc', id: 'examples/index'},
      items: [
        'examples/showcase',
        'examples/reference-videos',
        'examples/basics',
        'examples/text_and_math',
        'examples/animation',
        'examples/graphs_and_fields',
        'examples/dynamics'
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
};

export default sidebars;
