OS_TYPE := ""

ifeq ($(OS),Windows_NT)
    OS_TYPE := win
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        OS_TYPE := linux
    endif
    ifeq ($(UNAME_S),Darwin)
        OS_TYPE := osx
    endif
endif

.DEFAULT_GOAL := build

.PHONY: clean

clean:
	rm -rf XPHTTPBridge/

setup-plugin:
	mkdir -p XPHTTPBridge/64
	cp config.example.ini XPHTTPBridge/config.ini
	cp LICENSE XPHTTPBridge/LICENSE
	cp README.md XPHTTPBridge/README.md

copy-linux:
	cp target/debug/libxphttpbridge.so XPHTTPBridge/64/lin.xpl

copy-win:
	cp target/debug/xphttpbridge.dll XPHTTPBridge/64/win.xpl

copy-osx:
	cp target/debug/libxphttpbridge.dylib XPHTTPBridge/64/osx.xpl


build: clean setup-plugin
	echo "Building for $(OS_TYPE)"
	cargo build
	if [ "$(OS_TYPE)" = "linux" ]; then $(MAKE) copy-linux; fi
	if [ "$(OS_TYPE)" = "win" ]; then $(MAKE) copy-win; fi
	if [ "$(OS_TYPE)" = "osx" ]; then $(MAKE) copy-osx; fi
	echo "Output in XPHTTPBridge, copy as-is, enjoy!"
