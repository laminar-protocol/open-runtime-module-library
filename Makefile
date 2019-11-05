check-no-std: githooks
	cd example && cargo check --no-default-features && cd ..

check-tests: githooks
	cargo check --all --tests

test: githooks
	cargo test --all

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

githooks: .git/hooks $(GITHOOKS_DEST)

init: githooks
