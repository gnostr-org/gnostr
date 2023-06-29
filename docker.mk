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
docker:## 	docker commands
#                          docker                    docker
	@awk 'BEGIN {FS = ":.*?######	"} /^[a-zA-Z_-]+:.*?######	/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.ONESHELL:
docker-start:###### 	detect whether docker is running...
	@( \
	    while ! docker system info > /dev/null 2>&1; do\
	    echo 'Waiting for docker to start...';\
	    if [[ '$(OS)' == 'Linux' ]]; then\
	     systemctl restart docker.service;\
	    fi;\
	    if [[ '$(OS)' == 'Darwin' ]]; then\
	     open --background -a /./Applications/Docker.app/Contents/MacOS/Docker;\
	    fi;\
	sleep 1;\
	done\
	)
docker-pull:docker-start###### 	pull alpine image
	docker pull alpine

docker-build-debian:###### 	docker build -f ./packaging/debian/Dockerfile .
	docker buildx build --progress=plain -f gui/packaging/debian/Dockerfile .
	docker build -v -f gui/packaging/debian/Dockerfile .
# vim: set noexpandtab:
# vim: set setfiletype make
