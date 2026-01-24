# obsidian-journal-prepare

![workflow](https://github.com/Aethelflaed/obsidian-journal-prepare/actions/workflows/rust.yml/badge.svg?branch=main)


Prepare a [obsidian](https://obsidian.md/) journal entries by adding references to the week in
the daily journal entries.

The weeks each get a page (e.g. `2024/Week 39`) that embeds the days and link to the previous
and next week, and also to the month.

The months also each get a page with the days embedded for a monthly view.

## Usage

Prepare the journal at the given path for the provided period:

```sh
obsidian-journal-prepare --path path/to/obsidian --from 2024-09-01 --to 2024-09-30
```

## Configuration

### Command line options

```
$ obsidian-journal-prepare --help
Usage: obsidian-journal-prepare [OPTIONS] --path <PATH>

Options:
  -v, --verbose...
          Increase logging verbosity

  -q, --quiet...
          Decrease logging verbosity

  -p, --path <PATH>
          Path to notes

      --from <DATE>
          Only prepare journal start from given date

          [default: 2026-01-24]

      --to <DATE>
          Only prepare journal start from given date

          [default: 1 month after --from]

  -d, --day <day>
          Configure day pages

          Use --no-day-page instead to disable.

          [default: day week month nav events]

          Possible values:
          - day:    Add property day of week
          - week:   Add property link to week
          - month:  Add property link to month
          - nav:    Add property links to previous and next day
          - events: Add recurring events content, from events/recurring.md

      --no-day-page
          Do not update day pages

  -w, --week <week>
          Configure week pages

          Use --no-week-page instead to disable.

          [default: week month nav]

          Possible values:
          - week:  Add embedded week days
          - month: Add property link to month
          - nav:   Add property links to previous and next week

      --no-week-page
          Do not update week pages

  -m, --month <month>
          Configure month pages

          Use --no-month-page instead to disable.

          [default: month nav]

          Possible values:
          - month: Add embedded month days
          - nav:   Add property links to previous and next month

      --no-month-page
          Do not update month pages

  -y, --year <year>
          Configure year pages

          Use --no-year-page instead to disable.

          [default: month nav]

          Possible values:
          - month: Add link to months
          - nav:   Add property links to previous and next year

      --no-year-page
          Do not update year pages

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Configuration file

In addition to command line options, it's possible to add a `journal-prepation-config.md` page to
obsidian and to write the default options as TOML blocks, for example:

~~~md
# journal-preparation-config.md
```toml
[day]
# Add day of the week as page property
day_of_week = true
# Add link to the week as page property
link_to_week = true
# Add link to the month page as page property
link_to_month = true
# Add link to next and previous day as page property
nav_link = true
# Add matching events content in the page
events = false

[week]
# Embeds days of the week in the page
week = true
# Add link to the month page as page property
link_to_month = true
# Add link to next and previous week as page property
nav_link = true

[month]
# Embeds days of the month (grouped by week) in the page
month = true
# Add link to next and previous month as page property
nav_link = true

[year]
# Add links to the months in the page
month = true
# Add link to next and previous year as page property
nav_link = true
```
~~~


## Examples

Journal page for the 2025-12-09:
```md
---
day: "Tuesday"
week: "[[/2025/Week 50|Week 50]]"
month: "[[/2025/December|December]]"
next: "[[/journals/2025-12-10|2025-12-10]]"
prev: "[[/journals/2025-12-08|2025-12-08]]"
---
- [ ] stretching
```

The "2025/Week 50.md" page:
```md
---
next: "[[/2025/Week 51|Week 51]]"
prev: "[[/2025/Week 49|Week 49]]"
month: "[[/2025/December|December]]"
---
- Monday ![[/journals/2025-12-08|2025-12-08]]
- Tuesday ![[/journals/2025-12-09|2025-12-09]]
- Wednesday ![[/journals/2025-12-10|2025-12-10]]
- Thursday ![[/journals/2025-12-11|2025-12-11]]
- Friday ![[/journals/2025-12-12|2025-12-12]]
- Saturday ![[/journals/2025-12-13|2025-12-13]]
- Sunday ![[/journals/2025-12-14|2025-12-14]]

```
