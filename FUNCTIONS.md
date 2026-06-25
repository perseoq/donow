# DoNow Functions

Functions are reusable scripts stored in `~/.donow/funcs/` or provided as **builtins**. Functions accept arguments and can return values.

---

## Builtin Functions

Builtins are available without creating any files. Call them with `F<name>(args...)`.

### `F<sum>(array)`

Sum of an array's elements (integers only).

```
x = F<sum>(a[1, 2, 3, 4, 5])    # → 15
```

### `F<avg>(array)`

Average of an array's elements (truncated integer division).

```
x = F<avg>(a[10, 20, 30])        # → 20
```

### `F<len>(value)`

Length of a string, array, list, or dict.

```
F<len>("hello")                  # → 5
F<len>(a[1, 2, 3])              # → 3
F<len>(l["a", "b"])             # → 2
F<len>(d["k": 1])               # → 1
```

### `F<echo>(...)`

Print arguments to stdout. Returns `null`.

```
F<echo>("hello", "world")        # prints: hello world
```

### `F<date>(fmt)`

Return the current date/time. Optional format string with `%Y`, `%s` substitutions.

```
F<date>()                        # → "2026"
F<date>(%s)                      # → unix timestamp
```

### `F<read>(prompt)`

Read a line from stdin with an optional prompt.

```
F<read>()                        # reads a line
F<read>("Enter name: ")          # shows prompt, reads line
```

### `F<exit>(code)`

Exit the process with an optional exit code (default 0).

```
F<exit>()                        # exit 0
F<exit>(1)                       # exit 1
```

---

## User-Defined Functions

Create a file in `~/.donow/funcs/`:

```
~/.donow/funcs/
├── encrypt
├── notify
├── validate
└── greet
```

### Function File Format

Function files use the same DSL as `DoNow`. Arguments are available as positional variables `%1`, `%2`, etc.

**`~/.donow/funcs/greet`**
```
echo "Hello, %1!"
echo "You are in @env mode."
```

### Calling Functions

```
deploy:
    echo "F<greet>(Alice)"
```

```
$ donow deploy @env=staging
Hello, Alice!
You are in staging mode.
```

### Functions with Multiple Arguments

**`~/.donow/funcs/notify`**
```
echo "NOTIFICATION"
echo "To: %1"
echo "Message: %2"
```

```
alert:
    echo "F<notify>(admin, Server is down)"
```

```
$ donow alert
NOTIFICATION
To: admin
Message: Server is down
```

### Functions Calling Functions

Functions can call other functions and builtins:

**`~/.donow/funcs/report`**
```
echo "Report generated: F<date>(%Y-%m-%d)"
echo "Items: %1"
```

```
summary:
    len = F<len>(a[1, 2, 3])
    echo "F<report>(%len)"
```

---

## Builtins vs User Functions

| Aspect | Builtin | User Function |
|---|---|---|
| Location | Compiled into binary | `~/.donow/funcs/<name>` |
| Performance | Instant | File read + expansion |
| Arguments | Evaluated expressions | String positions (%1, %2) |
| Return | Value (can be assigned) | Expanded text (inline in command) |
| Examples | `sum`, `avg`, `len`, `echo` | Any custom script |

---

## Listing Functions

```bash
ls ~/.donow/funcs/
```

---

## Error Handling

If a function file does not exist and no builtin matches, DoNow stops:

```
error: func 'missing' not found at ~/.donow/funcs/missing
```

---

## Best Practices

1. **Use builtins when possible** — they're faster and handle types properly
2. **Keep functions focused** — one task per function
3. **Document arguments** — use comments to show `%1`, `%2` meanings
4. **Test independently** — run function content as a block first
