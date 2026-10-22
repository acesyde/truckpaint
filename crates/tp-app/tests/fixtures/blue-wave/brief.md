# Blue Wave Biarritz: the documentation's example livery

A fictional carrier from the Basque coast. Every screenshot and animation of the documentation that shows painted artwork shows this livery, on the sample truck and the sample trailer. The scene that builds it is `tests/common/blue_wave.rs`; the layout is its table.

The livery is inspired by a presentation board (a tractor and a curtainsider in white and blue, a wave logo, a lighthouse). The board is an illustration, not a model: it is not part of this repository, and the artwork here is original, drawn for the project, MIT licensed like the rest of the repository. No third-party mark is used.

## Identity

- Name: Blue Wave Biarritz. Place: Biarritz, Pays Basque, France.
- Tagline: "L'excellence du transport sur la Côte Basque", italic, with a light blue swoosh under it.
- Keywords: Mer, Transport, Confiance, Performance, Côte Basque.
- The name and the tagline are artwork: they read the same in all four documentation languages.

## Colors

| Swatch | Hex | Meaning | Use |
|---|---|---|---|
| White | `#FFFFFF` | purity, reliability | base of every texture |
| Light blue | `#3E9EDB` | ocean, openness | "Wave", the swoosh, the upper wave bands |
| Blue | `#033163` | performance, trust | "Blue", the lettering, the lower wave bands, the coast |

Sampled from the board. Artwork files use these three colors only.

## Logo lockup (the symbol "Logo")

The wave mark (`mark.svg`, an image), the words "Blue Wave" ("Blue" in Blue, "Wave" in Light blue, bold) and "BIARRITZ" small and letter-spaced between two thin rules (text and lines laid out by the app, so the result does not depend on the fonts of the machine).

## Motifs

- `waves.svg`: flowing bands (Light blue, then Blue with a Light blue line) that rise to the right.
- `coast.svg`: the lighthouse and the rocks, in Blue.

## Composition rules

- White base everywhere; blue only for graphics and lettering.
- Waves rise from the lower front toward the rear and end in the coast at the rear of the trailer.
- The logo is large on the trailer, small on the cab doors, centered on the rear.
- The tagline goes under the logo wherever a zone is large enough to read it; it is left out of small zones.
- The truck's side skirts are Blue with a Light blue rule, so the Side View shows a finished cab.
- Left and right sides use the same table; waves and coast are mirrored so they stay at the rear, the logo and text never are.

The placement of each element in each template zone is the layout table in `tests/common/blue_wave.rs`.
