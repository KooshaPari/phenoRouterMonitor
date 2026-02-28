.PHONY: lint generate breaking all clean

lint:
	buf lint

generate:
	buf generate

breaking:
	buf breaking --against '.git#branch=main'

all: lint generate

clean:
	rm -rf rust/src/gen python/src/gen
