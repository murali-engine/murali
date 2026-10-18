import type { ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import { Highlight, themes } from 'prism-react-renderer';

import styles from './index.module.css';

const pipeline = [
  {
    number: '01',
    name: 'Scene',
    title: 'Compose the world',
    body: 'Build typed scenes from shapes, text, mathematics, 3D objects, and reusable tattvas.',
  },
  {
    number: '02',
    name: 'Timeline',
    title: 'Choreograph time',
    body: 'Schedule motion, easing, cameras, callbacks, and transitions on one deterministic clock.',
  },
  {
    number: '03',
    name: 'Projection',
    title: 'Resolve every frame',
    body: 'Turn authored intent into explicit render primitives with predictable layout and geometry.',
  },
  {
    number: '04',
    name: 'Renderer',
    title: 'Draw on the GPU',
    body: 'Render through wgpu across Metal, Vulkan, and DirectX for native, computation-heavy work.',
  },
];

const learningPaths = [
  {
    label: 'Start',
    title: 'Create your first scene',
    body: 'Install the crate, place a few tattvas, preview the result, and understand the basic loop.',
    to: '/docs/intro',
  },
  {
    label: 'Build',
    title: 'Learn the visual vocabulary',
    body: 'Explore primitives, text, layout, mathematics, AI diagrams, and storytelling components.',
    to: '/docs/tattvas/',
  },
  {
    label: 'Deep dive',
    title: 'Understand the engine',
    body: 'Read how scenes, timelines, projection, ECS state, and the renderer fit together.',
    to: '/docs/architecture/overview/',
  },
];

const codeExample = `use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::{scene::Scene, timeline::Timeline};
use murali::frontend::{animation::Ease, collection::primitives::circle::Circle};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let dot = scene.add_tattva(
        Circle::new(0.5, 48, RED_B),
        Vec3::new(-2.0, 0.0, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline.animate(dot)
        .for_duration(1.2)
        .ease(Ease::OutCubic)
        .move_to(Vec3::new(2.0, 0.0, 0.0))
        .spawn();

    scene.play(timeline)?;
    App::new()?.with_scene(scene).run_app()
}`;

function HomepageHeader() {
  const logoUrl = useBaseUrl('img/murali_mark.svg');

  return (
    <header className={styles.hero}>
      <div className={styles.heroGrid} aria-hidden="true" />
      <div className={clsx('container', styles.heroInner)}>
        <div className={styles.heroCopy}>
          <p className={styles.identityLine}>
            <span className={styles.statusDot} />
            Murali is the Rust engine
          </p>
          <Heading as="h1" className={styles.heroTitle}>
            Animation,
            <span> native by design.</span>
          </Heading>
          <p className={styles.heroSubtitle}>
            A deterministic animation engine for mathematical visuals, technical storytelling,
            and workloads that belong close to the metal.
          </p>
          <div className={styles.heroActions}>
            <Link className={styles.primaryAction} to="/docs/intro">
              Start building <span aria-hidden="true">→</span>
            </Link>
            <Link className={styles.secondaryAction} href="https://github.com/murali-engine/murali">
              View source
            </Link>
          </div>
          <div className={styles.heroFacts} aria-label="Murali highlights">
            <span>Rust</span>
            <span>wgpu</span>
            <span>Deterministic timelines</span>
            <span>Native export</span>
          </div>
        </div>

        <div className={styles.heroStage} aria-hidden="true">
          <div className={styles.stageTopline}>
            <span>SCENE / MAIN</span>
            <span>00:04.20</span>
          </div>
          <div className={styles.markField}>
            <span className={styles.orbitOne} />
            <span className={styles.orbitTwo} />
            <img className={styles.heroMark} alt="" src={logoUrl} />
          </div>
          <div className={styles.timeline}>
            <div className={styles.timelineLabels}>
              <span>0s</span>
              <span>1s</span>
              <span>2s</span>
              <span>3s</span>
            </div>
            <div className={styles.timelineTrack}>
              <span className={styles.timelineFill} />
              <span className={clsx(styles.keyframe, styles.keyframeOne)} />
              <span className={clsx(styles.keyframe, styles.keyframeTwo)} />
              <span className={clsx(styles.keyframe, styles.keyframeThree)} />
              <span className={styles.playhead} />
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}

function SectionHeading({
  label,
  title,
  body,
}: {
  label: string;
  title: string;
  body: string;
}) {
  return (
    <div className={styles.sectionHeading}>
      <p className={styles.sectionLabel}>{label}</p>
      <Heading as="h2" className={styles.sectionTitle}>
        {title}
      </Heading>
      <p className={styles.sectionBody}>{body}</p>
    </div>
  );
}

function PipelineSection() {
  return (
    <section className={styles.pipelineSection}>
      <div className="container">
        <SectionHeading
          label="The native pipeline"
          title="Built like an engine. Authored like a story."
          body="Murali keeps composition, time, projection, and rendering explicit—so complex animation remains understandable as it grows."
        />
        <div className={styles.pipelineGrid}>
          {pipeline.map((item) => (
            <article className={styles.pipelineCard} key={item.number}>
              <div className={styles.pipelineMeta}>
                <span>{item.number}</span>
                <span>{item.name}</span>
              </div>
              <Heading as="h3">{item.title}</Heading>
              <p>{item.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function CodeSection() {
  return (
    <section className={styles.codeSection}>
      <div className={clsx('container', styles.codeLayout)}>
        <div className={styles.codeCopy}>
          <p className={styles.sectionLabel}>Rust from the first frame</p>
          <Heading as="h2" className={styles.sectionTitle}>
            One scene. One timeline. Every frame reproducible.
          </Heading>
          <p className={styles.sectionBody}>
            Murali is for work where explicit state, predictable performance, and native control
            matter. Your animation is regular Rust—not a hidden graph or proprietary project file.
          </p>
          <ul className={styles.checkList}>
            <li>Typed scene construction</li>
            <li>Seekable deterministic timelines</li>
            <li>GPU-backed rendering and export</li>
          </ul>
          <Link className={styles.textLink} to="/docs/first-scene">
            Walk through the first scene <span aria-hidden="true">→</span>
          </Link>
        </div>
        <div className={styles.codeWindow}>
          <div className={styles.codeBar}>
            <span>hello_motion.rs</span>
            <span>Rust</span>
          </div>
          <Highlight theme={themes.oneDark} code={codeExample.trim()} language="rust">
            {({ tokens, getLineProps, getTokenProps }) => (
              <pre className={styles.codeBlock}>
                <code>
                  {tokens.map((line, index) => (
                    <div key={index} {...getLineProps({ line })}>
                      {line.map((token, tokenIndex) => (
                        <span key={tokenIndex} {...getTokenProps({ token })} />
                      ))}
                    </div>
                  ))}
                </code>
              </pre>
            )}
          </Highlight>
        </div>
      </div>
    </section>
  );
}

function ShowcaseSection() {
  const shapesUrl = useBaseUrl('img/showcase/shapes.png');
  const motionUrl = useBaseUrl('img/showcase/animation_showcase.png');

  return (
    <section className={styles.showcaseSection}>
      <div className="container">
        <div className={styles.showcaseHeader}>
          <SectionHeading
            label="Rendered with Murali"
            title="Visual systems that move with intent."
            body="From geometric choreography to camera-led scenes, Murali keeps every result tied to authored code and a deterministic timeline."
          />
          <Link className={styles.textLink} href="https://www.youtube.com/@muraliengine">
            View the showcase <span aria-hidden="true">↗</span>
          </Link>
        </div>
        <div className={styles.showcaseGrid}>
          <Link className={styles.showcaseCard} href="https://youtu.be/rzQZHta2PQM">
            <div className={styles.showcaseImageWrap}>
              <img src={shapesUrl} alt="Colorful geometric shapes rendered by Murali" />
              <span className={styles.playButton} aria-hidden="true">▶</span>
            </div>
            <div className={styles.showcaseCaption}>
              <span>Geometry study</span>
              <Heading as="h3">Playful shapes, precisely choreographed</Heading>
            </div>
          </Link>
          <Link className={styles.showcaseCard} href="https://youtu.be/W8WQQbSo70Y">
            <div className={styles.showcaseImageWrap}>
              <img src={motionUrl} alt="Murali animation with camera movement" />
              <span className={styles.playButton} aria-hidden="true">▶</span>
            </div>
            <div className={styles.showcaseCaption}>
              <span>Camera and motion</span>
              <Heading as="h3">A moving tattva through a moving frame</Heading>
            </div>
          </Link>
        </div>
      </div>
    </section>
  );
}

function IdentitySection() {
  return (
    <section className={styles.identitySection}>
      <div className={clsx('container', styles.identityLayout)}>
        <div className={styles.identityCopy}>
          <p className={styles.sectionLabel}>Clear by design</p>
          <Heading as="h2" className={styles.sectionTitle}>
            Murali means Rust.
          </Heading>
          <p className={styles.sectionBody}>
            This project is the native animation engine: a Rust crate, a wgpu renderer, and a
            lower-level foundation for demanding visual workloads.
          </p>
          <p className={styles.identityAside}>
            Veena remains a separate browser-native engine for JavaScript and web authoring.
          </p>
        </div>
        <div className={styles.engineStack}>
          <div className={clsx(styles.engineRow, styles.engineRowActive)}>
            <span className={styles.engineMonogram}>M</span>
            <div>
              <strong>Murali</strong>
              <span>Rust · Native · wgpu</span>
            </div>
            <span className={styles.currentTag}>You are here</span>
          </div>
          <Link className={styles.engineRow} href="https://veenaengine.com/">
            <span className={styles.veenaMark} aria-hidden="true"><i /><i /><i /></span>
            <div>
              <strong>Veena</strong>
              <span>JavaScript · Browser · Web platform</span>
            </div>
            <span aria-hidden="true">↗</span>
          </Link>
        </div>
      </div>
    </section>
  );
}

function LearningSection() {
  return (
    <section className={styles.learningSection}>
      <div className="container">
        <SectionHeading
          label="Documentation"
          title="Begin with a scene. Go as deep as you need."
          body="The guides move from a first preview to reusable visual systems and the engine internals beneath them."
        />
        <div className={styles.learningGrid}>
          {learningPaths.map((path, index) => (
            <Link className={styles.learningCard} to={path.to} key={path.title}>
              <span className={styles.learningMeta}>
                <span className={styles.learningNumber}>0{index + 1}</span>
                <span className={styles.learningLabel}>{path.label}</span>
              </span>
              <Heading as="h3">{path.title}</Heading>
              <p>{path.body}</p>
              <span className={styles.learningArrow} aria-hidden="true">→</span>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}

function ClosingSection() {
  const logoUrl = useBaseUrl('img/murali_mark.svg');

  return (
    <section className={styles.closingSection}>
      <div className={clsx('container', styles.closingInner)}>
        <img src={logoUrl} alt="" aria-hidden="true" />
        <div>
          <p className={styles.sectionLabel}>Start at frame zero</p>
          <Heading as="h2">Build the difficult thing clearly.</Heading>
        </div>
        <Link className={styles.closingAction} to="/docs/intro">
          Read the introduction <span aria-hidden="true">→</span>
        </Link>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();

  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <HomepageHeader />
      <main className={styles.main}>
        <PipelineSection />
        <CodeSection />
        <ShowcaseSection />
        <IdentitySection />
        <LearningSection />
        <ClosingSection />
      </main>
    </Layout>
  );
}
