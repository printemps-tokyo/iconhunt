# iconhunt

Search open-source icons from the terminal, and know what each one asks of
you before you ship it.

iconhunt searches the [Iconify](https://iconify.design/) API, which serves
about 450,000 icons from 259 icon sets (Material Symbols, Tabler, Lucide,
Fluent Emoji, Game Icons and many more; counted on 2026-11-02). It adds four
things the API does not:

- License classes. Every result says whether its set is `public-domain`,
  `permissive`, `attribution` (credit required), `copyleft`,
  `non-commercial` or `unknown`, and `--no-attribution` keeps only icons you
  can use without visible credit.
- Japanese keywords. Iconify only matches English words, so `家` finds
  nothing. iconhunt maps about 190 common Japanese words (`家`, `ゴミ箱`,
  `歯車`, `ほーむ`) to the English words icon sets use.
- Previews in the terminal, drawn with half-block characters, and an HTML
  contact sheet for a closer look.
- `credits`, which writes the attribution text for the icons a project uses.

```
$ iconhunt search ゴミ箱 -n 6
iconhunt: ゴミ箱 -> trash, delete (searching: "trash", "delete")
                                          ▄▄▄▄
   ▄▄▄████▄▄▄        ▄▄      ▄▄          ██▀▀██          ▄▄▄████▄▄▄
    ▄▄▄▄▄▄▄▄         ██      ██       ▀█▀▀▀▀▀▀▀▀█▀      ▀█▀▀▀▀▀▀▀██▀
    ████████         ██      ██        █  ▄  ▄  █        ██ ▄  ▄ █▀
    ████████         ██      ██        █▄ █  █ ▄█         █ █  █ █
    ████████         ██      ██        ██ █  █ ██         █ ▀  ▀ █
    ████████         ▀█▄▄▄▄▄▄█▀        ▀█▄▄▄▄▄▄█▀         ▀▄▄▄▄▄▄▀
                                         ▀▀▀▀▀▀
      [1]               [2]               [3]               [4]

      ▄▄▄▄             ▄▄▄▄▄▄
  ▄▄▄██████▄▄▄         █▀▀▀▀█
  ▀▀▀▀▀▀▀▀▀▀▀▀     ▀██▀▀▀▀▀▀▀▀██▀
                     █  ▄  ▄  █
                     █  █  █  █
                     █  █  █  █
                     █▄▄▄▄▄▄▄▄█
                     ▀▀▀▀▀▀▀▀▀▀
      [5]               [6]

#  icon                        set                    license     class
1  mdi:trash                   Material Design Icons  Apache-2.0  permissive
2  line-md:trash               Material Line Icons    MIT         permissive
3  tabler:trash                Tabler Icons           MIT         permissive
4  iconoven:trash              IconOven               MIT         permissive
5  coolicons:trash-bulk-round  coolicons              MIT         permissive
6  boxicons:trash              Boxicons               MIT         permissive
```

Single-color icons are drawn in your terminal's own text color, so they read
on light and dark themes. Multicolor icons (emoji, logos) use 24-bit color.

## Install

```bash
cargo install --git https://github.com/printemps-tokyo/iconhunt
```

Or download a binary for Linux (x86_64) or macOS (x86_64, arm64) from the
[releases](https://github.com/printemps-tokyo/iconhunt/releases), with a
`.sha256` file next to each to check it. Building needs Rust 1.85 or later.

## Use

```bash
iconhunt search home                       # previews and a table
iconhunt search arrow left -n 40           # several words, more results
iconhunt search 家 --no-attribution        # Japanese, no credit needed
iconhunt search cat --style color          # multicolor sets only (emoji and the like)
iconhunt search star --set lucide,tabler   # only these sets
iconhunt search home --html home.html      # also an HTML contact sheet

iconhunt show mdi:home                     # one icon, large, with author and license
iconhunt get mdi:home lucide:search -o icons/
iconhunt get mdi:home --png --size 128 --color '#ba3329'
iconhunt sets lucide                       # sets whose prefix or name match
iconhunt sets --license attribution,copyleft
iconhunt credits icons.txt > CREDITS.md    # attribution for the icons you used
```

### search

Shows icons as a preview grid with numbers, then a table of the same numbers
with each icon's id, set, license and class. The preview is drawn when stdout
is a terminal; `--preview` forces it and `--no-preview` turns it off.

Results are spread across sets: at most 4 icons per set (`--per-set N`, 0 for
no cap), interleaved so the first match of every set comes first. That puts
different designs of the same icon side by side. With exactly one `--set`
there is no cap.

| Option | Meaning |
| --- | --- |
| `-n, --limit N` | Icons to show, 1-999 (default 24) |
| `--start N` | Skip N results, for paging (`--start` plus `--limit` at most 999) |
| `-s, --set a,b` | Only these icon sets, by prefix |
| `--license a,b` | Only sets in these license classes |
| `--no-attribution` | Only `public-domain` and `permissive` sets |
| `--style mono\|color` | Only single-color or only multicolor sets |
| `--category TEXT` | Only sets whose Iconify category contains TEXT (`UI 24px`, `Emoji`, `Logos`) |
| `--per-set N` | Most icons from one set (default 4) |
| `--size PX` | Preview size, 8-64 pixels (default 16; a cell holds 1x2 pixels) |
| `--html FILE` | Also write an HTML contact sheet (`--force` to overwrite) |
| `--no-translate` | Send the query as typed |
| `--json` | Machine-readable output |

Filters on sets are applied by the API (`prefixes=`), so `--limit` counts only
icons that pass them.

### show

`iconhunt show mdi:home` draws the icon large (`--size`, default 32) and
prints its set, icon count, category, author, license with its class and what
the class asks of you, the viewBox, the SVG URL and an
`<iconify-icon>` snippet. A misspelled name lists similar icons from the same
set and exits 1.

### get

Saves icons as SVG (as the API renders them) or, with `--png`, as PNG
rendered locally. `-o` names a file for one icon, a directory for several, or
`-` for stdout. Files are never overwritten without `--force`.

| Option | Meaning |
| --- | --- |
| `--size PX` | SVG: height attribute; PNG: canvas size (default 64) |
| `--color '#rrggbb'` | Color for single-color icons |
| `--flip h\|v\|hv` | Flip |
| `--rotate 90\|180\|270` | Rotate clockwise |

Single-color icons use `currentColor`. In a PNG without `--color` that
renders black. After saving, `get` says which icons need credit or carry
extra terms.

An SVG saved to disk runs scripts when opened in a browser, so `get` refuses
to save one that has a `<script>`, `<foreignObject>` or `<iframe>`, a CSS `@import`, an event
handler attribute, a `javascript:` URL, a DOCTYPE or entity, or an `href` or
`url()` pointing outside the file. Real icon sets pass; the check is there for
a compromised or hostile `--api` server.

### sets

Lists icon sets with prefix, name, icon count, license, class, style and
category. A word filters by prefix or name; `--license`, `--no-attribution`,
`--style` and `--category` work as in `search`. Sets the API marks hidden
(mostly retired duplicates) appear only with `--hidden`.

### credits

Takes icon ids, files with ids (one or more per line, `#` starts a comment),
or `-` for stdin, checks that every icon exists, and prints Markdown grouped
by set with the author, the license and its class. Sets that need credit come
first. Missing icons are reported and left out, and the exit code is 1.

```markdown
## Game Icons (`game-icons`)

- Author: [GameIcons](https://github.com/game-icons/icons)
- License: [CC BY 3.0](https://github.com/game-icons/icons/blob/master/license.txt) (SPDX: `CC-BY-3.0`)
- Class: attribution - Credit the author where the icon is used.
- Icons: `game-icons:broadsword`
```

## License classes

The class comes from the set's SPDX identifier as the Iconify API reports it.

| Class | SPDX identifiers | What it asks of you |
| --- | --- | --- |
| `public-domain` | CC0-1.0, Unlicense | Nothing |
| `permissive` | MIT, MIT-0, ISC, Apache-2.0, BSD-2/3-Clause, OFL-1.1, Zlib | Keep the license notice when you redistribute the icon files |
| `attribution` | CC-BY-* | Credit the author where the icon is used |
| `copyleft` | GPL-*, LGPL-*, AGPL-*, MPL-2.0, CC-BY-SA-* | Share-alike or copyleft terms: read the license before shipping |
| `non-commercial` | CC-BY-NC-* | Not for commercial use |
| `unknown` | Anything else, CC-BY-ND-*, or no identifier | Read the license |

An identifier the table does not know is `unknown`, never `permissive`. The
classes are a reading aid, not legal advice: the license text, and the terms
of the individual icon set, are what count.

## Japanese keywords

Japanese words are looked up in a fixed table in `src/ja.rs`. Before the
lookup, full-width ASCII becomes ASCII and hiragana becomes katakana, so
`ほーむ` and `ホーム` are the same word. A word made of known parts is split by
longest match (`猫の写真` becomes `cat camera`), with particles such as `の`
skipped. The table holds about 190 entries for things icons depict:
navigation, actions, people, objects, commerce, weather, animals, transport,
food and status.

The first query uses each word's best keyword, and up to two more swap in
alternatives (`家` searches `home`, then `house`). Words the table does not
know are reported, and nothing is sent for them. This is a keyword table,
not a translator; additions are welcome (see `CONTRIBUTING.md`).

## The Iconify API

iconhunt uses the public API at `https://api.iconify.design`, which is free
and asks that it not be abused. A search costs one request per query word
alternative (at most 3), plus one for the set list when filtering by license
or style, plus one per icon set for the previews (and one per rotated or flipped alias). Requests carry a
`User-Agent` naming iconhunt, time out after 20 seconds, and responses are
size-limited.

For heavy use, run your own instance of the
[Iconify API](https://iconify.design/docs/api/) and point iconhunt at it with
`--api URL` or `ICONHUNT_API`.

Icon data from the API is treated as untrusted. Ids are restricted to
`a-z`, `0-9` and `-` before they go into a URL or file name; previews are
rendered with external references and embedded images disabled; `get`
refuses SVGs that could run code or load resources (see above); the HTML
sheet embeds icons as `data:` images, which cannot run scripts; and control
characters in set names, authors and licenses are removed before they reach
your terminal or the credits file. New files are created exclusively, so an
existing file or a dangling symlink is never written through without
`--force`.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Found what was asked for |
| 1 | Nothing found: no matching icons or sets, an unknown set, or a missing icon |
| 2 | Error: bad input, network failure, refusal to overwrite or to save an unsafe SVG |

## Limits

- Search ranking and coverage are the API's. In testing on 2026-11-02 the
  public API returned at most 128 results per query, whatever the limit, and
  answered `start` with HTTP 400. iconhunt therefore always asks from the
  first result and pages (`--start`) through what came back.
- Per-set caps are applied to the results the API returned, so a capped
  search fetches up to six times `--start` plus `--limit` icons (at most 999).
- The license is per icon set as listed by Iconify. Some sets contain icons
  under other terms (brand logos, for example, are trademarks whatever the
  license of the drawing); check before relying on them.
- Previews are 16x16 pixels by default, enough to judge a shape, not detail.

## License

MIT
