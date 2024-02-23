DOCKER=$(shell which docker)
dockerx:docker-buildx## 	        docker-buildx
docker-build:gnostr-install## 		docker build -f Dockerfile -t miniscript .
	@gnostr-docker start
	@$(DOCKER) pull ghcr.io/gnostr-org/gnostr:latest
	@$(DOCKER) build -f Dockerfile -t gnostr .
docker-buildx:## 		docker buildx build sequence
	@gnostr-docker start
	@$(DOCKER) run --privileged --rm tonistiigi/binfmt --install all
	@$(DOCKER) buildx ls
	@$(DOCKER) buildx create --use --name gnostr-buildx || true
	@$(DOCKER) buildx build -t miniscript --platform linux/arm64,linux/amd64 .
	@$(DOCKER) buildx build -t miniscript --platform linux/$(TARGET) . --load
