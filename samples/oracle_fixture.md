# Oracle fixture

A wrapped paragraph that is deliberately long, spanning well past the edge of the printed column so it soft-wraps into more than one visual row, which is exactly the shape a vertical-motion sweep needs in order to cross a wrap boundary in both directions, from every column and every goal-x in the spread, without ever landing back on the row it started from.

## A bullet list

- first bullet, short
- second bullet, a little longer than the first one
- third bullet, shorter again

## A numbered list

1. first ordered item
2. second ordered item
3. third ordered item

Stepping with `visual_line_down` must land strictly below the row it started on, never back onto itself.

```rust
fn step(row: usize) -> usize {
    row + 1
}
```

## A small table

| World  | Face   | Mood  |
|--------|--------|-------|
| Tawny  | Bitter | warm  |
| Mopoke | Klee   | quiet |

这是一行中文文本，用于校验双宽字符的换行与列宽边界情况。
