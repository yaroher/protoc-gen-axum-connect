INSPIRED BY 
 - https://github.com/AThilenius/axum-connect
 - https://github.com/neoeinstein/protoc-gen-prost

Install:

```bash
cargo install --git https://github.com/yaroher/protoc-gen-axum-connect
```

Serde:
```bash
protoc --axum-connect_out=serde=true:. --axum-connect_opt=serde=true path/to/your/proto/file.proto
```