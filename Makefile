help: ## List targets & descriptions
	@cat Makefile* | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

setup:
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; \

depends:
	@rustup update

build:
	@cargo build

update:
	@cargo update

rebuild: update build

clean:
	@cargo clean

run: 
	RUST_LOG=info cargo run --release -- --port 8080

test:
	@cargo test;

clippy: 
	@cargo check; cargo clippy;

docker-build: 
	@docker build . -t rust-api-template

docker-run-local: docker-build
	bash -c "trap 'docker-compose down' EXIT; docker-compose up --remove-orphans"
