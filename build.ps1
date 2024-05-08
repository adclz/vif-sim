$version = "0.0.1-alpha.8"
#
# Web
#

# Run wasm-pack
wasm-pack build --out-dir pkg_web --target web

# Edit package.json
$packageJsonPath = "pkg_web/package.json"
$packageJson = Get-Content $packageJsonPath | ConvertFrom-Json
$packageJson.name = "@vifjs/sim-web"
$packageJson.version = $version
$packageJson.files += "vifsimlib_bg.wasm.d.ts"
$packageJson.files += "snippets"
$packageJson.files += "dist"
$packageJson | Add-Member -MemberType NoteProperty -Name "main" -Value "vifsimlib_bg.js"
$packageJson | Add-Member -MemberType NoteProperty -Name "exports" -Value @{}
$packageJson | Add-Member -MemberType NoteProperty -Name "type" -Value "module"
$packageJson.exports."." = @{ "import" = "./vifsimlib.js" }
$packageJson.exports."./vifsimlib_bg.wasm" = "./vifsimlib_bg.wasm"
$packageJson.exports."./plugin" = "./dist/plugin/plugin.js"
$packageJson.exports."./boot" = "./dist/boot/container.js"
$packageJson.PSObject.Properties.Remove('repository')
$packageJson.license = "MIT"
$packageJson | ConvertTo-Json | Set-Content $packageJsonPath

# Fixing weird PS json syntax
Set-Location "pkg_web/"
pnpm pkg fix
pnpm link --global
Set-Location "../"

# Create Plugin & Boot directories
New-Item -ItemType Directory -Force -Path "pkg_web/dist/plugin"
New-Item -ItemType Directory -Force -Path "pkg_web/dist/boot"
New-Item -ItemType Directory -Force -Path "pkg_web/dist/event"

# Copy tsconfig
Copy-Item "src/ts/tsconfig.json" -Destination "pkg_web/dist"

# Copy PluginBuilder to plugin dir & rename it to plugin.ts
Copy-Item "src/ts/plugin.ts" -Destination "pkg_web/dist/plugin"
Copy-Item "src/ts/event/event-emitter.ts" -Destination "pkg_web/dist/event"

# Copy Boot files
Copy-Item "src/ts/boot/browser/container.ts" -Destination "pkg_web/dist/boot"
Copy-Item "src/ts/boot/types.ts" -Destination "pkg_web/dist/boot"
Copy-Item "src/ts/boot/command-store.ts" -Destination "pkg_web/dist/boot"
Copy-Item "src/ts/boot/browser/worker.ts" -Destination "pkg_web/dist/boot"

# Add tsconfig for tsc
Copy-Item "src/ts/tsconfig.json" -Destination "pkg_web/dist"

# Set cwd for tsc
Set-Location "pkg_web/dist"

# Run tsc
npx tsc -b

# Back to the crate root
Set-Location "../.."

# Remove original ts files & tsconfig
Remove-Item "pkg_web/dist/plugin/plugin.ts"
Remove-Item "pkg_web/dist/boot/container.ts"
Remove-Item "pkg_web/dist/boot/worker.ts"
Remove-Item "pkg_web/dist/boot/types.ts"
Remove-Item "pkg_web/dist/boot/command-store.ts"
Remove-Item "pkg_web/dist/event/event-emitter.ts"
Remove-Item "pkg_web/dist/tsconfig.json"

#
# Node
#

# Run wasm-pack
wasm-pack build --out-dir pkg_node --target nodejs --features node

# Edit package.json
$packageJsonPath = "pkg_node/package.json"
$packageJson = Get-Content $packageJsonPath | ConvertFrom-Json
$packageJson.name = "@vifjs/sim-node"
$packageJson.version = $version
$packageJson.files += "vifsimlib_bg.wasm.d.ts"
$packageJson.files += "snippets"
$packageJson.files += "dist"
$packageJson | Add-Member -MemberType NoteProperty -Name "exports" -Value @{}
$packageJson.exports."." = @{ "import" = "./vifsimlib.js" }
$packageJson.exports."./plugin" = "./dist/plugin/plugin.js"
$packageJson.exports."./boot" = "./dist/boot/container.js"
$packageJson.PSObject.Properties.Remove('repository')
$packageJson.license = "MIT"
$packageJson | ConvertTo-Json | Set-Content $packageJsonPath

# Fixing weird PS json syntax
Set-Location "pkg_node/"
pnpm pkg fix
pnpm link --global
Set-Location "../"

# Create Plugin & Boot directories
New-Item -ItemType Directory -Force -Path "pkg_node/dist/plugin"
New-Item -ItemType Directory -Force -Path "pkg_node/dist/boot"
New-Item -ItemType Directory -Force -Path "pkg_node/dist/event"

# Copy tsconfig
Copy-Item "src/ts/tsconfig.json" -Destination "pkg_node/dist"

# Copy PluginBuilder to plugin dir & rename it to plugin.ts
Copy-Item "src/ts/plugin.ts" -Destination "pkg_node/dist/plugin"
Copy-Item "src/ts/event/event-emitter.ts" -Destination "pkg_node/dist/event"

# Copy Boot files
Copy-Item "src/ts/boot/node/container.ts" -Destination "pkg_node/dist/boot"
Copy-Item "src/ts/boot/types.ts" -Destination "pkg_node/dist/boot"
Copy-Item "src/ts/boot/command-store.ts" -Destination "pkg_node/dist/boot"
Copy-Item "src/ts/boot/node/worker.ts" -Destination "pkg_node/dist/boot"

# Add tsconfig for tsc
Copy-Item "src/ts/tsconfig.json" -Destination "pkg_node/dist"

# Set cwd for tsc
Set-Location "pkg_node/dist"

# Run tsc
npx tsc -b

# Back to the crate root
Set-Location "../.."

# Remove original ts files & tsconfig
Remove-Item "pkg_node/dist/plugin/plugin.ts"
Remove-Item "pkg_node/dist/boot/container.ts"
Remove-Item "pkg_node/dist/boot/worker.ts"
Remove-Item "pkg_node/dist/boot/types.ts"
Remove-Item "pkg_node/dist/boot/command-store.ts"
Remove-Item "pkg_node/dist/event/event-emitter.ts"
Remove-Item "pkg_node/dist/tsconfig.json"

#
# Finally
#

