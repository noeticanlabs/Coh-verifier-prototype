.PHONY: setup test audit demo clean fixtures bench

setup:
	@echo "Setting up workspace..."
	cd coh-node && cargo fetch
	cd ape && cargo fetch
	cd coh-dashboard && npm ci

test:
	@echo "Running all tests..."
	cd coh-node && cargo fmt --check
	cd coh-node && cargo clippy --all-targets -- -D warnings
	cd coh-node && cargo test --all
	cd ape && cargo fmt --check
	cd ape && cargo clippy --all-targets -- -D warnings
	cd ape && cargo test --all
	cd coh-dashboard && npm run test:run

audit:
	@echo "Running cargo audit..."
	cd coh-node && cargo audit
	cd ape && cargo audit

demo:
	@echo "Building dashboard and starting demo..."
	cd coh-dashboard && npm run build
	@echo "Demo instructions: run scripts or start sidecar."

clean:
	@echo "Cleaning workspace..."
	cd coh-node && cargo clean
	cd ape && cargo clean
	cd coh-dashboard && rm -rf node_modules dist

fixtures:
	@echo "Rebuilding fixtures..."
	# To be implemented in Phase 2

bench:
	@echo "Running benchmarks..."
	cd coh-node && cargo bench
