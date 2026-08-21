# Local Brand Fonts

Place licensed, non-redistributable font files in `assets/fonts/private/`. That directory is
ignored by Git.

The `kavriq_opening` example looks for:

```text
assets/fonts/private/Satoshi-Bold.ttf
```

Set `KAVRIQ_FONT` to use another local TrueType or OpenType font path. Public checkouts fall back
to Murali's embedded Inter font when neither source is available.
