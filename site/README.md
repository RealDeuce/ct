# Player documentation site

This directory contains the dependency-free GitHub Pages site for players.
Its visual language, in-world presentation, and usability principles are
recorded in [the website design direction](DESIGN.md).
The [ship catalog art guide](SHIP-ART-GUIDE.md) records the audited family,
shipyard, component, scale, and image-production rules for exterior plates.

The build has three authoritative content inputs:

- `docs/player-guide.md` becomes the Player Reference.
- Beginner bodies in `client/src/door_help.cpp` become searchable Beginner
  Help, checked against `DoorHelpTopic` in `client/include/ct/door_help.hpp`.
- Published records under `catalog/ships/` become the filtered Ship Catalog
  index and complete per-vessel dossiers; canonical plates are served from
  `site/assets/ships/`, while production manifests and masters remain under
  `site/ship-art/`.

Build and validate the site locally with:

```console
python3 site/build.py
```

The generated `site/_site/` directory is ignored. Serve it locally with any
static HTTP server, for example:

```console
python3 -m http.server --directory site/_site 8000
```

`.github/workflows/pages.yml` performs the same build and deploys the artifact
through GitHub Pages whenever a site or authoritative content source changes
on `main`.
