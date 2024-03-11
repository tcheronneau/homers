# HOMERS

This project has the purposed to be a replacement for [Varken](https://github.com/Boerderij/Varken).   
Since InfluxDB is not a good option for me, I decided to use prometheus and to build an exporter. 

It's not ready yet, but some features are already there. 

## Getting Started

The easiest way to start the project is to use docker.  
Image can be found at [Docker Hub](https://hub.docker.com/repository/docker/mcth/homers). 

``` 
docker run -d -p 8000:8000 -v ./config.toml:/app/config.toml mcth/homers
```
You can either use configuration file or environment variables.   
Each config key has a correspondent environment variable.  
Example: `config.toml`:
```toml
[server]
port=8000
address="0.0.0.0"
[sonarr]
address="http://localhost:8989"
api_key=""

[tautulli]
address="http://localhost:8181"
api_key=""
```

## Building the project 

To build the project you need to have `cargo` installed.  
Then you can run `cargo build --release`. 

Alternatively you can also use nix.  
To build the project using nix, you can run `nix build .#`.   
And for the docker image : 
```
nix build .#docker
docker load < ./result
```

## Advancement

So far it's not doing much.   
- Retrieve Sonarr today's calendar
- Retrieve Tautulli activity
- Retrieve Tautulli library information

## Roadmap

The point is to at least support what Varken was doing. 
There will also be a Grafana dashboard. 


## Acknowledgments

Since it's pretty much my first Rust project the code is far from perfect.  
It's heavily inspired from the work of [Lars Strojny](https://github.com/lstrojny/prometheus-weathermen) that's provide a really good example of exporter in rust.  
