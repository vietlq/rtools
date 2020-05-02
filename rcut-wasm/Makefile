.PHONY: all wasm webpack serve dist clean_dist

wasm:
	wasm-pack build

webpack:
	cd www && npm run build

serve:
	cd www && npm run start

all: wasm webpack

dist: clean_dist all

clean_dist:
	rm -rf www/dist
