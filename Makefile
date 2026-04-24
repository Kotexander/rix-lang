.PHONY: all run test clean

all: test

run: test
	-./test

test:
	cargo run
	gcc -o test test.o

clean:
	rm -f test test.o test.ll test-opt.ll
