# Murali Terminology Guide

This document defines standard terminology usage across all Murali documentation.

## Core Terms

### Tattva
- **Definition:** Any visual object in a scene (shape, text, composite, etc.)
- **Etymology:** Sanskrit word meaning "element" or "essence"
- **Usage:**
  - Capitalize when referring to the concept: "A Tattva is any object..."
  - Lowercase in code: `scene.add_tattva(...)`
  - Plural: "tattvas" (lowercase)
  - Possessive: "tattva's properties"

### Scene
- **Definition:** Container for all tattvas, the global timeline, and camera
- **Usage:**
  - Capitalize when referring to the concept: "The Scene owns all tattvas"
  - Lowercase in code: `let mut scene = Scene::new()`
  - Think of it as: "A stage in a theater"

### Timeline
- **Definition:** The scene's single global schedule of animations and callbacks over absolute time
- **Usage:**
  - Capitalize when referring to the concept: "A Timeline defines how properties evolve"
  - Lowercase in code: `let mut timeline = Timeline::new()`

### Clip
- **Definition:** A reusable animation schedule authored from its own local time `0.0`
- **Usage:**
  - Capitalize when referring to the concept: "A Clip defines one locally timed section"
  - Lowercase in code: `let mut clip = Clip::new()`
  - Say "append a clip," "overlay a clip," or "place a clip"
  - Do not describe clips as independent clocks; composition flattens them onto the timeline

### World Space
- **Definition:** Coordinate system using mathematical units, not pixels
- **Usage:**
  - Always "world space" (lowercase, two words)
  - "world units" when referring to measurements
  - "world coordinates" when referring to positions

## Animation Terms

### Preview vs Export
- **Preview:** Interactive window mode for development
  - Command: `cargo run --example my_scene`
  - Purpose: Fast iteration, debugging
- **Export:** Headless video rendering mode
  - Command: `cargo run --example my_scene -- --export`
  - Purpose: Final output, deterministic frames

### Animation Methods

#### Timeline composition and playback
- **`timeline.append(clip)`** - Place a clip at the composition cursor and advance it
- **`timeline.overlay(clip)`** - Share the most recent append origin
- **`timeline.place_at(time, clip)`** - Place a clip at an absolute scene time
- **`scene.play(timeline)`** - Install the scene's global timeline

**Recommendation in docs:** Use direct timeline authoring for small scenes and clips for independently timed sections.

## Capitalization Rules

### In Prose
- Capitalize when introducing concepts: "A Tattva is..."
- Capitalize in headings: "## Scene and Timeline"
- Lowercase when used as common nouns: "add tattvas to the scene"

### In Code Examples
- Always use actual Rust casing: `Scene`, `Timeline`, `Clip`, `TattvaId`
- Method names are lowercase: `add_tattva()`, `append()`, `play()`, `animate()`

## Common Phrases

### Preferred
- "add a tattva to the scene"
- "build a timeline"
- "author a clip from local time zero"
- "compose clips onto the timeline"
- "schedule animations"
- "world-space coordinates"
- "preview mode" / "export mode"
- "scene time" (the current time in the scene)

### Avoid
- "create a tattva" (use "add" instead - tattvas are added to scenes)
- "pixel coordinates" (always use world space)
- "frame-based animation" (Murali is time-based)
- "render mode" (use "preview" or "export")

## Cross-Reference Terms

When linking between docs:
- "See [Tattvas](./tattvas/)" - capitalize in link text
- "Learn about [animations](./animations)" - lowercase for action-oriented links
- "Read the [Scene and App](./scene-and-app) guide" - capitalize proper titles

## Consistency Checklist

When writing or reviewing docs, ensure:
- [ ] Tattva/Scene/Timeline/Clip capitalized when introducing concepts
- [ ] Code examples use correct Rust casing
- [ ] "preview" and "export" used consistently (not "render mode")
- [ ] "world space" / "world units" used instead of "pixels"
- [ ] Clip examples distinguish local time from absolute timeline time
- [ ] Links use consistent capitalization

## Examples

### Good
```markdown
A **Tattva** is any visual object in your scene. You add tattvas using `scene.add_tattva()`.

The **Scene** owns all tattvas and its global timeline. Create a scene with `Scene::new()`.

Use **preview mode** for development and **export mode** for final output.
```

### Needs Improvement
```markdown
A **tattva** is any visual object. You create tattvas with `scene.add_tattva()`.

The **scene** owns everything. Use render mode to see your animation.
```

## Updates

This terminology guide should be updated when:
- New core concepts are introduced
- API naming changes
- User feedback indicates confusion
- Documentation patterns evolve
