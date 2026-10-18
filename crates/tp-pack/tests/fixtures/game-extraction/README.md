# Historical extraction baseline

`validated-baseline.json` records the MAN TGX (`man.tgx_2020`) and DAF NGD
(`daf.2021`) experiments on ETS2 1.61.1.1: cabin/accessory ids, dimensions,
SVG edge counts, mounted sources and model/painted-definition counts. It
contains no model, texture or game geometry.

The original probes mounted the base archives plus their vehicle DLC only,
not additional tuning DLC. They selected a definition's look and variant,
visible parts, painted materials and the stream aliased `_TEXCOORD1`.
The converted-model coordinates were used without another vertical flip. The
prototype accepted ordinary truckpaint effects, refused alternate UVs and
required one crop-name-derived dimension per accessory. Main templates
superposed compatible painted options. Vehicle names, archive lists, cabin
sets and model-directory filtering were hardcoded and must not become
extractor rules. Includes and `variant_uk` were not independently resolved
by those probes; the maintained tool must account for them explicitly.

Native package validation, project save/reload, DDS dimensions and mod
references passed for 30 MAN and 15 DAF surfaces. The initial local reports
said game checks were pending. Later user confirmations are recorded in
`docs/game-extraction-plan.md`; no exhaustive configuration matrix exists.
New DLC contributions or changed geometry do not inherit that confirmation.

The retained Python tests also checked empty `Pieces` fields, an alias on
`_UV7`, a hidden variant, image coordinates, alternate-UV refusal and an
unclosed PIX section. These are selection regression cases to port to Rust,
not grounds for accepting other shaders.
