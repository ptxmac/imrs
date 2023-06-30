FROM ubuntu:22.04

ARG APP=/usr/src/app

EXPOSE 8080

COPY target/release/server $APP/server
COPY dist $APP/dist

WORKDIR $APP

CMD ["./server", "--addr", "0.0.0.0" ]
