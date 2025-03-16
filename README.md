# rtgen
## Rust Template Generator

### Overview
#### Supported files:
- YAML
- JSON

#### Purpose
Inspired by Terraform, rtgen is a CLI tool to easily aggregate various files homogenous file types to create a single file. Rather than having an extremely verbose file, due to repeating object definitions, you can create a template, and "compile" to a single file, aggregating small, modularized files, to help make your codebase much more maintainable.

### Usage
 Follow these steps or run the example

1. Create a template. `main.yaml`.
```yaml
greetings:
    - "{{ lookup('*') }}"
```
2. Create the directory `/greetings`
3. Create a file 2 files within this directory 

**esp.yaml**
```yaml
Hola Mundo!
```

**eng.yaml**
```yaml
Hello World
```
4. Run `rtgen -t main.yaml` and you should see
```yaml
greetings:
    - Hola Mundo!
    - Hello World
```
#### Example
Run `make run-example-yaml` to how `examples/main.yaml` is used as a template, and creates the output seen in `output.yaml`

## Caveats
- JSON output, although correct, may show map objects with keys in a different order
- Currently child tempaltes cannot contain refernces other files (it will not be evaulated and is returned as an empty string)

See the docs for more info