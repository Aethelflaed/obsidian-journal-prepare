# obsidian-journal-prepare

![workflow](https://github.com/Aethelflaed/obsidian-journal-prepare/actions/workflows/rust.yml/badge.svg?branch=main)


Prepare a [obsidian](https://obsidian.md/) journal entries by adding references to the week in
the daily journal entries.

The weeks each get a page (e.g. `2024/Week 39`) that embeds the days and link to the previous
and next week, and also to the month.

The months also each get a page with the days embedded for a monthly view.

## Usage

```sh
cargo run -- --path path/to/obsidian --from 2024-09-01 --to 2024-09-30
```
## Examples

