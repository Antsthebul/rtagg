help:
	cargo run -- --help

run-example-yaml:
	cargo run -- -t examples/yaml/main.yaml -e YAML

run-example-json:
	cargo run -- -t examples/json/main.json -e JSON