# WSL Port Forwarding Configuration Script
# Run in Windows PowerShell (Administrator)

$WSL_IP = "172.31.75.249"
$WIN_PORT = 17974
$WSL_PORT = 17974

Write-Host "========================================"
Write-Host "WSL Port Forwarding Configuration"
Write-Host "========================================"
Write-Host ""
Write-Host "Windows Host IP: 192.168.2.101"
Write-Host "WSL Internal IP: $WSL_IP"
Write-Host "Forwarding Port: $WIN_PORT -> $WSL_PORT"
Write-Host ""

Write-Host "Cleaning up existing rules..."
netsh interface portproxy delete v4tov4 listenport=$WIN_PORT listenaddress=0.0.0.0 2>$null
netsh interface portproxy delete v4tov4 listenport=$WIN_PORT listenaddress=192.168.2.101 2>$null

Write-Host "Adding port forwarding rules..."
netsh interface portproxy add v4tov4 listenport=$WIN_PORT listenaddress=0.0.0.0 connectport=$WSL_PORT connectaddress=$WSL_IP
netsh interface portproxy add v4tov4 listenport=$WIN_PORT listenaddress=192.168.2.101 connectport=$WSL_PORT connectaddress=$WSL_IP

Write-Host ""
Write-Host "Current port forwarding configuration:"
netsh interface portproxy show all

Write-Host ""
Write-Host "Configuring Windows Firewall..."

Remove-NetFirewallRule -DisplayName "NRCS P2P Port $WIN_PORT" -ErrorAction SilentlyContinue

New-NetFirewallRule -DisplayName "NRCS P2P Port $WIN_PORT" -Direction Inbound -Protocol TCP -LocalPort $WIN_PORT -Action Allow -Profile Any -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "========================================"
Write-Host "Configuration Complete!"
Write-Host "========================================"
Write-Host ""
Write-Host "Java NRCS can now access Rust node at:"
Write-Host "  192.168.2.101:$WIN_PORT"
Write-Host ""
Write-Host "Test connection:"
Write-Host "  netsh interface portproxy show all"
Write-Host ""
