---
sidebar_position: 2
---

# Chat Input Box

:::caution Beta API

`ChatInputBox` is published under `composite::beta`. Its API, defaults, layout behavior, returned
IDs, and visual styling may change while it is refined across real explainer videos.

:::

`ChatInputBox` is a small composite for scenes where text is typed inside a chat-style input box or
message bubble. It is useful for prompt-entry moments, dialogue scenes, and LLM explainers.

It creates ordinary Murali tattvas:

1. a single-boundary chat bubble body with a configurable bottom tip
2. a `Label` initialized for typewriter animation
3. an optional send button

The bubble body and tip are one primitive, so the fill and stroke share a continuous outline instead
of looking like a separate triangle attached to a rounded rectangle. The component does not own a
timeline. It returns IDs so the host scene can decide what appears, when text is typed, and how the
box participates in the larger animation.

## Import

```rust
use murali::frontend::sangrah::composite::beta::{
    ChatInputBox, ChatInputTipSide,
};
```

## Basic Usage

```rust
use glam::{Vec3, vec2};
use murali::frontend::animation::Ease;
use murali::frontend::sangrah::composite::beta::{
    ChatInputBox, ChatInputTipSide,
};

let ids = ChatInputBox::new("Why is the sky blue?")
    .with_size(5.8, 0.82)
    .with_tip(ChatInputTipSide::Right, 0.42, 0.28)
    .with_tip_inset(0.72)
    .with_text_inset(vec2(0.38, 0.0))
    .with_send_button(true)
    .add_to_scene(&mut scene, Vec3::new(0.0, 0.8, 0.0));

timeline
    .animate(ids.text)
    .at(0.4)
    .for_duration(1.5)
    .ease(Ease::Linear)
    .typewrite_text()
    .spawn();
```

The text label starts with `char_reveal = 0.0` and `typewriter_mode = true`, so it is ready for
`.typewrite_text()`.

## Tip Side

Use `ChatInputTipSide::Left` or `ChatInputTipSide::Right` to place the bottom tip:

```rust
ChatInputBox::new("User prompt")
    .with_tip(ChatInputTipSide::Right, 0.42, 0.28);

ChatInputBox::new("Assistant response")
    .with_tip(ChatInputTipSide::Left, 0.42, 0.28);
```

`with_tip_inset(...)` controls how far the tip center sits from the corresponding edge.

## Returned IDs

`ChatInputBoxIds` exposes:

| Field | Meaning |
| --- | --- |
| `bubble` | single filled/stroked chat bubble body, including the bottom tip |
| `text` | typewriter-ready label |
| `send_button` | optional rounded send-button body |

Use `ids.all()` when you want to fade or move the whole composite, and animate `ids.text` when you
want the typed prompt effect.

## Runnable Example

Run the reference example locally:

```bash
cargo run --example chat_input_box
```

The example shows both left and right bottom tips, an optional send button, and separate
typewriter timings for user and assistant text.
