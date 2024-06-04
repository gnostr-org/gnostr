.PHONY:chat
gnostr-chat:chat## chat
chat:
	cd chat && make &&  go build -v -o /usr/local/bin/gnostr-chat && cd ..
	cd chat && make &&  go build -v -o /usr/local/bin/git-chat && cd ..
	cd chat && make &&  go build -v -o gnostr-chat && cd ..
	cp ./chat/gnostr-chat .
