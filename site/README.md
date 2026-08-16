# Player documentation site

This directory contains the dependency-free GitHub Pages site for players.
The build has two authoritative content inputs:

- `docs/player-guide.md` becomes the Player Reference.
- Beginner bodies in `client/src/door_help.cpp` become searchable Beginner
  Help, checked against `DoorHelpTopic` in `client/include/ct/door_help.hpp`.

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
