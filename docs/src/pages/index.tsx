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
    label: 'Start',
  },
  {
    title: 'Write scenes',
    description: 'Coordinates, tattvas, timelines, camera, and export.',
    to: '/docs/tattvas/',
    label: 'Authoring',
  },
  {
    title: 'Teaching views',
    description: 'Themes, named colors, and lesson diagrams from the kit.',
    to: '/docs/murali-kit',
    label: 'Kit',
  },
  {
    title: 'Internals',
    description: 'Rust architecture and runtime notes, when you need them.',
    to: '/docs/architecture/overview/',
    label: 'Runtime',
  },
];

const highlights: Card[] = [
  {
    title: 'Python authoring',
    description: 'Write scenes, lessons, and integrations in Python.',
  },
  {
    title: 'Time-driven animation',
    description: 'Deterministic scenes built as explicit functions of time.',
  },
  {
    title: 'GPU-native rendering',
    description: 'A Rust core renders through wgpu across Metal, Vulkan, and DirectX 12.',
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
    title: 'Playful shapes animation',
    description: 'A polished motion study with many shapes moving through coordinated timing.',
    embedUrl: 'https://www.youtube.com/embed/rzQZHta2PQM',
  },
  {
    title: 'Tattva motion and camera follow',
    description: 'A moving tattva with a smooth camera follow and continuous rotation.',
    embedUrl: 'https://www.youtube.com/embed/W8WQQbSo70Y',
  },
];

const constructs = ['Scene', 'Timeline', 'Tattvas', 'Kit'];

function SurfaceCard({ title, description, to, href, label }: Card) {
  const content = (
    <>
      {label ? <span className={styles.cardLabel}>{label}</span> : null}
      <Heading as="h3" className={clsx('card__title', styles.cardTitle)}>
        {title}
      </Heading>
      <p className={styles.cardDescription}>{description}</p>
      <span className={styles.cardCta}>{href ? 'Open resource' : 'Read more'} →</span>
    </>
  );

  if (href) {
    return (
      <div className="col col--6 margin-bottom--lg">
        <Link className={clsx('card', styles.surfaceCard)} href={href}>
          {content}
        </Link>
      </div>
    );
  }

  return (
    <div className="col col--6 margin-bottom--lg">
      <Link className={clsx('card', styles.surfaceCard)} to={to!}>
        {content}
      </Link>
    </div>
  );
}

function HomepageHeader() {
  const logoLightUrl = useBaseUrl('img/murali_logo_light.png');
  const logoDarkUrl = useBaseUrl('img/murali_logo_dark.png');

  return (
    <header className={styles.hero}>
      <div className="container">
        <div className={clsx('row', styles.heroInner)}>
          <div className={clsx('col col--6', styles.heroCopy)}>
            <p className={styles.eyebrow}>Python animation engine</p>
            <Heading as="h1" className={styles.heroTitle}>
              <span>Build precise math and AI animations</span>
              <span className={styles.heroTitleAccent}>in Python.</span>
            </Heading>
            <p className={styles.heroSubtitle}>
              Author mathematical and AI explainers with deterministic timelines.
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
              <Link
                className={clsx('button button--secondary button--lg', styles.secondaryAction)}
                href="https://github.com/murali-engine/murali"
              >
                View GitHub
              </Link>
            </div>
          </div>
          <div className={clsx('col col--6', styles.heroArt)} aria-hidden="true">
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
            <div className="row">
              {highlights.map((item) => (
                <div key={item.title} className="col col--4 margin-bottom--lg">
                  <div className={clsx('card', styles.featureCard)}>
                    <div className="card__body">
                      <Heading as="h3" className={styles.featureTitle}>
                        {item.title}
                      </Heading>
                      <p className={styles.featureBody}>{item.description}</p>
                    </div>
                  </div>
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
              body="One guide: start in Python, write scenes, add teaching views, then open Internals only if you are changing the runtime."
            />
            <div className="row">
              {pathways.map((item) => (
                <SurfaceCard key={item.title} {...item} />
              ))}
            </div>
          </div>
        </section>

        <section className={clsx(styles.section, styles.sectionTint)}>
          <div className="container">
            <div className="row">
              <div className="col col--8">
                <SectionIntro
                  eyebrow="Explore"
                  title="Documentation, internals, examples, and showcase videos"
                  body="Start with the Python walkthrough, then kit examples. Architecture notes and Rust examples stay available for engine work."
                />
              </div>
              <div className="col col--4">
                <div className={clsx('card', styles.note)}>
                  <div className="card__body">
                    <p className={styles.noteTitle}>Suggested reading order</p>
                    <ol className={styles.noteList}>
                      <li>Introduction</li>
                      <li>Your first scene</li>
                      <li>Tattvas and animations</li>
                      <li>Teaching views</li>
                      <li>Python API</li>
                    </ol>
                  </div>
                </div>
              </div>
            </div>

            <div className="row">
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
