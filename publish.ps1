Set-Location -Path $PSScriptRoot\pkg_node
npm publish --access public

Set-Location -Path $PSScriptRoot\pkg_web
npm publish --access public
