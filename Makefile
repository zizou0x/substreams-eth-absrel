ENDPOINT ?= mainnet.eth.streamingfast.io:443
START_BLOCK ?= 12376729
STOP_BLOCK ?= +1000

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: run_output
run_output: build
	substreams run -e $(ENDPOINT) substreams.yaml map_output -s $(START_BLOCK) -t $(STOP_BLOCK) --debug-modules-output="map_output,store_mint_burn_liquidity,store_swap_liquidity"