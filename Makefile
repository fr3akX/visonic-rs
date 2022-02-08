HEAD_COMMIT := $(shell git rev-parse HEAD)
VERSION := $(shell git tag --contains ${HEAD_COMMIT}|tr -d 'v')
LATEST_TAG := $(shell git describe --tags --abbrev=0 --always | tr -d 'v')
ifeq ($(VERSION), )
VERSION := $(LATEST_TAG)
endif

build:
	DOCKER_BUILDKIT=1 docker build -t fr3akx/visonic-rs:${VERSION} -t fr3akx/visonic-rs:latest -f Dockerfile .