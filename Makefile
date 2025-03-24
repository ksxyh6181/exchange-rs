
tests:
	cargo test --all -- --nocapture

build:
	cargo build --release

doc:
	cargo doc --no-deps --open

up:
	docker-compose up -d
prune:
	docker system prune -a #docker清理命令
force:
	docker-compose up --force-recreate --build -d


