# Golang version

How to run:

- Install Golang SDK from the [official website](https://go.dev/doc/install)
- `go build -ldflags "-s -w"`
- Run benchmark:
  - Windows with PowerShell: `.\measure.ps1`
  - macOS/Linux: `./measure.sh`

How to clean after running:

- `go clean`
- Delete the `mockData` folder
