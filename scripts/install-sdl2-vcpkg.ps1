<#
Script d'automatisation minimal pour cloner vcpkg et installer SDL2 + extensions (x64-windows).
Exécuter depuis PowerShell en tant qu'utilisateur qui a git et Visual Studio (MSVC) installé.
#>

param(
    [string]$VcpkgDir = "$PSScriptRoot\..\vcpkg"
)

Write-Host "Vcpkg target directory: $VcpkgDir"

if (-not (Test-Path $VcpkgDir)) {
    git clone https://github.com/microsoft/vcpkg.git $VcpkgDir
}

Push-Location $VcpkgDir
if (-not (Test-Path .\vcpkg.exe)) {
    Write-Host "Bootstrapping vcpkg..."
    & .\bootstrap-vcpkg.bat
}

Write-Host "Installing sdl2 packages (x64-windows)..."
& .\vcpkg.exe install sdl2:x64-windows sdl2-image:x64-windows sdl2-ttf:x64-windows sdl2-gfx:x64-windows

Write-Host "Installation terminée. Si vous utilisez Visual Studio, ouvrez le ""x64 Native Tools Command Prompt"" avant de builder."
Write-Host "Ensuite: `cargo clean` puis `cargo build` dans la racine du projet." 

Pop-Location

Write-Host "Si le linker ne trouve toujours pas les .lib, ajoutez le dossier vcpkg\installed\x64-windows\lib à la variable d'environnement LIB, et vcpkg\installed\x64-windows\bin au PATH." 
