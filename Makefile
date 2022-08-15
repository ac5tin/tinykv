TAG := $(shell git describe --tags `git rev-list --tags --max-count=1`)
IMAGE_NAME := tinykv

check/tag:
	@echo "TAG: $(TAG)"
	@echo -n "Are you sure? [y/N] " && read ans && [ $${ans:-N} = y ]


test: 
	RUST_LOG=debug cargo test

run:
	RUST_LOG=info cargo run

@debug/run:
	RUST_LOG=debug cargo run

@build/podman:
	@echo "Using podman to build image"
	podman build --arch amd64 -t ${IMAGE_NAME} .


@clean/podman:
	@echo "Cleaning podman image"
	podman image prune
	podman rmi ${IMAGE_NAME}
