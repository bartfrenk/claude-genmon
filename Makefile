.PHONY: run install reinstall uninstall clean

run:
	uv run claude-genmon

install:
	uv tool install .

reinstall:
	uv tool install --reinstall .

uninstall:
	-uv tool uninstall claude-genmon

clean: uninstall
	rm -rf dist build *.egg-info
	find . -type d -name '__pycache__' -not -path './.venv/*' -exec rm -rf {} +
