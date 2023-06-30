FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 libfontconfig1 && rm -rf /var/lib/apt/lists/*

ARG APP=/usr/src/app

EXPOSE 8080

COPY target/release/server $APP/server
COPY dist $APP/dist

WORKDIR $APP

CMD ["./server", "--addr", "0.0.0.0" ]
