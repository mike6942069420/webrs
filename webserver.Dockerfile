FROM scratch

COPY ./target/x86_64-unknown-linux-musl/release/webrs /webserver

USER 1000:1000

EXPOSE 8080

# Run the binary
ENTRYPOINT ["/webserver"]