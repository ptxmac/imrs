# IM(Db) R(u)s(t)

A collection of IMDB tool written in rust.

Heavily inspired by [noxern/graph](https://github.com/noxern/graph)

## Features

- [x] Fetch star ratings for each episode of a TV Show
- [x] Generate a plot for the above data
- [ ] TDB

## Tools

This project is split in 3 parts

### imrs 

This is a library / cli tool that interacts with IMDb.

### Server

A small backend written in [axum](https://github.com/tokio-rs/axum). Hosts the frontend and APIs for generating images, and replying to Slack command hooks.

### Frontend

Simple [yew](https://yew.rs/) frontend that queries the backend for a image.


## References:
* axum/yew setup based on https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/