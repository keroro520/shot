alice = "<miner's privkey>"
basedir = "data"

[logger]
filter = "info,shot=debug"
file = "data/logs/shot.log"
color = true
log_to_file = true
log_to_stdout = true

[chain]
fee_rate = 1_000 # shannons/KB
rpc_urls = [ 
    "http://127.0.0.1:8114",
]

[controller]
tps = 100.0  # tps
