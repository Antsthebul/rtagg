run-example-yaml:
	cargo run -- -t examples/yaml/main.yaml -e YAML

help:
	cargo run -- --help

run-example-json:
	cargo run -- -t examples/json/main.json -e JSON