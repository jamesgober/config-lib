<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>config-lib</b><br>
    <sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./README.md" title="Documentation"><b>DOCS</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="./GUIDELINES.md" title="Developer Guidelines"><b>GUIDELINES</b></a>
    </sup>
</div>

<br>

## Table of Contents
- **[Installation](#installation)**
- **[Feature Flags](#feature-flags)**
- **[Getting Started](#getting-started)**
  - **[Basic Setup](#basic-setup)**
  - **[parse_from_file()](#parse_from_file)**

<hr>
<br>

<h2 id="installation">Installation</h2>


### 📋 Install Manually
```toml
[dependencies]
config-lib  = "0.4.0"
```
> Add this to your `Cargo.toml`:


#### Install Features
```toml
[dependencies]

# Single feature
config-lib = { version = "0.4.0", features = ["async"] }

# Multiple features
config-lib = { version = "0.4.0", features = ["async, noml"] }

# Disable Default
config-lib = { version = "0.4.0", features = ["async"] }
```
> **[Features](#feature-flags)**

<br>


### 📋 Install via Terminal
```bash
# Basic installation
cargo add config-lib

# Enable a feature
cargo add config-lib --features async

# Enable multiple features
cargo add config-lib --features async,noml

# Disable Default
cargo add config-lib --features async
```




<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="feature-flags">Feature Flags</h2>

| Feature               | Default | Description |
|-----------------------|:-------:|---------------------------------------------------------------|
| `conf`                |  ✅     | Conf file support (built in parser)                           |
| `noml`                |  ❌     | NOML file support.                                            |
| `toml`                |  ❌     | TOML file support.                                            |
| `json`                |  ❌     | JSON file support.                                            |
| `async`               |  ❌     | Enables async functions for file operations and HTTP includes | 
| `chrono`              |  ❌     | Enables DateTime support with chrono integration              |

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="getting-started">Getting Started</h2>

<br>

<h3 id="basic-setup">Basic Setup</h3>

```rust

// Example Basic Setup

```

<br>

<h3 id="basic-setup">Basic Setup</h3>

```rust

// Example Basic Setup

```


<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

---

<div align="center">
    <b>NOML Rust API Reference</b><br>
    <sub>The most advanced configuration language with revolutionary format preservation</sub>
</div>