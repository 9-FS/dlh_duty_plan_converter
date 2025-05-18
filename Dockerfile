ARG RUST_VERSION="1.86"

FROM rust:$RUST_VERSION AS builder
WORKDIR "/app/"
COPY "." "."
RUN cargo build --release

FROM alpine
WORKDIR "/app/"
COPY --from=builder "/app/target/release/dlh_duty_plan_converter" "."

CMD ["./dlh_duty_plan_converter"]


# MANUAL BUILD:

# build docker image, save in tar, remove image so only tar remains
# docker build -t "9-fs/dlh_duty_plan_converter:latest" --no-cache . && docker save "9-fs/dlh_duty_plan_converter:latest" > "docker-image.tar" && docker rmi "9-fs/dlh_duty_plan_converter:latest"

# on deployment environment load docker image from tar file
# docker load < "/mnt/user/appdata/docker-image.tar"