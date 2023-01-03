.PHONY: default
default:
	@echo "Try one of:"
	@echo "\tmake build-web"
	@echo "\tmake clean-web"

.PHONY: clean-web
clean-web:
	rm -rf ./dist

.PHONY: build-web
build-web: clean-web
	cargo build --release --no-default-features --target wasm32-unknown-unknown
	wasm-bindgen --out-dir ./dist/ --target web ./target/wasm32-unknown-unknown/release/talkie-game.wasm
	cp -R ./public/* ./dist/
	cp -R ./assets ./dist/
