pi-gcc:## 	pi-gcc
	@gcc src/gnostr-pi.c -o $@ || $(shell which $@)
	@install ./$@ /usr/local/bin/ || $(shell which $@)
pi-clang:## 	pi-clang
	@clang src/gnostr-pi.c -o $@ || $(shell which $@)
	@install ./$@ /usr/local/bin/ || $(shell which $@)
pi-test:pi-gcc-test pi-clang-test gnostr-pi-test## 	pi-test
.PHONY:gnostr-pi-test
gnostr-pi-test:
	$(MAKE) gnostr-pi > /dev/null
	mkdir -p logs
	./gnostr-pi 360   > ./logs/360.txt
	./gnostr-pi 1000  > ./logs/1000.txt
	./gnostr-pi 360 1 > ./logs/360_1.txt
	./gnostr-pi 100 2 > ./logs/100_2.txt
	./gnostr-pi 1000 -253 > ./logs/1000_-253.txt
	./gnostr-pi 1000 -205 > ./logs/1000_-205.txt
	./gnostr-pi 1000 -250 > ./logs/1000_-250.txt
	git diff logs > diff.log #&& cat diff.log
pi-gcc-test:
	$(MAKE) pi-gcc > /dev/null
	mkdir -p logs
	./pi-gcc    360   > ./logs/360.txt
	./pi-gcc    1000  > ./logs/1000.txt
	./pi-gcc    360 1 > ./logs/360_1.txt
	./pi-gcc    100 2 > ./logs/100_2.txt
	./pi-gcc    1000 -253 > ./logs/1000_-253.txt
	./pi-gcc    1000 -205 > ./logs/1000_-205.txt
	./pi-gcc    1000 -250 > ./logs/1000_-250.txt
	git diff    logs > diff.log #&& cat diff.log
pi-clang-test:
	$(MAKE) pi-clang > /dev/null
	mkdir -p logs
	./pi-clang  360   > ./logs/360.txt
	./pi-clang  1000  > ./logs/1000.txt
	./pi-clang  360 1 > ./logs/360_1.txt
	./pi-clang  100 2 > ./logs/100_2.txt
	./pi-clang  1000 -253 > ./logs/1000_-253.txt
	./pi-clang  1000 -205 > ./logs/1000_-205.txt
	./pi-clang  1000 -250 > ./logs/1000_-250.txt
	git diff logs > diff.log #&& cat diff.log
pi-gcc-test2:
	@( \
	bash -c "pi-gcc   11111 | sed 's/\./_/'" \
)
pi-clang-test2:
	@( \
	bash -c "pi-clang 11111 | sed 's/\./_/'" \
)

pi-gcc-test3:
	@( \
	bash -c "pi-gcc   11111 | sed 's/3\.//'" \
)

pi-clang-test3:
	@( \
	bash -c "pi-clang 11111 | sed 's/3\.//'" \
)

xor-logs:pi-test## 	xor-logs
	gnostr-xor $(shell cat logs/1000_-250.txt) $(shell cat logs/1000_-205.txt)
	gnostr-xor $(shell cat logs/1000_-205.txt) $(shell cat logs/1000_-250.txt) > logs/gnostr-xor.log || $(MAKE) gnostr-install
	git add logs diff.log || true
xor-gnostr-post:## 	xor-gnostr-post
	gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) --content "$(shell make xor-logs)" -t gnostr -t gnostr-pi -t gnostr-xor -t nostr
pi-clean:
	@rm pi        2>/dev/null || true
	@rm pi-*      2>/dev/null || true
	@rm a.out     2>/dev/null || true
