venv:
	python -m venv venv
	./venv/bin/pip install setuptools-rust

build: venv
	./venv/bin/python setup.py develop

run-simple: build
	./venv/bin/python simple.py
