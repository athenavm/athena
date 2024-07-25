# Find all subdirectories
# Except minimal test, which needs to be built separately
SUBDIRS := $(shell find . -mindepth 1 -maxdepth 1 -type d -not -name minimal)

all:
	@for dir in $(SUBDIRS); do \
		echo "Building in $$dir..."; \
		cd $$dir && cargo athena build || { echo "Failed at command: cd $$dir && cargo athena build"; exit 1; }; \
    cd -; \
	done

.PHONY: all
