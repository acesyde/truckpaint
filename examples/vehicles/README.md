# Sample vehicles

Two fictional vehicles let you try vehicle projects without game files. They are also the reference examples for package authors. Their templates are original drawings, not game assets, and are MIT licensed like the rest of the repository.

**TruckPaint Sample Truck** (`community.truckpaint.sample_truck`) is a truck whose two cabins have different layouts. It has one main texture per cabin layout, and accessories:

| Version | Main textures | Accessories |
|---|---|---|
| 1.0.0 | Standard cab 4096, High roof 4096 | Chassis 2048, Cab accessories 1024 |
| 1.1.0 | Standard cab 4096 (layout changed), High roof 4096 | Chassis 4096, Cab accessories 1024, Side skirts 1024 (new) |

**TruckPaint Sample Trailer** (`community.truckpaint.sample_trailer`) is a curtainsider. It has a single main texture and its bodies as accessories:

| Version | Main texture | Accessories |
|---|---|---|
| 1.0.0 | Base 2048 | Curtain body 13.6 m 4096, Curtain body 10.5 m 4096, Mudflaps 512 |

Each main texture also has a hand-made preview mesh in `preview/` (boxes whose faces cover the labeled panels of its template), used by the Side View. The truck's 1.1.0 meshes also hold a `sideskirt` slot (the Side skirts texture), and the trailer's mesh a `body` slot with its two curtain bodies, to show how accessories are written.

The built packages sit next to their sources:

```
community.truckpaint.sample_truck-1.0.0.tpv
community.truckpaint.sample_truck-1.1.0.tpv
community.truckpaint.sample_trailer-1.0.0.tpv
sample-truck/<version>/vehicle.json
sample-truck/<version>/templates/*.svg
sample-truck/<version>/preview/*.json
sample-trailer/<version>/vehicle.json
sample-trailer/<version>/templates/*.svg
sample-trailer/<version>/preview/*.json
```

## Trying Update Template

1. Install `community.truckpaint.sample_truck-1.0.0.tpv` (Vehicle › Vehicle Library… › Install…, or drop it on the window).
2. Create a project on the truck, keep the **Standard cab**, and paint something.
3. Install `community.truckpaint.sample_truck-1.1.0.tpv`, then run **Vehicle › Update Template…**. The dialog:
   - flags the Standard cab as "Layout changed";
   - scales the Chassis artwork to 4096;
   - offers the new **Side skirts** accessory, checked.

TruckPaint also offers **Install the sample vehicles** (the truck 1.1.0 and the trailer) while no vehicle is installed.

## Rebuilding

After editing a source, rebuild the packages:

```sh
mise run sample-vehicles
```

The test suite fails if a committed package no longer matches its sources. See [the package format](https://truckpaint.ace-dev.me/en/vehicles/package-format/) to make your own.
