SUBDIRS=computer-lab-benchmark llama2-c gapbs wanco-artifacts wamrc-artifacts wasmedge-artifacts

all: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@

clean:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -C $$dir clean; \
	done

.PHONY: all clean $(SUBDIRS)
