# Find all subdirectories
SUBDIRS := $(shell find . -mindepth 1 -maxdepth 1 -type d)

all:
	@for dir in $(SUBDIRS); do \
		echo "Building in $$dir..."; \
		cd $$dir && cargo athena build || { echo "Failed at command: cd $$dir && cargo athena build"; exit 1; }; \
    cd -; \
	done

.PHONY: all
