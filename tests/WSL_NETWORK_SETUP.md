# WSL 环境下 P2P 连接配置指南

## 网络拓扑

```
Java NRCS 节点 (192.168.2.164)
        |
        |  P2P 连接
        v
Windows 主机 (192.168.2.101:17974)
        |
        |  端口转发
        v
   WSL2 (172.31.75.249:17974)
        |
        v
   Rust NRCS 节点
```

## IP 地址信息

| 组件 | IP 地址 | 说明 |
|------|---------|------|
| Windows 主机 | 192.168.2.101 | 对外可见的IP地址 |
| WSL2 内部 | 172.31.75.249 | WSL2虚拟网络IP |
| Java NRCS | 192.168.2.164 | 目标节点 |
| 默认网关 | 172.31.64.1 | WSL2网关 |

## 配置步骤

### 1. 配置 Windows 端口转发

在 Windows PowerShell (管理员权限) 中运行：

```powershell
# 以管理员身份运行 PowerShell
# 然后执行以下命令：

# 删除现有规则（如果存在）
netsh interface portproxy delete v4tov4 listenport=17974 listenaddress=0.0.0.0
netsh interface portproxy delete v4tov4 listenport=17974 listenaddress=192.168.2.101

# 添加端口转发规则
netsh interface portproxy add v4tov4 listenport=17974 listenaddress=0.0.0.0 connectport=17974 connectaddress=172.31.75.249
netsh interface portproxy add v4tov4 listenport=17974 listenaddress=192.168.2.101 connectport=17974 connectaddress=172.31.75.249

# 验证配置
netsh interface portproxy show all
```

或者运行提供的脚本：

```powershell
# 在 Windows PowerShell (管理员) 中
.\tests\scripts\setup_wsl_port_forwarding.ps1
```

### 2. 配置 Windows 防火墙

在 Windows PowerShell (管理员权限) 中运行：

```powershell
# 添加防火墙规则允许端口 17974
New-NetFirewallRule -DisplayName "NRCS P2P Port 17974" `
    -Direction Inbound `
    -Protocol TCP `
    -LocalPort 17974 `
    -Action Allow `
    -Profile Any

# 验证规则
Get-NetFirewallRule -DisplayName "NRCS P2P Port 17974"
```

### 3. 配置 Rust 节点

配置文件：`config/local.toml`

```toml
[p2p]
listen_addr = "0.0.0.0:17974"
external_addr = "192.168.2.101:17974"  # Windows主机IP
bootstrap_nodes = ["192.168.2.164:17974"]
max_connections = 100
connection_ttl_secs = 600
protocol_id = "NRCS"
```

### 4. 启动 Rust 节点

```bash
cd /mnt/d/workspace/git/rust-nrcs
cargo run --release --bin nrcs-node
```

## 验证连接

### 测试端口转发

在 Windows 命令提示符中：

```cmd
# 测试端口是否可访问
telnet 192.168.2.101 17974

# 或者使用 PowerShell
Test-NetConnection -ComputerName 192.168.2.101 -Port 17974
```

### 检查 P2P 连接

在 Rust 节点日志中查找：
- `Handshake successful`
- `Peer connected`
- `Received getInfo response`

### 检查 Java NRCS 节点

```bash
# 检查对等节点列表
curl "http://192.168.2.164:17976/nrcs?requestType=getPeers"

# 检查区块链状态
curl "http://192.168.2.164:17976/nrcs?requestType=getBlockchainStatus"
```

## 故障排除

### 问题1：端口转发不工作

**解决方案：**
```powershell
# 检查端口转发状态
netsh interface portproxy show all

# 重启端口转发
netsh interface portproxy reset

# 重新添加规则
netsh interface portproxy add v4tov4 listenport=17974 listenaddress=0.0.0.0 connectport=17974 connectaddress=172.31.75.249
```

### 问题2：防火墙阻止连接

**解决方案：**
```powershell
# 检查防火墙规则
Get-NetFirewallRule -DisplayName "NRCS*"

# 删除并重新创建规则
Remove-NetFirewallRule -DisplayName "NRCS P2P Port 17974"
New-NetFirewallRule -DisplayName "NRCS P2P Port 17974" -Direction Inbound -Protocol TCP -LocalPort 17974 -Action Allow
```

### 问题3：WSL IP 地址变化

**解决方案：**
创建启动脚本自动更新配置：

```bash
#!/bin/bash
# update_wsl_ip.sh

WSL_IP=$(ip addr show eth0 | grep "inet " | awk '{print $2}' | cut -d/ -f1)
echo "WSL IP: $WSL_IP"

# 更新配置文件
sed -i "s/connectaddress=.*/connectaddress=$WSL_IP/" /mnt/d/workspace/git/rust-nrcs/tests/scripts/setup_wsl_port_forwarding.ps1
```

## 自动化脚本

### WSL 启动时自动配置

在 Windows 中创建任务计划程序任务：

1. 打开任务计划程序
2. 创建基本任务
3. 触发器：当特定用户登录时
4. 操作：启动程序
5. 程序：`C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe`
6. 参数：`-File "D:\workspace\git\rust-nrcs\tests\scripts\setup_wsl_port_forwarding.ps1"`

## 参考链接

- [WSL 网络配置文档](https://docs.microsoft.com/en-us/windows/wsl/networking)
- [Windows 端口转发](https://docs.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
