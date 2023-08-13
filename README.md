# Rust Editor

a simple editor implemented by rust

features has been implemented:

1. edit file

2. save file

3. search

4. Highlighting

if you want to add highlighting rules, please add related files to src/highlightkeys. The filename should be the suffix of corresponding language

eg. add rust highlighting rules

create a file called `rs.json` in src/highlightkeys

```json
"rs": {
        "primary_keys": [
            "as",
            "break",
            ...
        ],
        "secondary_keys": [
            "bool",
            "char",
            ...
        ]
    }
```

PS: the editor only supports comment semantic `// this is a comment`