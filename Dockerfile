FROM rust:1.64-buster AS app_builder
COPY api-server /app/src/api-server
COPY api-lib /app/src/api-lib
WORKDIR /app/src/api-server
RUN cargo build --release
RUN strip target/release/api-server


FROM debian:buster AS app
ENV LANG=C.UTF-8
# Install openssl
RUN apt-get update && apt-get install -y openssl curl net-tools
# Copy over the build artifact from the previous step and create a non root user
RUN useradd --create-home app
WORKDIR /home/app
RUN mkdir bin
COPY --chown=app --from=app_builder /app/src/api-server/target/release/api-server /home/app/bin/

USER app
CMD ["/home/app/bin/api-server"]
