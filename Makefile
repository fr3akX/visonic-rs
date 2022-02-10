HEAD_COMMIT := $(shell git rev-parse --short HEAD)
ifeq ($(VERSION), )
VERSION := $(HEAD_COMMIT)-snapshot
endif

version:
	@echo $(VERSION)
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
