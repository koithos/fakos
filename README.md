# fakos

fakos (φακός; greek for lens) enables users to efficiently access the state, logs and metadata of different Kubernetes resources.

## Features

This cli utility can:

- [x] Get pod labels and annotations
- [x] Get node labels and annotations
- [ ] Get envs on pods
- [ ] Get pods by node
- [ ] Get coloured logs for different pods if getting logs by pod label
- [ ] Get Node health and metrics
- [ ] Get GPU state
- [ ] Get resources in unhealthy state
- [ ] Rollout restart changes watch

## Installation

### Via Cargo

If you have Rust installed, you can install fakos using cargo:

```bash
cargo install fakos
```

### Via kubectl krew

Once published to the krew index, you can install fakos as a kubectl plugin:

```bash
kubectl krew install fakos
kubectl fakos get pods
```

### Via homebrew

```bash
brew tap koithos/cli/fakos
brew install fakos
```

### From Source

Clone the repository and build from source:

```bash
git clone https://github.com/koithos/fakos.git
cd fakos
cargo build --jobs $(sysctl -n hw.logicalcpu) --release
# The binary will be available at target/release/fakos
```

## Usage

### Global Options

- `--kubeconfig <PATH>`: Path to kubeconfig file (default: `~/.kube/config` or `$KUBECONFIG`)
- `-v, --verbose`: Enable verbose logging (use multiple times for increased verbosity)
  - `-v`: WARN level
  - `-vv`: INFO level
  - `-vvv`: DEBUG level
  - `-vvvv`: TRACE level
- `--log-format <FORMAT>`: Log format to use (default: `plain`, options: `plain`, `json`)

### Get Pods

List and inspect pods in your Kubernetes cluster.

#### Output Formats

```bash
# Normal output (default)
fakos get pods -o normal

# Wide output (shows additional columns)
fakos get pods -o wide
```

#### Labels and Annotations

```bash
# Display only labels for pods
fakos get pods --labels

# Display only annotations for pods
fakos get pods --annotations

# Display both labels and annotations
fakos get pods --labels --annotations

# Get labels for a specific pod
fakos get pods my-pod --labels -n default

# Get labels for a specific pod on a specific node in a specific namespace
fakos get pods my-pod --labels --annotations -n default --node node-1
```

### Get Nodes

#### Labels and Annotations

```bash
# Display only labels for nodes
fakos get nodes --labels

# Display only annotations for nodes
fakos get nodes --annotations

# Display both labels and annotations
fakos get nodes --labels --annotations

# Get labels for a specific node
fakos get nodes node-1 --labels
```

## Examples

```bash
# Get all pods in production namespace with verbose logging
fakos get pods -n production -vv

# Get all pods on a specific node with labels
fakos get pods --node worker-1 --labels

# Get node information with custom kubeconfig
fakos get nodes --kubeconfig /path/to/kubeconfig

# Get wide output for all pods across all namespaces
fakos get pods -A -o wide

# Get annotations for a specific pod with debug logging
fakos get pods my-app-pod -n default --annotations -vvv
```
