# rsid3 - a simple, command line ID3v2 tag editor designed for scripting
# Copyright (C) 2024  Randoragon
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along
# with this program; if not, write to the Free Software Foundation, Inc.,
# 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
DESTDIR=
PREFIX=/usr/local
MANPREFIX=$(PREFIX)/share/man

.PHONY: all usage install uninstall

all: usage

usage:
	@echo 'This Makefile (un)installs the rsid3 binary and associated man pages.'
	@echo 'It exists because cargo cannot handle installing man pages on its own.'
	@echo 'For development, building, running tests, etc., use cargo as normal.'
	@echo
	@echo 'Available targets:'
	@echo '- install:   Installs rsid3 at "$$DESTDIR/$$PREFIX/bin/rsid3"'
	@echo '             and the man pages under "$$DESTDIR/$$MANPREFIX/"'
	@echo '             (defaults: PREFIX=/usr/local, MANPREFIX="$$PREFIX/share/man")'
	@echo '- uninstall: Removes installed files, if any'

install:
	cargo install --path . --root $(DESTDIR)$(PREFIX)
	@chmod -- 755 $(DESTDIR)$(PREFIX)/bin/rsid3
	@mkdir -p $(DESTDIR)$(MANPREFIX)/man1
	cp -- rsid3.1 $(DESTDIR)$(MANPREFIX)/man1
	@chmod -- 644 $(DESTDIR)$(MANPREFIX)/man1/rsid3.1

uninstall:
	rm -f -- $(DESTDIR)$(PREFIX)/bin/rsid3
	rm -f -- $(DESTDIR)$(MANPREFIX)/man1/rsid3.1
