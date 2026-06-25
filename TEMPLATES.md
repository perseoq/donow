# DoNow Templates

Templates are reusable text snippets stored in `~/.donow/templates/`. They are expanded inline wherever `T<name>` appears in a command.

---

## Creating a Template

Create a file in `~/.donow/templates/`:

```
~/.donow/templates/
├── header
├── footer
├── greeting
└── notice
```

Each file contains plain text:

**`~/.donow/templates/header`**
```
========================
BUILD START
========================
```

**`~/.donow/templates/footer`**
```
========================
BUILD END
========================
```

---

## Using Templates

Reference a template with `T<name>` anywhere in a command:

```
build:
    echo "T<header>"
    cargo build
    echo "T<footer>"
```

Execution:

```
$ donow build
========================
BUILD START
========================
   Compiling ...
========================
BUILD END
========================
```

---

## Templates with Variables

Template files can contain `%var`, `@param`, and `$var` references. When expanded, they inherit the current block's scope:

**`~/.donow/templates/notice`**
```
NOTICE: deploying %app_name to @env
```

**DoNow block:**
```
deploy:
    app_name = "my-service"
    echo "T<notice>"
```

```
$ donow deploy @env=staging
NOTICE: deploying my-service to staging
```

---

## Templates with Functions

Templates can call builtin functions:

**`~/.donow/templates/report`**
```
Date: F<date>(%Y-%m-%d)
Items: %count
```

```
summary:
    count = F<len>(a[1, 2, 3])
    echo "T<report>"
```

```
$ donow summary
Date: 2026-06-24
Items: 3
```

---

## Listing Templates

```bash
ls ~/.donow/templates/
```

---

## Error Handling

If a template file does not exist, DoNow stops with an error:

```
error: template 'missing' not found at ~/.donow/templates/missing
```

---

## Best Practices

1. **Keep templates small** — they are expanded inline into a single shell command
2. **Use simple text** — template content is inserted literally
3. **Use variables** — `%var`, `@param` make templates dynamic
4. **Organize by purpose** — one file per template
