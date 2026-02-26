.PHONY: srpm

srpm:
	# Extract version from Cargo.toml
	$(eval VERSION := $(shell grep '^version' ../Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'))

	# Update spec with current version
	sed -i 's/^Version:.*/Version:        $(VERSION)/' ../wallflow.spec

	# Create source tarball matching spec's expected layout
	mkdir -p /tmp/wallflow-$(VERSION)
	cp -r ../* /tmp/wallflow-$(VERSION)/ 2>/dev/null || true
	cp -r ../.[!.]* /tmp/wallflow-$(VERSION)/ 2>/dev/null || true
	tar czf /tmp/wallflow-$(VERSION).tar.gz -C /tmp wallflow-$(VERSION)

	# Build SRPM
	rpmbuild -bs ../wallflow.spec \
		--define "_sourcedir /tmp" \
		--define "_srcrpmdir $(outdir)"

	# Cleanup
	rm -rf /tmp/wallflow-$(VERSION) /tmp/wallflow-$(VERSION).tar.gz
