# DoNow Language Manual

## 1. Overview

DoNow files live at `~/.donow/DoNow` and contain **named blocks**. Each block is a sequence of statements that can be executed by name from the CLI.

```
build:
    cargo build
    cargo test

deploy:
    echo "deploying to @env"
    git push origin main
```

---

## 2. Blocks

A block starts with a **name** followed by `:` and a body. The body can be **indented** (4 spaces) or wrapped in `{ }`:

```
# Indented body (4 spaces)
build:
    echo "building..."
    cargo build

# Brace-delimited body (enables : assignment)
deploy:
    {
    "deploying to @env" : $msg
    echo "%msg"
    }
```

---

## 3. Statements

### 3.1 Commands

Any line that isn't an expression or control flow is treated as a **shell command**:

```
cargo build
git push origin main
echo "hello %name"
```

Variables (`%var`, `@param`, `$var`) and references (`T<name>`, `F<name>`) are expanded before execution.

### 3.2 Variable Assignment

Use `=` outside `{ }`:

```
x = 5
name = "Alice"
result = $x + 10 * 2
```

Use `:` inside `{ }` (target on left, `$var` on right):

```
{
x + 5 : $result
"status: ok" : $msg
}
```

### 3.3 String Literals

Double-quoted strings:

```
x = "hello world"
echo "the value is %x"
```

### 3.4 Comments

Line comments use `#`:

```
# This is a comment
x = 5  # inline comment
```

Multiline comments use `''`:

```
'' this is a
multiline comment ''
```

---

## 4. Variables

### 4.1 Variable Types

| Type | Syntax | Example |
|---|---|---|
| Integer | `123` | `x = 42` |
| String | `"..."` | `name = "Alice"` |
| Bool | `true` / `false` | `flag = true` |
| Array | `a[...]` | `nums = a[1, 2, 3]` |
| List | `l[...]` | `items = l["a", "b"]` |
| Dict | `d[...]` | `cfg = d["host": "local"]` |
| Null | - | returned by some operations |

### 4.2 Variable References

| Prefix | Scope | Example |
|---|---|---|
| `$var` | Local variable | `$x`, `$name` |
| `%var` | Expands in commands | `echo %name` |
| `@var` | CLI parameter | `echo @env` |

### 4.3 Arrays (`a[]`)

Numeric arrays with mathematical operations:

```
nums = a[10, 20, 30]
first = $nums[0]       # index access → 10
```

### 4.4 Lists (`l[]`)

Simple lists (heterogeneous):

```
items = l["apple", "banana", 42]
second = $items[1]     # → "banana"
```

### 4.5 Dicts (`d[]`)

Key-value dictionaries (JSON-like):

```
config = d["host": "localhost", "port": 8080]
host = $config.host    # dot access → "localhost"
```

### 4.6 Index and Dot Access

```
$arr[0]                # array/list index
$arr[$i + 1]           # computed index
$dict.key              # dict dot access
$arr[0][1]             # nested
```

---

## 5. Operators

### 5.1 Arithmetic

| Op | Meaning | Example |
|---|---|---|
| `+` | Addition | `x + 5` |
| `-` | Subtraction | `x - 1` |
| `*` | Multiplication | `x * 2` |
| `/` | Integer division | `x / 3` |

Precedence: `* /` > `+ -` (standard arithmetic).

### 5.2 Comparison

| Op | Meaning |
|---|---|
| `==` | Equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |
| `>!` | Greater than or equal (same as `>=`) |
| `<!` | Less than or equal (same as `<=`) |
| `!=` | Not equal (structural) |

### 5.3 Logical

| Op | Meaning |
|---|---|
| `and` | Logical AND |
| `or` | Logical OR |
| `!` | Logical NOT |

### 5.4 Builtin Functions

Called via `F<name>(args...)`:

```
sum = F<sum>(a[1, 2, 3])        # → 6
avg = F<avg>(a[10, 20, 30])     # → 20
len = F<len>("hello")           # → 5
```

See [FUNCTIONS.md](FUNCTIONS.md) for the full list.

### 5.5 Templates

Inline template expansion via `T<name>`:

```
echo "T<header>"
```

Loads `~/.donow/templates/<name>` and inserts the content.

See [TEMPLATES.md](TEMPLATES.md).

---

## 6. Control Flow

### 6.1 If / Else

```
? if $x == 42:
    echo "x is 42"
e?:
    echo "x is not 42"
```

Nested if-else chains work:

```
? if $score >= 90:
    echo "grade A"
e?:
    ? if $score >= 80:
        echo "grade B"
    e?:
        echo "grade C or lower"
```

### 6.2 While

```
i = 0
w! $i < 5:
    echo "i = %i"
    i = $i + 1
```

### 6.3 For

```
# Over array
f! $i in a[10, 20, 30]:
    echo "%i"

# Over list
f! $name in l["Alice", "Bob"]:
    echo "hello %name"

# Over variable array
nums = a[1, 2, 3]
f! $n in $nums:
    sum = $sum + $n
```

---

## 7. Execution Order

### 7.1 Priority Blocks `( ... )`

Content inside `()` executes **before** normal statements:

```
x = 1
(echo "runs first")
echo "x = %x"        # → x = 1
```

### 7.2 Deferred Blocks `[ ... ]`

Content inside `[]` executes **after** normal statements:

```
x = 1
[echo "runs last"]
x = 2
echo "x = %x"        # → x = 2
# then deferred runs
```

### 7.3 Combined

```
x = 10
[x = 99]             # deferred: runs last
(x = 5)              # priority: runs first
echo "x = %x"        # normal: runs second
# deferred runs third → x = 99
```

---

## 8. Templates, Functions, Classes

| Feature | Syntax | Location |
|---|---|---|
| Template | `T<name>` | `~/.donow/templates/<name>` |
| Function | `F<name>(args)` | `~/.donow/funcs/<name>` or builtin |
| Class | `C<name>` | `~/.donow/classes/<name>` |

See the dedicated documentation files for each.

---

## 9. CLI Reference

```
donow <block> [@param=value ...] [--flag ...]

Commands:
  <block>              Name of the block to execute
  @param=value         Set a CLI parameter (accessible via @param)
  --flag               Shorthand for @flag=true
  --key=value          Shorthand for @key=value
  --list / -l          List available blocks
  --init               Create ~/.donow/DoNow with example
  --help / -h          Show help

Examples:
  donow build
  donow deploy @env=prod --verbose
  donow test
  donow --list
```

---

## 10. Error Handling

| Error | Behavior |
|---|---|
| Block not found | Shows error + lists available blocks |
| Variable undefined | Stops execution, shows variable name |
| Command fails (exit ≠ 0) | Stops execution, shows command and exit code |
| Parse error | Shows line:column + description |
| File not found | Shows path and error |
