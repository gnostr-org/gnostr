.PHONY:go-chat
gnostr-go-chat:go-chat## go
go:go-chat
go-chat:
		cd go && go build -v -o /usr/local/bin/gnostr-go-chat && cd ..
