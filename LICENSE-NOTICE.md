# Licence notice — RFTools Utility

- **Upstream:** RFTools Utility, https://github.com/McJtyMods/RFToolsUtility.
- **Licence as read:** MIT. The jar's `META-INF/neoforge.mods.toml` says
  `license="MIT License https://github.com/McJtyMods/RFToolsUtility/blob/1.16/LICENCE"`;
  the jar carries no licence file of its own, so the text was read from that
  repository path (`LICENSE`, `Copyright (c) 2019 McJtyMods`) and is kept in
  [`LICENSE`](LICENSE) beside the port's own copyright.
- **Assets:** the tank's blockstate shape, the nine fill-level textures, the
  machine base textures, the models, the recipe, the loot table, the
  advancement and the lang lines are upstream's own MIT assets, read from the
  exact ATM10 jar `rftoolsutility-1.21-7.0.13.jar`
  (SHA-256 `615642b1…d3ff`); see [`FINDINGS.md`](FINDINGS.md) and
  [`CONTENT.md`](CONTENT.md).
- **What travels with the artifact:** the same notices are packaged inside the
  distributed `.hnmod` (`mixins/rftoolsutility/LICENSE` and `NOTICE.md`) and
  inside the embedded content archive, so an artifact-only download carries
  them (MIT notice condition).
- **Status:** local-only until the maintainer approves publishing
  (ARCHITECTURE.md §8).
