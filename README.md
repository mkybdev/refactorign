# refactorign

.gitignore file refactoring tool

## Usage

### Install

To install *refactorign*, run:
```bash
cargo install --path .
``` 

### Usage
```bash
refactorign [OPTIONS]
```

#### Options
`-p, --path <PATH>`

Path to the .gitignore file to refactor.
If not provided, *refactorign* will look for a .gitignore file in the current directory.

`-d, --destination <DESTINATION>`

Destination path to the directory to place the refactored .gitignore file.
If not provided, the same directory as the original .gitignore file will be used.

`-o, --overwrite`

Whether to overwrite the original .gitignore file

`-r, --report`

Whether to generate a detailed report on refactoring

`-l, --level <LEVEL>`

Refactoring level (1 - 3, Higher level means more aggressive refactoring) [default: 2]

`-h, --help`

Print help

`-V, --version`

Print version