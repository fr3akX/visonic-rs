HEAD_COMMIT := $(shell git rev-parse HEAD)
VERSION := $(shell git tag --contains ${HEAD_COMMIT})
ifeq ($(VERSION), )
VERSION := latest
endif

build:
	DOCKER_BUILDKIT=1 docker build -t fr3akx/visonic-rs:${VERSION} -f Dockerfile .