---
title: Design Guidelines
description: Visual direction for Murali docs and related web surfaces.
---

# Design Guidelines

This page captures the visual direction we want for Murali docs and related web surfaces.

The goal is not to freeze every pixel. The goal is to preserve the **taste**, **constraints**, and **interaction principles** that make the product feel sharp, modern, and coherent over time.

## Design intent

Murali should feel:

- precise
- modern
- technical, but not cold
- polished without becoming flashy
- clean enough for documentation

The aesthetic target is a balance of:

- Apple-like restraint and spacing
- modern developer-tool clarity
- subtle motion and depth
- strong typography and hierarchy

## Core principles

### 1. Clarity first

Documentation is the product here. Visual styling should improve scanability, not compete with content.

- Prefer fewer, stronger visual decisions.
- Reduce noise before adding ornament.
- Use whitespace as structure.
- Let headings and section rhythm do most of the work.

### 2. Polished, not loud

We want polish, not visual chaos.

- Avoid overly saturated rainbow palettes.
- Avoid stacking too many gradients, borders, glows, and shadows in one place.
- Use motion sparingly and deliberately.
- Prefer subtle depth over decorative clutter.

### 3. Strong hierarchy

The page should be understandable at a glance.

- Hero: one dominant idea.
- Sections: one clear purpose each.
- Cards: compact, scannable, and consistent.
- Supporting copy: quieter than headlines.

### 4. Documentation-friendly design

This is not a startup marketing splash page.

- Layouts should remain readable for long-form technical audiences.
- Interactive effects should never reduce legibility.
- Surfaces should support text, not overwhelm it.
- Navigation between sections should feel obvious and calm.

## Color direction

We maintain separate light and dark mode palettes, but both should feel like the same product.

### Light mode

Use light surfaces with crisp contrast and restrained accent color.

- Base background: soft off-white, never pure sterile white everywhere.
- Surface cards: translucent white or lightly tinted white.
- Accent: indigo / blue family.
- Text: near-black or deep slate.

Suggested palette characteristics:

- background: `#f5f5f7` range
- primary text: `#111827` range
- muted text: `#4b5563` range
- accent: `#2563eb` range

### Dark mode

Dark mode should feel rich and calm, not neon.

- Base background: charcoal or very deep gray, not flat black everywhere.
- Surfaces: slightly lifted dark panels.
- Accent: softened cool blue.
- Text: high contrast, but not full white for every element.

Suggested palette characteristics:

- background: `#0b0b0d` range
- surface: `#1c1c1e` range
- primary text: `#f5f5f7` range
- muted text: translucent white
- accent: `#7aa2ff` range

## Typography

Typography should carry most of the interface quality.

- Large headlines with tight tracking.
- High contrast between headline and body copy.
- Quiet eyebrow labels in uppercase.
- Comfortable line height for all paragraph text.

### Headings

- bold to extra-bold
- slightly negative letter spacing
- short, direct language

### Body copy

- medium contrast
- generous line height
- avoid long dense paragraphs on landing pages

## Layout rules

### Hero section

The hero should be clean and decisive.

- one main headline
- one supporting paragraph
- one primary CTA
- one secondary CTA at most
- avoid competing panels unless they add real information density

The hero should feel spacious, not crowded.

### Horizontal sections

Each horizontal section should have a distinct job.

- overview
- getting started / pathways
- exploration / resources

Avoid repeating the same visual pattern too many times in a row.

If one section uses cards, the next section should introduce a different rhythm:

- text + cards
- split layout
- lighter list treatment

### Cards

Cards should feel refined and lightweight.

- large radius
- subtle border
- soft shadow
- minimal decorative chrome
- clear hover state

Avoid:

- heavy gradients inside every card
- loud glow effects everywhere
- too many badges, icons, and dividers in one card

## Motion guidelines

Motion should support polish, not spectacle.

- use short easing-based transitions
- hover lift should be subtle
- buttons can shimmer only if the effect remains restrained
- avoid continuous motion that distracts from reading

Preferred motion qualities:

- smooth
- soft
- slightly springy or cubic-bezier refined
- never hyperactive

## Effects and depth

Use visual depth carefully.

Allowed:

- soft shadows
- subtle glassmorphism
- mild backdrop blur
- gentle gradient backgrounds

Avoid:

- excessive glow stacks
- multiple layered animations on every component
- visually noisy textures
- strong gradients behind dense text

## Landing page checklist

Before considering a landing page section done, verify:

- the hierarchy is obvious without reading every line
- the hero has one dominant message
- sections feel distinct without becoming busy
- cards are fully clickable and clearly interactive
- dark mode and light mode both feel intentional
- text remains readable at a glance
- hover states feel polished but restrained
- page rhythm feels calm, not cramped

## Anti-patterns

Do not drift into these:

- AI-generated “safe SaaS” look
- overdecorated gradient overload
- too many boxed panels above the fold
- repeated card grids with no visual pacing
- hero sections with multiple competing focal points
- color palettes that look unrelated between light and dark mode

## Current working direction

For Murali docs, the preferred direction is:

- Apple-like clean sections
- large whitespace
- restrained surfaces
- crisp typography
- limited accent color
- polished but quiet interaction

This should be the baseline unless there is a strong reason to deviate.

## Origin

This direction was distilled from our homepage design iteration work, including external ideation help, and is being preserved here so future design changes stay consistent with the decisions we already made.
