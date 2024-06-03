# Find all subdirectories
SUBDIRS := $(shell find . -mindepth 1 -maxdepth 1 -type d)

all:
	@for dir in $(SUBDIRS); do \
  	if [ -d "$${dir}/script" ]; then \
		  echo "Building in $${dir}/script..."; \
		  cd $${dir}/script && cargo check || { echo "Failed at command: cd $${dir}script && cargo check"; exit 1; }; \
      cd -; \
    else \
			echo "No script directory in $${dir}, skipping..."; \
    fi; \
	done

.PHONY: all
