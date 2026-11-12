# CncCraft: kurze Wege zu build.sh
.PHONY: run test lint docs mac mac-intel linux windows all clean

run:
	cargo run --release

test:
	cargo test --workspace

mac:
	./build.sh mac

mac-intel:
	./build.sh mac-intel

linux:
	./build.sh linux

windows:
	./build.sh windows

all:
	./build.sh all

clean:
	./build.sh clean

# Bauen und prüfen, dass es keine einzige Warnung gibt
# die Website (docs/) neu erzeugen und ihre Links prüfen
docs:
	python3 -I tools/build_docs.py
	python3 -I tools/check_docs.py

lint:
	@touch src/main.rs crates/shape-edit/src/lib.rs
	@out=$$(cargo build --release --workspace --all-targets 2>&1); echo "$$out" | tail -3; if echo "$$out" | grep -q '^warning'; then echo "Es gibt Warnungen."; exit 1; fi
