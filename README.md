# logseq-journal-prepare

Prepare a [logseq](https://logseq.com/) journal entries by adding references to the week in
the daily journal entries.

The weeks each get a page (e.g. `2024/Week 39`) that embeds the days and link to the previous
and next week, and also to the month.

The months also each get a page with the days embedded for a monthly view.

## Usage

```sh
cargo run -- --path path/to/logseq --from 2024-09-01 --to 2024-09-30
```
