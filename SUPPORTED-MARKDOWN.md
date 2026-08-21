# Supported Markdown

This page is generated from awl's Markdown dialect roster. Do not edit the
generated block; run `scripts/regen-reference.sh` after changing the roster.

<!-- GENERATED:supported-markdown:BEGIN -->
awl keeps documents as ordinary plain text. This page describes the syntax awl renders; unsupported syntax remains editable text. It does not promise parity with an unnamed Markdown dialect.

## Syntax awl renders

### ATX headings

```markdown
# Heading
## Subheading
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A six-level heading ladder with larger type. | The caret or a selection on the line shows its # markers. | Heading; Cycle heading | Core CommonMark |

### Bold, italic, and bold italic

```markdown
**bold** and *italic* and ***both***
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Weighted or italic text. | The caret or a selection on the line shows the delimiters. | Bold; Italic | Core CommonMark |

### Inline code

```markdown
Use `code` here.
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Monospace text with a quiet pill. | The caret or a selection on the line shows the backticks. | Inline code | Core CommonMark |

### Fenced code blocks

````markdown
```rust
let answer = 42;
```
````

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A monospace block panel. Recognised language tags add quiet syntax roles. | The caret anywhere in the block, or a selection touching it, shows its fences and language tag. | Code block | Core CommonMark |

### Indented code blocks

```markdown
    plain code
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A plain monospace code block. | Its source remains visible; it has no fence to conceal. | — | Core CommonMark |

### Blockquotes

```markdown
> quoted text
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Quiet quoted text with a hanging quotation mark. | The caret or a selection on the line shows the > marker. | Blockquote | Core CommonMark |

### Bulleted and numbered lists

```markdown
- first
- second

1. one
2. two
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A list with quiet markers. | The caret or a selection on a line shows its marker. | Bullet list; Numbered list | Core CommonMark |

### Task lists

```markdown
- [ ] open
- [x] done
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Open and completed task markers; completed text recedes. | The source remains editable on its line. | Task list | Widely used extension |

### Inline links

```markdown
[awl](https://awl-editor.fly.dev)
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Link text in the document ink. | The caret or a selection on the line shows brackets and destination. | Insert link… | Core CommonMark |

### Images

```markdown
![A description](image.png)
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A local image when inline images are enabled; otherwise its source stays visible. | The caret or a selection on the line overlays editable source on a dimmed image. | Paste can insert an image reference | Core CommonMark |

### YAML frontmatter

```markdown
---
title: A note
lang: en
---
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Metadata is hidden away from the block when WYSIWYG is on. | The caret anywhere in the block, or a selection touching it, reveals the whole block. | Tag document language | Widely used extension |

### Thematic breaks

```markdown
---
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A centred section-break ornament. | The caret or a selection on the line shows the typed break. | — | Core CommonMark |

### Tables

```markdown
| Name | Value |
| --- | --- |
| awl | editor |
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A composed grid; header cells and structure stay legible. | The caret row, or rows touched by a selection, show their source over the grid. | Align table | Widely used extension |

### Highlight

```markdown
==highlighted text==
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Full-ink text on a warm highlighter wash. | The caret or a selection on the line shows the == delimiters. | Highlight | awl-specific extension |

### Strikethrough

```markdown
~~removed text~~
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| Receding text with a strike line. | The caret or a selection on the line shows the ~~ delimiters. | Strikethrough | Widely used extension |

### Footnotes

```markdown
A note[^source]

[^source]: Its text
```

| awl renders | Caret and selection | Formatting command | Portability |
|---|---|---|---|
| A quiet superscript reference and a composed numbered definition. | The caret or a selection on the line shows the exact label, marker, and indentation. | Insert footnote | Widely used extension |

## Not supported / deliberately different

### Setext headings

```markdown
A heading
---
```

Not a heading in awl. A run of three or more dashes renders as a thematic break.

### Reference-style links

```markdown
[text][key]

[key]: https://example.com
```

No special link preview. The source remains editable text.

### Raw HTML

```markdown
<mark>text</mark>
```

No HTML rendering. The source remains editable text.

<!-- GENERATED:supported-markdown:END -->
