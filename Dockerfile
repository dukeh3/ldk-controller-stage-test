FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl gpg \
    && curl -fsSL http://apt.h3/h3.gpg | gpg --dearmor -o /etc/apt/keyrings/h3.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/h3.gpg] http://apt.h3 trixie main" \
       > /etc/apt/sources.list.d/h3.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends ldk-controller \
    && apt-get purge -y curl gpg \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /etc/ldk-controller
USER ldk-controller
EXPOSE 9735
ENTRYPOINT ["/usr/bin/ldk-controller"]
