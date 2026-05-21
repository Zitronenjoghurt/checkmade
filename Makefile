.PHONY: up down build migration entities check dev

up:
	docker compose -f docker/docker-compose.yml up -d

down:
	docker compose -f docker/docker-compose.yml down

build:
	docker image prune -f
	docker compose -f docker/docker-compose.yml build

migration:
	cd ./core && sea-orm-cli migrate generate $(NAME)

entities:
	sea-orm-cli generate entity -o ./core/src/data/entity --database-url postgresql://admin:root@localhost:5432/checkmade

check:
	cargo check -p checkmade-core
	cargo check -p checkmade-core --features bitcode
	cargo check -p checkmade-core --features data
	cargo check -p checkmade-core --features serde
	cargo test -p checkmade-app

dev:
	docker compose -f docker/docker-compose.yml --profile dev up server-dev db postgres-exporter prometheus grafana