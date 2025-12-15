Pourquoi l'édition sans CMake échoue
-----------------------------------

Vous aviez initialement retiré la feature `bundled` pour la crate `sdl2` afin d'éviter que le build invoque CMake/Ninja.
Ceci est possible, mais alors `sdl2-sys` s'attend à trouver les bibliothèques de développement SDL2 (fichiers `.lib` et headers) sur votre machine.

L'erreur de l'éditeur de liens indique que `SDL2_image.lib`, `SDL2_ttf.lib` et `SDL2_gfx.lib` sont manquants : cela signifie que vous n'avez pas installé les paquets de développement correspondants pour MSVC.

Options pour corriger
---------------------

1) Installer via `vcpkg` (recommandé pour MSVC)

  - Installe `vcpkg`, puis installez les paquets `sdl2`, `sdl2-image`, `sdl2-ttf`, `sdl2-gfx` (x64-windows).
  - Exemple (PowerShell) :

```powershell
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg.exe install sdl2:x64-windows sdl2-image:x64-windows sdl2-ttf:x64-windows sdl2-gfx:x64-windows
```

  - Après installation, vcpkg place les `.lib` dans `vcpkg\installed\x64-windows\lib` et les `.dll` dans `vcpkg\installed\x64-windows\bin`. Si le linker ne trouve pas les `.lib`, ajoutez ce dossier à la variable d'environnement `LIB` et `PATH`.

2) Télécharger les archives de développement SDL2 (Visual C++)

  - Téléchargez les paquets de développement (SDL2, SDL2_image, SDL2_ttf, SDL2_gfx) depuis les sites officiels ou builds tiers qui fournissent des `.lib`/`.dll` pour MSVC.
  - Placez les `.lib` dans un dossier et ajoutez ce dossier à la variable d'environnement `LIB` (ou copiez les .lib dans un emplacement déjà recherché par le linker).
  - Ajoutez les `.dll` dans le `PATH` (ou à côté de l'exécutable) pour l'exécution.

3) Réactiver `bundled` (construire via CMake/Ninja)

  - Si vous réactivez `bundled`, le build tentera de compiler SDL2 à partir des sources en utilisant CMake/Ninja.
  - Attention : `bundled` construit typiquement SDL2 core, mais les extensions `SDL2_image`, `SDL2_ttf`, `SDL2_gfx` peuvent nécessiter des étapes supplémentaires ou produire des DLLs sans fichiers d'import (`.lib`) compatibles avec le linker MSVC.

Remarques spécifiques pour `bundled` et les extensions
-----------------------------------------------------

Si vous gardez la feature `bundled` dans `pain_graphics/Cargo.toml` (ce que vous avez choisi), deux options pratiques :

- Installer les paquets `sdl2-image`, `sdl2-ttf`, `sdl2-gfx` via `vcpkg` (recommandé). Le script `scripts/install-sdl2-vcpkg.ps1` installe ces paquets.
- Ou retirer les features `image`, `ttf`, `gfx` de la dépendance `sdl2` si le projet n'a pas besoin de ces fonctionnalités.

Commande d'exemple après installation vcpkg :

```powershell
cd C:\src\house_of_pain
.\scripts\install-sdl2-vcpkg.ps1
cargo clean
cargo build
```

Si vous préférez, je peux :
- exécuter le script d'installation (`scripts/install-sdl2-vcpkg.ps1`) pour vous (nécessite accès réseau), ou
- enlever les features `image`, `ttf`, `gfx` de `pain_graphics/Cargo.toml` si vous voulez tout faire avec le `bundled` minimal.
