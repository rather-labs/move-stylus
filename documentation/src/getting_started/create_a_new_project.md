# Create a New Project

To create a new Stylus project, you can use the Stylus CLI tool that you installed in the previous step. Open your terminal and run the following command:

```bash
move-stylus new counter
```

This command will create a new directory called `counter` with the basic structure of a Stylus project with all the necessary files and folders to get you started:

```
counter
├── Move.toml
├── .gitignore
└── sources/
    └── counter.move
```

Where:
- `Move.toml`: This is the manifest file for your Stylus project. It contains metadata about your project, such as name, and dependencies.
- `.gitignore`: This file specifies which files and directories should be ignored by Git version control.
- `sources/`: This directory contains the Move source files for your project. The `counter.move` the first module of the package.

In the next section we are going to implement a simple counter smart contract in the `counter.move` file.
