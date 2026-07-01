<#
.SYNOPSIS
SQLRustGo 发布门禁脚本

.DESCRIPTION
执行完整的发布前检查，包括编译、测试、代码质量、格式、安全审计和依赖更新检查。

.EXAMPLE
.\gatekeeper.ps1
#>

$ErrorActionPreference = "Stop"

$totalChecks = 6
$passedChecks = 0
$failedChecks = 0

function Run-Check {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "[$passedChecks/$totalChecks] $Name" -ForegroundColor Yellow
    Write-Host "--------------------------------------------" -ForegroundColor Yellow

    try {
        & $Command
        Write-Host "✓ $Name 通过" -ForegroundColor Green
        $script:passedChecks++
        return $true
    } catch {
        Write-Host "✗ $Name 失败" -ForegroundColor Red
        Write-Host "  错误: $_" -ForegroundColor Red
        $script:failedChecks++
        return $false
    }
}

Write-Host "============================================" -ForegroundColor Yellow
Write-Host "  SQLRustGo 发布门禁检查" -ForegroundColor Yellow
Write-Host "=============================================" -ForegroundColor Yellow

Run-Check "1. 编译检查" { cargo build --all-features }

Run-Check "2. 测试检查" { cargo test --all-features }

Run-Check "3. Clippy 代码质量检查" { cargo clippy --all-features -- -D warnings }

Run-Check "4. rustfmt 代码格式检查" { cargo fmt --check --all }

Run-Check "5. cargo-audit 安全审计" { cargo audit }

Run-Check "6. cargo-outdated 依赖更新检查" { cargo outdated --exit-code 1 }

Write-Host ""
Write-Host "============================================" -ForegroundColor Yellow
Write-Host "  门禁检查结果" -ForegroundColor Yellow
Write-Host "=============================================" -ForegroundColor Yellow

Write-Host ""
Write-Host "通过: $passedChecks/$totalChecks" -ForegroundColor Green
Write-Host "失败: $failedChecks/$totalChecks" -ForegroundColor Red

if ($failedChecks -eq 0) {
    Write-Host ""
    Write-Host "🎉 所有门禁检查通过！可以发布。" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "❌ 门禁检查未通过，修复后重试。" -ForegroundColor Red
    exit 1
}