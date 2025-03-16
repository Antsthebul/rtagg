# rtgen
## Rust Template Generator

### Overview
#### Supported files:
- YAML
- JSON

#### Purpose
Inspired by Terraform, **rtgen** is a CLI tool to easily aggregate various homogenous file types to produce a single, complete file. Rather than having an extremely verbose file, due to repeating object definitions, **rtgen** allows you to create a template, and "compile" to a single file, from smaller, modularized files, which results in much more maintable codebase.

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