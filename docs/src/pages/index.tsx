import type { ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import ThemedImage from '@theme/ThemedImage';

import styles from './index.module.css';

type Card = {
  title: string;
  description: string;
  to?: string;
  href?: string;
  label?: string;
};

const pathways: Card[] = [
  {
    title: 'Get started',
    description: 'Install murali-kit and write your first Python scene.',
    to: '/docs/intro',
    label: 'Quickstart',
  },
  {
    title: 'Write a scene',
    description: 'Scene, tattvas, timeline, preview, and export in Python.',
    to: '/docs/first-scene',
    label: 'Tutorial',
  },
  {
    title: 'Murali Kit',
    description: 'Themes, named colors, and reusable teaching views.',
    to: '/docs/murali-kit',
    label: 'Authoring',
  },
  {
    title: 'Engine internals',
    description: 'Rust architecture, renderer, and contributor notes.',
    to: '/docs/architecture/overview/',
    label: 'Internals',
  },
];

const highlights: Card[] = [
  {
    title: 'Python authoring',
    description: 'Write scenes in Python. The Rust engine renders them.',
  },
  {
    title: 'Time-driven animation',
    description: 'Deterministic scenes built as explicit functions of time.',
  },
  {
    title: 'GPU-native rendering',
    description: 'Powered by wgpu across Metal, Vulkan, and DirectX 12.',
  },
];

const resources: Card[] = [
  {
    title: 'Python examples',
    description: 'Runnable kit examples for shapes, motion, math, and AI lessons.',
    href: 'https://github.com/murali-engine/murali-kit/tree/main/examples',
    label: 'Examples',
  },
  {
    title: 'YouTube showcase',
    description: 'Watch Murali showcase videos and visual demos on the official channel.',
    href: 'https://www.youtube.com/@muraliengine',
    label: 'Videos',
  },
  {
    title: 'Python API',
    description: 'The murali_engine surface for scenes, tattvas, timelines, and export.',
    to: '/docs/python-bindings',
  },
  {
    title: 'Engine internals',
    description: 'Architecture notes for people changing the Rust runtime.',
    to: '/docs/architecture/overview/',
  },
];

type VideoShowcase = {
  title: string;
  description: string;
  embedUrl: string;
};

const showcaseVideos: VideoShowcase[] = [
  {
    title: 'Payful shapes animation',
    description: 'A polished animation showing large no of shapes doing movement',
    embedUrl: 'https://www.youtube.com/embed/rzQZHta2PQM',
  },
  {
    title: 'Tattva move animation, alongwith camera smooth movement',
    description: 'We show a Tattva moving, and then also the camera following that Tattva, while it keeps rotating',
    embedUrl: 'https://www.youtube.com/embed/W8WQQbSo70Y',
  },
];

const constructs = ['Scene', 'Timeline', 'Tattvas', 'Kit'];

function SurfaceCard({ title, description, to, href, label }: Card) {
  const content = (
    <>
      {label ? <span className={styles.cardLabel}>{label}</span> : null}
      <Heading as="h3" className={styles.cardTitle}>
        {title}
      </Heading>
      <p className={styles.cardDescription}>{description}</p>
      <span className={styles.cardCta}>{href ? 'Open resource' : 'Read more'} →</span>
    </>
  );

  if (href) {
    return (
      <Link className={styles.card} href={href}>
        {content}
      </Link>
    );
  }

  return (
    <Link className={styles.card} to={to!}>
      {content}
    </Link>
  );
}

function HomepageHeader() {
  const logoLightUrl = useBaseUrl('img/murali_logo_light.png');
  const logoDarkUrl = useBaseUrl('img/murali_logo_dark.png');

  return (
    <header className={styles.hero}>
      <div className={clsx('container', styles.heroInner)}>
        <div className={styles.heroCopy}>
          <p className={styles.eyebrow}>Python animation engine</p>
          <Heading as="h1" className={styles.heroTitle}>
            Build precise teaching visuals.
          </Heading>
          <p className={styles.heroSubtitle}>
            Write mathematical and AI animation in Python. Murali renders on a modern GPU runtime
            with deterministic, timeline-driven scenes.
          </p>
          <p className={styles.heroNote}>
            Developed in part for{' '}
            <Link href="https://kavriq.com/">Kavriq</Link>, where production use will continue to
            help shape the path forward for Murali.
          </p>
          <div className={styles.constructRow} aria-label="Murali building blocks">
            {constructs.map((item) => (
              <span key={item} className={styles.constructChip}>
                {item}
              </span>
            ))}
          </div>
          <div className={styles.heroActions}>
            <Link className="button button--primary button--lg" to="/docs/intro">
              Start with the docs
            </Link>
            <Link className={styles.secondaryAction} href="https://github.com/murali-engine/murali">
              View GitHub
            </Link>
          </div>
        </div>
        <div className={styles.heroArt} aria-hidden="true">
          <ThemedImage
            className={styles.heroLogo}
            alt="Murali logo"
            sources={{
              light: logoLightUrl,
              dark: logoDarkUrl,
            }}
          />
        </div>
      </div>
    </header>
  );
}

function SectionIntro({
  eyebrow,
  title,
  body,
}: {
  eyebrow: string;
  title: string;
  body: string;
}) {
  return (
    <div className={styles.sectionIntro}>
      <p className={styles.sectionEyebrow}>{eyebrow}</p>
      <Heading as="h2" className={styles.sectionTitle}>
        {title}
      </Heading>
      <p className={styles.sectionBody}>{body}</p>
    </div>
  );
}

function VideoSection() {
  return (
    <section className={clsx(styles.section, styles.sectionPlain)}>
      <div className="container">
        <SectionIntro
          eyebrow="Showcase"
          title="See Murali in action"
          body="A few short examples of the kinds of visuals you can build with Murali."
        />
        <div className={styles.videoGrid}>
          {showcaseVideos.map((video) => (
            <div key={video.title} className={styles.videoCard}>
              <div className={styles.videoFrame}>
                <iframe
                  src={video.embedUrl}
                  title={video.title}
                  loading="lazy"
                  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                  allowFullScreen
                />
              </div>
              <Heading as="h3" className={styles.featureTitle}>
                {video.title}
              </Heading>
              <p className={styles.featureBody}>{video.description}</p>
            </div>
          ))}
        </div>
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
        <section className={clsx(styles.section, styles.sectionSoft)}>
          <div className="container">
            <SectionIntro
              eyebrow="Overview"
              title="A cleaner way to build mathematical animation"
              body="Write scenes in Python. Murali keeps tattvas, timelines, and rendering as one deterministic system, with kit components for lessons and themes."
            />
            <div className={styles.threeUp}>
              {highlights.map((item) => (
                <div key={item.title} className={styles.featureCard}>
                  <Heading as="h3" className={styles.featureTitle}>
                    {item.title}
                  </Heading>
                  <p className={styles.featureBody}>{item.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <VideoSection />

        <section className={clsx(styles.section, styles.sectionPlain)}>
          <div className="container">
            <SectionIntro
              eyebrow="Paths"
              title="Choose the right place to begin"
              body="Start in Python, add kit teaching views when you need them, and open Core Rust Engine only if you are changing the runtime."
            />
            <div className={styles.cardGrid}>
              {pathways.map((item) => (
                <SurfaceCard key={item.title} {...item} />
              ))}
            </div>
          </div>
        </section>

        <section className={clsx(styles.section, styles.sectionTint)}>
          <div className="container">
            <div className={styles.split}>
              <SectionIntro
                eyebrow="Explore"
                title="Documentation, internals, examples, and showcase videos"
                body="Start with the Python walkthrough, then kit examples. Architecture notes and Rust examples stay available for engine work."
              />
              <div className={styles.note}>
                <p className={styles.noteTitle}>Suggested reading order</p>
                <ol className={styles.noteList}>
                  <li>Introduction</li>
                  <li>Your first scene</li>
                  <li>Mental model</li>
                  <li>Murali Kit</li>
                  <li>Python API</li>
                </ol>
              </div>
            </div>

            <div className={styles.cardGrid}>
              {resources.map((item) => (
                <SurfaceCard key={item.title} {...item} />
              ))}
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
