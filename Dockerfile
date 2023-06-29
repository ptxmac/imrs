FROM debian:buster-slim
ARG APP=/usr/src/app

EXPOSE 8080

COPY target/release/server $APP/server
COPY dist $APP/dist

WORKDIR $APP

CMD ["./server"]
