OFFICE=$(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
HOST=$(shell uname -n |cut -d . -f 1)

BUILD=${OFFICE}/.build-${HOST}

PROGS=strategy_find strategy_check strategy_draw
STRATEGIES=$(shell find . -iname "*.strategy" -type f)
STRAT_TEXS=$(patsubst %.strategy,%.tex,$(STRATEGIES))
STRAT_PDFS=$(patsubst %.strategy,%.pdf,$(STRATEGIES))


.SUFFIXES:

.PHONY: all

all : ${PROGS}

${PROGS} : % : ${BUILD}/%
	rm -f $@
	ln -s $< $@

${BUILD}/% : ${BUILD}/Makefile force
	make -C ${BUILD} $*

${BUILD}/Makefile : CMakeLists.txt Makefile
	cmake -B ${BUILD} -S ${OFFICE}

.PHONY: force
force : ;

.PHONY: test
test :
	make -C ${BUILD} test

${STRAT_TEXS} : %.tex : %.strategy strategy_draw
	./strategy_draw < $< > $@

${STRAT_PDFS} : %.pdf : %.tex
	lualatex --output-directory ${BUILD} $<
	mv ${BUILD}/$@ $@
