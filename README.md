# DNS Updater

A lightweight, efficient, and reliable dynamic DNS (DDNS) updater service written in Rust. It periodically checks for changes in your public IP address and automatically updates your configured dynamic DNS records.

Designed for simplicity and robustness, it leverages Nix with Home Manager for easy and declarative configuration on NixOS or any system with Nix.

## Features

- **Multiple Provider Support**: Out-of-the-box support for DuckDNS, FreeDNS, and OVH.
- **IPv4 and IPv6**: Can update records for both `A` (IPv4) and `AAAA` (IPv6) records.
- **Interface Monitoring**: Monitors a specific network interface for IP changes.
- **Efficient**: Uses an asynchronous (tokio) runtime and only sends updates when your IP address actually changes.
- **Persistent State**: Remembers the last-sent IP to avoid redundant API calls to your DNS provider.
- **Easy Deployment**: Comes with a Nix Flake for simple, reproducible setups using Home Manager.
- **Configurable Polling**: Set a custom polling interval for each DNS record you want to update.

## Supported Providers

The service is configured via environment variables. The `DNS_TUPLES` variable holds the configuration for one or more DNS records. Each record is a semicolon-delimited string, and multiple records are separated by commas.

The general format is: `PROVIDER;...`

Here are the specific formats for each supported provider:

- **DuckDNS**: `DD;TOKEN;VERSION;POLL_SECS;SUBDOMAIN_NAME`
  - `TOKEN`: Your DuckDNS account token.
  - `VERSION`: `ipv4` or `ipv6`.
  - `POLL_SECS`: The interval in seconds to check for an IP change. Set to `0` to check only once on startup.
  - `SUBDOMAIN_NAME`: The DuckDNS subdomain you want to update (e.g., `my-domain`).

- **FreeDNS**: `FD;TOKEN;VERSION;POLL_SECS`
  - `TOKEN`: The update token for your FreeDNS record.
  - `VERSION`: `ipv4` or `ipv6`.
  - `POLL_SECS`: The interval in seconds to check for an IP change. Set to `0` to check only once on startup.

- **OVH**: `OVH;USERNAME;PASSWORD;SUBDOMAIN;VERSION;POLL_SECS`
  - `USERNAME`: Your DynHost username.
  - `PASSWORD`: Your DynHost password.
  - `SUBDOMAIN`: The full subdomain you want to update (e.g., `home.example.com`).
  - `VERSION`: `ipv4` or `ipv6`.
  - `POLL_SECS`: The interval in seconds to check for an IP change. Set to `0` to check only once on startup.

## Configuration

The application is configured using two environment variables:

- `INTERFACE`: The network interface to monitor for IP address changes (e.g., `eth0`, `wlan0`).
- `DNS_TUPLES`: A comma-separated list of DNS provider configurations (see formats above).

**Example `DNS_TUPLES` value**:

```
"DD;ax...1;ipv4;300;my-domain,OVH;user-dyn;pa...ss;home.example.com;ipv6;600"
```

## Usage

### With Nix & Home Manager (Recommended)

This project includes a [Nix Flake](https://nixos.wiki/wiki/Flakes) that provides a Home Manager module for easy configuration.

1.  Add the flake to your `home.nix` inputs:

    ```nix
    # home.nix
    {
      inputs = {
        # ... your other inputs
        dns-updater.url = "github:juancabe/dns-updater";
      };
    }
    ```

2.  Import the module and configure the service:

    ```nix
    # home.nix
    { inputs, pkgs, ... }: {
      imports = [
        # ... your other imports
        inputs.dns-updater.homeManagerModules.default
      ];

      services.dns-updater = {
        enable = true;
        interface = "wlan0"; # The network interface to watch
        dnsTuples = [
          # Update my-domain.duckdns.org for IPv4 every 5 minutes
          "DD;your-duckdns-token;ipv4;300;my-domain"

          # Update home.example.com at OVH for IPv6 every 10 minutes
          "OVH;your-ovh-username;your-ovh-password;home.example.com;ipv6;600"

          # Update a FreeDNS record for IPv4 once at startup
          "FD;your-freedns-token;ipv4;0"
        ];
      };
    }
    ```

3.  Rebuild your Home Manager configuration:
    ```sh
    home-manager switch --flake .
    ```

### With Cargo

You can also build and run the service manually using Cargo.

1.  **Clone the repository**:

    ```sh
    git clone https://github.com/your-github-username/dns-updater
    cd dns-updater
    ```

2.  **Set the environment variables**:

    ```sh
    export INTERFACE="eth0"
    export DNS_TUPLES="DD;your-token;ipv4;300;your-domain"
    ```

3.  **Run the service**:
    ```sh
    cargo run
    ```

## Nix Flake

The `flake.nix` provides the following outputs:

- **`packages.<system>.default`**: The `dns-updater` binary, built for the specified system.
  You can run it directly with `nix run github:your-github-username/dns-updater`.

- **`homeManagerModules.default`**: The Home Manager module for declaratively configuring the service, as shown in the usage guide above. This is the intended way to use the flake.
