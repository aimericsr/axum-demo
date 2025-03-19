ci: 
	cargo fmt --all -- --check && cargo clippy -- -D warnings && cargo test

compose-dev-up: 
	docker compose --profile dev up -d   

compose-dev-down: 
	docker compose --profile dev down  