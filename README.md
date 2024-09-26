# logseq-journal-prepare

![workflow](https://github.com/Aethelflaed/logseq-journal-prepare/actions/workflows/rust.yml/badge.svg?branch=main)


Prepare a [logseq](https://logseq.com/) journal entries by adding references to the week in
the daily journal entries.

The weeks each get a page (e.g. `2024/Week 39`) that embeds the days and link to the previous
and next week, and also to the month.

The months also each get a page with the days embedded for a monthly view.

## Usage

```sh
cargo run -- --path path/to/logseq --from 2024-09-01 --to 2024-09-30
```
## Examples

![image](https://github.com/user-attachments/assets/4b39612a-52d7-44f7-acdc-8fd72c0df187)

![image](https://github.com/user-attachments/assets/00b8d2ae-e15d-471c-871d-9da2425491e9)

![image](https://github.com/user-attachments/assets/4e08c239-15b0-4ef2-82b0-f86bcb1adc63)
