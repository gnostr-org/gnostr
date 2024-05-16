.PHONY:go-chat
gnostr-go-chat:go-chat## go
go:go-chat
go-chat:
	cd go && make &&  go build -v -o /usr/local/bin/gnostr-go-chat && cd ..
	cd go && make &&  go build -v -o gnostr-go-chat-test-0.0.0-linux-amd64.tar.gz && cd ..
	cp ./go/gnostr-go-chat .
