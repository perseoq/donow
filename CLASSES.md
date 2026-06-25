# DoNow Classes

Classes are reusable module definitions stored in `~/.donow/classes/`. They are referenced with `C<name>` and can contain multiple blocks, variables, and logic that can be composed into your workflows.

---

## Creating a Class

Create a file in `~/.donow/classes/`:

```
~/.donow/classes/
├── Server
├── Database
└── Config
```

Each file uses the same DSL as `DoNow`:

**`~/.donow/classes/Server`**
```
deploy:
    echo "deploying server..."
    echo "host: @host"
    echo "region: @region"

status:
    echo "checking server status..."
    curl -s http://@host/health

cleanup:
    echo "cleaning up server @host"
```

---

## Using Classes

Reference a class with `C<name>` in a command:

```
provision:
    echo "C<Server>"
```

When expanded, the entire class file content is injected into the command.

---

## Use Cases

### Configuration Classes

**`~/.donow/classes/Config`**
```
app_name = "my-service"
version = "3.1.0"
```

```
build:
    echo "C<Config>"
    echo "building %app_name v%version"
```

### Environment Classes

**`~/.donow/classes/Production`**
```
@env = "production"
@host = "api.example.com"
@port = "443"
```

```
deploy-prod:
    echo "C<Production>"
    echo "deploying to @host:@port"
```

---

## Classes vs Templates vs Functions

| Feature | Template `T<>` | Function `F<>` | Class `C<>` |
|---|---|---|---|
| Content | Text/strings | Executable DSL | Multi-block module |
| Args | No | Yes (`%1`, `%2`) | No |
| Return | Inline text | Inline text | Inline text |
| Use case | Static snippets | Computed logic | Module definitions |

---

## Listing Classes

```bash
ls ~/.donow/classes/
```

---

## Error Handling

If a class file does not exist, DoNow stops:

```
error: class 'missing' not found at ~/.donow/classes/missing
```

---

## Best Practices

1. **Keep classes focused** — one domain per file
2. **Use for configuration** — centralize constants and defaults
3. **Combine with templates** — `T<name>` inside class files for modular snippets
4. **Document parameters** — comment what `@params` the class expects
