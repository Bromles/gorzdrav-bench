# Golang version

How to run in PowerShell:

- Install Golang SDK from the [official website](https://go.dev/doc/install)
- `go build -ldflags "-s -w"`
- `.\measure.ps1`

How to clean after running:

- `go clean`
- Delete the `mockData` folder
