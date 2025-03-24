FROM m.daocloud.io/docker.io/library/ubuntu:latest AS exchange-service
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/LISTS/*
#FROM alpine:latest AS opc-mqtt
#ENV TZ=Asia/Shanghai
#RUN ln -snf /usr/share/zoneinfo/$TimeZone /etc/localtime && echo $TimeZone > /etc/timezone
WORKDIR /app
COPY   ./target/release/exchange-service ./
ENTRYPOINT ./exchange-service -e prod

FROM m.daocloud.io/docker.io/library/ubuntu:latest AS order-service
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/LISTS/*
#FROM alpine:latest AS opc-mqtt
#ENV TZ=Asia/Shanghai
#RUN ln -snf /usr/share/zoneinfo/$TimeZone /etc/localtime && echo $TimeZone > /etc/timezone
WORKDIR /app
COPY   ./target/release/order-service ./
ENTRYPOINT ./order-service -e prod

FROM m.daocloud.io/docker.io/library/ubuntu:latest AS user-service
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/LISTS/*
#FROM alpine:latest AS opc-mqtt
#ENV TZ=Asia/Shanghai
#RUN ln -snf /usr/share/zoneinfo/$TimeZone /etc/localtime && echo $TimeZone > /etc/timezone
WORKDIR /app
COPY   ./target/release/user-service ./
ENTRYPOINT ./user-service -e prod

FROM m.daocloud.io/docker.io/library/ubuntu:latest AS risk-service
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/LISTS/*
#FROM alpine:latest AS opc-mqtt
#ENV TZ=Asia/Shanghai
#RUN ln -snf /usr/share/zoneinfo/$TimeZone /etc/localtime && echo $TimeZone > /etc/timezone
WORKDIR /app
COPY   ./target/release/risk-service ./
ENTRYPOINT ./risk-service -e prod