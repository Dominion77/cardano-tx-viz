# Test script to verify Koios API endpoint

$testHash = "f2754b2d3a9e9e6f4b3e3d9f8c5e5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8"
$url = "https://api.koios.rest/api/v1/tx_info"

Write-Host "Testing Koios API..." -ForegroundColor Yellow
Write-Host "URL: $url" -ForegroundColor Cyan
Write-Host "Hash: $testHash" -ForegroundColor Cyan
Write-Host ""

# Test with POST request
$body = @{
    "_tx_hashes" = @($testHash)
} | ConvertTo-Json

Write-Host "Request Body:" -ForegroundColor Green
Write-Host $body
Write-Host ""

try {
    $response = Invoke-WebRequest -Uri $url -Method POST -Body $body -ContentType "application/json" -Headers @{"Accept"="application/json"}
    Write-Host "Success! Status Code: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Response:" -ForegroundColor Green
    Write-Host $response.Content
} catch {
    Write-Host "Error!" -ForegroundColor Red
    Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "Error Message: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response Body: $responseBody" -ForegroundColor Red
    }
}
