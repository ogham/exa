COMPILER = rustc

all:
	$(COMPILER) exa.rs

test:
	$(COMPILER) --test exa.rs -o exa-test
	./exa-test

clean:
	rm -f exa
	rm -f exa-test



