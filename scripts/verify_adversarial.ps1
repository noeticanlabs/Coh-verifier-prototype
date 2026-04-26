# PowerShell equivalent of verify_adversarial.sh
# Verifies that coh-validator rejects all adversarial vectors

$VALIDATOR = "coh-node/target/release/coh-validator.exe"
$VECTORS_DIR = "coh-node/vectors/adversarial"

if (-not (Test-Path $VALIDATOR)) {
    Write-Error "Validator binary not found at $VALIDATOR. Please build with cargo build --release."
    exit 1
}

$files = Get-ChildItem -Path $VECTORS_DIR -Filter "reject_*.jsonl"
$total = $files.Count
$passed = 0
$failed = 0

Write-Host "Starting Adversarial Verification (PS) for $total vectors..." -ForegroundColor Cyan

foreach ($file in $files) {
    $relative_path = $file.FullName.Replace((Get-Location).Path + "\", "")
    Write-Host "  Checking $relative_path..." -NoNewline
    
    # Run validator
    $output = & $VALIDATOR verify-chain $file.FullName 2>&1
    $exit_code = $LASTEXITCODE

    if ($exit_code -eq 0) {
        Write-Host " [FAILED] - Vector was ACCEPTED (expected rejection)" -ForegroundColor Red
        $failed++
    } elseif ($exit_code -gt 3) {
        Write-Host " [CRASH] - Exit code $exit_code" -ForegroundColor Red
        Write-Host $output
        $failed++
    } else {
        Write-Host " [OK] - Rejected with code $exit_code" -ForegroundColor Green
        $passed++
    }
}

$statusColor = if ($passed -eq $total) { "Green" } else { "Yellow" }
Write-Host "`nResults: $passed/$total vectors correctly rejected." -ForegroundColor $statusColor

if ($failed -gt 0) {
    exit 1
}
