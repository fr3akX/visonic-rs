HEAD_COMMIT := $(shell git rev-parse HEAD)
VERSION := $(shell git tag --contains ${HEAD_COMMIT}|tr -d 'v')
LATEST_TAG := $(shell git describe --tags --abbrev=0 --always | tr -d 'v')
ifeq ($(VERSION), )
VERSION := $(LATEST_TAG)
endif

build:
	DOCKER_BUILDKIT=1 docker build -t fr3akx/visonic-rs:${VERSION} -t fr3akx/visonic-rs:latest \
			--platform linux/armv7 --platform linux/amd64 -f Dockerfile .
extract:
	docker container create --name extract fr3akx/visonic-rs:${VERSION}
	docker container cp extract:/visonic/visonic ./bin/visonic
	docker container cp extract:/visonic/visonic-arm ./bin/visonic-arm
	docker container rm -f extract

publish:
	docker push fr3akx/visonic-rs:${VERSION}
	docker push fr3akx/visonic-rs:latest

all: build publish