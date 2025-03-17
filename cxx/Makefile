OFFICE=$(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
HOST=$(shell uname -n |cut -d . -f 1)

BUILD=${OFFICE}/.build-${HOST}

PROGS=strategy_find strategy_check

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
