docker:## 	docker commands
#                          docker                    docker
	@awk 'BEGIN {FS = ":.*?######	"} /^[a-zA-Z_-]+:.*?######	/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.ONESHELL:
docker-start:######	detect whether docker is running...
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
docker-pull:docker-start######	pull alpine image
	docker pull alpine

# vim: set noexpandtab:
# vim: set setfiletype make
